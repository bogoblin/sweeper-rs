mod eventlog;

use std::collections::VecDeque;
use axum::extract::{Path, State};
use axum::extract::{ws::WebSocket, WebSocketUpgrade};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body, Router};
use clap::Parser;
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use include_dir::{include_dir, Dir};
use serde_json::{Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc};
use axum::extract::ws::Message;
use mime_guess::mime::TEXT_HTML;
use tokio::net::TcpListener;
use log::{error, info, trace};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use world::client_messages::ClientMessage;
use world::client_messages::ClientMessage::*;
use world::events::Event;
use world::player::Player;
use world::server_messages::ServerMessage;
use world::World;
use crate::eventlog::{EventLogReader, EventLogWriter, SourcedEvent};

#[derive(Parser)]
struct Cli {
    #[arg(short, long, value_name = "PORT NUMBER")]
    port: Option<u16>,

    #[arg(short, long, value_name = "PATH")]
    world_file: Option<PathBuf>
}

#[derive(Clone)]
struct AppState {
    world: Arc<Mutex<World>>,
    broadcast_tx: Arc<Mutex<Sender<Message>>>,
    event_log_writer: Arc<Mutex<EventLogWriter>>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    env_logger::init();

    let mut world = World::new();
    if let Ok(mut reader) = EventLogReader::open("eventlog".into()).await {
        while let Some(event) = reader.read().await {
            match event {
                SourcedEvent::Click(position) => {
                    world.click(position, "");
                }
                SourcedEvent::DoubleClick(position) => {
                    world.double_click(position, "");
                }
                SourcedEvent::Flag(position) |
                SourcedEvent::Unflag(position) => {
                    // TODO: should probably handle this properly
                    world.flag(position, "");
                }
                SourcedEvent::ChunkGenerated(position, mines) => {
                    if world.get_chunk(position.position()).is_none() {
                        let chunk = mines.to_chunk(position);
                        world.insert_chunk(chunk);
                    }
                }
            }
        }
    } else {
        info!("No event log found, starting a new world.");
    }
    // We don't need to broadcast these events, so clear the queue:
    world.events.clear();
    world.players.clear();

    let event_log_writer = EventLogWriter::new("eventlog".into()).await
        .expect("Unable to create event log writer");

    let (tx, _) = broadcast::channel(32);
    let app = AppState {
        world: Arc::new(Mutex::new(world)),
        broadcast_tx: Arc::new(Mutex::new(tx)),
        event_log_writer: Arc::new(Mutex::new(event_log_writer)),
    };

    let router: Router<> = Router::new()
        .route("/", get(root))
        .route("/ws", get(ws_upgrade_handler))
        .with_state(app)
        .route("/static/*path", get(static_path))
        ;
    let port = cli.port.unwrap_or(80);
    info!("Hosting on port {port}");
    let addr = SocketAddr::from(([0,0,0,0], port));
    let tcp = TcpListener::bind(&addr).await.unwrap();

    axum::serve(tcp, router).await.unwrap();
}

async fn ws_upgrade_handler(ws: WebSocketUpgrade, State(app): State<AppState>) -> Response {
    ws.on_upgrade(|socket| handle_socket(socket, app))
}

async fn handle_socket(ws: WebSocket, app: AppState) {
    let (ws_tx, ws_rx) = ws.split();
    let ws_tx = Arc::new(Mutex::new(ws_tx));
    
    let player_id = app.world.lock().await.new_player_id();

    let tx_clone = ws_tx.clone();
    {
        let broadcast_rx = app.broadcast_tx.lock().await.subscribe();
        tokio::spawn(async move {
            recv_broadcast(tx_clone, broadcast_rx).await;
        });
    }
    
    recv_from_client(ws_rx, ws_tx, app.broadcast_tx, app.event_log_writer, app.world, &player_id).await;
}

async fn recv_broadcast(
    client_tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    mut broadcast_rx: Receiver<Message>,
) {
    while let Ok(msg) = broadcast_rx.recv().await {
        trace!("Broadcasting message: {:?}", msg);
        if client_tx.lock().await.send(msg).await.is_err() {
            return; // disconnected.
        }
    }
}

