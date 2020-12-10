use crate::{
    KahootError,
    KahootResult,
};
use bytes::Buf;
use ducc::Ducc;
use http::{
    header::HeaderName,
    StatusCode,
};
use log::trace;
use serde::Deserialize;
#[cfg(debug_assertions)]
use std::time::Instant;
use std::{
    collections::HashMap,
    string::FromUtf8Error,
};

const JS_ENV_PATCHES: &str = include_str!("js_env_patches.js");

fn epoch_time_millis() -> u128 {
    use std::time::{
        SystemTime,
        UNIX_EPOCH,
    };

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Valid SystemTime")
        .as_millis()
}

/// Challenge Client
pub struct Client {
    client: hyper::Client<hyper_tls::HttpsConnector<hyper::client::HttpConnector>>,
}

impl Client {
    /// Make a new challenge client
    pub fn new() -> Self {
        let https = hyper_tls::HttpsConnector::new();
        let client = hyper::Client::builder().build::<_, hyper::Body>(https);

        Self { client }
    }

    /// Probe a code
    pub async fn probe_code(&self, code: &str) -> KahootResult<ProbeResult> {
        trace!("probing code '{}'", code);

        let url = format!(
            "https://kahoot.it/reserve/session/{}/?{}",
            code,
            epoch_time_millis()
        );

        let req = hyper::Request::builder()
            .uri(url)
            .header("User-Agent", crate::USER_AGENT_STR)
            .body(hyper::Body::empty())?;

        let res = self.client.request(req).await?;
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

    /// Get the token for a code
    pub async fn get_token(&self, code: &str) -> KahootResult<String> {
        let res = self.probe_code(code).await?;

        #[cfg(debug_assertions)]
        let start = Instant::now();

        let token = tokio::task::spawn_blocking(move || {
            crate::challenge::decode(&res.token, &res.response.challenge)
        })
        .await??;

        #[cfg(debug_assertions)]
        trace!("decoded challenge in {:?}", Instant::now() - start);

        Ok(token)
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
    }
}

/// The probe response
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

#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
    /// Challenge decode error
    #[error("{0}")]
    ChallegeDecode(#[from] SendDuccError),

    /// Boa Challenge decode error
    #[error("boa challenge decode error")]
    ChallegeDecodeBoa(String),

    /// Token deocode error
    #[error("{0}")]
    TokenDecode(#[from] DecodeTokenError),
}

impl From<ducc::Error> for DecodeError {
    fn from(e: ducc::Error) -> Self {
        DecodeError::ChallegeDecode(e.into())
    }
}

/// Sendable DuccError
#[derive(Debug, thiserror::Error)]
pub struct SendDuccError {
    /// Error Kind
    #[source]
    pub kind: SendDuccErrorKind,

    /// Error Context
    pub context: Vec<String>,
}

impl std::fmt::Display for SendDuccError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.kind.fmt(f)
    }
}

