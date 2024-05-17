use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::mpsc;
use std::thread;

use axum::Router;
use futures_util::FutureExt;
use serde_json::{json, Value};
use socketioxide::extract::{Bin, Data, SocketRef};
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::world::{Position, RevealResult, World};

mod world;

#[tokio::main]
async fn main() {
    let mut world = World::new();
    let (tx, rx) = mpsc::channel();
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", |socket: SocketRef, Data(data): Data<Value>| {
        socket.on("click", move |socket_ref: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
            match data {
                Value::Array(array) => {
                    if array.len() == 2 {
                        if let Some(Value::Number(x)) = array.get(0) {
                            if let Some(Value::Number(y)) = array.get(1) {
                                let x = x.as_f64().unwrap().floor() as i32;
                                let y = y.as_f64().unwrap().floor() as i32;
                                let position = Position(x, y);
                                tx.send((position, socket_ref)).unwrap();
                            }
                        }
                    }
                },
                _ => {}
            }
        })
    });

    let handle = thread::spawn(move || {
        for (received, socket_ref) in rx {
            let result = world.reveal(received);
            match &result {
                RevealResult::Death(_) => {}
                RevealResult::Revealed(chunks) => {
                    let tile_to_u8 = |mine: bool, flag: bool, revealed: bool, adjacent: u8| -> u8 {
                        let mut result = adjacent;
                        if mine {result += 1<<4}
                        if flag {result += 1<<5}
                        if revealed {result += 1<<6}
                        result
                    };
                    for &chunk_id in chunks.keys() {
                        if let Some(coords) = world.positions.get(chunk_id) {
                            let mines = world.mines.get(chunk_id).unwrap();
                            let flags = world.flags.get(chunk_id).unwrap();
                            let revealed = world.revealed.get(chunk_id).unwrap();
                            let adjacent = world.adjacent_mines.get(chunk_id).unwrap().as_ref().unwrap();
                            let mut tiles = Vec::new();
                            for x in 0..16 {
                                for y in 0..16 {
                                    tiles.push(tile_to_u8(
                                        mines.get(Position(x, y)),
                                        flags.get(Position(x, y)),
                                        true,
                                        adjacent.get(Position(x, y)),
                                    ))
                                }
                            }
                            match socket_ref.emit("chunk", json!({
                                "coords": [coords.0, coords.1],
                                "tiles": tiles,
                            })) {
                                Ok(_) => { println!("sent") }
                                Err(err) => { eprintln!("{}", err) }
                            }
                        }
                    }
                }
                RevealResult::Nothing => {}
            }
            world.apply_reveal(result);
        }
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

}