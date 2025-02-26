use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, SendError, Sender};
use log::{error, info};
use websocket_lite::{Client, ClientBuilder, Error, Message, NetworkStream};
use url::Url;
use world::client_messages::ClientMessage;
use world::{World};
use world::server_messages::{ServerMessage, ServerMessageError};
use crate::sweeper_socket::SweeperSocket;

pub struct WebSocketWorld {
    world: World,
    send_queue: VecDeque<ClientMessage>,
    receive_queue: VecDeque<ServerMessage>,
    connection: ConnectionState,
}

impl WebSocketWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            send_queue: Default::default(),
            receive_queue: Default::default(),
            connection: ConnectionState::Disconnected,
        }
    }
}

enum ConnectionState {
    Connected(Connection),
    Disconnected,
}

struct Connection {
    client_tx: Sender<ClientMessage>,
    server_rx: Receiver<WebSocketMessage>,
}

enum WebSocketMessage {
    Disconnect,
    Message(ServerMessage)
}

impl Connection {
    pub fn receive(&mut self) -> Option<WebSocketMessage> {
        match self.server_rx.try_recv() {
            Ok(message) => {Some(message)}
            Err(_) => {None}
        }
    }

    pub fn send(&mut self, message: ClientMessage) -> Result<(), SendError<ClientMessage>> {
        self.client_tx.send(message)
    }

    pub fn new() -> Self {
        // This implementation is kinda shitty but I'm not targetting desktop so I don't think it's worth improving
        info!("Connecting to websocket...");
        
        let (client_tx, client_rx) = mpsc::channel();
        let (server_tx, server_rx) = mpsc::channel();
        
        let (tx, rx) = mpsc::channel();
        
        std::thread::spawn(move || {
            let mut ws = ClientBuilder::from_url(Url::parse("ws://64.23.244.167/ws").unwrap())
                .connect().unwrap();
            
            loop {
                if let Ok(message) = client_rx.recv() {
                    ws.send(Message::text(serde_json::to_string(&message).unwrap().to_string()));
                    tx.send(message);
                }
            }
        });
        
        std::thread::spawn(move || {
            let mut ws = ClientBuilder::from_url(Url::parse("ws://64.23.244.167/ws").unwrap())
                .connect().unwrap();

            loop {
                loop {
                    match &rx.try_recv() {
                        Ok(message) => {
                            ws.send(Message::text(serde_json::to_string(&message).unwrap().to_string()));
                        }
                        Err(_) => {break}
                    }
                }
                if let Ok(message) = ws.receive() {
                    if let Some(message) = message {
                        match ServerMessage::from_compressed(message.data().to_vec()) {
                            Ok(message) => {
                                server_tx.send(WebSocketMessage::Message(message));
                            }
                            Err(error) => {
                                error!("Bad message: {:?}", error);
                            }
                        }
                    } else {
                    }
                }
            }
        });

        Self {
            client_tx,
            server_rx,
        }
    }
}

impl SweeperSocket for WebSocketWorld {
    fn send(&mut self, message: ClientMessage) {
        self.send_queue.push_back(message);
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        match &mut self.connection {
            ConnectionState::Connected(connection) => {
                // Send any messages in the queue:
                while let Some(message) = self.send_queue.pop_front() {
                    match connection.send(message) {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                }

                if let Some(message) = connection.receive() {
                    match message {
                        WebSocketMessage::Disconnect => {
                            self.connection = ConnectionState::Disconnected;
                            None
                        }
                        WebSocketMessage::Message(message) => {
                            match message {
                                ServerMessage::Connected => {
                                    self.send(ClientMessage::Connected);
                                }
                                _ => {}
                            }
                            Some(message)
                        }
                    }
                } else { None }
            }
            ConnectionState::Disconnected => {
                self.connection = ConnectionState::Connected(Connection::new());
                info!("Disconnected from web socket");
                None
            }
        }

    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}