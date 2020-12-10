pub mod client;
pub mod packet;
mod transport;

pub use crate::client::Client;
pub use async_trait::async_trait;
pub use serde_json::json;

pub type CometResult<T> = Result<T, CometError>;

/// Comet Error
#[derive(Debug, thiserror::Error)]
pub enum CometError {
    /// Websocket Error
    #[error("{0}")]
    Ws(#[from] tungstenite::Error),

    /// Json decode Error
    #[error("{0}")]
    Json(#[from] serde_json::Error),

    /// IO Error
    #[error("{0}")]
    Io(#[from] std::io::Error),

    /// The client is shutting down
    #[error("the client is shutting down")]
    ClientExited,

    /// The client id is missing
    #[error("missing client id")]
    MissingClientId,
}
