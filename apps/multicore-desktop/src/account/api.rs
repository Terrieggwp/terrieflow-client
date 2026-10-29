use super::{AccountUser, StoredSession};
use reqwest::StatusCode;
use reqwest::blocking::{Client, Response};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::time::{Duration, SystemTime};

const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(8);

#[derive(Clone)]
pub(crate) struct AccountApi {
    client: Client,
    base_url: reqwest::Url,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
pub(crate) struct LoginRequest {
    pub token: String,
    pub bot_username: String,
    pub expires_in: u64,
}

pub(crate) enum LoginPoll {
    Pending,
    Authenticated(StoredSession),
    Expired,
    RateLimited(Duration),
}

#[derive(Debug)]
pub(crate) enum EmailLoginError {
    InvalidCredentials,
    EmailNotVerified,
    RateLimited(Duration),
    Network,
    ServerUnavailable,
    InvalidResponse,
    UnexpectedStatus,
}

#[derive(Debug)]
pub(crate) enum AccountApiError {
    InvalidConfiguration,
    Network,
    Unauthorized,
    ServerUnavailable,
    UnexpectedStatus(u16),
    InvalidResponse,
}

impl AccountApiError {
    pub(crate) fn is_offline(&self) -> bool {
        matches!(self, Self::Network | Self::ServerUnavailable)
    }
}

impl fmt::Display for AccountApiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::InvalidConfiguration => "invalid cabinet API configuration",
            Self::Network => "cabinet API is unreachable",
            Self::Unauthorized => "cabinet session is no longer valid",
            Self::ServerUnavailable => "cabinet API is temporarily unavailable",
            Self::UnexpectedStatus(status) => {
                return write!(formatter, "cabinet API returned status {status}");
            }
            Self::InvalidResponse => "cabinet API returned an invalid response",
        };
        formatter.write_str(message)
    }
}

impl std::error::Error for AccountApiError {}

#[derive(Deserialize)]
struct AuthResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
    user: AccountUser,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: u64,
}

#[derive(Serialize)]
struct PollRequest<'a> {
    token: &'a str,
}

#[derive(Serialize)]
struct RefreshRequest<'a> {
    refresh_token: &'a str,
}

