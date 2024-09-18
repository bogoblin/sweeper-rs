mod client_messages;

use axum::body::Bytes;
use axum::Router;
use serde_json::{json, Value};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{fs, thread};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use world::{Position, World};
use crate::client_messages::ClientMessage::*;

#[tokio::main]
async fn main() {
    let saved_world: Option<World> = {
        if let Ok(saved_world) = fs::read("worldfile") {
            match postcard::from_bytes::<World>(saved_world.as_slice()) {
                Ok(world) => Some(world),
                Err(e) => {
                    eprintln!("{:?}", e);
                    None
                }
            }
        } else {
            eprintln!("World file not found. Creating new world...");
            None
        }
    };
    let mut world = saved_world.unwrap_or(World::new());
    let (tx, rx) = mpsc::channel();
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", |socket: SocketRef| {
        if let Ok(_) = tx.send((Connected, socket.clone())) {
            socket.join("default").expect("TODO: panic message");
            let _ = socket.emit("join", json!({
                "player_id": socket.id
            }));
            socket.on("message", move |socket_ref: SocketRef, Data::<Value>(data)| {
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
                                _ => return,
                            };
                            tx.send((message, socket_ref)).unwrap_or_default();
                        },
                        _ => {}
                    }
                }
            });
        }
    });

    let _handle = thread::spawn(move || {
        let mut next_event = 0;
        let mut last_backup: Option<Instant> = None;
        for (received, socket_ref) in rx {
            let player_id = socket_ref.id.as_str();
            match received {
                Click(position) => {
                    world.click(position, player_id);
                }
                Flag(position) => {
                    world.flag(position, player_id);
                }
                DoubleClick(position) => {
                    world.double_click(position, player_id);
                }
                Connected => {
                    let mut chunks_to_send = vec![];
                    for chunk in &world.chunks {
                        if chunk.should_send() {
                            chunks_to_send.push(chunk.compress());
                        }
                    }
                    match &socket_ref.bin(chunks_to_send)
                        .emit("hello", "hello") {
                        Ok(_) => {}
                        Err(_) => {
                            socket_ref.disconnect().ok();
                            continue
                        }
                    }
                }
            }
            send_recent_events(&world, &socket_ref, next_event);
            next_event = world.events.len();

            let now = Instant::now();
            let do_backup = match last_backup {
                None => true,
                Some(backup_time) =>
                    now - backup_time > Duration::from_secs(5)
            };
            if do_backup {
                last_backup = Some(now);
                if let Ok(serialized) = postcard::to_allocvec(&world) {
                    let num_bytes = serialized.len();
                    println!("Writing {num_bytes} bytes to backup file...");
                    if let Err(err) = fs::write("worldfile", serialized) {
                        eprintln!("{err}");
                    }
                    println!("Done");
                }
            }
        }
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([0,0,0,0], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

}

fn send_recent_events(world: &World, socket_ref: &SocketRef, next_event: usize) {
    for event in &world.events[next_event..] {
        println!("{:?}", event);
        socket_ref
            .bin(vec![Bytes::from(event.compress())])
            .within("default")
            .emit("e", vec![""]).ok();
    }
}