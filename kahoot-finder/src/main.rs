use http::StatusCode;
use kahoot::{
    Context,
    KahootError,
};
use std::time::Duration;

const CODE_LIMIT: u64 = 9_999_999; // Longest Code seen: 7,800,484
const BACKOFF_TIME: Duration = Duration::from_secs(10);
const DEFAULT_NAME: &str = "test123"; // TODO: Make bot react to invalid names and randomize

struct ProbeHandler;

#[kahoot::async_trait]
impl kahoot::Handler for ProbeHandler {
    async fn on_login(&self, ctx: Context) {
        ctx.shutdown().await.expect("Shutdown");
    }

    async fn on_error(&self, ctx: Context, _error: kahoot::KahootError) {
        ctx.shutdown().await.expect("Shutdown");
    }
}

#[tokio::main]
async fn main() {
    let client = kahoot::challenge::Client::new();
    let mut code = rand::random::<u64>() % CODE_LIMIT;

    loop {
        println!("Testing code {}..", code);

        let code_str = format!("{:06}", code); // Min length code: ?
        let challenge = client.get_token(&code_str).await;

        match challenge {
            Ok(c) => {
                println!("Located potential kahoot: {}", code_str);
                println!("Decoded Challenge: {}", c);
                println!("Testing Session validity...");

                let client_connect = kahoot::Client::connect_with_handler(
                    code_str.clone(),
                    DEFAULT_NAME.into(),
                    ProbeHandler,
                )
                .await;

                match client_connect {
                    Ok(mut client) => match client.run().await {
                        Ok(_) => {
                            println!("Located Valid Kahoot code: {}", code_str);
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to join Kahoot session '{}', with name '{}', got error: {:#?}", code_str.clone(), DEFAULT_NAME, e);
                        }
                    },
                    Err(e) => {
                        eprintln!(
                            "Unable to connect to Bauyex server using websockets, got error: {:#?}",
                            e
                        );
                    }
                }
            }
            Err(KahootError::InvalidCode) => {}
            Err(KahootError::InvalidStatus(StatusCode::INTERNAL_SERVER_ERROR)) => {
                eprintln!("500 status, backing down for 10 seconds...");
                tokio::time::delay_for(BACKOFF_TIME).await;
            }
            Err(e) => {
                eprintln!("Terminating on unknown error: {:#?}", e);
                break;
            }
        }

        code = code.wrapping_add(1) % CODE_LIMIT;
    }
}
