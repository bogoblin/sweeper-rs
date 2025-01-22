mod backup;

use axum::body::Bytes;
use axum::extract::Path;
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body, Router};
use clap::Parser;
use include_dir::{include_dir, Dir};
use serde_json::{json, Value};
use socketioxide::extract::{Data, SocketRef};
use socketioxide::SocketIo;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use mime_guess::mime::TEXT_HTML;
use tokio::net::TcpListener;
use log::debug;
use crate::backup::Backup;
use world::client_messages::ClientMessage;
use world::client_messages::ClientMessage::*;
use world::World;

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "PORT NUMBER")]
    port: Option<u16>,

    #[arg(short, long, value_name = "PATH")]
    world_file: Option<PathBuf>
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    let mut backup = Backup::new(cli.world_file.unwrap_or("worldfile".into()));

    let mut world = backup.load().unwrap_or_else(|_| World::new());
    
    let (tx, rx) = mpsc::channel();
    let (socket_layer, io) = SocketIo::new_layer();
    io.ns("/", |socket: SocketRef| {
        if let Ok(_) = tx.send((Connected, socket.clone())) {
            socket.join("default").expect("TODO: panic message");
            let _ = socket.emit("join", json!({
                "player_id": socket.id
            }));
            socket.on("message", move |socket_ref: SocketRef, Data::<Value>(data)| {
                if let Some(message) = ClientMessage::decode(data) {
                    tx.send((message, socket_ref)).unwrap_or_default();
                }
            });
        }
    });

    let _handle = thread::spawn(move || {
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
                Connected => {},
                QueryChunks(rect) => {
                    let mut chunks_to_send = vec![];
                    let query = world.query_chunks(&rect);
                    for chunk in query {
                        if chunk.should_send() {
                            chunks_to_send.push(chunk.compress());
                        }
                    }
                    match &socket_ref.bin(chunks_to_send)
                        .emit("e", vec![""]) {
                        Ok(_) => {}
                        Err(_) => {
                            socket_ref.disconnect().ok();
                            continue
                        }
                    }
                }
            }
            send_recent_events(&mut world, &socket_ref);
            
            match backup.save(&world) {
                Ok(bytes_written) => {
                    debug!("{} bytes written to {}", bytes_written, backup.location().to_string_lossy());
                }
                Err(err) => {
                    eprintln!("Error while writing worldfile: {err}")
                }
            }
        }
    });

    let router: Router<> = Router::new()
        .layer(socket_layer)
        .route("/", get(root))
        .route("/static/*path", get(static_path))
        ;
    let port = cli.port.unwrap_or(80);
    println!("Hosting on port {port}");
    let addr = SocketAddr::from(([0,0,0,0], port));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}

static STATIC_DIR: Dir<'_> = include_dir!("crates/server/static");
// Thanks to https://matze.github.io/axum-notes/notes/misc/serve_static_from_binary/index.html
async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    println!("{path}");
    let mime_type = mime_guess::from_path(path).first_or(TEXT_HTML);
    println!("{path}: {mime_type}");

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::Body::empty())
            .unwrap(),
        Some(file) => Response::builder()
            .status(StatusCode::OK)
            .header(
                header::CONTENT_TYPE,
                HeaderValue::from_str(mime_type.as_ref()).unwrap(),
            )
            .body(body::Body::from(file.contents()))
            .unwrap(),
    } 
}

async fn root() -> impl IntoResponse {
    static_path(Path("index.html".to_string())).await
}

fn send_recent_events(world: &mut World, socket_ref: &SocketRef) {
    while let Some(event) = world.events.pop_front() {
        println!("{:?}", event);
        socket_ref
            .bin(vec![Bytes::from(event.compress())])
            .within("default")
            .emit("e", vec![""]).ok();
    }
}