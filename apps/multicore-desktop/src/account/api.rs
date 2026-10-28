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

}
