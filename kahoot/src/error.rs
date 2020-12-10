use cometd::CometError;

/// Kahoot Result
pub type KahootResult<T> = Result<T, KahootError>;

/// Kahoot Error
#[derive(Debug, thiserror::Error)]
pub enum KahootError {
    /// Hyper HTTP Error
    #[error("{0}")]
    Hyper(#[from] hyper::Error),

    /// Invalid http uri
    #[error("{0}")]
    InvalidUrl(#[from] http::uri::InvalidUri),

    /// Invalid Http Status
    #[error("invalid http status {0}")]
    InvalidStatus(http::StatusCode),

    /// Json Error
    #[error("{0}")]
    Json(#[from] serde_json::Error),

    /// Comet Error
    #[error("{0}")]
    Comet(#[from] CometError),

    /// Http Error
    #[error("{0}")]
    Http(#[from] http::Error),

    /// Challenge decode error
    #[error("{0}")]
    ChallengeDecodeError(#[from] crate::challenge::DecodeError),

    /// Invalid Game Code
    #[error("invalid game code")]
    InvalidCode,

    /// Missing Token
    #[error("missing token")]
    MissingToken,

    /// Missing name
    #[error("missing name")]
    MissingName,

    /// Invalid Login
    #[error("invalid login")]
    InvalidLogin(crate::LoginResponse),
}
