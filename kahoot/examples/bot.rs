use kahoot::Context;
use rand::Rng;

struct BotHandler;

#[kahoot::async_trait]
impl kahoot::Handler for BotHandler {
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

#[tokio::main(threaded_scheduler)]
async fn main() {
    let code = "7800484";
    let mut client = kahoot::Client::connect_with_handler(code.into(), "bob1".into(), BotHandler)
        .await
        .unwrap();

    let res = client.run().await;
    res.unwrap();
}