impl From<ducc::Error> for SendDuccError {
    fn from(e: ducc::Error) -> Self {
        Self {
            kind: e.kind.into(),
            context: e.context,
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum SendDuccErrorKind {
    #[error("to js conversion error")]
    ToJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    #[error("from js conversion error")]
    FromJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    #[error("runtime error")]
    RuntimeError {
        code: ducc::RuntimeErrorCode,
        name: String,
    },
    #[error("recursive mut callback")]
    RecursiveMutCallback,

    #[error("external error")]
    ExternalError,

    #[error("not a function")]
    NotAFunction,
}

impl From<ducc::ErrorKind> for SendDuccErrorKind {
    fn from(e: ducc::ErrorKind) -> Self {
        match e {
            ducc::ErrorKind::ToJsConversionError { from, to } => {
                SendDuccErrorKind::ToJsConversionError { from, to }
            }
            ducc::ErrorKind::FromJsConversionError { from, to } => {
                SendDuccErrorKind::FromJsConversionError { from, to }
            }
            ducc::ErrorKind::RuntimeError { code, name } => {
                SendDuccErrorKind::RuntimeError { code, name }
            }
            ducc::ErrorKind::RecursiveMutCallback => SendDuccErrorKind::RecursiveMutCallback,
            ducc::ErrorKind::ExternalError(_) => SendDuccErrorKind::ExternalError,
            ducc::ErrorKind::NotAFunction => SendDuccErrorKind::NotAFunction,
        }
    }
}

/// Decode a token
pub fn decode(encoded_token: &str, encoded_challenge: &str) -> Result<String, DecodeError> {
    let decoded_challenge = decode_challenge(encoded_challenge)?;
    let decoded_token = decode_token(encoded_token, &decoded_challenge)?;
    Ok(decoded_token)
}

/// Decode a challenge
pub fn decode_challenge(challenge_str: &str) -> ducc::Result<String> {
    thread_local!(static DUCC: Ducc = Ducc::new());

    DUCC.with(|ducc| {
        ducc.exec(JS_ENV_PATCHES, Some("env_patches.js"), Default::default())?;
        ducc.exec(challenge_str, Some("challenge.js"), Default::default())
    })
}

/// Decode a token with boa
pub fn decode_boa(encoded_token: &str, encoded_challenge: &str) -> Result<String, DecodeError> {
    let decoded_challenge =
        decode_challenge_boa(encoded_challenge).map_err(DecodeError::ChallegeDecodeBoa)?;
    let decoded_token = decode_token(encoded_token, &decoded_challenge)?;
    Ok(decoded_token)
}

/// Decode a challenge with boa. Boa currently cannot do this so this will always fail
pub fn decode_challenge_boa(challenge_str: &str) -> Result<String, String> {
    let mut ctx = boa::Context::new();
    let _patches = ctx.eval(JS_ENV_PATCHES).map_err(|e| {
        e.to_string(&mut ctx)
            .map(|s| s.as_str().to_string())
            .unwrap_or("Missing Error String".into())
    })?;

    let challenge = ctx
        .eval(challenge_str)
        .map_err(|e| {
            e.to_string(&mut ctx)
                .map(|s| s.as_str().to_string())
                .unwrap_or("Missing Error String".into())
        })?
        .to_string(&mut ctx)
        .map(|s| s.as_str().to_string())
        .map_err(|_| "Missing Error String".to_string())?;

    Ok(challenge)
}

/// Decode Token Error
#[derive(Debug, thiserror::Error)]
pub enum DecodeTokenError {
    /// Base64 decode error
    #[error("{0}")]
    Base64(#[from] base64::DecodeError),

    /// Invalid string error
    #[error("{0}")]
    InvalidString(#[from] FromUtf8Error),
}

pub fn decode_token(token: &str, challenge: &str) -> Result<String, DecodeTokenError> {
    let mut raw_token = base64::decode(token)?;
    let challenge_bytes = challenge.as_bytes();
    let challenge_len = challenge_bytes.len();

    for (i, byte) in raw_token.iter_mut().enumerate() {
        *byte ^= challenge_bytes[i % challenge_len];
    }

    Ok(String::from_utf8(raw_token)?)
}

#[cfg(test)]
mod test {
    use super::*;
    use std::time::Instant;

    const SAMPLE_1: (&str, &str) = (
        "UFJ5AUhQO1J9SlcHA3BBYUQCU1xzCFFiPjYyekFDAwZIDzN+ISAIfwIgDVtfUjh2MAAJP0JpXnZjR0QicA5/BlkLQEQCGElMflFDSlkHAUpZa1MODAwHTnhHHg1XaT9+",
        "decode.call(this, \'NlcrzmYQJ6lBmnIQ1OInvpMg3eyRwK6SyxH4jcPbH2YzAMk7p7LYqwpDQgDSACYcRyKrcJ5cq2xhOtR276MTh5V8QHCJndzntSpL\'); function decode(message) {var offset = 75 \u{2003} *\u{2003}\t 47 \u{2003} *\u{2003}\t 32\t\u{2003}+\u{2003}55; if(\u{2003}\t this \u{2003} .\u{2003}angular\u{2003}.\u{2003}\t isDate\u{2003}( \u{2003} offset \u{2003} ))\t\u{2003}console\u{2003}\t . \u{2003} log \u{2003} (\"Offset derived as: {\", offset, \"}\"); return  \u{2003} _\t\u{2003}.\t\u{2003}replace \u{2003} ( message,/./g, function(char, position) {return String.fromCharCode((((char.charCodeAt(0)*position)+ offset ) % 77) + 48);});}",
    );

    #[test]
    fn decode_sample_1() {
        let start = Instant::now();
        assert_eq!(decode(SAMPLE_1.0, SAMPLE_1.1).unwrap(), "2f8648fc7031b16045414732dde566f309a8aa296e2720d7db9a82a7827a7f7d7f854b946e839ef3481140a994d5a8b2");
        let end = Instant::now();
        dbg!(end - start);

        let start = Instant::now();
        assert_eq!(decode(SAMPLE_1.0, SAMPLE_1.1).unwrap(), "2f8648fc7031b16045414732dde566f309a8aa296e2720d7db9a82a7827a7f7d7f854b946e839ef3481140a994d5a8b2");
        let end = Instant::now();
        dbg!(end - start);
    }

    #[test]
    #[ignore]
    fn decode_sample_1_boa() {
        assert_eq!(decode_boa(SAMPLE_1.0, SAMPLE_1.1).unwrap(), "2f8648fc7031b16045414732dde566f309a8aa296e2720d7db9a82a7827a7f7d7f854b946e839ef3481140a994d5a8b2");
    }
}
