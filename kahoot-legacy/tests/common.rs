/*
#[test]
fn common() {
    let mut handles: Vec<_> = (0..3)
        .map(|i| {
            std::thread::spawn(move || {
                let mut client = kahoot::Client::new();
                let code = "69412";
                let token = client.try_code(code).unwrap(); //716981, 463837, 31612, 69412
                let res = client.join_with_token_and_code(code, &token);
                let msg = kahoot::Request::HandShake;
                client.send(&msg);
                let initial_response: serde_json::Value =
                    serde_json::from_str(&client.recv().unwrap()).unwrap();
                let id = &initial_response[0]["clientId"].as_str().unwrap();
                let msg = kahoot::Request::Json(json!({
                    "channel": "/meta/connect",
                    "clientId": id,
                    "connectionType": "websocket",
                    "advice": {
                        "timeout": 0
                    }
                }));
                dbg!(&msg);
                client.send(&msg);
                dbg!(client.recv());
                let msg = kahoot::Request::Login {
                    name: &format!("hekkermein{}", i),
                    game_code: code,
                    client_id: id,
                };
                dbg!(&msg);
                client.send(&msg);
                dbg!(client.recv());
                loop {
                    let msg = kahoot::Request::KeepAlive { client_id: id };
                    std::thread::sleep(std::time::Duration::from_secs(6));
                    client.send(&msg);
                    //dbg!(client.recv());
                }
                dbg!(&res);
            })
        })
        .collect();
    handles.remove(0).join();
    handles.remove(1).join();
    handles.remove(2).join();
}
*/
