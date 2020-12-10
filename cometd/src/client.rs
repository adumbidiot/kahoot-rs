mod context;
mod handler;

pub use self::{
    context::Context,
    handler::{
        DefaultHandler,
        Handler,
    },
};
use crate::{
    packet::{
        Channel,
        Packet,
    },
    transport::WsTransport,
    CometError,
    CometResult,
};
use std::sync::Arc;
use tungstenite::error::Error as TError;

/// A cometd client
pub struct Client<T> {
    ctx: Context,
    transport: WsTransport,

    /// The event handler
    pub handler: Arc<T>,
}

impl Client<DefaultHandler> {
    /// Connect to the url with the default handler
    pub async fn connect(url: &str) -> CometResult<Self> {
        Self::connect_with_handler(url, DefaultHandler).await
    }
}

impl<T: Handler + 'static> Client<T> {
    /// Connect to the url with the given handler
    pub async fn connect_with_handler(url: &str, handler: T) -> CometResult<Self> {
        let (stream, _response) = tokio_tungstenite::connect_async(url).await?;
        let transport = WsTransport::new(stream);
        let client = Client {
            ctx: Context::new(transport.clone()),
            transport,

            handler: Arc::new(handler),
        };

        client.ctx.send_handshake().await?;

        Ok(client)
    }

    /// Run client
    pub async fn run(&mut self) -> CometResult<()> {
        loop {
            let next_packet = self.transport.next_packet().await;
            match next_packet {
                Ok(packets) => {
                    self.process_packets(packets).await;
                }
                Err(e) => match e {
                    CometError::ClientExited => {
                        return Ok(());
                    }
                    CometError::Ws(TError::Io(_)) => {
                        return Err(e);
                    }
                    e => {
                        self.handler.on_error(self.ctx.clone(), e).await;
                    }
                },
            }
        }
    }

    async fn process_packets(&mut self, packets: Vec<Packet>) {
        for packet in packets {
            match packet.channel {
                Channel::Handshake => {
                    if let (Some(true), Some(client_id)) = (packet.successful, packet.client_id) {
                        {
                            let mut lock = self.ctx.inner.lock().unwrap();
                            lock.client_id = Some(client_id);
                            lock.is_reconnect = true;
                        }

                        if let Err(e) = self.ctx.queue_connect() {
                            let handler = self.handler.clone();
                            let ctx = self.ctx.clone();

                            tokio::spawn(async move {
                                handler.on_error(ctx, e).await;
                            });
                        }
                    } else {
                        self.ctx.queue_handshake();
                    }
                }
                Channel::Connect => {
                    if packet.successful == Some(false) {
                        self.ctx.queue_handshake();
                    } else {
                        let is_reconnect = {
                            let mut lock = self.ctx.inner.lock().unwrap();
                            if lock.is_reconnect {
                                lock.is_reconnect = false;
                                true
                            } else {
                                false
                            }
                        };

                        if is_reconnect {
                            let handler = self.handler.clone();
                            let ctx = self.ctx.clone();

                            tokio::spawn(async move { handler.on_reconnect(ctx).await });
                        }
                    }

                    // TODO: Find out why this has to be here
                    if let Err(e) = self.ctx.queue_connect() {
                        let handler = self.handler.clone();
                        let ctx = self.ctx.clone();

                        tokio::spawn(async move { handler.on_error(ctx, e).await });
                    }
                }
                Channel::Subscribe => {
                    // TODO: Figure out how to handle failed subscriptions
                    assert_eq!(packet.successful, Some(true));
                }
                _ => {
                    let handler = self.handler.clone();
                    let ctx = self.ctx.clone();

                    tokio::spawn(async move {
                        handler.on_message(ctx, packet).await;
                    });
                }
            }
        }

        if let Err(e) = self.ctx.send_buffered_packets().await {
            let handler = self.handler.clone();
            let ctx = self.ctx.clone();

            tokio::spawn(async move { handler.on_error(ctx, e).await });
        }
    }

    pub async fn graceful_shutdown(&self) -> CometResult<()> {
        // TODO: Unsub, disconnect packet
        self.transport.graceful_shutdown().await
    }
}
