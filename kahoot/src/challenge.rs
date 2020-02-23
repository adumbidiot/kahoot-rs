use crate::KahootError;
use crate::KahootResult;
use bytes::Buf;
use ducc::Ducc;
use http::header::HeaderName;
use http::StatusCode;
use serde::Deserialize;
use std::collections::HashMap;
use std::string::FromUtf8Error;

const JS_ENV_PATCHES: &str = include_str!("js_env_patches.js");

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

#[derive(Debug)]
pub enum DecodeError {
    ChallegeDecode(SendDuccError),
    TokenDecode(DecodeTokenError),
}

impl From<DecodeTokenError> for DecodeError {
    fn from(e: DecodeTokenError) -> Self {
        Self::TokenDecode(e)
    }
}

#[derive(Debug)]
pub struct SendDuccError {
    pub kind: SendDuccErrorKind,
    pub context: Vec<String>,
}

impl From<ducc::Error> for SendDuccError {
    fn from(e: ducc::Error) -> Self {
        Self {
            kind: e.kind.into(),
            context: e.context,
        }
    }
}

#[derive(Debug)]
pub enum SendDuccErrorKind {
    ToJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    FromJsConversionError {
        from: &'static str,
        to: &'static str,
    },
    RuntimeError {
        code: ducc::RuntimeErrorCode,
        name: String,
    },
    RecursiveMutCallback,
    ExternalError,
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

pub fn decode(encoded_token: &str, encoded_challenge: &str) -> Result<String, DecodeError> {
    let decoded_challenge = decode_challenge(encoded_challenge)
        .map_err(|e| DecodeError::ChallegeDecode(SendDuccError::from(e)))?;
    let decoded_token = decode_token(encoded_token, &decoded_challenge)?;
    Ok(decoded_token)
}

pub fn decode_challenge(challenge_str: &str) -> ducc::Result<String> {
    let ducc = Ducc::new();
    ducc.exec(JS_ENV_PATCHES, Some("env_patches.js"), Default::default())?;
    ducc.exec(challenge_str, Some("challenge.js"), Default::default())
}

#[derive(Debug)]
pub enum DecodeTokenError {
    Base64(base64::DecodeError),
    InvalidString(FromUtf8Error),
}

pub fn decode_token(token: &str, challenge: &str) -> Result<String, DecodeTokenError> {
    let mut raw_token = base64::decode(token).map_err(DecodeTokenError::Base64)?;
    let challenge_bytes = challenge.as_bytes();
    let challenge_len = challenge_bytes.len();

    for (i, byte) in raw_token.iter_mut().enumerate() {
        *byte ^= challenge_bytes[i % challenge_len];
    }

    String::from_utf8(raw_token).map_err(DecodeTokenError::InvalidString)
}

#[cfg(test)]
mod test {
    use super::*;

    const SAMPLE_1: (&str, &str) = (
        "UFJ5AUhQO1J9SlcHA3BBYUQCU1xzCFFiPjYyekFDAwZIDzN+ISAIfwIgDVtfUjh2MAAJP0JpXnZjR0QicA5/BlkLQEQCGElMflFDSlkHAUpZa1MODAwHTnhHHg1XaT9+",
        "decode.call(this, \'NlcrzmYQJ6lBmnIQ1OInvpMg3eyRwK6SyxH4jcPbH2YzAMk7p7LYqwpDQgDSACYcRyKrcJ5cq2xhOtR276MTh5V8QHCJndzntSpL\'); function decode(message) {var offset = 75 \u{2003} *\u{2003}\t 47 \u{2003} *\u{2003}\t 32\t\u{2003}+\u{2003}55; if(\u{2003}\t this \u{2003} .\u{2003}angular\u{2003}.\u{2003}\t isDate\u{2003}( \u{2003} offset \u{2003} ))\t\u{2003}console\u{2003}\t . \u{2003} log \u{2003} (\"Offset derived as: {\", offset, \"}\"); return  \u{2003} _\t\u{2003}.\t\u{2003}replace \u{2003} ( message,/./g, function(char, position) {return String.fromCharCode((((char.charCodeAt(0)*position)+ offset ) % 77) + 48);});}",
    );

    #[test]
    fn decode_sample_1() {
        assert_eq!(decode(SAMPLE_1.0, SAMPLE_1.1).unwrap(), "2f8648fc7031b16045414732dde566f309a8aa296e2720d7db9a82a7827a7f7d7f854b946e839ef3481140a994d5a8b2");
    }
}