async fn recv_from_client(
    mut client_rx: SplitStream<WebSocket>,
    client_tx: Arc<Mutex<SplitSink<WebSocket, Message>>>,
    broadcast_tx: Arc<Mutex<Sender<Message>>>,
    event_log_writer: Arc<Mutex<EventLogWriter>>,
    world: Arc<Mutex<World>>,
    player_id: &str,
) {
    while let Some(Ok(msg)) = client_rx.next().await {
        let mut to_broadcast = vec![];
        let mut to_client = vec![];
        let mut new_chunks = VecDeque::new();
        match msg {
            Message::Text(text) => {
                if let Ok(message) = serde_json::from_str::<Value>(&text) {
                    if let Some(message) = ClientMessage::decode(message) {
                        let mut world = world.lock().await;
                        match message {
                            // Click, Flag, and DoubleClick return a safety rect to send to the client
                            // in case nothing has been updated.
                            Click(position) => {
                                let rect = world.click(position, player_id);
                                to_client.push(ServerMessage::Rect(rect))
                            }
                            Flag(position) => {
                                let rect = world.flag(position, player_id);
                                to_client.push(ServerMessage::Rect(rect))
                            }
                            DoubleClick(position) => {
                                let rect = world.double_click(position, player_id);
                                to_client.push(ServerMessage::Rect(rect))
                            }
                            Connected => {
                                for (_player_id, player) in &world.players {
                                    to_client.push(ServerMessage::Player(player.clone()))
                                }
                                let player = Player::new(player_id.to_string());
                                to_broadcast.push(ServerMessage::Player(player.clone()));
                                to_client.push(ServerMessage::Welcome(player));
                            },
                            Disconnected(player_id) => {
                                world.players.remove(&player_id);
                                to_broadcast.push(ServerMessage::Disconnected(player_id));
                            }
                            Query(rect) => {
                                let query = world.query_chunks(&rect);
                                for chunk in query {
                                    if chunk.should_send() {
                                        // TODO: cloning all these chunks might be expensive
                                        to_client.push(ServerMessage::Chunk(chunk.clone()));
                                    }
                                }
                            }
                        }
                        while let Some(event) = world.events.pop_front() {
                            if event.should_send() {
                                to_broadcast.push(ServerMessage::Event(event))
                            }
                        }
                        std::mem::swap(&mut world.generated_chunks, &mut new_chunks);
                    }
                }
            }
            Message::Binary(_) => {}
            Message::Ping(_) => {}
            Message::Pong(_) => {}
            Message::Close(_) => return
        }

        {
            let messages = to_client.into_iter()
                .map(|message| {
                    Message::Binary(Vec::<u8>::from(&message))
                });
            let mut lock = client_tx.lock().await;
            for message in messages {
                lock.send(message).await.unwrap_or_default();
            }
        }
        
        {
            let lock = broadcast_tx.lock().await;
            
            for message in &to_broadcast {
                match lock.send(Message::Binary(Vec::<u8>::from(message))) {
                    Ok(_) => {}
                    Err(err) => {
                        error!("Unable to broadcast message: {}", err);
                    }
                }
            }
        }

        {
            let mut lock = event_log_writer.lock().await;
            
            for (position, mines) in new_chunks {
                lock.write(SourcedEvent::ChunkGenerated(position, mines)).await.unwrap();
            }

            for message in &to_broadcast {
                if let Some(sourced) = match message {
                    ServerMessage::Event(event) => {
                        match event {
                            Event::Clicked { at, .. } => {
                                Some(SourcedEvent::Click(*at))
                            }
                            Event::DoubleClicked { at, .. } => {
                                Some(SourcedEvent::DoubleClick(*at))
                            }
                            Event::Flag { at, .. } => {
                                Some(SourcedEvent::Flag(*at))
                            }
                            Event::Unflag { at, .. } => {
                                Some(SourcedEvent::Unflag(*at))
                            }
                        }
                    }
                    _ => None
                } {
                    match lock.write(sourced).await {
                        Ok(_) => {}
                        Err(err) => {
                            error!("Unable to write to event log: {}", err);
                        }
                    }
                }
            }
            
            lock.flush().await.expect("Unable to flush event log");
        }
        
    }
}

static STATIC_DIR: Dir<'_> = include_dir!("crates/server/static");
// Thanks to https://matze.github.io/axum-notes/notes/misc/serve_static_from_binary/index.html
async fn static_path(Path(path): Path<String>) -> impl IntoResponse {
    let path = path.trim_start_matches('/');
    let mime_type = mime_guess::from_path(path).first_or(TEXT_HTML);
    trace!("Serving {path} as {mime_type}");

    match STATIC_DIR.get_file(path) {
        None => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(body::Body::empty())
            .unwrap(),
        Some(file) => {
            // The server compiles into one binary, including the files from the static directory,
            // but in development, we want to serve the files from the directory so that we don't
            // have to recompile whenever we change something:
            if cfg!(debug_assertions) {
                let static_dir_path: PathBuf = PathBuf::from("crates/server/static");
                let path = static_dir_path.join(file.path());
                trace!("Serving file from filesystem: {:?}", path);
                if let Ok(contents) = tokio::fs::read(path).await {
                    Response::builder()
                        .status(StatusCode::OK)
                        .header(
                            header::CONTENT_TYPE,
                            HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                        )
                        .body(body::Body::from(contents))
                        .unwrap()
                } else {
                    Response::builder()
                        .status(StatusCode::INTERNAL_SERVER_ERROR)
                        .body(body::Body::empty())
                        .unwrap()
                }
            } else {
                trace!("Serving file from binary");
                Response::builder()
                    .status(StatusCode::OK)
                    .header(
                        header::CONTENT_TYPE,
                        HeaderValue::from_str(mime_type.as_ref()).unwrap(),
                    )
                    .body(body::Body::from(file.contents()))
                    .unwrap()
            }
        }
    }
}

async fn root() -> impl IntoResponse {
    static_path(Path("index.html".to_string())).await
}