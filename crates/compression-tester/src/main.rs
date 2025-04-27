use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio::time::Instant;
use tokio_tungstenite::tungstenite::{Message, Utf8Bytes};
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use world::Position;
use world::ServerMessage;
use world::{ClientMessage, Rect, ServerMessageBundle};

#[tokio::main]
async fn main() {
    Client::spawn().await
}

struct Request {
    message: ClientMessage,
}

struct SentMessage {
    request: Request,
}

struct Client {
    sent_messages: Vec<SentMessage>,
    outfile: BufWriter<File>
}

impl Client {
    pub async fn spawn() {
        let (stream, _response) = connect_async("ws://infinitesweeper.online/ws")
            .await.expect("couldn't connect");

        let (write, read) = stream.split();

        let chunk_file = OpenOptions::new()
            .write(true)
            .create(true)
            .open("chunks").await.unwrap();
        let client = Arc::new(Mutex::new(Client {
            sent_messages: vec![],
            outfile: BufWriter::new(chunk_file),
        }));

        let sender = Client::sender(client.clone(), write);
        let receiver = Client::receiver(client.clone(), read);
        let sender = tokio::spawn(sender);
        let receiver = tokio::spawn(receiver);

        tokio::time::sleep(Duration::from_secs(10)).await;
        sender.abort();
        receiver.abort();
    }

    async fn sender(client: Arc<Mutex<Client>>, mut write: SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
        {
            let mut client = client.lock().await;
            client.send_message(ClientMessage::Connected, &mut write).await
        }
        let mut client = client.lock().await;
        let message = ClientMessage::Query(Rect::from_center_and_size(Position::origin(), 1024, 1024));
        client.send_message(message, &mut write).await;
    }

    async fn send_message(&mut self, message: ClientMessage, write: &mut SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>) {
        eprintln!("sending {:?}", message);
        self.sent_messages.push(SentMessage {
            request: Request {
                message,
            },
        });
        let message = &self.sent_messages.last().unwrap().request.message;
        write.send(Message::Text(Utf8Bytes::from(serde_json::to_string(message).expect("couldn't serialize message")))).await.expect("TODO: panic message");
    }

    async fn receiver(client: Arc<Mutex<Client>>, read: SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>) {
        read.for_each(|message| async {
            match message {
                Ok(data) => {
                    let mut client = client.lock().await;
                    let decompression_started = Instant::now();
                    let mut decompression_finished = Instant::now();
                    let data_length = data.len();
                    let mut chunks_received = 0;
                    if let Ok(ServerMessageBundle(messages)) = ServerMessageBundle::from_compressed(&*data.into_data()) {
                        decompression_finished = Instant::now();
                        for message in messages {
                            match message {
                                ServerMessage::Chunk(chunk) => {
                                    client.outfile.write_all(chunk.tiles.bytes()).await.unwrap();
                                    client.outfile.write(&[255]).await.unwrap();
                                    chunks_received += 1;
                                }
                                _ => {}
                            }
                        }
                        client.outfile.flush().await.unwrap();
                    }
                    if chunks_received > 0 {
                        let time_taken = decompression_finished - decompression_started;
                        eprintln!("Received {chunks_received} chunks using {data_length} bytes");
                        let bytes_per_chunk = (data_length as f64)/(chunks_received as f64);
                        let bytes_per_tile = bytes_per_chunk/256.0;
                        let tiles_per_byte = 1.0/bytes_per_tile;
                        eprintln!("{tiles_per_byte} tiles per byte (more is better)");
                        eprintln!("Took {time_taken:?} (less is better)");
                    }
                }
                Err(err) => {
                    eprintln!("{:?}", err)
                }
            }
        }).await
    }
}
