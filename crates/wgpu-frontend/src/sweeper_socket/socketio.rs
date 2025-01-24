use socketio_local::io;
use socketio_local::SocketIo;
use std::collections::VecDeque;
use std::sync::mpsc;
use std::sync::mpsc::Receiver;
use wasm_bindgen::closure::Closure;
use web_sys::js_sys::{Array, Object, Uint8Array};
use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use world::{Chunk, Rect, World};
use crate::sweeper_socket::SweeperSocket;

pub struct IoWorld {
    world: World,
    socket: SocketIo,
    messages: VecDeque<ServerMessage>,
    on_packet: Closure<dyn FnMut(Object)>,
    packet_receiver: Receiver<Object>,
}

impl IoWorld {
    pub fn new(url: &str) -> Self {
        let socket = io(url);
        let (tx, rx) = mpsc::channel();
        let on_packet = Closure::new(move |data: Object| {
            tx.send(data).expect("TODO: panic message");
        });
        let result = Self {
            world: World::new(),
            socket,
            messages: VecDeque::new(),
            on_packet,
            packet_receiver: rx,
        };
        result.socket.io().engine().on("packet", &result.on_packet);
        result
    }

    fn send_message(&self, message: ClientMessage) {
        self.socket.emit("message", serde_wasm_bindgen::to_value(&message).unwrap());
    }

    fn update(&mut self) {
        for packet in self.packet_receiver.try_iter() {
            let entries: Vec<_> = Object::entries(&packet).iter().collect();
            for entry in entries {
                let array: Array = Array::from(&entry);
                if let Some(message_type) = array.get(0).as_string() {
                    if message_type != "data" {
                        continue
                    }
                } else {
                    continue
                }
                let data = Uint8Array::new(&array.get(1));
                if let Ok(message) = ServerMessage::from_compressed(data.to_vec()) {
                    self.messages.push_back(message);
                }
            }
        }
    }
}

impl SweeperSocket for IoWorld {
    fn send(&mut self, message: ClientMessage) {
        self.send_message(message)
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        self.update();
        self.messages.pop_front()
    }

    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk> {
        // We should query the server for chunks
        self.send_message(ClientMessage::Query(rect));
        self.world.query_chunks(&rect)
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}

