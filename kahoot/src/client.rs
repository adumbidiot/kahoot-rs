mod handler;

pub use self::handler::{
    DefaultHandler,
    Handler,
};
use crate::{
    async_trait,
    KahootError,
    KahootResult,
    LoginResponse,
    Message,
    USER_AGENT_STR,
};
use cometd::{
    json,
    packet::Packet,
    CometError,
};
use std::sync::{
    Arc,
    Mutex,
};

pub const DEFAULT_DEVICE_WIDTH: u64 = 1920;
pub const DEFAULT_DEVICE_HEIGHT: u64 = 1080;
pub const DEFAULT_LAG: u64 = 0;
pub const CONTROLLER_CHANNEL: &str = "/service/controller";
pub const PLAYER_CHANNEL: &str = "/service/player";
pub const STATUS_CHANNEL: &str = "/service/status";

pub(crate) struct KahootHandler<T> {
    pub(crate) code: Arc<String>,
    pub(crate) name: Arc<String>,
    pub(crate) handler: Arc<T>,

    pub(crate) exit_error: Arc<Mutex<Option<KahootError>>>,
}

impl<T> KahootHandler<T> {
    pub(crate) fn new(code: String, name: String, handler: T) -> Self {
        Self {
            code: Arc::new(code),
            name: Arc::new(name),
            handler: Arc::new(handler),
            exit_error: Arc::new(Mutex::new(None)),
        }
    }

    pub(crate) fn kahoot_ctx(&self, ctx: &cometd::client::Context) -> Context {
        Context::new(ctx.clone(), self.code.clone(), self.name.clone())
    }
}

#[derive(Clone)]
pub struct Context {
    pub ctx: cometd::client::Context,
    pub code: Arc<String>,
    pub name: Arc<String>,
}

impl Context {
    pub fn new(ctx: cometd::client::Context, code: Arc<String>, name: Arc<String>) -> Self {
        Context { ctx, code, name }
    }

    pub fn get_device_data_str(&self) -> KahootResult<String> {
        let content = json!({
            "device": {
                "userAgent": USER_AGENT_STR,
                "screen": {
                    "width": DEFAULT_DEVICE_WIDTH,
                    "height": DEFAULT_DEVICE_HEIGHT,
                }
            }
        });

        serde_json::to_string(&content).map_err(From::from)
    }

    pub async fn login(&self, name: &str) -> KahootResult<()> {
        let packet = Packet::new()
            .channel(CONTROLLER_CHANNEL.into())
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
                "content": self.get_device_data_str()?,
            }));

        self.ctx.send_packet(packet).await?;

        self.ctx.subscribe(CONTROLLER_CHANNEL).await?;
        self.ctx.subscribe(PLAYER_CHANNEL).await?;
        self.ctx.subscribe(STATUS_CHANNEL).await?;

        Ok(())
    }

    pub async fn submit_answer(&self, choice: usize) -> KahootResult<()> {
        let content = json!({
            "choice": choice,
            "meta": {
                "lag": DEFAULT_LAG,
                "device": self.get_device_data_str()?,
            }
        });

        let packet = Packet::new()
            .channel(CONTROLLER_CHANNEL.into())
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

    pub fn get_username(&self) -> Arc<String> {
        self.name.clone()
    }

    pub async fn shutdown(&self) -> KahootResult<()> {
        self.ctx.shutdown().await?;
        Ok(())
    }
}

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
            CONTROLLER_CHANNEL => {
                let data = match packet.data.as_ref() {
                    Some(d) => d,
                    None => &serde_json::Value::Null,
                };

                if let Some(login_response) = LoginResponse::from_value(data) {
                    if login_response.error.as_ref().is_some() {
                        *self.exit_error.lock().unwrap() =
                            Some(KahootError::InvalidLogin(login_response));
                        ctx.shutdown().await.expect("Shutdown");
                    } else {
                        self.handler.on_login(self.kahoot_ctx(&ctx)).await;
                    }
                } else {
                    // println!("Controller Packet: {:#?}", packet);
                }
            }
            STATUS_CHANNEL => {
                // println!("Status Packet: {:#?}", packet);
            }
            PLAYER_CHANNEL => {
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

pub struct Client<T> {
    client: cometd::Client<KahootHandler<T>>,
}

impl<T: Handler + Send + 'static> Client<T> {
    pub async fn connect_with_handler(
        code: String,
        name: String,
        handler: T,
    ) -> KahootResult<Client<T>> {
        let token = crate::challenge::Client::new().get_token(&code).await?;
        let url = format!("wss://kahoot.it/cometd/{}/{}", &code, token);
        let handler = KahootHandler::new(code, name, handler);
        let client = cometd::Client::connect_with_handler(&url, handler).await?;
        let client = Client { client };

        Ok(client)
    }

    pub async fn run(&mut self) -> KahootResult<()> {
        self.client.run().await;

        if let Some(e) = self.client.handler.exit_error.lock().unwrap().take() {
            return Err(e);
        }

        Ok(())
    }

    pub fn handler(&self) -> Arc<T> {
        self.client.handler.handler.clone()
    }
}
