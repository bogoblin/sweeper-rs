mod eventlog;

use std::collections::VecDeque;
use axum::extract::{Path, State};
use axum::extract::{ws::WebSocket, WebSocketUpgrade};
use axum::http::{header, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{body, Router};
use clap::Parser;
use futures_util::{stream::{SplitSink, SplitStream}, SinkExt, StreamExt};
use include_dir::{include_dir, Dir};
use serde_json::{Value};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::{Arc};
use std::time::{Duration, Instant};
use axum::extract::ws::Message;
use mime_guess::mime::TEXT_HTML;
use tokio::net::TcpListener;
use log::{error, info, trace};
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use world::client_messages::ClientMessage;
use world::client_messages::ClientMessage::*;
use world::player::Player;
use world::server_messages::ServerMessage;
use world::{Rect, World};
use crate::eventlog::{EventLogReader, EventLogWriter, EventReadResult, SourcedEvent};

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
    broadcast_tx: Arc<Sender<Message>>,
    event_log_writer: Arc<UnboundedSender<SourcedEvent>>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();
    
    env_logger::init();

    let mut world = World::new();
    let start_time = Instant::now();
    if let Ok(reader) = EventLogReader::open("eventlog".into()).await {
        let mut events = reader.events();
        let mut events_read = 0;
        while let Some(event) = events.next().await {
            events_read += 1;
            if events_read % 1000 == 0 {
                info!("Read {} events", events_read);
            }
            trace!("read");
            match event {
                EventReadResult::Ok(event) => {
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
                EventReadResult::Invalid(text) => {
                    error!("Skipping invalid event: {}", text)
                }
                EventReadResult::EOF => {}
            }
        }
        info!("{} events read in {:?}", events_read, Instant::now() - start_time);
    } else {
        info!("No event log found, starting a new world.");
    }
    world.players.clear();

    let mut event_log_writer = EventLogWriter::new("eventlog".into()).await
        .expect("Unable to create event log writer");

    let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn((|| async move {
        let mut events = vec![];
        while event_rx.recv_many(&mut events, 1024).await != 0 {
            let events = std::mem::replace(&mut events, vec![]);
            for event in events {
                event_log_writer.write(event).await.unwrap()
            }
            event_log_writer.flush().await.unwrap();
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    })());
    
    let (broadcast_tx, _) = broadcast::channel(1024);
    
    let app = AppState {
        world: Arc::new(Mutex::new(world)),
        broadcast_tx: Arc::new(broadcast_tx),
        event_log_writer: Arc::new(event_tx),
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

    let player_id = app.world.lock().await.new_player_id();

    let (client_tx, client_rx) = tokio::sync::mpsc::unbounded_channel();
    let client_tx_clone = client_tx.clone();
    {
        let broadcast_rx = app.broadcast_tx.subscribe();
        tokio::spawn(async move {
            recv_broadcast(broadcast_rx, client_tx_clone).await;
        });
        tokio::spawn(async move {
            send_client_messages(ws_tx, client_rx).await;
        });
    }
    
    recv_from_client(ws_rx, client_tx, app.broadcast_tx, app.event_log_writer, app.world, &player_id).await;
}

async fn recv_broadcast(
    mut broadcast_rx: Receiver<Message>,
    client_tx: UnboundedSender<Message>,
) {
    while let Ok(msg) = broadcast_rx.recv().await {
        if client_tx.send(msg).is_err() {
            return;
        }
    }
}

async fn send_client_messages(
    mut client_tx: SplitSink<WebSocket, Message>,
    mut client_rx: UnboundedReceiver<Message>,
) {
    let mut messages = vec![];
    while client_rx.recv_many(&mut messages, 1024).await != 0 {
        let messages = std::mem::replace(&mut messages, vec![]);
        // TODO: find a way to bundle messages together
        for message in messages {
            if client_tx.feed(message).await.is_err() {
                return; // Disconnected
            }
        }
        if client_tx.flush().await.is_err() {
            return;
        }
    }
}

async fn recv_from_client(
    mut client_rx: SplitStream<WebSocket>,
    client_tx: UnboundedSender<Message>,
    broadcast_tx: Arc<Sender<Message>>,
    event_log_writer: Arc<UnboundedSender<SourcedEvent>>,
    world: Arc<Mutex<World>>,
    player_id: &str,
) {
    let broadcast = move |message: &ServerMessage| {
        if broadcast_tx.send(Message::Binary(message.into())).is_err() {
            error!("Unable to broadcast message");
        }
    };
    while let Some(Ok(msg)) = client_rx.next().await {
        let mut to_client = vec![];
        let mut new_chunks = VecDeque::new();
        let mut event = None;
        match msg {
            Message::Text(text) => {
                if let Ok(message) = serde_json::from_str::<Value>(&text) {
                    if let Some(message) = ClientMessage::decode(message) {
                        let mut world = world.lock().await;
                        match message {
                            // Click, Flag, and DoubleClick return a safety rect to send to the client
                            // in case nothing has been updated.
                            Click(position) => {
                                event = world.click(position, player_id);
                                if event.is_none() {
                                    let rect = world.get_rect(&Rect::from_center_and_size(position, 1, 1));
                                    to_client.push(ServerMessage::Rect(rect));
                                }
                            }
                            Flag(position) => {
                                event = world.flag(position, player_id);
                                if event.is_none() {
                                    let rect = world.get_rect(&Rect::from_center_and_size(position, 1, 1));
                                    to_client.push(ServerMessage::Rect(rect));
                                }
                            }
                            DoubleClick(position) => {
                                event = world.double_click(position, player_id);
                                if event.is_none() {
                                    let rect = world.get_rect(&Rect::from_center_and_size(position, 3, 3));
                                    to_client.push(ServerMessage::Rect(rect));
                                }
                            }
                            Connected => {
                                for (_player_id, player) in &world.players {
                                    to_client.push(ServerMessage::Player(player.clone()))
                                }
                                let player = Player::new(player_id.to_string());
                                to_client.push(ServerMessage::Welcome(player.clone()));
                                broadcast(&ServerMessage::Player(player));
                            },
                            Disconnected(player_id) => {
                                world.players.remove(&player_id);
                                broadcast(&ServerMessage::Disconnected(player_id));
                            }
                            Query(rect) => {
                                for chunk in world.query_chunks(&rect)
                                    .iter().map(|chunk_id| &world.chunks[*chunk_id]) {
                                    if chunk.should_send() {
                                        // TODO: cloning all these chunks might be expensive
                                        to_client.push(ServerMessage::Chunk(chunk.clone()));
                                    }
                                }
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
        
        if let Some(event) = event {
            event_log_writer.send(SourcedEvent::from_event(&event))
                .unwrap_or_default();
            broadcast(&ServerMessage::Event(event));
        }

        for message in to_client {
            let message = Message::Binary(Vec::<u8>::from(&message));
            client_tx.send(message).unwrap_or_default();
        }
        
        for (position, mines) in new_chunks {
            event_log_writer.send(SourcedEvent::ChunkGenerated(position, mines)).unwrap_or_default();
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