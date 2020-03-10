pub mod client;
pub mod packet;
mod transport;

pub use crate::client::Client;
pub use async_trait::async_trait;
pub use serde_json::json;

pub type CometResult<T> = Result<T, CometError>;

#[derive(Debug)]
pub enum CometError {
    Ws(tungstenite::error::Error),
    Json(serde_json::Error),
    Io(std::io::Error),

    ClientExited,
    MissingClientId,
}

impl From<tungstenite::error::Error> for CometError {
    fn from(e: tungstenite::error::Error) -> Self {
        Self::Ws(e)
    }
}
