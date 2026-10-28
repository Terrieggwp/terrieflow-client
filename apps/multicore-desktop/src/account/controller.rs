use super::{
    AccountApi, AccountApiError, AccountUser, CredentialSessionStore, LoginPoll, SessionStore,
    StoredSession, telegram_login_url,
};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::thread;
use std::time::{Duration, Instant, SystemTime};

const LOGIN_POLL_INTERVAL: Duration = Duration::from_secs(3);

pub(crate) enum AccountCommand {
    Restore,
    Login,
    CancelLogin,
    Refresh,
    Logout,
    Shutdown,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AccountViewState {
    pub phase: &'static str,
    pub display_name: String,
    pub username: String,
    pub email: String,
    pub detail: String,
    pub authenticated: bool,
    pub busy: bool,
}

pub(crate) enum AccountEvent {
    State(AccountViewState),
    OpenBrowser(String),
}

pub(crate) struct AccountController {
    commands: Sender<AccountCommand>,
    events: Receiver<AccountEvent>,
}

impl AccountController {
    pub(crate) fn start(api: AccountApi, store: CredentialSessionStore) -> Self {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        thread::Builder::new()
            .name("terrieflow-account".into())
            .spawn(move || worker(api, store, command_rx, event_tx))
            .expect("start account worker");
        Self { commands, events }
    }

    pub(crate) fn send(&self, command: AccountCommand) {
        let _ = self.commands.send(command);
    }

