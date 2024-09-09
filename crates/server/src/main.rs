use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::mpsc;
use std::{fs, thread};
use std::time::{Duration, Instant};
use axum::Router;
use serde_json::{json, Value};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::socket::Sid;
use socketioxide::{SendError, SocketIo};
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use world::{AuthKey, Chunk, Position, UpdatedRect, World};
use world::client_messages::ClientMessage::*;
use world::player::Player;
use world::server_messages::{chunk_message, flag_message, player_message, unflag_message, updated_rect_message};
use world::events::Event;

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
        let _ = tx.send((Connected, socket.clone())).or_else(|err| Ok(println!("{err}")));
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
                            "move" => Move(position),
                            _ => return,
                        };
                        let _ = tx.send((message, socket_ref)).or_else(|err| Ok(println!("{err}")));
                    },
                    [Value::String(message_type), Value::String(string)] => {
                        match message_type.as_str() {
                            "register" => {
                                let username = string.to_string();
                                let _ = tx.send((Register(username), socket_ref)).or_else(|err| Ok(println!("{err}")));
                            }
                            "login" => {
                                let auth_key = AuthKey(string.to_string());
                                let _ = tx.send((Login(auth_key), socket_ref)).or_else(|err| Ok(println!("{err}")));
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
        });
    });

    let mut socket_id_to_player_id: HashMap<Sid, usize> = HashMap::new();
    let _handle = thread::spawn(move || {
        let mut next_event = 0;
        let mut last_backup: Option<Instant> = None;
        for (received, socket_ref) in rx {
            let player_id = socket_id_to_player_id.get(&socket_ref.id).cloned();
            match received {
                Click(position) => {
                    if let Some(player_id) = player_id { world.click(position, player_id); }
                }
                Flag(position) => {
                    if let Some(player_id) = player_id { world.flag(position, player_id); }
                }
                DoubleClick(position) => {
                    if let Some(player_id) = player_id { world.double_click(position, player_id); }
                }
                Move(position) => {
                    if let Some(player_id) = player_id { world.players[player_id].position = position; }
                }
                Connected => {
                    for chunk in &world.chunks {
                        send_chunk(&socket_ref, chunk);
                    }
                }
                Register(username) => {
                    let (AuthKey(auth_key), player) = world.register_player(username);
                    let username = player.username.clone();
                    let _ = socket_ref.emit("login_details", json!({
                        "username": username,
                        "authKey": auth_key
                    })).or_else(|err| Ok(println!("{err}")));
                    // TODO: Handle failure by removing player from world
                }
                Login(auth_key) => {
                    if let Some(( player_id, player )) = world.authenticate_player(&auth_key) {
                        socket_id_to_player_id.insert(socket_ref.id, player_id);
                        send_player(&socket_ref, &player);
                        let _ = socket_ref.emit("welcome", &player.username).or_else(|err| Ok(println!("{err}")));
                    } else {
                        let _ = socket_ref.emit("error", json!({
                            "error": "Auth key not recognised"
                        })).or_else(|err| Ok(println!("{err}")));
                    }
                }
            }
            send_recent_events(&world, &socket_ref, next_event);
            next_event = world.events.len();
            if let Some(player) = player_id.map(|player_id| world.players.get(player_id)).flatten() {
                eprintln!("{:?}", player);
            }

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
                    let _ = fs::write("worldfile", serialized).or_else(|err| Ok(println!("{err}")));
                    println!("Done");
                }
            }
        }
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

}

fn send_recent_events(world: &World, socket_ref: &SocketRef, next_event: usize) {
    for event in &world.events[next_event..] {
        println!("{:?}", event);
        match event {
            Event::Registered { player_id } => {
                let player = unsafe {world.players.get_unchecked(*player_id)};
                send_player(socket_ref, player);
            }
            Event::Clicked { player_id, at: _, updated } => {
                send_updated_rect(socket_ref, &updated);
                let player = unsafe {world.players.get_unchecked(*player_id)};
                send_player(socket_ref, player);
            },
            Event::DoubleClicked { player_id, at: _, updated } => {
                send_updated_rect(socket_ref, &updated);
                let player = unsafe {world.players.get_unchecked(*player_id)};
                send_player(socket_ref, player);
            }
            Event::Flag { player_id, at } => {
                let (event, data) = flag_message(at);
                send_all(socket_ref, event, data);
                let player = unsafe {world.players.get_unchecked(*player_id)};
                send_player(socket_ref, player);
            }
            Event::Unflag { player_id, at } => {
                let (event, data) = unflag_message(at);
                send_all(socket_ref, event, data);
                let player = unsafe {world.players.get_unchecked(*player_id)};
                send_player(socket_ref, player);
            }
        }
    }
}

fn send_chunk(socket_ref: &SocketRef, chunk: &Chunk) {
    let (event, data) = chunk_message(chunk);
    send_all(socket_ref, event, data);
}

fn send_player(socket_ref: &SocketRef, player: &Player) {
    let (event, data) = player_message(player);
    send_all(socket_ref, event, data);
}

fn send_updated_rect(socket_ref: &SocketRef, updated_rect: &UpdatedRect) {
    if updated_rect.updated.is_empty() {
        return;
    }
    let (event, data) = updated_rect_message(updated_rect);
    send_all(socket_ref, event, data);
}

fn send_all(socket_ref: &SocketRef, event: &'static str, data: Value) {
    socket_ref.emit(event, &data).ok();
    socket_ref.broadcast().emit(event, &data).ok();
}