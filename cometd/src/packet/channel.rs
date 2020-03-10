use serde::Deserialize;
use serde::Serialize;
use std::borrow::Cow;
use std::convert::Infallible;
use std::str::FromStr;

const HANDSHAKE_PATH: &str = "/meta/handshake";
const CONNECT_PATH: &str = "/meta/connect";
const SUBSCRIBE_PATH: &str = "/meta/subscribe";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, Hash)]
#[serde(from = "&str", into = "Cow<'static, str>")]
pub enum Channel {
    Handshake,
    Connect,
    Subscribe,
    Other(String),
}

impl Channel {
    pub fn from_string(s: String) -> Self {
        match s.as_str() {
            HANDSHAKE_PATH => Channel::Handshake,
            CONNECT_PATH => Channel::Connect,
            SUBSCRIBE_PATH => Channel::Subscribe,
            _ => Channel::Other(s),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Channel::Handshake => HANDSHAKE_PATH,
            Channel::Connect => CONNECT_PATH,
            Channel::Subscribe => SUBSCRIBE_PATH,
            Channel::Other(ref s) => s,
        }
    }

    pub fn into_cow(self) -> Cow<'static, str> {
        match self {
            Channel::Handshake => HANDSHAKE_PATH.into(),
            Channel::Connect => CONNECT_PATH.into(),
            Channel::Subscribe => SUBSCRIBE_PATH.into(),
            Channel::Other(s) => s.into(),
        }
    }

    pub fn into_string(self) -> String {
        self.into_cow().into_owned()
    }
}

impl Default for Channel {
    fn default() -> Self {
        Self::Handshake
    }
}

impl FromStr for Channel {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s {
            HANDSHAKE_PATH => Channel::Handshake,
            CONNECT_PATH => Channel::Connect,
            SUBSCRIBE_PATH => Channel::Subscribe,
            _ => Channel::Other(String::from(s)),
        })
    }
}

impl<'a> From<&'a str> for Channel {
    fn from(s: &'a str) -> Self {
        Self::from_str(s).unwrap()
    }
}

impl From<String> for Channel {
    fn from(s: String) -> Self {
        Self::from_string(s)
    }
}

impl Into<Cow<'static, str>> for Channel {
    fn into(self) -> Cow<'static, str> {
        self.into_cow()
    }
}

impl Into<String> for Channel {
    fn into(self) -> String {
        self.into_string()
    }
}
