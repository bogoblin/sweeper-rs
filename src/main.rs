use std::net::SocketAddr;

use axum::Router;
use futures_util::FutureExt;
use serde_json::Value;
use socketioxide::extract::{Bin, Data, SocketRef};
use socketioxide::SocketIo;
use tokio::net::TcpListener;
use tower_http::services::ServeDir;

mod world;

#[tokio::main]
async fn main() {
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", |socket: SocketRef, Data(data): Data<Value>| {
        socket.on("click", |socket_ref: SocketRef, Data::<Value>(data), Bin(bin)| {
            println!("Received event: {:?} {:?}", data, bin);
        })
    });

    let router: Router<> = Router::new()
        .fallback_service(ServeDir::new("static"))
        .layer(socket_layer);
    let addr = SocketAddr::from(([127,0,0,1], 8000));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}