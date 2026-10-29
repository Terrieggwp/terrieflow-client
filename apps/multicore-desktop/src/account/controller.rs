use super::{
    AccountApi, AccountApiError, AccountSubscriptionAccess, AccountUser, CredentialSessionStore,
    EmailLoginError, LoginPoll, SessionStore, StoredSession, telegram_login_url,
};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant, SystemTime};
use zeroize::{Zeroize, ZeroizeOnDrop};

const LOGIN_POLL_INTERVAL: Duration = Duration::from_secs(3);

pub(crate) enum AccountCommand {
    Restore,
    Login,
    EmailLogin(EmailCredentials),
    CancelLogin,
    CheckSubscription(u64),
    Logout,
    Shutdown,
}

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct EmailCredentials {
    pub(crate) email: String,
    pub(crate) password: String,
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
    ImportSubscription(String),
    SubscriptionAccess(AccountSubscriptionAccess),
    SubscriptionAccessUnavailable,
    SubscriptionCheckStarted,
    SubscriptionLinkUnavailable,
}

pub(crate) struct AccountController {
    commands: Sender<AccountCommand>,
    events: Receiver<AccountEvent>,
    subscription_check_pending: Arc<AtomicBool>,
    session_generation: Arc<AtomicU64>,
}

impl AccountController {
    pub(crate) fn start(api: AccountApi, store: CredentialSessionStore) -> Self {
        let (commands, command_rx) = mpsc::channel();
        let (event_tx, events) = mpsc::channel();
        let subscription_check_pending = Arc::new(AtomicBool::new(false));
        let worker_check_pending = subscription_check_pending.clone();
        let session_generation = Arc::new(AtomicU64::new(0));
        let worker_session_generation = session_generation.clone();
        thread::Builder::new()
            .name("terrieflow-account".into())
            .spawn(move || {
                worker(
                    api,
                    store,
                    command_rx,
                    event_tx,
                    worker_check_pending,
                    worker_session_generation,
                )
            })
            .expect("start account worker");
        Self {
            commands,
            events,
            subscription_check_pending,
            session_generation,
        }
    }

    pub(crate) fn send(&self, command: AccountCommand) {
        if matches!(
            &command,
            AccountCommand::Login | AccountCommand::EmailLogin(_) | AccountCommand::Logout
        ) {
            self.session_generation.fetch_add(1, Ordering::AcqRel);
        }
        let _ = self.commands.send(command);
    }

    pub(crate) fn try_event(&self) -> Option<AccountEvent> {
        self.events.try_recv().ok()
    }

