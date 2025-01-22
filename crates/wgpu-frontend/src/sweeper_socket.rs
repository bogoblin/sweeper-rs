use world::client_messages::ClientMessage;
use log::info;
use world::{Chunk, Position, Rect, World};
use world::events::Event;
use world::server_messages::ServerMessage;

pub trait SweeperSocket {
    fn send(&mut self, message: ClientMessage);
    fn next_message(&mut self) -> Option<ServerMessage>;
    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk>;
    fn world(&mut self) -> &mut World;
}


#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;
#[cfg(target_arch = "wasm32")]
use serde_json::json;
#[cfg(target_arch = "wasm32")]
use socketio_local::{SocketIo};
#[cfg(target_arch = "wasm32")]
use socketio_local::io;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::closure::Closure;
#[cfg(target_arch = "wasm32")]
use std::collections::VecDeque;
#[cfg(target_arch = "wasm32")]
use std::sync::{Arc, Mutex};
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc;
#[cfg(target_arch = "wasm32")]
use std::sync::mpsc::Receiver;
#[cfg(target_arch = "wasm32")]
use web_sys::js_sys::{Object, Array, ArrayBuffer, DataView, Uint8Array};
#[cfg(target_arch = "wasm32")]
use once_cell::unsync::Lazy;

#[cfg(target_arch = "wasm32")]
pub struct IoWorld {
    world: World,
    socket: SocketIo,
    events: Vec<Event>,
    chunks: VecDeque<Chunk>,
    messages: VecDeque<ServerMessage>,
    next_event: usize,
    on_packet: Closure<dyn FnMut(Object)>,
    packet_receiver: Receiver<Object>,
}

#[cfg(target_arch = "wasm32")]
impl IoWorld {
    pub fn new(url: &str) -> Self {
        let socket = io(url);
        let (tx, rx) = mpsc::channel();
        let on_packet = Closure::new(move |data: Object| {
            tx.send(data).expect("TODO: panic message");
        });
        let mut result = Self {
            world: World::new(),
            socket,
            events: vec![],
            chunks: VecDeque::new(),
            messages: VecDeque::new(),
            next_event: 0,
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

#[cfg(target_arch = "wasm32")]
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
        self.send_message(ClientMessage::QueryChunks(rect));
        self.world.query_chunks(&rect)
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}

pub struct LocalWorld {
    world: World,
}

impl LocalWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
}

impl SweeperSocket for LocalWorld {
    fn send(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::Connected => {}
            ClientMessage::Click(position) => { self.world.click(position, "") }
            ClientMessage::Flag(position) => { self.world.flag(position, "") }
            ClientMessage::DoubleClick(position) => { self.world.double_click(position, "") }
            ClientMessage::QueryChunks(_) => {}
        }
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        if let Some(event) = self.world.events.pop_front() {
            Some(ServerMessage::Event(event))
        } else {
            None
        }
    }

    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk> {
        let chunks_to_get = rect.chunks_contained();
        chunks_to_get.iter()
            .map(|chunk_position| self.world.get_chunk(chunk_position.position()))
            .filter_map(|v| v)
            .collect()
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}