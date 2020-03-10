use kahoot::challenge::Client as ChallengeClient;
use std::io::{
    stdin,
    stdout,
    Write,
};

use lazy_static::lazy_static;
use rand::Rng;
use std::sync::Mutex;

lazy_static! {
    pub static ref KAHOOT_RT: Mutex<tokio::runtime::Runtime> =
        { Mutex::new(tokio::runtime::Runtime::new().unwrap()) };
}

pub type KahootResult<T> = Result<T, KahootError>;

#[derive(Debug)]
pub enum KahootError {
    Network,
}

pub struct Client {
    client: Option<kahoot::Client<BotHandler>>,
}

impl Client {
    pub fn new() -> Self {
        Client { client: None }
    }

    pub fn join(&mut self, settings: JoinSettings) -> KahootResult<()> {
        self.client = Some(
            KAHOOT_RT
                .lock()
                .unwrap()
                .block_on(kahoot::Client::connect_with_handler(
                    settings.code.into(),
                    settings.name,
                    BotHandler,
                ))
                .unwrap(),
        );
        Ok(())
    }

    pub fn update(&mut self) {
        // asa
    }
}

pub struct JoinSettings<'a> {
    pub code: &'a str,
    pub token: &'a str,
    pub name: String,
}

struct BotHandler;

#[kahoot::async_trait]
impl kahoot::Handler for BotHandler {
    async fn on_login(&self, ctx: kahoot::Context) {
        println!("Logged in as: {}", ctx.get_username());
    }

    async fn on_get_ready(&self, _ctx: kahoot::Context, msg: kahoot::message::GetReadyMessage) {
        dbg!(msg);
    }

    async fn on_start_question(
        &self,
        ctx: kahoot::Context,
        msg: kahoot::message::StartQuestionMessage,
    ) {
        tokio::time::delay_for(std::time::Duration::from_millis(250)).await; // Needed or kahoot thinks you were too fast

        let choice = rand::thread_rng().gen_range(0, msg.quiz_question_answers[msg.question_index]);
        ctx.submit_answer(choice).await.unwrap();
    }

    async fn on_error(&self, _ctx: kahoot::Context, error: kahoot::KahootError) {
        dbg!(error);
    }
}

fn add_client(
    challenge_client: &ChallengeClient,
    clients: &mut Vec<Client>,
    code: &str,
    name: String,
) {
    let mut client = Client::new();
    let token = KAHOOT_RT
        .lock()
        .unwrap()
        .block_on(challenge_client.get_token(code))
        .unwrap();
    client
        .join(JoinSettings {
            code,
            token: &token,
            name,
        })
        .unwrap();
    clients.push(client);
}

fn read_line() -> String {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    String::from(s.trim())
}

fn main() {
    //let max_clients = 1;
    //let base_name = String::from("the_zucc");

    let challenge_client = ChallengeClient::new();
    let should_yield = true;

    let code = loop {
        print!("Code: ");
        let _ = stdout().flush();
        let code = read_line();

        match KAHOOT_RT
            .lock()
            .unwrap()
            .block_on(challenge_client.get_token(&code))
        {
            Ok(_challenge) => {
                break code;
            }
            Err(e) => {
                println!("Failed to get challenge for {}.", code);
                println!("Got error: {:#?}", e);
                println!();
            }
        }
    };

    let max_clients = loop {
        print!("Max_Clients: ");
        let _ = stdout().flush();

        match read_line().parse::<usize>() {
            Ok(max_clients) => break max_clients,
            Err(e) => {
                println!("Invalid value.");
                println!("Got Error: {:#?}", e);
                println!();
            }
        }
    };

    print!("Base Name: ");
    let _ = stdout().flush();
    let base_name = read_line();

    let mut clients: Vec<Client> = Vec::new();
    loop {
        for client in clients.iter_mut() {
            client.update();
        }

        if clients.len() < max_clients {
            let name = format!("{}{}", &base_name, clients.len());
            println!("Adding client {}", &name);
            add_client(&challenge_client, &mut clients, &code, name);
        }

        if should_yield {
            std::thread::yield_now();
        }
    }
}
