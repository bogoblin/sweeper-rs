use std::net::SocketAddr;
use std::sync::mpsc;
use std::thread;

use axum::Router;
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use world::{Chunk, Position, UpdatedRect, World};
use world::client_messages::ClientMessage::*;
use world::player::Player;
use world::server_messages::{chunk_message, player_message, updated_rect_message};

#[tokio::main]
async fn main() {
    let mut world = World::new();
    let (tx, rx) = mpsc::channel();
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", |socket: SocketRef| {
        tx.send((Welcome, socket.clone())).expect("Can't send welcome message");
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
                        tx.send((message, socket_ref)).expect("Can't send game message");
                    },
                    _ => {}
                }
            }
        });
    });

    let _handle = thread::spawn(move || {
        for (received, socket_ref) in rx {
            let player_id = world.register_player(format!("{}", socket_ref.id));
            eprintln!("Player {player_id} : {:?}", received);
            match received {
                Click(position) => {
                    let result = world.reveal(vec![position], player_id);
                    send_reveal_result(&world, &socket_ref, result, player_id);
                }
                Flag(position) => {
                    let result = world.flag(position, player_id);
                    send_reveal_result(&world, &socket_ref, result, player_id);
                }
                DoubleClick(position) => {
                    let result = world.double_click(position, player_id);
                    send_reveal_result(&world, &socket_ref, result, player_id);
                }
                Welcome => {
                    let player = unsafe {world.players.get_unchecked(player_id)};
                    let username = player.username.clone();
                    send_player(&socket_ref, player);
                    socket_ref.emit("welcome", username).expect("Couldn't send welcome message");
                    for chunk in &world.chunks {
                        send_chunk(&socket_ref, chunk);
                    }
                }
                Move(position) => {
                    world.players[player_id].position = position;
                }
            }
            let player = unsafe {world.players.get_unchecked(player_id)};
            eprintln!("{:?}", player);
        }
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

}

fn send_chunk(socket_ref: &SocketRef, chunk: &Chunk) {
    let (event, data) = chunk_message(chunk);
    socket_ref.emit(event, &data).expect("TODO: panic message");
    socket_ref.broadcast().emit(event, &data).expect("TODO: panic message");
}

fn send_player(socket_ref: &SocketRef, player: &Player) {
    let (event, data) = player_message(player);
    socket_ref.emit(event, &data).expect("TODO: panic message");
    socket_ref.broadcast().emit(event, &data).expect("TODO: panic message");
}

fn send_updated_rect(socket_ref: &SocketRef, updated_rect: &UpdatedRect) {
    if updated_rect.updated.is_empty() {
        return;
    }
    let (event, data) = updated_rect_message(updated_rect);
    socket_ref.emit(event, &data).expect("TODO: panic message");
    socket_ref.broadcast().emit(event, &data).expect("TODO: panic message");
}

fn send_reveal_result(world: &World, socket_ref: &SocketRef, updated: UpdatedRect, by_player_id: usize) {
    send_updated_rect(socket_ref, &updated);
    let player = unsafe {world.players.get_unchecked(by_player_id)};
    send_player(socket_ref, player);
}