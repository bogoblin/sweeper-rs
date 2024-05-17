use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::mpsc;

use axum::Router;
use futures_util::FutureExt;
use serde_json::Value;
use socketioxide::extract::{Bin, Data, SocketRef};
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

use crate::world::{Position, World};

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
                                tx.send(position).unwrap();
                            }
                        }
                    }
                },
                _ => {}
            }
        })
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();

    for received in rx {
        let result = world.reveal(received);
        world.apply_reveal(result);
    }
}