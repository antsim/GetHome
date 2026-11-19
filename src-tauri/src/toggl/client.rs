//! HTTP client for the Toggl Track API v9.
//!
//! Authentication is HTTP Basic with the API token as the user name and the
//! literal string `api_token` as the password. All requests are recorded in a
//! shared [`RateBudget`].

use std::sync::{Arc, Mutex, RwLock};
use std::time::Duration as StdDuration;

use chrono::{DateTime, SecondsFormat, Utc};
use reqwest::{Method, RequestBuilder, Response, StatusCode, Url};
use serde::de::DeserializeOwned;

use super::models::{Me, NewTimeEntry, TimeEntry};
use super::rate_budget::RateBudget;
use crate::clock::Clock;

pub const DEFAULT_BASE_URL: &str = "https://api.track.toggl.com/api/v9";
const REQUEST_TIMEOUT: StdDuration = StdDuration::from_secs(15);
const USER_AGENT: &str = concat!("tyoaikalaskuri/", env!("CARGO_PKG_VERSION"));

#[derive(Debug, thiserror::Error)]
pub enum TogglError {
    #[error("no API token configured")]
    MissingToken,
    #[error("API token was rejected (HTTP 401/403)")]
    Unauthorized,
    #[error("request quota exceeded (HTTP 402)")]
    QuotaExceeded,
    #[error("rate limited (HTTP 429)")]
    RateLimited,
    #[error("network error: {0}")]
    Network(#[source] reqwest::Error),
    #[error("unexpected HTTP status {0}")]
    Status(u16),
    #[error("invalid response from Toggl: {0}")]
    Decode(#[source] reqwest::Error),
    #[error("invalid request URL: {0}")]
    Url(#[from] url::ParseError),
}

impl TogglError {
    /// Whether the error means Toggl asked us to slow down.
    pub fn is_rate_limit(&self) -> bool {
        matches!(self, Self::QuotaExceeded | Self::RateLimited)
    }
}

pub type Result<T> = std::result::Result<T, TogglError>;

/// Thread-safe Toggl API client. Cheap to share behind an `Arc`.
pub struct TogglClient {
    http: reqwest::Client,
    base_url: String,
    token: RwLock<Option<String>>,
    budget: Mutex<RateBudget>,
    clock: Arc<dyn Clock>,
}

impl TogglClient {
    pub fn new(base_url: impl Into<String>, clock: Arc<dyn Clock>) -> Result<Self> {
        let http = reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(USER_AGENT)
            .build()
            .map_err(TogglError::Network)?;
        Ok(Self {
            http,
            base_url: base_url.into().trim_end_matches('/').to_owned(),
            token: RwLock::new(None),
            budget: Mutex::new(RateBudget::default()),
            clock,
        })
    }

    /// Replaces the API token used for subsequent requests.
    pub fn set_token(&self, token: Option<String>) {
        *self.token.write().unwrap_or_else(|e| e.into_inner()) = token;
    }

    pub fn has_token(&self) -> bool {
        self.token
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .is_some()
    }

    /// Requests made during the last hour (sliding window).
    pub fn requests_in_last_hour(&self) -> usize {
        let now = self.clock.now();
        self.budget
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .count(now)
    }

    /// `GET /me` with workspace data, using the stored token.
    pub async fn me(&self) -> Result<Me> {
        let token = self.current_token()?;
        self.get_me(&token).await
    }

    /// `GET /me` with an explicit token. Used to validate a token before it
    /// is stored.
    pub async fn verify_token(&self, token: &str) -> Result<Me> {
        self.get_me(token).await
    }

    /// Time entries whose start time is within `[start, end)`.
    pub async fn time_entries(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<TimeEntry>> {
        let url = self.url_with_query(
            "/me/time_entries",
            &[("start_date", rfc3339(start)), ("end_date", rfc3339(end))],
        )?;
        self.json(Method::GET, url, None::<&()>).await
    }

    /// Most recent time entries without a date filter (Toggl returns roughly
    /// the latest couple of weeks).
    pub async fn recent_time_entries(&self) -> Result<Vec<TimeEntry>> {
        let url = self.url("/me/time_entries")?;
        self.json(Method::GET, url, None::<&()>).await
    }

    /// The running time entry, or `None` if no timer is running.
    pub async fn current_time_entry(&self) -> Result<Option<TimeEntry>> {
        let url = self.url("/me/time_entries/current")?;
        self.json(Method::GET, url, None::<&()>).await
    }

    /// Starts a new time entry.
    pub async fn start_time_entry(&self, entry: &NewTimeEntry) -> Result<TimeEntry> {
        let url = self.url(&format!("/workspaces/{}/time_entries", entry.workspace_id))?;
        self.json(Method::POST, url, Some(entry)).await
    }

    /// Stops a running time entry.
    pub async fn stop_time_entry(&self, workspace_id: i64, entry_id: i64) -> Result<TimeEntry> {
        let url = self.url(&format!(
            "/workspaces/{workspace_id}/time_entries/{entry_id}/stop"
        ))?;
        self.json(Method::PATCH, url, None::<&()>).await
    }

    async fn get_me(&self, token: &str) -> Result<Me> {
        let url = self.url_with_query("/me", &[("with_related_data", "true".to_owned())])?;
        let request = self.http.get(url).basic_auth(token, Some("api_token"));
        decode(self.send(request).await?).await
    }

    async fn json<B, T>(&self, method: Method, url: Url, body: Option<&B>) -> Result<T>
    where
        B: serde::Serialize + ?Sized,
        T: DeserializeOwned,
    {
        let token = self.current_token()?;
        let mut request = self
            .http
            .request(method, url)
            .basic_auth(token, Some("api_token"));
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(self.send(request).await?).await
    }

    /// Sends a request, records it in the rate budget and maps error statuses.
    async fn send(&self, request: RequestBuilder) -> Result<Response> {
        self.budget
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .record(self.clock.now());

        let response = request.send().await.map_err(TogglError::Network)?;
        match response.status() {
            status if status.is_success() => Ok(response),
            StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(TogglError::Unauthorized),
            StatusCode::PAYMENT_REQUIRED => Err(TogglError::QuotaExceeded),
            StatusCode::TOO_MANY_REQUESTS => Err(TogglError::RateLimited),
            status => Err(TogglError::Status(status.as_u16())),
        }
    }

    fn current_token(&self) -> Result<String> {
        self.token
            .read()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
            .ok_or(TogglError::MissingToken)
    }

    fn url(&self, path: &str) -> Result<Url> {
        Ok(Url::parse(&format!("{}{path}", self.base_url))?)
    }

    fn url_with_query(&self, path: &str, query: &[(&str, String)]) -> Result<Url> {
        Ok(Url::parse_with_params(
            &format!("{}{path}", self.base_url),
            query,
        )?)
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T> {
    response.json::<T>().await.map_err(TogglError::Decode)
}

fn rfc3339(time: DateTime<Utc>) -> String {
    time.to_rfc3339_opts(SecondsFormat::Secs, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::SystemClock;
    use wiremock::matchers::{basic_auth, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    async fn client_for(server: &MockServer) -> TogglClient {
        let client = TogglClient::new(server.uri(), Arc::new(SystemClock)).expect("client");
        client.set_token(Some("secret-token".into()));
        client
    }

    #[tokio::test]
    async fn sends_basic_auth_and_parses_current_entry() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me/time_entries/current"))
            .and(basic_auth("secret-token", "api_token"))
            .respond_with(ResponseTemplate::new(200).set_body_string("null"))
            .expect(1)
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        assert_eq!(client.current_time_entry().await.expect("ok"), None);
        assert_eq!(client.requests_in_last_hour(), 1);
    }

    #[tokio::test]
    async fn passes_date_range_as_rfc3339() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/me/time_entries"))
            .and(query_param("start_date", "2026-03-01T22:00:00Z"))
            .and(query_param("end_date", "2026-03-02T22:00:00Z"))
            .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
            .expect(1)
            .mount(&server)
            .await;

        let client = client_for(&server).await;
        let start = "2026-03-01T22:00:00Z".parse().expect("date");
        let end = "2026-03-02T22:00:00Z".parse().expect("date");
        assert!(client
            .time_entries(start, end)
            .await
            .expect("ok")
            .is_empty());
    }

    #[tokio::test]
    async fn maps_error_statuses() {
        for (status, check) in [
            (
                401,
                (|e| matches!(e, TogglError::Unauthorized)) as fn(&TogglError) -> bool,
            ),
            (402, |e| matches!(e, TogglError::QuotaExceeded)),
            (429, |e| matches!(e, TogglError::RateLimited)),
            (500, |e| matches!(e, TogglError::Status(500))),
        ] {
            let server = MockServer::start().await;
            Mock::given(method("GET"))
                .respond_with(ResponseTemplate::new(status))
                .mount(&server)
                .await;
            let client = client_for(&server).await;
            let err = client.current_time_entry().await.expect_err("should fail");
            assert!(check(&err), "status {status} mapped to {err:?}");
        }
    }

    #[tokio::test]
    async fn fails_fast_without_token() {
        let server = MockServer::start().await;
        let client = TogglClient::new(server.uri(), Arc::new(SystemClock)).expect("client");
        let err = client.current_time_entry().await.expect_err("no token");
        assert!(matches!(err, TogglError::MissingToken));
        assert_eq!(client.requests_in_last_hour(), 0);
    }
}
