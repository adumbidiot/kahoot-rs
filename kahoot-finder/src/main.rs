use http::StatusCode;
use kahoot::KahootError;
use std::time::Duration;

const CODE_LIMIT: u32 = 999_999;
const BACKOFF_TIME: Duration = Duration::from_secs(10);

#[tokio::main]
async fn main() {
    let client = kahoot::Client::new();
    let mut code = rand::random::<u32>() % CODE_LIMIT;

    loop {
        println!("Testing code {}..", code);

        let code_str = format!("{:06}", code);
        let challenge = client.get_token(&code_str).await;

        match challenge {
            Ok(c) => {
                println!("Located kahoot: {}", code_str);
                println!("Decoded Challenge: {}", c);
                break;
            }
            Err(KahootError::InvalidCode) => {}
            Err(KahootError::InvalidStatus(StatusCode::INTERNAL_SERVER_ERROR)) => {
                println!("500 status, backing down for 10 seconds...");
                tokio::time::delay_for(BACKOFF_TIME).await;
            }
            Err(e) => {
                println!("Terminating on unknown error: {:#?}", e);
                break;
            }
        }

        code = code.wrapping_add(1) % CODE_LIMIT;
    }
}
