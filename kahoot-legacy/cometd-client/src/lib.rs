//https://docs.cometd.org/current/reference/

pub mod handler;
pub mod packet;

use crate::handler::{Handler, NoopHandler};
pub use packet::{Channel, Packet};
use std::{
    collections::VecDeque,
    io::ErrorKind,
    time::{Duration, Instant},
};
pub use websocket::client::builder::Url;
use websocket::{
    client::sync::Client as WebSocketClient,
    message::{Message, OwnedMessage},
    result::WebSocketError,
    stream::sync::NetworkStream,
    ClientBuilder,
};

pub type CometResult<T> = Result<T, CometError>;

#[derive(Debug)]
pub enum CometError {
    MissingClientId,

    Block,

    Io(std::io::Error),

    Generic(&'static str),
}

#[derive(Debug, PartialEq)]
pub enum State {
    Disconnected,
    Connecting,
    Connected,
}

pub struct RequestBuffer {
    buf: VecDeque<Packet>,
}

impl RequestBuffer {
    pub fn new() -> Self {
        RequestBuffer {
            buf: VecDeque::new(),
        }
    }

    pub fn push_packet(&mut self, p: Packet) {
        self.buf.push_back(p);
    }

    pub fn get_packet(&mut self) -> Option<Packet> {
        self.buf.pop_front()
    }
}

pub struct ClientState {
    state: State,
    client_id: Option<String>,
    last_ping: Option<Instant>,
    sent_connect: bool,
}

impl ClientState {
    pub fn new() -> Self {
        ClientState {
            state: State::Disconnected,
            client_id: None,
            last_ping: None,
            sent_connect: false,
        }
    }

    pub fn get_client_id(&self) -> Option<&str> {
        self.client_id.as_ref().map(|s| s.as_str())
    }
}

pub struct Client<T: Handler> {
    client: WebSocketClient<Box<dyn NetworkStream + Send>>,

    state: ClientState,

    request_buffer: RequestBuffer,
    handler: T,
}

impl Client<NoopHandler> {
    pub fn connect(url: Url) -> CometResult<Self> {
        Self::connect_with_handler(url, NoopHandler)
    }
}

impl<T> Client<T>
where
    T: Handler,
{
    pub fn connect_with_handler(url: Url, handler: T) -> CometResult<Self> {
        let client = ClientBuilder::from_url(&url)
            .connect(None)
            .map_err(|_| CometError::Generic("Failed to connect"))?;

        client
            .set_nonblocking(true)
            .map_err(|_| CometError::Generic("Failed to enable nonblocking mode"))?;

        Ok(Client {
            client,

            state: ClientState::new(),

            request_buffer: RequestBuffer::new(),
            handler,
        })
    }

    pub fn send_handshake(&mut self) {
        let p = Packet::new()
            .channel(String::from("/meta/handshake"))
            .version("1.0".to_string())
            .minimum_version("1.0".to_string())
            .supported_connection_types(vec!["websocket".to_string()]);
        self.request_buffer.push_packet(p);
    }

    pub fn send_connect(&mut self) -> CometResult<()> {
        let p = Packet::new()
            .channel(String::from("/meta/connect"))
            .client_id(
                self.state
                    .client_id
                    .as_ref()
                    .ok_or(CometError::MissingClientId)?
                    .to_string(),
            )
            .connection_type(String::from("websocket"));
        self.request_buffer.push_packet(p);
        Ok(())
    }

    fn process_packet(&mut self, p: &Packet) {
        match p.get_channel() {
            Channel::Handshake => {
                self.handler
                    .on_handshake(&self.state, &mut self.request_buffer);
                if let Some(s) = p.get_client_id() {
                    self.state.state = State::Connected;
                    self.state.client_id = Some(s.to_string());
                } else {
                    self.state.state = State::Disconnected;
                }
            }
            Channel::Connect => {
                self.handler
                    .on_connect(&self.state, &mut self.request_buffer);
                self.state.last_ping = Some(Instant::now());
                self.state.sent_connect = false;
            }
            _ => self
                .handler
                .on_unknown(&p, &self.state, &mut self.request_buffer),
        }
    }

    fn process_incoming(&mut self) -> CometResult<()> {
        loop {
            match self.client.recv_message() {
                Ok(data) => match data {
                    OwnedMessage::Text(t) => {
                        match serde_json::from_str::<Vec<Packet>>(&t) {
                            Ok(packets) => {
                                for p in packets.iter() {
                                    if !self.handler.on_packet(&p) {
                                        self.process_packet(p);
                                    }
                                }
                            }
                            Err(_) => {
                                //TODO: Handle invalid packets with handler?
                            }
                        };
                    }
                    _ => {
                        //TODO: Handle non text packet?
                        //println!("Got: {:#?}", data)
                    }
                },
                Err(WebSocketError::IoError(e)) => match e.kind() {
                    ErrorKind::WouldBlock => {
                        break;
                    }
                    _ => {
                        return Err(CometError::Io(e));
                    }
                },
                Err(e) => panic!("{:#?}", e),
            }
        }

        Ok(())
    }

    fn process_request_buffer(&mut self) -> CometResult<()> {
        while let Some(packet) = self.request_buffer.get_packet() {
            let data = serde_json::to_string(&[packet]) //TODO: Can this be optimized? send multi packets at once?
                .map_err(|_| CometError::Generic("Failed to serialize packet"))?;

            self.client
                .send_message(&Message::text(data))
                .map_err(|_| CometError::Generic("Failed to send packet"))?;
        }

        Ok(())
    }

    pub fn update(&mut self) -> CometResult<()> {
        self.process_incoming()?;

        match self.state.state {
            State::Disconnected => {
                self.send_handshake();
                self.state.state = State::Connecting;
            }
            State::Connected => {
                if self
                    .state
                    .last_ping
                    .map(|ping| ping.elapsed() > Duration::from_secs(6))
                    .unwrap_or(true)
                    && !self.state.sent_connect
                {
                    self.send_connect()?;
                    self.state.sent_connect = true;
                }
            }
            _ => (),
        }

        self.process_request_buffer()?;

        Ok(())
    }
}
