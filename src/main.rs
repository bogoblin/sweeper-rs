use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;

use axum::Router;
use futures_util::FutureExt;
use serde::Serialize;
use serde_json::Value;
use socketioxide::extract::{Bin, Data, SocketRef};
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use ClientMessage::{Click, Flag};

use crate::client_messages::ClientMessage;
use crate::server_messages::chunk_message;
use crate::world::{Chunk, FlagResult, Position, RevealResult, World};

mod world;
mod server_messages;
mod client_messages;

#[tokio::main]
async fn main() {
    let mut world = World::new();
    let (tx, rx) = mpsc::channel();
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", move |socket: SocketRef, Data(data): Data<Value>| {

        let tx_click = mpsc::Sender::clone(&tx);
        socket.on("click", move |socket_ref: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Click: {:?} {:?}", data, bin);
            match data {
                Value::Array(array) => {
                    if array.len() == 2 {
                        if let Some(Value::Number(x)) = array.get(0) {
                            if let Some(Value::Number(y)) = array.get(1) {
                                let x = x.as_f64().unwrap().floor() as i32;
                                let y = y.as_f64().unwrap().floor() as i32;
                                let position = Position(x, y);
                                tx_click.send((Click(position), socket_ref)).unwrap();
                            }
                        }
                    }
                },
                _ => {}
            }
        });

        let tx_flag = mpsc::Sender::clone(&tx);
        socket.on("flag", move |socket_ref: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Flag: {:?} {:?}", data, bin);
            match data {
                Value::Array(array) => {
                    if array.len() == 2 {
                        if let Some(Value::Number(x)) = array.get(0) {
                            if let Some(Value::Number(y)) = array.get(1) {
                                let x = x.as_f64().unwrap().floor() as i32;
                                let y = y.as_f64().unwrap().floor() as i32;
                                let position = Position(x, y);
                                tx_flag.send((Flag(position), socket_ref)).unwrap();
                            }
                        }
                    }
                },
                _ => {}
            }
        });
    });

    let handle = thread::spawn(move || {
        for (received, socket_ref) in rx {
            match received {
                Click(position) => {
                    let result = world.reveal(position);
                    match &result {
                        RevealResult::Death(_) => {}
                        RevealResult::Revealed(chunks) => {
                            for &chunk_id in chunks {
                                if let Some(chunk) = world.chunks.get(chunk_id) {
                                    send_chunk(&socket_ref, chunk);
                                }
                            }
                        }
                        RevealResult::Nothing => {}
                    }
                    if let Some(chunk) = world.get_chunk(position) {
                        send_chunk(&socket_ref, chunk);
                    }
                }
                Flag(position) => {
                    let result = world.flag(position);
                    match &result {
                        FlagResult::Flagged(_) => {}
                        FlagResult::Unflagged(_) => {}
                        FlagResult::Nothing => {}
                    }
                    if let Some(chunk) = world.get_chunk(position) {
                        send_chunk(&socket_ref, chunk);
                    }
                }
                ClientMessage::DoubleClick(_) => {}
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

fn send_chunk(socket_ref: &SocketRef, chunk: &Chunk) {
    let (event, data) = chunk_message(chunk);
    socket_ref.emit(event, &data).expect("TODO: panic message");
    socket_ref.broadcast().emit(event, &data).expect("TODO: panic message");
}