    pub(crate) fn try_event(&self) -> Option<AccountEvent> {
        self.events.try_recv().ok()
    }
}

impl Drop for AccountController {
    fn drop(&mut self) {
        let _ = self.commands.send(AccountCommand::Shutdown);
    }
}

fn worker(
    api: AccountApi,
    store: CredentialSessionStore,
    commands: Receiver<AccountCommand>,
    events: Sender<AccountEvent>,
) {
    let mut session: Option<StoredSession> = None;
    let mut login: Option<PendingLogin> = None;
    publish(&events, guest_state());

    loop {
        let timeout = login
            .as_ref()
            .map(|pending| pending.next_poll.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::from_secs(60));
        match commands.recv_timeout(timeout) {
            Ok(AccountCommand::Restore) => restore(&api, &store, &events, &mut session),
            Ok(AccountCommand::Login) => {
                if login.is_none() {
                    begin_login(&api, &events, &mut login);
                }
            }
            Ok(AccountCommand::CancelLogin) => {
                login = None;
                publish(
                    &events,
                    session.as_ref().map(authenticated_state).unwrap_or_else(guest_state),
                );
            }
            Ok(AccountCommand::Refresh) => {
                if let Some(current) = session.take() {
                    session = refresh_session(&api, &store, &events, current);
                }
            }
            Ok(AccountCommand::Logout) => {
                login = None;
                if let Some(current) = session.take() {
                    let _ = api.logout(&current.refresh_token);
                }
                let _ = store.clear();
                publish(&events, guest_state());
            }
            Ok(AccountCommand::Shutdown) | Err(RecvTimeoutError::Disconnected) => break,
            Err(RecvTimeoutError::Timeout) => {
                if let Some(pending) = login.as_mut() {
                    if SystemTime::now() >= pending.expires_at {
                        login = None;
                        publish(&events, expired_state());
                    } else if Instant::now() >= pending.next_poll {
                        match api.poll_login(&pending.token, SystemTime::now()) {
                            Ok(LoginPoll::Pending) => {
                                pending.next_poll = Instant::now() + LOGIN_POLL_INTERVAL;
                            }
                            Ok(LoginPoll::RateLimited(wait)) => {
                                pending.next_poll = Instant::now() + wait;
                            }
                            Ok(LoginPoll::Expired) => {
                                login = None;
                                publish(&events, expired_state());
                            }
                            Ok(LoginPoll::Authenticated(current)) => {
                                login = None;
                                if store.save(&current).is_err() {
                                    publish(&events, error_state("Не удалось безопасно сохранить сессию"));
                                } else {
                                    publish(&events, authenticated_state(&current));
                                    session = Some(current);
                                }
                            }
                            Err(error) if error.is_offline() => {
                                pending.next_poll = Instant::now() + LOGIN_POLL_INTERVAL;
                                publish(&events, waiting_state("Нет связи с кабинетом. Повторяем…"));
                            }
                            Err(_) => {
                                login = None;
                                publish(&events, error_state("Не удалось завершить вход"));
                            }
                        }
                    }
                }
            }
        }
    }
}

struct PendingLogin {
    token: String,
    expires_at: SystemTime,
    next_poll: Instant,
}

fn begin_login(
    api: &AccountApi,
    events: &Sender<AccountEvent>,
    login: &mut Option<PendingLogin>,
) {
    publish(events, requesting_state());
    match api.request_login() {
        Ok(request) => {
            let Some(url) = telegram_login_url(&request.token) else {
                publish(events, error_state("Кабинет вернул некорректную ссылку входа"));
                return;
            };
            *login = Some(PendingLogin {
                token: request.token,
                expires_at: SystemTime::now() + Duration::from_secs(request.expires_in),
                next_poll: Instant::now() + LOGIN_POLL_INTERVAL,
            });
            let _ = events.send(AccountEvent::OpenBrowser(url));
            publish(events, waiting_state("Подтвердите вход в Telegram"));
        }
        Err(error) if error.is_offline() => {
            publish(events, offline_state(None, "Кабинет временно недоступен"));
        }
        Err(_) => publish(events, error_state("Не удалось начать вход")),
    }
}

fn restore(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    session: &mut Option<StoredSession>,
) {
    let loaded = match store.load() {
        Ok(Some(value)) => value,
        Ok(None) => {
            publish(events, guest_state());
            return;
        }
        Err(_) => {
            publish(events, error_state("Не удалось открыть защищённую сессию"));
            return;
        }
    };
    *session = restore_loaded(api, store, events, loaded);
}

fn restore_loaded(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    mut current: StoredSession,
) -> Option<StoredSession> {
    publish(events, refreshing_state());
    if !current.access_is_fresh(SystemTime::now()) {
        match refresh_tokens(api, store, &mut current) {
            Ok(()) => {}
            Err(error) if error.is_offline() => {
                publish(
                    events,
                    offline_state(Some(&current.user), "Нет связи с кабинетом"),
                );
                return Some(current);
            }
            Err(_) => {
                let _ = store.clear();
                publish(events, expired_state());
                return None;
            }
        }
    }
    match api.profile(&current.access_token) {
        Ok(user) => {
            current.user = user;
            let _ = store.save(&current);
            publish(events, authenticated_state(&current));
            Some(current)
        }
        Err(AccountApiError::Unauthorized) => {
            if refresh_tokens(api, store, &mut current).is_ok() {
                match api.profile(&current.access_token) {
                    Ok(user) => {
                        current.user = user;
                        let _ = store.save(&current);
                        publish(events, authenticated_state(&current));
                        Some(current)
                    }
                    _ => {
                        let _ = store.clear();
                        publish(events, expired_state());
                        None
                    }
                }
            } else {
                let _ = store.clear();
                publish(events, expired_state());
                None
            }
        }
        Err(error) if error.is_offline() => {
            publish(events, offline_state(Some(&current.user), "Нет связи с кабинетом"));
            Some(current)
        }
        Err(_) => {
            publish(events, error_state("Не удалось проверить аккаунт"));
            Some(current)
        }
    }
}

fn refresh_session(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    mut current: StoredSession,
) -> Option<StoredSession> {
    publish(events, refreshing_state());
    match refresh_tokens(api, store, &mut current) {
        Ok(()) => restore_loaded(api, store, events, current),
        Err(error) if error.is_offline() => {
            publish(events, offline_state(Some(&current.user), "Нет связи с кабинетом"));
            Some(current)
        }
        Err(_) => {
            let _ = store.clear();
            publish(events, expired_state());
            None
        }
    }
}

fn refresh_tokens(
    api: &AccountApi,
    store: &CredentialSessionStore,
    current: &mut StoredSession,
) -> Result<(), AccountApiError> {
    let (access, refresh, expires_in) = api.refresh(&current.refresh_token)?;
    current.rotate(access, refresh, expires_in, SystemTime::now());
    store
        .save(current)
        .map_err(|_| AccountApiError::InvalidResponse)
}

fn publish(events: &Sender<AccountEvent>, state: AccountViewState) {
    let _ = events.send(AccountEvent::State(state));
}

fn user_state(user: &AccountUser, phase: &'static str, detail: &str) -> AccountViewState {
    AccountViewState {
        phase,
        display_name: user.display_name(),
        username: user
            .username
            .as_deref()
            .map(|value| format!("@{value}"))
            .unwrap_or_default(),
        email: user.email.clone().unwrap_or_default(),
        detail: detail.into(),
        authenticated: true,
        busy: phase == "refreshing",
    }
}

fn authenticated_state(session: &StoredSession) -> AccountViewState {
    user_state(&session.user, "authenticated", "Аккаунт подключён")
}

fn guest_state() -> AccountViewState {
    AccountViewState {
        phase: "guest",
        display_name: "Войти в аккаунт".into(),
        detail: "Через Telegram".into(),
        ..Default::default()
    }
}

fn requesting_state() -> AccountViewState {
    AccountViewState {
        phase: "requesting",
        display_name: "Подключаем Telegram…".into(),
        detail: "Получаем безопасную ссылку".into(),
        busy: true,
        ..Default::default()
    }
}

fn waiting_state(detail: &str) -> AccountViewState {
    AccountViewState {
        phase: "waiting",
        display_name: "Ожидаем подтверждение".into(),
        detail: detail.into(),
        busy: true,
        ..Default::default()
    }
}

fn refreshing_state() -> AccountViewState {
    AccountViewState {
        phase: "refreshing",
        display_name: "Проверяем аккаунт…".into(),
        busy: true,
        ..Default::default()
    }
}

fn expired_state() -> AccountViewState {
    AccountViewState {
        phase: "expired",
        display_name: "Сессия завершена".into(),
        detail: "Войдите снова через Telegram".into(),
        ..Default::default()
    }
}

fn offline_state(user: Option<&AccountUser>, detail: &str) -> AccountViewState {
    user.map(|user| user_state(user, "offline", detail))
        .unwrap_or_else(|| AccountViewState {
            phase: "offline",
            display_name: "Кабинет недоступен".into(),
            detail: detail.into(),
            ..Default::default()
        })
}

fn error_state(detail: &str) -> AccountViewState {
    AccountViewState {
        phase: "error",
        display_name: "Ошибка аккаунта".into(),
        detail: detail.into(),
        ..Default::default()
    }
}
