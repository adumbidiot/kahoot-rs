use crate::{
    client::Context,
    packet::Packet,
    CometError,
};

#[crate::async_trait]
pub trait Handler: Send + Sync {
    async fn on_error(&self, _ctx: Context, _error: CometError) {}
    async fn on_reconnect(&self, _ctx: Context) {}
    async fn on_message(&self, _ctx: Context, _packet: Packet) {}
}

pub struct DefaultHandler;
#[crate::async_trait]
impl Handler for DefaultHandler {}
