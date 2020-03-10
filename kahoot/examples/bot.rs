use kahoot::Context;
use rand::Rng;
use std::io::stdin;

struct BotHandler;

#[kahoot::async_trait]
impl kahoot::Handler for BotHandler {
    async fn on_login(&self, ctx: Context) {
        println!("Logged in as: {}", ctx.get_username());
    }

    async fn on_get_ready(&self, _ctx: Context, msg: kahoot::message::GetReadyMessage) {
        dbg!(msg);
    }

    async fn on_start_question(&self, ctx: Context, msg: kahoot::message::StartQuestionMessage) {
        tokio::time::delay_for(std::time::Duration::from_millis(250)).await; // Needed or kahoot thinks you were too fast

        let choice = rand::thread_rng().gen_range(0, msg.quiz_question_answers[msg.question_index]);
        ctx.submit_answer(choice).await.unwrap();
    }

    async fn on_error(&self, _ctx: Context, error: kahoot::KahootError) {
        dbg!(error);
    }
}

fn read_line() -> String {
    let mut s = String::new();
    stdin().read_line(&mut s).unwrap();
    String::from(s.trim())
}

#[tokio::main(threaded_scheduler)]
async fn main() {
    println!("Enter a Code: ");
    let code = read_line();

    println!("Enter a Name: ");
    let name = read_line();

    let mut client =
        match kahoot::Client::connect_with_handler(code.clone(), name, BotHandler).await {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to connect to quiz '{}', got error: {:#?}", code, e);
                return;
            }
        };

    match client.run().await {
        Ok(_) => {
            println!("Client exited sucessfully");
        }
        Err(e) => {
            eprintln!("Client exited with error: {:#?}", e);
        }
    }
}
