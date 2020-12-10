use kahoot::KahootResult;
use rand::Rng;
use std::{
    io::{
        stdin,
        stdout,
        Write,
    },
    sync::{
        atomic::{
            AtomicU64,
            Ordering,
        },
        Arc,
    },
};
use tokio::sync::mpsc::{
    UnboundedReceiver,
    UnboundedSender,
};

use std::sync::Mutex;

struct BotHandler {
    id: u64,
    tx: UnboundedSender<TaskMessage>,
}

#[kahoot::async_trait]
impl kahoot::Handler for BotHandler {
    async fn on_login(&self, ctx: kahoot::Context) {
        let _ = self.tx.send(TaskMessage {
            id: self.id,
            data: TaskMessageData::Login {
                name: ctx.get_username().to_string(),
            },
        });
    }

    async fn on_start_question(
        &self,
        ctx: kahoot::Context,
        msg: kahoot::message::StartQuestionMessage,
    ) {
        let choice = rand::thread_rng().gen_range(0, msg.quiz_question_answers[msg.question_index]);
        println!("Client {} submitting answer...", self.id);
        tokio::time::delay_for(std::time::Duration::from_millis(
            250 + ((self.id as f32 / 100.0) * 1000.0) as u64, // TODO: Can we go faster here?
        ))
        .await; // Needed or kahoot thinks you were too fast
        match ctx.submit_answer(choice).await {
            Ok(_) => {}
            Err(e) => {
                self.on_error(ctx.clone(), e).await;
            }
        }
    }

    async fn on_error(&self, _ctx: kahoot::Context, e: kahoot::KahootError) {
        println!("Error: {}", e);
    }
}

pub struct TaskMessage {
    id: u64,
    data: TaskMessageData,
}

pub enum TaskMessageData {
    Login { name: String },
    Exit(KahootResult<()>),
}

#[derive(Clone)]
pub struct Swarm {
    code: String,
    base_name: String,

    task_tx: UnboundedSender<TaskMessage>,
    rx: Arc<Mutex<UnboundedReceiver<TaskMessage>>>,

    num_workers: Arc<AtomicU64>,
}

impl Swarm {
    pub fn new(code: String, base_name: String) -> Self {
        let (task_tx, rx) = tokio::sync::mpsc::unbounded_channel();
        Self {
            code,
            base_name,
            task_tx,
            rx: Arc::new(Mutex::new(rx)),
            num_workers: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn add_n_workers(&self, n: usize) -> KahootResult<()> {
        // TODO: Optimize
        let mut futures: Vec<_> = (0..n as u64)
            .map(|_i| Box::pin(async move { self.add_worker().await }))
            .collect();

        for chunk in futures.chunks_mut(10) {
            futures::future::join_all(chunk).await;
        }

        Ok(())
    }

    pub async fn add_worker(&self) -> KahootResult<()> {
        let id = self.num_workers.fetch_add(1, Ordering::SeqCst);
        self.add_worker_with_id(id).await?;
        Ok(())
    }

    async fn add_worker_with_id(&self, id: u64) -> KahootResult<()> {
        let tx = self.task_tx.clone();

        let mut client = loop {
            let res = kahoot::Client::connect_with_handler(
                self.code.clone(),
                format!("{}{}", self.base_name, id),
                BotHandler { id, tx: tx.clone() },
            )
            .await;

            match res {
                Ok(client) => break client,
                Err(e) => {
                    eprintln!("Failed to join, got error: {:#?}", e);
                }
            }
        };

        tokio::spawn(async move {
            let res = client.run().await;
            let _ = tx.send(TaskMessage {
                id: client.handler().id,
                data: TaskMessageData::Exit(res),
            });
        });

        Ok(())
    }

    pub async fn run(&self) {
        while let Some(msg) = self.rx.lock().unwrap().recv().await {
            match &msg.data {
                TaskMessageData::Login { name } => {
                    println!("Worker #{} logged in as {}", msg.id, name);
                }
                TaskMessageData::Exit(_res) => loop {
                    match self.add_worker_with_id(msg.id).await {
                        Ok(_) => break,
                        Err(e) => {
                            eprintln!("Error readding dead worker: {:#?}", e);
                        }
                    }
                },
            }
        }
    }
}

fn read_line() -> String {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    String::from(s.trim())
}

#[tokio::main(threaded_scheduler)]
async fn main() {
    let challenge_client = kahoot::challenge::Client::new();

    let code = loop {
        print!("Code: ");
        let _ = stdout().flush();
        let code = read_line();

        match challenge_client.get_token(&code).await {
            Ok(_challenge) => {
                break code;
            }
            Err(e) => {
                println!(
                    "Failed to get challenge for {}, got error: {:#?}\n",
                    code, e
                );
            }
        }
    };

    let max_clients = loop {
        print!("Max Clients: ");
        let _ = stdout().flush();

        match read_line().parse::<usize>() {
            Ok(max_clients) => break max_clients,
            Err(e) => {
                println!("Invalid value, got error: {:#?}\n", e);
            }
        }
    };

    print!("Base Name: ");
    let _ = stdout().flush();
    let base_name = read_line();

    let swarm = Swarm::new(code, base_name);
    let swarm1 = swarm.clone();
    tokio::spawn(async move {
        swarm1.add_n_workers(max_clients).await.unwrap();
    });
    swarm.run().await;
}
