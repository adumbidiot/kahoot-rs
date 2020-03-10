use crate::{
    packet::{
        Advice,
        Channel,
        ConnectionType,
        Packet,
    },
    transport::WsTransport,
    CometError,
    CometResult,
};
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct Context {
    pub(crate) inner: Arc<Mutex<ContextState>>,
    transport: WsTransport, // TODO: Replace with request buffer?
}

impl Context {
    pub(crate) fn new(transport: WsTransport) -> Self {
        Self {
            inner: Arc::new(Mutex::new(ContextState {
                client_id: None,
                is_reconnect: true,

                request_buffer: Vec::new(),
            })),
            transport,
        }
    }

    pub async fn send_packet(&self, packet: Packet) -> CometResult<()> {
        self.transport.send_packet(vec![packet]).await // TODO: Batch and send
    }

    pub async fn send_buffered_packets(&self) -> CometResult<()> {
        self.transport
            .send_packet(self.inner.lock().request_buffer.drain(..).collect())
            .await
    }

    pub async fn send_handshake(&self) -> CometResult<()> {
        let handshake_packet = Packet::new()
            .channel(Channel::Handshake)
            .version("1.0".to_string())
            .minimum_version("1.0".to_string())
            .supported_connection_types(vec![ConnectionType::WebSocket])
            .advice(Advice::new().timeout(60_000).interval(0));

        self.send_packet(handshake_packet).await
    }

    pub async fn send_connect(&self) -> CometResult<()> {
        let connect_packet = Packet::new()
            .channel(Channel::Connect)
            .client_id(self.get_client_id().ok_or(CometError::MissingClientId)?)
            .advice(Advice::new())
            .connection_type(ConnectionType::WebSocket);

        self.send_packet(connect_packet).await
    }

    pub async fn subscribe(&self, s: &str) -> CometResult<()> {
        let packet = Packet::new()
            .channel(Channel::Subscribe)
            .client_id(self.get_client_id().ok_or(CometError::MissingClientId)?)
            .subscription(s.into());

        self.send_packet(packet).await
    }

    pub async fn shutdown(&self) -> CometResult<()> {
        self.transport.graceful_shutdown().await
    }

    pub fn get_client_id(&self) -> Option<String> {
        self.inner.lock().client_id.as_ref().cloned()
    }
}

pub struct ContextState {
    pub(crate) client_id: Option<String>,
    pub(crate) is_reconnect: bool,

    pub(crate) request_buffer: Vec<Packet>,
}
