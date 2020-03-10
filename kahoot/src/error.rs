use cometd::CometError;

pub type KahootResult<T> = Result<T, KahootError>;

#[derive(Debug)]
pub enum KahootError {
    Hyper(hyper::Error),
    InvalidUrl(http::uri::InvalidUri),
    InvalidStatus(http::StatusCode),
    Json(serde_json::Error),
    Comet(CometError),
    Http(http::Error),

    ChallengeDecodeError(crate::challenge::DecodeError),
    InvalidCode,
    MissingToken,
    MissingName,
    InvalidLogin(crate::LoginResponse),
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

impl From<cometd::CometError> for KahootError {
    fn from(e: cometd::CometError) -> Self {
        Self::Comet(e)
    }
}

impl From<crate::challenge::DecodeError> for KahootError {
    fn from(e: crate::challenge::DecodeError) -> Self {
        Self::ChallengeDecodeError(e)
    }
}
