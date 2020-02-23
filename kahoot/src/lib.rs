pub mod challenge;

use bytes::Buf;
use http::header::HeaderName;
use http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;

pub type KahootResult<T> = Result<T, KahootError>;

#[derive(Debug)]
pub enum KahootError {
    Hyper(hyper::Error),
    InvalidUrl(http::uri::InvalidUri),
    InvalidStatus(http::StatusCode),
    Json(serde_json::Error),

    ChallengeDecodeError(crate::challenge::DecodeError),
    InvalidCode,
    MissingToken,
}

impl From<hyper::Error> for KahootError {
    fn from(e: hyper::Error) -> Self {
        Self::Hyper(e)
    }
}

impl From<http::uri::InvalidUri> for KahootError {
    fn from(e: http::uri::InvalidUri) -> Self {
        Self::InvalidUrl(e)
    }
}

impl From<serde_json::Error> for KahootError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

impl From<crate::challenge::DecodeError> for KahootError {
    fn from(e: crate::challenge::DecodeError) -> Self {
        Self::ChallengeDecodeError(e)
    }
}

fn epoch_time_millis() -> u128 {
    use std::time::SystemTime;
    use std::time::UNIX_EPOCH;

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis()
}

pub struct Client {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl Client {
    pub fn new() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        Self { client }
    }

    pub async fn probe_code(&self, code: &str) -> KahootResult<ProbeResult> {
        let url = format!(
            "https://kahoot.it/reserve/session/{}/?{}",
            code,
            epoch_time_millis()
        );

        let res = self.client.get(url.parse()?).await?;
        let status = res.status();

        if status == StatusCode::NOT_FOUND {
            return Err(KahootError::InvalidCode);
        }

        if !status.is_success() {
            return Err(KahootError::InvalidStatus(status));
        }

        let token = res
            .headers()
            .get(HeaderName::from_static("x-kahoot-session-token"))
            .and_then(|h| h.to_str().ok())
            .ok_or(KahootError::MissingToken)?
            .to_string();
        let body = hyper::body::aggregate(res.into_body()).await?;
        let response: GetChallengeJsonResponse = serde_json::from_slice(body.bytes())?;
        Ok(ProbeResult { token, response })
    }

    pub async fn get_token(&self, code: &str) -> KahootResult<String> {
        let res = self.probe_code(code).await?;
        let token = crate::challenge::decode(&res.token, &res.response.challenge)?;
        Ok(token)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct ProbeResult {
    token: String,
    response: GetChallengeJsonResponse,
}

#[derive(Debug, Deserialize)]
pub struct GetChallengeJsonResponse {
    #[serde(rename = "twoFactorAuth")]
    two_factor_auth: bool,

    namerator: bool,

    #[serde(rename = "participantId")]
    participant_id: Option<serde_json::Value>,

    #[serde(rename = "smartPractice")]
    smart_practice: bool,

    challenge: String,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}
