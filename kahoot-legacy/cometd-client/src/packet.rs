use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Packet {
    channel: String,
	
	#[serde(rename = "clientId")]
    client_id: Option<String>,
	
	 #[serde(rename = "connectionType")]
    connection_type: Option<String>,
	
    version: Option<String>,

    #[serde(rename = "minimumVersion")]
    minimum_version: Option<String>,

    #[serde(rename = "supportedConnectionTypes")]
    supported_connection_types: Option<Vec<String>>, // ["websocket", "long-polling"]

    successful: Option<bool>,

    data: Option<serde_json::Value>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Packet {
    pub fn new() -> Self {
        Packet {
            channel: String::from("/meta/handshake"),
            version: None,
            minimum_version: None,
            supported_connection_types: None,
            client_id: None,
            connection_type: None,
            successful: None,
            data: None,
            extra: Default::default(),
        }
    }

    pub fn channel(mut self, channel: String) -> Self {
        self.channel = channel;
        self
    }

    pub fn get_channel(&self) -> Channel {
        Channel::from_str(&self.channel)
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    pub fn minimum_version(mut self, minimum_version: String) -> Self {
        self.minimum_version = Some(minimum_version);
        self
    }

    pub fn supported_connection_types(mut self, supported_connection_types: Vec<String>) -> Self {
        self.supported_connection_types = Some(supported_connection_types);
        self
    }

    pub fn client_id(mut self, id: String) -> Self {
        self.client_id = Some(id);
        self
    }

    pub fn get_client_id(&self) -> Option<&str> {
        self.client_id.as_ref().map(|s| s.as_str())
    }

    pub fn connection_type(mut self, c: String) -> Self {
        self.connection_type = Some(c);
        self
    }

    pub fn data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }

    pub fn get_data(&self) -> Option<&serde_json::Value> {
        self.data.as_ref()
    }
}

//TODO: Write custom deser impl so packets dont fuck around with Strings so much
#[derive(Debug, PartialEq)]
pub enum Channel {
    Handshake,
    Connect,
    Other(String),
}

impl Channel {
    pub fn from_str(s: &str) -> Self {
        match s {
            "/meta/handshake" => Channel::Handshake,
            "/meta/connect" => Channel::Connect,
            _ => Channel::Other(String::from(s)),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Channel::Handshake => "/meta/handshake",
            Channel::Connect => "/meta/connect",
            Channel::Other(ref s) => s,
        }
    }
}
