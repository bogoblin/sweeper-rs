use futures_util::future::join_all;
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use rand::{thread_rng, RngCore};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::sleep;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use world::{ClientMessage, ServerMessageBundle};
use world::Event;
use world::ServerMessage;
use world::Position;

static THREADS: usize = 200;
static TEST_DURATION: Duration = Duration::from_secs(600);
static TIME_BETWEEN_MESSAGES: Duration = Duration::from_millis(1000);

#[tokio::main]
async fn main() {
    let mut handles = vec![];
    for _ in 0..THREADS {
        handles.push(tokio::spawn(Client::spawn()));
        sleep(TIME_BETWEEN_MESSAGES/THREADS as u32).await
    }
    join_all(handles).await;
}

struct Request {
    message: ClientMessage,
    sent_at: Instant,
}

struct Response {
    message: ServerMessage,
    received_at: Instant,
}

struct SentMessage {
    request: Request,
    response: Option<Response>,
}

struct Client {
    sent_messages: Vec<SentMessage>,
    player_id: Option<String>,
}

impl Client {
    pub async fn spawn() {
        let (stream, _response) = connect_async("ws://infinitesweeper.online/ws")
            .await.expect("couldn't connect");

        let (write, read) = stream.split();

        let client = Arc::new(Mutex::new(Client {
            sent_messages: vec![],
            player_id: None,
        }));

        let sender = Client::sender(client.clone(), write);
        let receiver = Client::receiver(client.clone(), read);
        let sender = tokio::spawn(sender);
        let receiver = tokio::spawn(receiver);

        tokio::time::sleep(TEST_DURATION).await;
        sender.abort();
        receiver.abort();

        let client = client.lock().await;
        for sent in &client.sent_messages {
            match &sent.response {
                None => {
                    // println!("{:?}: no matched _response", sent.request.message);
                }
                Some(response) => {
                    println!("{:?}: {:?}", response.message, response.received_at - sent.request.sent_at)
                }
            }
        }
    }

    async fn sender(client: Arc<Mutex<Client>>, mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
        {
            let mut client = client.lock().await;
            client.send_message(ClientMessage::Connected, &mut write).await
        }
        loop {
            let mut client = client.lock().await;
            let message = ClientMessage::Click(Position(thread_rng().next_u32() as i32%5000, thread_rng().next_u32() as i32%5000));
            client.send_message(message, &mut write).await;
            tokio::time::sleep(TIME_BETWEEN_MESSAGES).await;
        }
    }

    async fn send_message(&mut self, message: ClientMessage, write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
        println!("sending {:?}", message);
        self.sent_messages.push(SentMessage {
            request: Request {
                message,
                sent_at: Instant::now()
            },
            response: None
        });
        let message = &self.sent_messages.last().unwrap().request.message;
        write.send(Message::Text(Utf8Bytes::from(serde_json::to_string(message).expect("couldn't serialize message")))).await.expect("TODO: panic message");
    }

    async fn receiver(client: Arc<Mutex<Client>>, read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
        read.for_each(|message| async {
            match message {
                Ok(data) => {
                    println!("got data {:?}", data);
                    if let Ok(ServerMessageBundle(messages)) = ServerMessageBundle::from_compressed(&*data.into_data()) {
                        for message in messages {
                            println!("got message {:?}", message);
                            let mut client = client.lock().await;
                            client.match_server_message(message);
                        }
                    }
                }
                Err(err) => {
                    println!("{:?}", err)
                }
            }
        }).await
    }

    fn match_server_message(&mut self, server_message: ServerMessage) {
        let sm_clone = server_message.clone();
        match &server_message {
            ServerMessage::Event(event) => {
                let player = event.player();
                if Some(player.player_id) == self.player_id {
                    // it was a message we sent, so find it:
                    let corresponding_client_message = match event {
                        Event::Clicked { at, .. } => ClientMessage::Click(*at),
                        Event::DoubleClicked { at, .. } => ClientMessage::DoubleClick(*at),
                        Event::Flag { at, .. } |
                        Event::Unflag { at, .. } => ClientMessage::Flag(*at),
                    };
                    for sent in &mut self.sent_messages {
                        if sent.response.is_none() && sent.request.message == corresponding_client_message {
                            sent.response = Some(Response {
                                message: sm_clone.clone(),
                                received_at: Instant::now(),
                            })
                        }
                    }
                }
            }
            ServerMessage::Chunk(_) => {}
            ServerMessage::Rect(_) => {}
            ServerMessage::Player(_) => {}
            ServerMessage::Welcome(player) => {
                self.player_id = Some(player.player_id.clone());
            }
            ServerMessage::Disconnected(_) => {}
            ServerMessage::Connected => {}
        }
    }
}