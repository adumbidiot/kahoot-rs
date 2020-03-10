use serde::{
    Deserialize,
    Serialize,
};
use std::{
    convert::Infallible,
    str::FromStr,
};

const WEBSOCKET_STR: &str = "websocket";
const LONG_POLLING_STR: &str = "long-polling";
const CALLBACK_POLLING_STR: &str = "callback-polling";

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(from = "&str", into = "String")]
pub enum ConnectionType {
    WebSocket,
    LongPolling,
    CallbackPolling,
    Other(String),
}

impl ConnectionType {
    pub fn from_string(s: String) -> Self {
        match s.as_str() {
            WEBSOCKET_STR => Self::WebSocket,
            LONG_POLLING_STR => Self::LongPolling,
            CALLBACK_POLLING_STR => Self::CallbackPolling,
            _ => Self::Other(s),
        }
    }

    pub fn into_string(self) -> String {
        match self {
            Self::WebSocket => String::from(WEBSOCKET_STR),
            Self::LongPolling => String::from(LONG_POLLING_STR),
            Self::CallbackPolling => String::from("callback-polling"),
            Self::Other(s) => s,
        }
    }
}

impl Default for ConnectionType {
    fn default() -> Self {
        Self::WebSocket
    }
}

impl FromStr for ConnectionType {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            WEBSOCKET_STR => Self::WebSocket,
            LONG_POLLING_STR => Self::LongPolling,
            CALLBACK_POLLING_STR => Self::CallbackPolling,
            _ => Self::Other(String::from(s)),
        })
    }
}

impl<'a> From<&'a str> for ConnectionType {
    fn from(s: &'a str) -> Self {
        ConnectionType::from_str(s).unwrap()
    }
}

impl From<String> for ConnectionType {
    fn from(s: String) -> Self {
        ConnectionType::from_string(s)
    }
}

impl Into<String> for ConnectionType {
    fn into(self) -> String {
        self.into_string()
    }
}
