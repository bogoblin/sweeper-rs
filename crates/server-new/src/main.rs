use std::sync::{Arc, Condvar, mpsc, Mutex, RwLock};
use std::thread;
use serde_json::Value;
use websocket::OwnedMessage;
use websocket::sync::Server;
use world::client_messages::ClientMessage::{Click, DoubleClick, Flag, Move};
use world::Position;

fn main() {
    let handles = vec![
        thread::spawn(websocket_server),
        thread::spawn(http_server),
    ];
    for handle in handles {
        handle.join().unwrap();
    }
}

fn http_server() {
    let _ = std::process::Command::new("simple-http-server")
        .args(["--index"])
        .current_dir("static")
        .status();
}

fn websocket_server() {
    let server = Server::bind("127.0.0.1:2794").unwrap();

    let mut world = world::World::new();
    let mut serialized_events = Arc::new((RwLock::new(vec![]), Condvar::new()));
    let (client_tx, client_rx) = mpsc::channel();

    for request in server.filter_map(Result::ok) {
        let mut client = request.accept().unwrap();

        let ip = client.peer_addr().unwrap();

        println!("Connection from {}", ip);

        let (mut receiver, mut sender) = client.split().unwrap();

        thread::spawn(move || {
            // TODO: send the current state of the world to the client here

            let (events, events_cvar) = &*serialized_events;
            let mut next_message_id = {
                events.read().unwrap().len()
            };

            loop { // TODO: only loop while we're connected, but will probably panic if we're not
                events_cvar.wait()
            }
        });

        thread::spawn(move || {
            for message in receiver.incoming_messages() {
                let message = message.unwrap();

                match message {
                    OwnedMessage::Text(json_message) => {
                        let message : Value = serde_json::from_str(json_message.as_str()).unwrap();
                        match message {
                            Value::Array(data) => {
                                if let Value::Array(array) = data {
                                    match &array[..] {
                                        [Value::String(message_type), Value::Number(x), Value::Number(y)] => {
                                            let x = x.as_f64().unwrap().floor() as i32;
                                            let y = y.as_f64().unwrap().floor() as i32;
                                            let position = Position(x, y);
                                            let message = match message_type.as_str() {
                                                "click" => Click(position),
                                                "flag" => Flag(position),
                                                "doubleClick" => DoubleClick(position),
                                                "move" => Move(position),
                                                _ => return,
                                            };
                                            client_tx.send(message).expect("Can't send game message");
                                        },
                                        _ => {}
                                    }
                                }

                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });
    }
}