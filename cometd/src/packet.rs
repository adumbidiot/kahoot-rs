mod advice;
mod channel;
mod connection_type;

pub use self::advice::Advice;
pub use self::advice::Reconnect;
pub use self::channel::Channel;
pub use self::connection_type::ConnectionType;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Packet {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub advice: Option<Advice>,

    pub channel: Channel,

    #[serde(rename = "clientId", skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,

    #[serde(rename = "connectionType", skip_serializing_if = "Option::is_none")]
    pub connection_type: Option<ConnectionType>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub ext: Option<serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    #[serde(rename = "minimumVersion", skip_serializing_if = "Option::is_none")]
    pub minimum_version: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub subscription: Option<Channel>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub successful: Option<bool>,

    #[serde(
        rename = "supportedConnectionTypes",
        skip_serializing_if = "Option::is_none"
    )]
    pub supported_connection_types: Option<Vec<ConnectionType>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            advice: None,
            channel: Default::default(),
            client_id: None,
            connection_type: None,
            data: None,
            error: None,
            ext: None,
            id: None,
            minimum_version: None,
            subscription: None,
            successful: None,
            supported_connection_types: None,
            version: None,

            extra: Default::default(),
        }
    }

    pub fn advice(mut self, advice: Advice) -> Self {
        self.advice = Some(advice);
        self
    }

    pub fn channel(mut self, channel: Channel) -> Self {
        self.channel = channel;
        self
    }

    pub fn client_id(mut self, id: String) -> Self {
        self.client_id = Some(id);
        self
    }

    pub fn connection_type(mut self, connection_type: ConnectionType) -> Self {
        self.connection_type = Some(connection_type);
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn minimum_version(mut self, minimum_version: String) -> Self {
        self.minimum_version = Some(minimum_version);
        self
    }

    pub fn supported_connection_types(
        mut self,
        supported_connection_types: Vec<ConnectionType>,
    ) -> Self {
        self.supported_connection_types = Some(supported_connection_types);
        self
    }

    pub fn ext(mut self, ext: serde_json::Value) -> Self {
        self.ext = Some(ext);
        self
    }

    pub fn subscription(mut self, channel: Channel) -> Self {
        self.subscription = Some(channel);
        self
    }
}

impl Default for Packet {
    fn default() -> Self {
        Packet::new()
    }
}
