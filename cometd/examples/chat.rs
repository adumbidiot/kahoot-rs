use cometd::{
    async_trait,
    client::Context,
    json,
    packet::Packet,
    CometError,
};

const DEFAULT_URL: &str = "ws://localhost:8080/cometd";
const DEFAULT_NAME: &str = "rob";

pub struct ChatHandler;

#[async_trait]
impl cometd::client::Handler for ChatHandler {
    async fn on_reconnect(&self, ctx: Context) {
        ctx.subscribe("/chat/demo").await.unwrap();
        ctx.subscribe("/members/demo").await.unwrap();

        let packet = Packet::new()
            .channel("/chat/demo".into())
            .client_id(ctx.get_client_id().unwrap())
            .data(json!({
                "user": DEFAULT_NAME,
                "membership": "join",
                "chat": format!("{} has joined", DEFAULT_NAME),
            }));

        ctx.send_packet(packet).await.unwrap();

        let packet = Packet::new()
            .channel("/service/members".into())
            .client_id(ctx.get_client_id().unwrap())
            .data(json!({
                "user": DEFAULT_NAME,
                "room": "/chat/demo",
            }));

        ctx.send_packet(packet).await.unwrap();

        dbg!("Reconnect");
    }

    async fn on_error(&self, _ctx: Context, error: CometError) {
        dbg!(error);
    }

    async fn on_message(&self, _ctx: Context, packet: Packet) {
        dbg!(packet);
    }
}

#[tokio::main(threaded_scheduler)]
async fn main() {
    let mut client = cometd::Client::connect_with_handler(DEFAULT_URL, ChatHandler)
        .await
        .unwrap();

    client.run().await;
}