    /// Starts the one shared subscription check, or joins the one already in
    /// flight. The worker is single-threaded as well, so no stale parallel
    /// response can overtake a newer one.
    pub(crate) fn request_subscription_check(&self) -> bool {
        if self
            .subscription_check_pending
            .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_err()
        {
            return false;
        }
        let generation = self.session_generation.load(Ordering::Acquire);
        if self
            .commands
            .send(AccountCommand::CheckSubscription(generation))
            .is_err()
        {
            self.subscription_check_pending
                .store(false, Ordering::Release);
            return false;
        }
        true
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
    subscription_check_pending: Arc<AtomicBool>,
    session_generation: Arc<AtomicU64>,
) {
    let mut session: Option<StoredSession> = None;
    let mut login: Option<PendingLogin> = None;
    let mut last_access_blocks_routes: Option<bool> = None;
    publish(&events, guest_state());

    loop {
        let timeout = login
            .as_ref()
            .map(|pending| pending.next_poll.saturating_duration_since(Instant::now()))
            .unwrap_or(Duration::from_secs(60));
        match commands.recv_timeout(timeout) {
            Ok(AccountCommand::Restore) => restore(
                &api,
                &store,
                &events,
                &mut session,
                &mut last_access_blocks_routes,
            ),
            Ok(AccountCommand::Login) => {
                if login.is_none() {
                    begin_login(&api, &events, &mut login);
                }
            }
            Ok(AccountCommand::EmailLogin(credentials)) => {
                login = None;
                last_access_blocks_routes = None;
                begin_email_login(
                    &api,
                    &store,
                    &events,
                    &mut session,
                    &mut last_access_blocks_routes,
                    credentials,
                );
            }
            Ok(AccountCommand::CancelLogin) => {
                login = None;
                publish(
                    &events,
                    session.as_ref().map(authenticated_state).unwrap_or_else(guest_state),
                );
            }
            Ok(AccountCommand::CheckSubscription(generation)) => {
                let _ = events.send(AccountEvent::SubscriptionCheckStarted);
                if generation != session_generation.load(Ordering::Acquire) {
                    // Account/logout changed while the request was queued.
                } else if let Some(current) = session.as_mut() {
                    check_subscription(
                        &api,
                        &store,
                        &events,
                        current,
                        &mut last_access_blocks_routes,
                        &session_generation,
                        generation,
                    );
                } else {
                    let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
                }
                subscription_check_pending.store(false, Ordering::Release);
            }
            Ok(AccountCommand::Logout) => {
                login = None;
                if let Some(current) = session.take() {
                    let _ = api.logout(&current.refresh_token);
                }
                let _ = store.clear();
                last_access_blocks_routes = None;
                subscription_check_pending.store(false, Ordering::Release);
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
                                last_access_blocks_routes = None;
                                if store.save(&current).is_err() {
                                    publish(&events, error_state("Не удалось безопасно сохранить сессию"));
                                } else {
                                    session = restore_loaded(
                                        &api,
                                        &store,
                                        &events,
                                        current,
                                        &mut last_access_blocks_routes,
                                    );
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

fn begin_email_login(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    session: &mut Option<StoredSession>,
    last_access_blocks_routes: &mut Option<bool>,
    mut credentials: EmailCredentials,
) {
    credentials.email = credentials.email.trim().to_lowercase();
    if !valid_email(&credentials.email) || credentials.password.is_empty() {
        publish(
            events,
            email_error_state("Введите корректную почту и пароль"),
        );
        return;
    }

    publish(events, email_login_state());
    match api.email_login(
        &credentials.email,
        &credentials.password,
        SystemTime::now(),
    ) {
        Ok(current) => {
            if store.save(&current).is_err() {
                publish(
                    events,
                    email_error_state("Не удалось безопасно сохранить сессию"),
                );
                return;
            }
            *session = restore_loaded(
                api,
                store,
                events,
                current,
                last_access_blocks_routes,
            );
        }
        Err(EmailLoginError::InvalidCredentials) => publish(
            events,
            email_error_state("Не удалось войти. Проверьте почту и пароль"),
        ),
        Err(EmailLoginError::EmailNotVerified) => publish(
            events,
            email_error_state("Подтвердите почту по ссылке из письма и повторите вход"),
        ),
        Err(EmailLoginError::RateLimited(wait)) => publish(
            events,
            email_error_state(&format!(
                "Слишком много попыток. Повторите через {} с",
                wait.as_secs()
            )),
        ),
        Err(EmailLoginError::Network | EmailLoginError::ServerUnavailable) => publish(
            events,
            email_error_state("Кабинет временно недоступен. Проверьте соединение"),
        ),
        Err(EmailLoginError::InvalidResponse | EmailLoginError::UnexpectedStatus) => publish(
            events,
            email_error_state("Не удалось войти. Повторите попытку позже"),
        ),
    }
}

fn valid_email(email: &str) -> bool {
    if email.is_empty() || email.len() > 254 || email.contains(char::is_whitespace) {
        return false;
    }
    let Some((local, domain)) = email.rsplit_once('@') else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && domain.contains('.')
        && !domain.starts_with('.')
        && !domain.ends_with('.')
}

fn restore(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    session: &mut Option<StoredSession>,
    last_access_blocks_routes: &mut Option<bool>,
) {
    *last_access_blocks_routes = None;
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
    *session = restore_loaded(api, store, events, loaded, last_access_blocks_routes);
}

fn restore_loaded(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    mut current: StoredSession,
    last_access_blocks_routes: &mut Option<bool>,
) -> Option<StoredSession> {
    publish(events, refreshing_state(Some(&current.user)));
    if !current.access_is_fresh(SystemTime::now()) {
        match refresh_tokens(api, store, &mut current) {
            Ok(()) => {}
            Err(error) if error.is_offline() => {
                publish(
                    events,
                    offline_state(Some(&current.user), "Нет связи с кабинетом"),
                );
                let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
                return Some(current);
            }
            Err(_) => {
                let _ = store.clear();
                publish(events, expired_state());
                let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
                return None;
            }
        }
    }
    match api.profile(&current.access_token) {
        Ok(user) => {
            current.user = user;
            let _ = store.save(&current);
            publish(events, authenticated_state(&current));
            publish_subscription(api, events, &current, last_access_blocks_routes, true);
            Some(current)
        }
        Err(AccountApiError::Unauthorized) => {
            if refresh_tokens(api, store, &mut current).is_ok() {
                match api.profile(&current.access_token) {
                    Ok(user) => {
                        current.user = user;
                        let _ = store.save(&current);
                        publish(events, authenticated_state(&current));
                        publish_subscription(api, events, &current, last_access_blocks_routes, true);
                        Some(current)
                    }
                    _ => {
                        let _ = store.clear();
                        publish(events, expired_state());
                        let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
                        None
                    }
                }
            } else {
                let _ = store.clear();
                publish(events, expired_state());
                let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
                None
            }
        }
        Err(error) if error.is_offline() => {
            publish(events, offline_state(Some(&current.user), "Нет связи с кабинетом"));
            let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
            Some(current)
        }
        Err(_) => {
            publish(events, error_state("Не удалось проверить аккаунт"));
            let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
            Some(current)
        }
    }
}

fn publish_subscription(
    api: &AccountApi,
    events: &Sender<AccountEvent>,
    current: &StoredSession,
    last_access_blocks_routes: &mut Option<bool>,
    import_on_first_success: bool,
) {
    match api.subscription_access(&current.access_token) {
        Ok(Some(access)) => {
            let blocks_routes = access.blocks_routes();
            let restored = last_access_blocks_routes.is_some_and(|previous| previous)
                && !blocks_routes;
            let first = last_access_blocks_routes.is_none();
            *last_access_blocks_routes = Some(blocks_routes);
            let _ = events.send(AccountEvent::SubscriptionAccess(access));
            if restored || (first && import_on_first_success) {
                publish_connection_link(api, events, current);
            }
        }
        Ok(None) | Err(_) => {
            let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
        }
    }
}

fn publish_connection_link(
    api: &AccountApi,
    events: &Sender<AccountEvent>,
    current: &StoredSession,
) {
    match api.connection_link(&current.access_token) {
        Ok(Some(url)) => {
            let _ = events.send(AccountEvent::ImportSubscription(url));
        }
        Ok(None) | Err(_) => {
            let _ = events.send(AccountEvent::SubscriptionLinkUnavailable);
        }
    }
}

fn check_subscription(
    api: &AccountApi,
    store: &CredentialSessionStore,
    events: &Sender<AccountEvent>,
    current: &mut StoredSession,
    last_access_blocks_routes: &mut Option<bool>,
    session_generation: &AtomicU64,
    expected_generation: u64,
) {
    if !current.access_is_fresh(SystemTime::now())
        && refresh_tokens(api, store, current).is_err()
    {
        let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
        return;
    }
    let result = api.subscription_access(&current.access_token);
    if session_generation.load(Ordering::Acquire) != expected_generation {
        return;
    }
    match result {
        Ok(Some(access)) => {
            let blocks_routes = access.blocks_routes();
            let restored = last_access_blocks_routes.is_some_and(|previous| previous)
                && !blocks_routes;
            *last_access_blocks_routes = Some(blocks_routes);
            let _ = events.send(AccountEvent::SubscriptionAccess(access));
            if restored {
                let link = api.connection_link(&current.access_token);
                if session_generation.load(Ordering::Acquire) != expected_generation {
                    return;
                }
                match link {
                    Ok(Some(url)) => {
                        let _ = events.send(AccountEvent::ImportSubscription(url));
                    }
                    Ok(None) | Err(_) => {
                        let _ = events.send(AccountEvent::SubscriptionLinkUnavailable);
                    }
                }
            }
        }
        Ok(None) | Err(_) => {
            let _ = events.send(AccountEvent::SubscriptionAccessUnavailable);
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
        detail: "Telegram или email".into(),
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

fn refreshing_state(user: Option<&AccountUser>) -> AccountViewState {
    user.map(|user| user_state(user, "refreshing", "Проверяем аккаунт…"))
        .unwrap_or_else(|| AccountViewState {
            phase: "refreshing",
            display_name: "Проверяем аккаунт…".into(),
            busy: true,
            ..Default::default()
        })
}

fn expired_state() -> AccountViewState {
    AccountViewState {
        phase: "expired",
        display_name: "Сессия завершена".into(),
        detail: "Войдите снова удобным способом".into(),
        ..Default::default()
    }
}

fn email_login_state() -> AccountViewState {
    AccountViewState {
        phase: "email-login",
        display_name: "Входим в аккаунт…".into(),
        detail: "Проверяем почту и пароль".into(),
        busy: true,
        ..Default::default()
    }
}

fn email_error_state(detail: &str) -> AccountViewState {
    AccountViewState {
        phase: "email-error",
        display_name: "Войти в TerrieFlow".into(),
        detail: detail.into(),
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

#[cfg(test)]
mod tests {
    use super::{refreshing_state, valid_email, AccountUser};

    #[test]
    fn email_validation_rejects_incomplete_or_unsafe_values() {
        assert!(valid_email("user@example.com"));
        assert!(valid_email("user.name+vpn@sub.example.com"));
        assert!(!valid_email("user"));
        assert!(!valid_email("@example.com"));
        assert!(!valid_email("user@example"));
        assert!(!valid_email("user @example.com"));
    }

    #[test]
    fn refreshing_an_existing_session_keeps_it_authenticated() {
        let user = AccountUser {
            id: 1,
            telegram_id: None,
            first_name: Some("Alex".into()),
            last_name: None,
            username: Some("alex".into()),
            email: None,
            email_verified: false,
            balance_kopeks: 0,
            balance_rubles: 0.0,
            referral_code: None,
            language: String::new(),
            created_at: String::new(),
            auth_type: "email".into(),
        };
        let state = refreshing_state(Some(&user));
        assert!(state.authenticated);
        assert!(state.busy);
        assert_eq!(state.phase, "refreshing");
        assert_eq!(state.display_name, "Alex");
    }
}
