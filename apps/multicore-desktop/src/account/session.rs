use serde::{Deserialize, Serialize};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use zeroize::Zeroize;

const EXPIRY_SAFETY_MARGIN: Duration = Duration::from_secs(30);

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct AccountUser {
    pub id: u64,
    pub telegram_id: Option<i64>,
    pub username: Option<String>,
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub email: Option<String>,
    #[serde(default)]
    pub email_verified: bool,
    #[serde(default)]
    pub balance_kopeks: i64,
    #[serde(default)]
    pub balance_rubles: f64,
    pub referral_code: Option<String>,
    #[serde(default)]
    pub language: String,
    #[serde(default)]
    pub created_at: String,
    #[serde(default)]
    pub auth_type: String,
}

impl AccountUser {
    pub(crate) fn display_name(&self) -> String {
        let full_name = [self.first_name.as_deref(), self.last_name.as_deref()]
            .into_iter()
            .flatten()
            .filter(|part| !part.trim().is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        if !full_name.is_empty() {
            return full_name;
        }
        if let Some(username) = self.username.as_deref().filter(|value| !value.is_empty()) {
            return format!("@{username}");
        }
        if let Some(email) = self.email.as_deref().filter(|value| !value.is_empty()) {
            return email.to_owned();
        }
        "TerrieFlow".to_owned()
    }
}

#[derive(Deserialize, Serialize)]
pub(crate) struct StoredSession {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at_unix: u64,
    pub user: AccountUser,
}

impl StoredSession {
    pub(crate) fn new(
        access_token: String,
        refresh_token: String,
        expires_in: Duration,
        user: AccountUser,
        now: SystemTime,
    ) -> Self {
        let now_unix = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        Self {
            access_token,
            refresh_token,
            access_expires_at_unix: now_unix.saturating_add(expires_in.as_secs()),
            user,
        }
    }

    pub(crate) fn access_is_fresh(&self, now: SystemTime) -> bool {
        let now_unix = now.duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
        now_unix.saturating_add(EXPIRY_SAFETY_MARGIN.as_secs()) < self.access_expires_at_unix
    }

    pub(crate) fn rotate(
        &mut self,
        access_token: String,
        refresh_token: String,
        expires_in: Duration,
        now: SystemTime,
    ) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
        self.access_token = access_token;
        self.refresh_token = refresh_token;
        self.access_expires_at_unix = now
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs()
            .saturating_add(expires_in.as_secs());
    }
}

impl Drop for StoredSession {
    fn drop(&mut self) {
        self.access_token.zeroize();
        self.refresh_token.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> AccountUser {
        AccountUser {
            id: 7,
            telegram_id: Some(8),
            username: Some("terrie".into()),
            first_name: Some("Terrie".into()),
            last_name: Some("Flow".into()),
            email: None,
            email_verified: false,
            balance_kopeks: 0,
            balance_rubles: 0.0,
            referral_code: None,
            language: "ru".into(),
            created_at: String::new(),
            auth_type: "telegram".into(),
        }
    }

    #[test]
    fn session_refreshes_before_the_server_expiry_boundary() {
        let now = UNIX_EPOCH + Duration::from_secs(1_000);
        let session = StoredSession::new(
            "access".into(),
            "refresh".into(),
            Duration::from_secs(120),
            user(),
            now,
        );
        assert!(session.access_is_fresh(now + Duration::from_secs(89)));
        assert!(!session.access_is_fresh(now + Duration::from_secs(90)));
    }

    #[test]
    fn user_display_name_has_stable_fallbacks() {
        let mut value = user();
        assert_eq!(value.display_name(), "Terrie Flow");
        value.first_name = None;
        value.last_name = None;
        assert_eq!(value.display_name(), "@terrie");
        value.username = None;
        value.email = Some("user@example.test".into());
        assert_eq!(value.display_name(), "user@example.test");
    }
}
