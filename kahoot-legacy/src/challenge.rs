use crate::error::{
    KahootError,
    KahootResult,
};

pub struct ChallengeClient {
    client: kahoot::challenge::Client,
}

impl ChallengeClient {
    pub fn new() -> Self {
        ChallengeClient {
            client: kahoot::challenge::Client::new(),
        }
    }

    pub fn get_challenge(&self, code: &str) -> KahootResult<String> {
		let mut rt = tokio::runtime::Runtime::new().expect("Valid Runtime");
        rt.block_on(self.client.get_token(code)).map_err(|_e| KahootError::Generic("Error"))
    }
}
