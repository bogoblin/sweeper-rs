use log::info;
use world::{Chunk, Position, Rect, World};
use world::events::Event;
use world::server_messages::ServerMessage;

pub trait SweeperSocket {
    fn update(&mut self);
    fn click(&mut self, position: Position);
    fn double_click(&mut self, position: Position);
    fn flag(&mut self, position: Position);
    fn next_message(&mut self) -> Option<&Event>;
    fn next_chunk(&mut self) -> Option<&Chunk>;
    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk>;
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
use world::client_messages::ClientMessage;

#[cfg(target_arch = "wasm32")]
pub struct IoWorld {
    world: World,
    socket: SocketIo,
    events: Vec<Event>,
    chunks: VecDeque<Chunk>,
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
            info!("{:?}", data);
            tx.send(data).expect("TODO: panic message");
        });
        let mut result = Self {
            world: World::new(),
            socket,
            events: vec![],
            chunks: VecDeque::new(),
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
    
}

#[cfg(target_arch = "wasm32")]
impl SweeperSocket for IoWorld {
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
                    match message {
                        ServerMessage::Event(event) => {
                            self.events.push(event);
                        }
                        ServerMessage::Chunk(chunk) => {
                            info!("added chunk: {:?}", chunk.position);
                            self.chunks.push_back(chunk);
                        }
                    }
                } else {
                    info!("bad message");
                }
            }
        }
    }

    fn click(&mut self, position: Position) {
        self.send_message(ClientMessage::Click(position))
    }

    fn double_click(&mut self, position: Position) {
        self.send_message(ClientMessage::DoubleClick(position))
    }

    fn flag(&mut self, position: Position) {
        self.send_message(ClientMessage::Flag(position))
    }

    fn next_message(&mut self) -> Option<&Event> {
        if let Some(event) = self.events.get(self.next_event) {
            self.next_event += 1;
            Some(event)
        } else {
            None
        }
    }

    fn next_chunk(&mut self) -> Option<&Chunk> {
        if let Some(chunk) = self.chunks.pop_front() {
            let chunk_position = chunk.position.position();
            self.world.insert_chunk(chunk);
            self.world.get_chunk(chunk_position)
        } else {
            None
        }
    }

    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk> {
        // We should query the server for chunks
        self.send_message(ClientMessage::QueryChunks(rect));
        self.world.query_chunks(&rect)
    }
}

pub struct LocalWorld {
    world: World,
    next_event: usize,
}

impl LocalWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            next_event: 0,
        }
    }
}

impl SweeperSocket for LocalWorld {
    fn update(&mut self) {
    }

    fn click(&mut self, position: Position) {
        self.world.click(position, "")
    }

    fn double_click(&mut self, position: Position) {
        self.world.double_click(position, "")
    }

    fn flag(&mut self, position: Position) {
        self.world.flag(position, "")
    }

    fn next_message(&mut self) -> Option<&Event> {
        if let Some(event) = self.world.events.get(self.next_event) {
            self.next_event += 1;
            Some(event)
        } else {
            None
        }
    }

    fn next_chunk(&mut self) -> Option<&Chunk> {
        None
    }

    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk> {
        let chunks_to_get = rect.chunks_contained();
        chunks_to_get.iter()
            .map(|chunk_position| self.world.get_chunk(chunk_position.position()))
            .filter_map(|v| v)
            .collect()
    }
}