use kahoot::challenge::ChallengeClient;
use std::io::{
    stdin,
    stdout,
    Write,
};

fn add_client(
    challenge_client: &ChallengeClient,
    clients: &mut Vec<kahoot::Client>,
    code: &str,
    name: String,
) {
    let mut client = kahoot::Client::new();
    let token = challenge_client.get_challenge(code).unwrap();
    client
        .join(kahoot::JoinSettings {
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

        match challenge_client.get_challenge(&code) {
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

    let mut clients: Vec<kahoot::Client> = Vec::new();
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
