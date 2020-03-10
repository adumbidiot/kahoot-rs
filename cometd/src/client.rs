mod context;
mod handler;

pub use self::context::Context;
pub use self::handler::DefaultHandler;
pub use self::handler::Handler;
use crate::packet::Channel;
use crate::transport::WsTransport;
use crate::CometError;
use crate::CometResult;
use tungstenite::error::Error as TError;

pub struct Client<T> {
    ctx: Context,
    transport: WsTransport,

    handler: T,
}

impl Client<DefaultHandler> {
    pub async fn connect(url: &str) -> CometResult<Self> {
        Self::connect_with_handler(url, DefaultHandler).await
    }
}

impl<T: Handler> Client<T> {
    pub async fn connect_with_handler(url: &str, handler: T) -> CometResult<Self> {
        let (stream, _response) = tokio_tungstenite::connect_async(url).await?;
        let transport = WsTransport::new(stream);
        let client = Client {
            ctx: Context::new(transport.clone()),
            transport,

            handler,
        };

        client.ctx.send_handshake().await?;

        Ok(client)
    }

    pub async fn run(&mut self) {
        loop {
            match self.transport.next_packet().await {
                Ok(packets) => {
                    for packet in packets {
                        match packet.channel {
                            Channel::Handshake => {
                                if let (Some(true), Some(client_id)) =
                                    (packet.successful, packet.client_id)
                                {
                                    self.ctx.inner.lock().client_id = Some(client_id);
                                    self.ctx.inner.lock().is_reconnect = true;

                                    match self.ctx.send_connect().await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            self.handler.on_error(self.ctx.clone(), e).await;
                                        }
                                    }
                                } else {
                                    match self.ctx.send_handshake().await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            self.handler.on_error(self.ctx.clone(), e).await;
                                        }
                                    }
                                }
                            }
                            Channel::Connect => {
                                if packet.successful == Some(false) {
                                    match self.ctx.send_handshake().await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            self.handler.on_error(self.ctx.clone(), e).await;
                                        }
                                    }
                                } else {
                                    let mut lock = self.ctx.inner.lock();
                                    if lock.is_reconnect {
                                        lock.is_reconnect = false;
                                        drop(lock);
                                        self.handler.on_reconnect(self.ctx.clone()).await;
                                    }
                                }

                                // TODO: Find out why this has to be here
                                match self.ctx.send_connect().await {
                                    Ok(_) => {}
                                    Err(e) => {
                                        self.handler.on_error(self.ctx.clone(), e).await;
                                    }
                                }
                            }
                            Channel::Subscribe => {
                                // WIP
                                assert_eq!(packet.successful, Some(true));
                            }
                            _ => {
                                self.handler.on_message(self.ctx.clone(), packet).await;
                            }
                        }
                    }
                }
                Err(CometError::ClientExited) => {
                    break;
                }
                Err(CometError::Ws(TError::Io(e)))
                    if e.kind() == std::io::ErrorKind::ConnectionReset =>
                {
                    break;
                }
                Err(e) => {
                    self.handler.on_error(self.ctx.clone(), e).await;
                }
            }
        }
    }

    pub async fn graceful_shutdown(&self) -> CometResult<()> {
        // TODO: Unsub, disconnect packet
        self.transport.graceful_shutdown().await
    }
}