#[derive(Serialize)]
struct EmailLoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct ConnectionLinkResponse {
    subscription_url: Option<String>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct AccountSubscriptionAccess {
    pub(crate) status: String,
    pub(crate) tariff_name: String,
    pub(crate) end_date: String,
}

impl AccountSubscriptionAccess {
    pub(crate) fn blocks_routes(&self) -> bool {
        matches!(self.status.as_str(), "expired" | "disabled")
    }
}

#[derive(Deserialize)]
struct SubscriptionResponse {
    subscription: Option<CabinetSubscription>,
}

#[derive(Deserialize)]
struct CabinetSubscription {
    status: String,
    #[serde(default)]
    is_expired: bool,
    tariff_name: Option<String>,
    end_date: Option<String>,
}

impl AccountApi {
    pub(crate) fn new(base_url: &str) -> Result<Self, AccountApiError> {
        let mut base_url = reqwest::Url::parse(base_url)
            .map_err(|_| AccountApiError::InvalidConfiguration)?;
        if base_url.scheme() != "https" && !cfg!(test) {
            return Err(AccountApiError::InvalidConfiguration);
        }
        if !base_url.path().ends_with('/') {
            let path = format!("{}/", base_url.path());
            base_url.set_path(&path);
        }
        let client = Client::builder()
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(REQUEST_TIMEOUT)
            .user_agent(concat!("TerrieFlow/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| AccountApiError::InvalidConfiguration)?;
        Ok(Self { client, base_url })
    }

    fn endpoint(&self, path: &str) -> Result<reqwest::Url, AccountApiError> {
        self.base_url
            .join(path)
            .map_err(|_| AccountApiError::InvalidConfiguration)
    }

    pub(crate) fn request_login(&self) -> Result<LoginRequest, AccountApiError> {
        let response = self
            .client
            .post(self.endpoint("cabinet/auth/deeplink/request")?)
            .json(&serde_json::json!({}))
            .send()
            .map_err(map_transport_error)?;
        parse_success(response)
    }

    pub(crate) fn poll_login(
        &self,
        token: &str,
        now: SystemTime,
    ) -> Result<LoginPoll, AccountApiError> {
        let response = self
            .client
            .post(self.endpoint("cabinet/auth/deeplink/poll")?)
            .json(&PollRequest { token })
            .send()
            .map_err(map_transport_error)?;
        match response.status() {
            StatusCode::OK => {
                let auth: AuthResponse = response
                    .json()
                    .map_err(|_| AccountApiError::InvalidResponse)?;
                validate_tokens(&auth.access_token, &auth.refresh_token)?;
                Ok(LoginPoll::Authenticated(StoredSession::new(
                    auth.access_token,
                    auth.refresh_token,
                    Duration::from_secs(auth.expires_in),
                    auth.user,
                    now,
                )))
            }
            StatusCode::ACCEPTED => Ok(LoginPoll::Pending),
            StatusCode::GONE => Ok(LoginPoll::Expired),
            StatusCode::TOO_MANY_REQUESTS => Ok(LoginPoll::RateLimited(retry_after(&response))),
            status => Err(classify_status(status)),
        }
    }

    pub(crate) fn email_login(
        &self,
        email: &str,
        password: &str,
        now: SystemTime,
    ) -> Result<StoredSession, EmailLoginError> {
        let response = self
            .client
            .post(
                self.endpoint("cabinet/auth/email/login")
                    .map_err(|_| EmailLoginError::InvalidResponse)?,
            )
            .json(&EmailLoginRequest { email, password })
            .send()
            .map_err(|_| EmailLoginError::Network)?;
        match response.status() {
            StatusCode::OK => {
                let auth: AuthResponse = response
                    .json()
                    .map_err(|_| EmailLoginError::InvalidResponse)?;
                validate_tokens(&auth.access_token, &auth.refresh_token)
                    .map_err(|_| EmailLoginError::InvalidResponse)?;
                Ok(StoredSession::new(
                    auth.access_token,
                    auth.refresh_token,
                    Duration::from_secs(auth.expires_in),
                    auth.user,
                    now,
                ))
            }
            StatusCode::UNAUTHORIZED => Err(EmailLoginError::InvalidCredentials),
            StatusCode::FORBIDDEN => Err(EmailLoginError::EmailNotVerified),
            StatusCode::TOO_MANY_REQUESTS => {
                Err(EmailLoginError::RateLimited(retry_after(&response)))
            }
            status if status.is_server_error() => Err(EmailLoginError::ServerUnavailable),
            _ => Err(EmailLoginError::UnexpectedStatus),
        }
    }

    pub(crate) fn profile(&self, access_token: &str) -> Result<AccountUser, AccountApiError> {
        validate_single_token(access_token)?;
        let response = self
            .client
            .get(self.endpoint("cabinet/auth/me")?)
            .bearer_auth(access_token)
            .send()
            .map_err(map_transport_error)?;
        parse_success(response)
    }

    pub(crate) fn connection_link(
        &self,
        access_token: &str,
    ) -> Result<Option<String>, AccountApiError> {
        validate_single_token(access_token)?;
        let response = self
            .client
            .get(self.endpoint("cabinet/subscription/connection-link")?)
            .bearer_auth(access_token)
            .send()
            .map_err(map_transport_error)?;
        let link: ConnectionLinkResponse = parse_success(response)?;
        link.subscription_url
            .map(|value| validate_subscription_url(&value))
            .transpose()
    }

    pub(crate) fn subscription_access(
        &self,
        access_token: &str,
    ) -> Result<Option<AccountSubscriptionAccess>, AccountApiError> {
        validate_single_token(access_token)?;
        let response = self
            .client
            .get(self.endpoint("cabinet/subscription")?)
            .bearer_auth(access_token)
            .send()
            .map_err(map_transport_error)?;
        let response: SubscriptionResponse = parse_success(response)?;
        response.subscription.map(normalize_subscription_access).transpose()
    }

    pub(crate) fn refresh(
        &self,
        refresh_token: &str,
    ) -> Result<(String, String, Duration), AccountApiError> {
        validate_single_token(refresh_token)?;
        let response = self
            .client
            .post(self.endpoint("cabinet/auth/refresh")?)
            .json(&RefreshRequest { refresh_token })
            .send()
            .map_err(map_transport_error)?;
        let tokens: TokenResponse = parse_success(response)?;
        validate_tokens(&tokens.access_token, &tokens.refresh_token)?;
        Ok((
            tokens.access_token,
            tokens.refresh_token,
            Duration::from_secs(tokens.expires_in),
        ))
    }

    pub(crate) fn logout(&self, refresh_token: &str) -> Result<(), AccountApiError> {
        validate_single_token(refresh_token)?;
        let response = self
            .client
            .post(self.endpoint("cabinet/auth/logout")?)
            .json(&RefreshRequest { refresh_token })
            .send()
            .map_err(map_transport_error)?;
        if response.status().is_success() || response.status() == StatusCode::UNAUTHORIZED {
            Ok(())
        } else {
            Err(classify_status(response.status()))
        }
    }
}

fn validate_single_token(token: &str) -> Result<(), AccountApiError> {
    if token.is_empty() || token.len() > 16 * 1024 || token.bytes().any(|byte| byte == 0) {
        Err(AccountApiError::InvalidResponse)
    } else {
        Ok(())
    }
}

fn validate_tokens(access_token: &str, refresh_token: &str) -> Result<(), AccountApiError> {
    validate_single_token(access_token)?;
    validate_single_token(refresh_token)
}

fn validate_subscription_url(value: &str) -> Result<String, AccountApiError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 16 * 1024 {
        return Err(AccountApiError::InvalidResponse);
    }
    let parsed = reqwest::Url::parse(value).map_err(|_| AccountApiError::InvalidResponse)?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || !parsed.username().is_empty()
        || parsed.password().is_some()
    {
        return Err(AccountApiError::InvalidResponse);
    }
    Ok(value.to_owned())
}

fn normalize_subscription_access(
    subscription: CabinetSubscription,
) -> Result<AccountSubscriptionAccess, AccountApiError> {
    let mut status = subscription.status.trim().to_ascii_lowercase();
    if status.is_empty()
        || status.len() > 32
        || !status
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
    {
        return Err(AccountApiError::InvalidResponse);
    }
    // Cabinet exposes the definitive expiry flag separately from its broader
    // lifecycle status. During Grace the server keeps `is_expired` false, so
    // the client must not infer expiry from dates or traffic counters.
    if subscription.is_expired {
        status = "expired".to_owned();
    }
    fn bounded(value: Option<String>) -> Result<String, AccountApiError> {
        let value = value.unwrap_or_default().trim().to_owned();
        if value.len() > 256 || value.chars().any(char::is_control) {
            Err(AccountApiError::InvalidResponse)
        } else {
            Ok(value)
        }
    }
    Ok(AccountSubscriptionAccess {
        status,
        tariff_name: bounded(subscription.tariff_name)?,
        end_date: bounded(subscription.end_date)?,
    })
}

fn map_transport_error(_error: reqwest::Error) -> AccountApiError {
    AccountApiError::Network
}

fn classify_status(status: StatusCode) -> AccountApiError {
    if status == StatusCode::UNAUTHORIZED || status == StatusCode::FORBIDDEN {
        AccountApiError::Unauthorized
    } else if status.is_server_error() {
        AccountApiError::ServerUnavailable
    } else {
        AccountApiError::UnexpectedStatus(status.as_u16())
    }
}

fn parse_success<T: for<'de> Deserialize<'de>>(response: Response) -> Result<T, AccountApiError> {
    if !response.status().is_success() {
        return Err(classify_status(response.status()));
    }
    response
        .json::<T>()
        .map_err(|_| AccountApiError::InvalidResponse)
}

fn retry_after(response: &Response) -> Duration {
    response
        .headers()
        .get(reqwest::header::RETRY_AFTER)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.parse::<u64>().ok())
        .map(Duration::from_secs)
        .unwrap_or(Duration::from_secs(3))
        .clamp(Duration::from_secs(1), Duration::from_secs(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_errors_do_not_expose_server_bodies_or_tokens() {
        assert!(matches!(
            classify_status(StatusCode::UNAUTHORIZED),
            AccountApiError::Unauthorized
        ));
        assert!(matches!(
            classify_status(StatusCode::BAD_GATEWAY),
            AccountApiError::ServerUnavailable
        ));
        assert!(matches!(
            classify_status(StatusCode::BAD_REQUEST),
            AccountApiError::UnexpectedStatus(400)
        ));
    }

    #[test]
    fn production_client_rejects_a_non_tls_endpoint() {
        if !cfg!(test) {
            assert!(AccountApi::new("http://example.test/api").is_err());
        }
        assert!(AccountApi::new("not a url").is_err());
    }

    #[test]
    fn subscription_links_require_tls_and_no_embedded_credentials() {
        assert_eq!(
            validate_subscription_url(" https://example.test/sub?token=secret ")
                .expect("valid subscription URL"),
            "https://example.test/sub?token=secret"
        );
        assert!(validate_subscription_url("http://example.test/sub").is_err());
        assert!(validate_subscription_url("https://user:pass@example.test/sub").is_err());
        assert!(validate_subscription_url("not a url").is_err());
    }

    #[test]
    fn only_confirmed_expired_or_disabled_access_blocks_routes() {
        for status in ["active", "trial", "limited", "grace", "unknown"] {
            assert!(!AccountSubscriptionAccess {
                status: status.into(),
                ..Default::default()
            }
            .blocks_routes());
        }
        for status in ["expired", "disabled"] {
            assert!(AccountSubscriptionAccess {
                status: status.into(),
                ..Default::default()
            }
            .blocks_routes());
        }
    }

    #[test]
    fn subscription_access_is_normalized_without_inventing_grace() {
        let access = normalize_subscription_access(CabinetSubscription {
            status: " ACTIVE ".into(),
            is_expired: false,
            tariff_name: Some(" Premium Pro ".into()),
            end_date: Some(" 2026-11-10T12:00:00Z ".into()),
        })
        .expect("valid subscription access");
        assert_eq!(access.status, "active");
        assert_eq!(access.tariff_name, "Premium Pro");
        assert_eq!(access.end_date, "2026-11-10T12:00:00Z");
        assert!(!access.blocks_routes());
    }

    #[test]
    fn server_expiry_flag_wins_without_expiring_active_grace() {
        let expired = normalize_subscription_access(CabinetSubscription {
            status: "active".into(),
            is_expired: true,
            tariff_name: None,
            end_date: None,
        })
        .expect("confirmed expired subscription");
        assert_eq!(expired.status, "expired");
        assert!(expired.blocks_routes());

        let grace = normalize_subscription_access(CabinetSubscription {
            status: "active".into(),
            is_expired: false,
            tariff_name: None,
            end_date: None,
        })
        .expect("active grace subscription");
        assert_eq!(grace.status, "active");
        assert!(!grace.blocks_routes());
    }

}
