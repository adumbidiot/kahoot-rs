use crate::{
    client::Context,
    error::KahootError,
    message::{
        GetReadyMessage,
        StartQuestionMessage,
    },
};

#[crate::async_trait]
pub trait Handler: Send + Sync {
    async fn on_login(&self, _ctx: Context) {}
    async fn on_get_ready(&self, _ctx: Context, _msg: GetReadyMessage) {}
    async fn on_start_question(&self, _ctx: Context, _msg: StartQuestionMessage) {}

    async fn on_error(&self, _ctx: Context, _e: KahootError) {}
}

pub struct DefaultHandler;

#[crate::async_trait]
impl Handler for DefaultHandler {}
