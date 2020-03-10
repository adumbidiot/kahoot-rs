use crate::Context;
use crate::KahootError;
use parking_lot::Mutex;
use std::sync::Arc;

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
        Context::new(ctx.clone(), self.code.clone())
    }
}
