use crate::{
    packet::Packet,
    CometError,
    CometResult,
};
use futures::{
    sink::SinkExt,
    stream::{
        SplitSink,
        SplitStream,
        StreamExt,
    },
};
use std::sync::{
    atomic::{
        AtomicU64,
        Ordering,
    },
    Arc,
};
use tokio::{
    io::AsyncWriteExt,
    net::TcpStream,
    sync::Mutex as TokioMutex,
};
use tokio_native_tls::TlsStream;
use tokio_tungstenite::stream::Stream as TStream;
use tungstenite::Message as TMessage;

type WebSocketStream = tokio_tungstenite::WebSocketStream<TStream<TcpStream, TlsStream<TcpStream>>>;

#[derive(Clone)]
pub(crate) struct WsTransport {
    tx: Arc<TokioMutex<Option<SplitSink<WebSocketStream, TMessage>>>>,
    rx: Arc<TokioMutex<Option<SplitStream<WebSocketStream>>>>,
    packet_id: Arc<AtomicU64>,
}

impl WsTransport {
    pub fn new(stream: WebSocketStream) -> Self {
        let (tx, rx) = stream.split();
        let tx = Arc::new(TokioMutex::new(Some(tx)));
        let rx = Arc::new(TokioMutex::new(Some(rx)));

        WsTransport {
            rx,
            tx,
            packet_id: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn send_packet(&self, mut packets: Vec<Packet>) -> CometResult<()> {
        for packet in packets.iter_mut() {
            packet.id = Some(self.get_new_packet_id().to_string());
        }

        let data = serde_json::to_string(&packets).map_err(CometError::Json)?;

        self.tx
            .lock()
            .await
            .as_mut()
            .ok_or(CometError::ClientExited)?
            .send(TMessage::Text(data))
            .await
            .map_err(CometError::Ws)?;

        Ok(())
    }

    pub async fn next_packet(&self) -> CometResult<Vec<Packet>> {
        loop {
            let msg = self
                .rx
                .lock()
                .await
                .as_mut()
                .ok_or(CometError::ClientExited)?
                .next()
                .await
                .ok_or(CometError::ClientExited)?;

            match msg {
                Ok(msg) => match msg {
                    TMessage::Text(txt) => {
                        return serde_json::from_str::<Vec<Packet>>(&txt).map_err(CometError::Json);
                    }
                    TMessage::Close(_frame) => {
                        self.handle_server_shutdown().await?;
                        return Err(CometError::ClientExited);
                    }
                    _ => {}
                },
                Err(e) => return Err(CometError::Ws(e)),
            }
        }
    }

    async fn steal_stream(&self) -> CometResult<WebSocketStream> {
        let tx = self
            .tx
            .lock()
            .await
            .take()
            .ok_or(CometError::ClientExited)?;

        let rx = self
            .rx
            .lock()
            .await
            .take()
            .ok_or(CometError::ClientExited)?;

        let stream = tx.reunite(rx).expect("Valid Reunite");

        Ok(stream)
    }

    pub async fn graceful_shutdown(&self) -> CometResult<()> {
        let mut stream = self.steal_stream().await?;
        stream.close(None).await.map_err(CometError::Ws)?;

        Ok(())
    }

    async fn handle_server_shutdown(&self) -> CometResult<()> {
        self.steal_stream()
            .await?
            .get_mut()
            .shutdown()
            .await
            .map_err(CometError::Io)?;

        Ok(())
    }

    fn get_new_packet_id(&self) -> u64 {
        self.packet_id.fetch_add(1, Ordering::SeqCst)
    }
}
