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
use std::sync::{
    Arc,
    Mutex,
};

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

    /// Send a single packet. Inefficient.
    pub async fn send_packet(&self, packet: Packet) -> CometResult<()> {
        self.transport.send_packet(vec![packet]).await // TODO: Batch and send?
    }

    /// Queue a single packet. Will not send immediately until the next call to `send_buffered_packets`.
    pub fn queue_packet(&self, packet: Packet) {
        self.inner.lock().unwrap().request_buffer.push(packet);
    }

    /// Send buffered packets
    pub async fn send_buffered_packets(&self) -> CometResult<()> {
        let packets = {
            let mut lock = self.inner.lock().unwrap();
            let request_buffer = &mut lock.request_buffer;

            if request_buffer.is_empty() {
                return Ok(());
            }

            std::mem::take(request_buffer)
        };
        self.transport.send_packet(packets).await
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

    /// Queue a handshake packet
    pub fn queue_handshake(&self) {
        let handshake_packet = Packet::new()
            .channel(Channel::Handshake)
            .version("1.0".to_string())
            .minimum_version("1.0".to_string())
            .supported_connection_types(vec![ConnectionType::WebSocket])
            .advice(Advice::new().timeout(60_000).interval(0));

        self.queue_packet(handshake_packet)
    }

    pub async fn send_connect(&self) -> CometResult<()> {
        let client_id = self.get_client_id().ok_or(CometError::MissingClientId)?;
        let connect_packet = Packet::new()
            .channel(Channel::Connect)
            .client_id(client_id)
            .advice(Advice::new())
            .connection_type(ConnectionType::WebSocket);

        self.send_packet(connect_packet).await
    }

    /// Queue a connect packet
    pub fn queue_connect(&self) -> CometResult<()> {
        let client_id = self.get_client_id().ok_or(CometError::MissingClientId)?;
        let connect_packet = Packet::new()
            .channel(Channel::Connect)
            .client_id(client_id)
            .advice(Advice::new())
            .connection_type(ConnectionType::WebSocket);

        self.queue_packet(connect_packet);

        Ok(())
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
        self.inner.lock().unwrap().client_id.as_ref().cloned()
    }
}

pub struct ContextState {
    pub(crate) client_id: Option<String>,
    pub(crate) is_reconnect: bool,

    pub(crate) request_buffer: Vec<Packet>,
}
