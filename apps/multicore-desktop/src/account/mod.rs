mod api;
mod controller;
mod secure_store;
mod session;

pub(crate) use api::{AccountApi, AccountApiError, EmailLoginError, LoginPoll};
pub(crate) use controller::{
    AccountCommand, AccountController, AccountEvent, AccountViewState, EmailCredentials,
};
pub(crate) use secure_store::{CredentialSessionStore, SessionStore};
pub(crate) use session::{AccountUser, StoredSession};

pub(crate) const CABINET_API_BASE_URL: &str = "https://cp.terrieflow.shop/api";
pub(crate) const CABINET_URL: &str = "https://cp.terrieflow.shop";
pub(crate) const CABINET_LOGIN_URL: &str = "https://cp.terrieflow.shop/login";
pub(crate) const TELEGRAM_BOT_USERNAME: &str = "TerrieFlow_bot";

pub(crate) fn telegram_login_url(token: &str) -> Option<String> {
    let token = token.trim();
    if token.is_empty()
        || token.len() > 512
        || !token
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return None;
    }
    Some(format!(
        "https://t.me/{TELEGRAM_BOT_USERNAME}?start=webauth_{token}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn telegram_url_accepts_only_a_safe_deeplink_token() {
        assert_eq!(
            telegram_login_url("abc_DEF-123").as_deref(),
            Some("https://t.me/TerrieFlow_bot?start=webauth_abc_DEF-123")
        );
        assert!(telegram_login_url("").is_none());
        assert!(telegram_login_url("abc&start=attacker").is_none());
        assert!(telegram_login_url("abc/def").is_none());
    }
}
