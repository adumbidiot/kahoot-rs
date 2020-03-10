use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Advice {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub reconnect: Option<Reconnect>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<i64>,

    #[serde(rename = "maxInterval", skip_serializing_if = "Option::is_none")]
    pub max_interval: Option<i64>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Advice {
    pub fn new() -> Self {
        Self {
            timeout: None,
            reconnect: None,
            interval: None,
            max_interval: None,
            extra: Default::default(),
        }
    }

    pub fn timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn interval(mut self, interval: i64) -> Self {
        self.interval = Some(interval);
        self
    }
}

impl Default for Advice {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum Reconnect {
    #[serde(rename = "retry")]
    Retry,
    #[serde(rename = "handshake")]
    Handshake,
    #[serde(rename = "none")]
    None,
}
