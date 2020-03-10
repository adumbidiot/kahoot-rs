pub mod challenge;
pub mod client;
pub mod error;
pub mod message;

use crate::client::KahootHandler;
pub use crate::error::KahootError;
pub use crate::error::KahootResult;
use crate::message::GetReadyMessage;
pub use crate::message::Message;
use crate::message::StartQuestionMessage;
pub use async_trait::async_trait;
use cometd::json;
use cometd::packet::Packet;
use cometd::CometError;
use std::sync::Arc;

const USER_AGENT_STR: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36";

#[async_trait]
impl<T: Handler + 'static> cometd::client::Handler for KahootHandler<T> {
    async fn on_reconnect(&self, ctx: cometd::client::Context) {
        let ctx = self.kahoot_ctx(&ctx);

        let handler = self.handler.clone();
        let name = self.name.clone();
        tokio::spawn(async move {
            // Needs to satisfy 500 < x < 1000. Upper bound is optional (within reason).
            tokio::time::delay_for(std::time::Duration::from_millis(1000)).await;
            match ctx.login(&name).await {
                Ok(_) => {}
                Err(e) => handler.on_error(ctx, e).await,
            }
        });
    }

    async fn on_message(&self, ctx: cometd::client::Context, packet: Packet) {
        match packet.channel.as_str() {
            "/service/controller" => {
                let data = match packet.data.as_ref() {
                    Some(d) => d,
                    None => &serde_json::Value::Null,
                };

                if let Some(login_response) = LoginResponse::from_value(data) {
                    if login_response.error.as_ref().is_some() {
                        *self.exit_error.lock() = Some(KahootError::InvalidLogin(login_response));
                        ctx.shutdown().await.expect("Shutdown");
                    } else {
                        self.handler.on_login(self.kahoot_ctx(&ctx)).await;
                    }
                } else {
                    // println!("Controller Packet: {:#?}", packet);
                }
            }
            "/service/status" => {
                // println!("Status Packet: {:#?}", packet);
            }
            "/service/player" => {
                let data = match packet.data {
                    Some(d) => d,
                    None => serde_json::Value::Null,
                };

                match Message::from_value(data) {
                    Message::GetReady { msg } => {
                        self.handler.on_get_ready(self.kahoot_ctx(&ctx), msg).await;
                    }
                    Message::StartQuestion { msg } => {
                        self.handler
                            .on_start_question(self.kahoot_ctx(&ctx), msg)
                            .await;
                    }
                    _msg => {
                        // dbg!(msg);
                    }
                }
            }
            _ => {
                // dbg!("Unknown Packet: ", packet);
            }
        }
    }

    async fn on_error(&self, ctx: cometd::client::Context, error: CometError) {
        self.handler.on_error(self.kahoot_ctx(&ctx), error.into());
    }
}

#[derive(Clone)]
pub struct Context {
    pub ctx: cometd::client::Context,
    pub code: Arc<String>,
}

impl Context {
    pub fn new(ctx: cometd::client::Context, code: Arc<String>) -> Self {
        Context { ctx, code }
    }

    pub async fn login(&self, name: &str) -> KahootResult<()> {
        let content = json!({
            "device": {
                "userAgent": USER_AGENT_STR,
                "screen": {
                    "width": 1920,
                    "height": 1080,
                }
            }
        });

        let packet = Packet::new()
            .channel("/service/controller".into())
            .client_id(
                self.ctx
                    .get_client_id()
                    .ok_or(KahootError::Comet(CometError::MissingClientId))?,
            )
            .data(json!({
                "type": "login",
                "gameid": self.code.as_str(),
                "host": "kahoot.it",
                "name": name,
                "content": serde_json::to_string(&content)?,
            }));

        self.ctx.send_packet(packet).await?;

        self.ctx.subscribe("/service/controller").await?;
        self.ctx.subscribe("/service/player").await?;
        self.ctx.subscribe("/service/status").await?;

        Ok(())
    }

    pub async fn submit_answer(&self, choice: usize) -> KahootResult<()> {
        let content = json!({
            "choice": choice,
            "meta": {
                "lag": 23,
                "device": {
                    "userAgent": USER_AGENT_STR,
                    "screen" : {
                        "width": 1920,
                        "height": 1080,
                    }
                }
            }
        });

        let packet = Packet::new()
            .channel("/service/controller".into())
            .client_id(
                self.ctx
                    .get_client_id()
                    .ok_or(KahootError::Comet(CometError::MissingClientId))?,
            )
            .data(json!({
                "content": serde_json::to_string(&content)?,
                "gameid": self.code.as_str(),
                "host": "kahoot.it",
                "id": 45,
                "type": "message",
            }));

        self.ctx.send_packet(packet).await?;

        Ok(())
    }
}

#[async_trait]
pub trait Handler: Send + Sync {
    async fn on_login(&self, _ctx: Context) {}
    async fn on_get_ready(&self, _ctx: Context, _msg: GetReadyMessage) {}
    async fn on_start_question(&self, _ctx: Context, _msg: StartQuestionMessage) {}

    async fn on_error(&self, _ctx: Context, _e: KahootError) {}
}

pub struct Client<T> {
    client: cometd::Client<KahootHandler<T>>,
}

impl<T: Handler + 'static> Client<T> {
    pub async fn connect_with_handler(
        code: String,
        name: String,
        handler: T,
    ) -> KahootResult<Self> {
        let token = crate::challenge::Client::new().get_token(&code).await?;
        let url = format!("wss://kahoot.it/cometd/{}/{}", &code, token);
        let handler = KahootHandler::new(code, name, handler);
        let client = cometd::Client::connect_with_handler(&url, handler).await?;
        let client = Client { client };

        Ok(client)
    }

    pub async fn run(&mut self) -> KahootResult<()> {
        self.client.run().await;

        if let Some(e) = self.client.handler.exit_error.lock().take() {
            return Err(e);
        }

        Ok(())
    }
}

#[derive(Debug)]
pub struct LoginResponse {
    pub error: Option<String>,
    pub description: Option<String>,
    pub cid: Option<String>,
}

impl LoginResponse {
    pub fn from_value(value: &serde_json::Value) -> Option<Self> {
        if value.get("type")?.as_str()? != "loginResponse" {
            return None;
        }

        let error = value
            .get("error")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let description = value
            .get("description")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        let cid = value
            .get("cid")
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());

        Some(Self {
            error,
            description,
            cid,
        })
    }
}
