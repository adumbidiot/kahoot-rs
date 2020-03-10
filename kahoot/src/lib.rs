pub mod challenge;
pub mod client;
pub mod error;
pub mod message;

pub use crate::{
    client::{
        Client,
        Context,
        Handler,
    },
    error::{
        KahootError,
        KahootResult,
    },
    message::Message,
};
pub use async_trait::async_trait;

const USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[derive(Debug)]
pub struct LoginResponse {
    pub error: Option<String>,
    pub description: Option<String>,

    pub cid: Option<String>,
}

impl LoginResponse {
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        if value.get("type")?.as_str()? != "loginResponse" {
            return None;
        }

        let error = value
            .get("error")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let description = value
            .get("description")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let cid = value
            .get("cid")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        Some(Self {
            error,
            description,
            cid,
        })
    }
}
