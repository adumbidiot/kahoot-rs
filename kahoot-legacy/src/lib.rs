use cometd_client::{
    handler::Handler,
    ClientState,
    Packet,
    RequestBuffer,
    Url,
};
use rand::Rng;
use serde::Deserialize;
use serde_json::json;
use std::collections::HashMap;

const QUIZ_INTRO_ID: u64 = 1;
const QUIZ_ANSWERS_ID: u64 = 2;

pub type KahootResult<T> = Result<T, KahootError>;

#[derive(Debug)]
pub enum KahootError {
    Network,
    InvalidCode,
    InvalidStatus,
    Json,
    ChallengeDecode,
    MissingToken,

    NoConnection,

    Generic(&'static str),
}

#[derive(Debug, Deserialize)]
struct QuizIntroContent {
    #[serde(rename = "quizQuestionAnswers")]
    quiz_question_answers: Vec<u32>,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

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
        rt.block_on(self.client.get_token(code))
            .map_err(|_e| KahootError::Generic("Error"))
    }
}

#[derive(Debug, Deserialize)]
struct QuestionAnswersContent {
    #[serde(rename = "questionIndex")]
    question_index: u32,

    #[serde(rename = "quizQuestionAnswers")]
    quiz_question_answers: Vec<u32>,

    #[serde(flatten)]
    extra: HashMap<String, serde_json::Value>,
}

struct KahootHandler {
    code: String,
    name: String,
    is_first_connect: bool,
}

impl KahootHandler {
    fn new(code: &str, name: String) -> Self {
        KahootHandler {
            code: String::from(code),
            is_first_connect: true,
            name,
        }
    }
}

impl Handler for KahootHandler {
    fn on_connect(&mut self, state: &ClientState, request_buffer: &mut RequestBuffer) {
        if self.is_first_connect {
            self.is_first_connect = false;
            let p = Packet::new()
                .channel("/service/controller".into())
                .client_id(state.get_client_id().unwrap().to_string())
                .data(json!({
                    "type": "login",
                    "gameid": self.code,
                    "host": "kahoot.it",
                    "name": self.name,
                }));
            request_buffer.push_packet(p);
            println!("Sent Login");
        }
    }

    fn on_unknown(&mut self, p: &Packet, state: &ClientState, request_buffer: &mut RequestBuffer) {
        //{\"questionIndex\":0,\"gameBlockType\":\"quiz\",\"quizQuestionAnswers\":[4]}

        match p.channel.as_str() {
            "/service/controller" => {
                //Do nothing for now
                //Remove noise
            }
            "/service/status" => {
                //Do nothing for now
                //Remove noise
            }
            "/service/player" => {
                if let Some(data) = &p.data {
                    match data.get("id").and_then(|id| id.as_u64()) {
                        Some(QUIZ_INTRO_ID) => {
                            if let Some(intro) = data
                                .get("content")
                                .and_then(|data| data.as_str())
                                .and_then(|s| serde_json::from_str::<QuizIntroContent>(&s).ok())
                            {
                                //println!("Question Intro Data: {:#?}", intro);
                            }
                        }
                        Some(QUIZ_ANSWERS_ID) => {
                            if let Some(data) = data
                                .get("content")
                                .and_then(|data| data.as_str())
                                .and_then(|s| {
                                    serde_json::from_str::<QuestionAnswersContent>(&s).ok()
                                })
                            {
                                let choice = rand::thread_rng().gen_range(
                                    0,
                                    data.quiz_question_answers[data.question_index as usize],
                                );

                                let content = json!({
                                    "choice": choice, //index of answer
                                    "meta": {
                                        "lag": 23,
                                        "device": {
                                            "userAgent": "reqwest",
                                            "screen" : {
                                                "width": 1920,
                                                "height": 1080,
                                            }
                                        }
                                    }
                                });

                                let content_str = serde_json::to_string(&content).unwrap();
                                let p = Packet::new()
                                    .channel("/service/controller".into())
                                    .client_id(state.get_client_id().unwrap().to_string())
                                    .data(json!({
                                        "content": content_str,
                                        "gameid": self.code,
                                        "host": "kahoot.it",
                                        "id": 45,
                                        "type": "message",
                                    }));

                                request_buffer.push_packet(p);
                            }
                        }
                        Some(id) => {
                            //println!("Id: {}\nData: {:#?}", id, data);
                        }
                        None => {
                            println!("Got data from '/service/player': {:#?}", data);
                        }
                    }
                }
            }
            _ => {
                println!("Unknown Packet: {:#?}", p);
            }
        }
    }
}

pub struct Connection {
    client: cometd_client::Client<KahootHandler>,
}

impl Connection {
    pub fn connect(settings: JoinSettings) -> KahootResult<Self> {
        let mut url = Url::parse("wss://kahoot.it/cometd").unwrap();

        url.path_segments_mut()
            .unwrap()
            .push(settings.code)
            .push(settings.token);

        let client = cometd_client::Client::connect_with_handler(
            url,
            KahootHandler::new(settings.code, settings.name),
        )
        .map_err(|_| KahootError::Generic("Failed to connect"))?;

        Ok(Connection { client })
    }

    pub fn update(&mut self) {
        self.client.update().unwrap();
    }
}

pub struct Client {
    client: Option<Connection>,
}

impl Client {
    pub fn new() -> Self {
        Client { client: None }
    }

    pub fn join(&mut self, settings: JoinSettings) -> KahootResult<()> {
        self.client = Some(Connection::connect(settings)?);
        Ok(())
    }

    pub fn update(&mut self) {
        if let Some(c) = self.client.as_mut() {
            c.update();
        }
    }
}

pub struct JoinSettings<'a> {
    pub code: &'a str,
    pub token: &'a str,
    pub name: String,
}
