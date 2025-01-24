use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use world::{Chunk, Rect, World};

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub mod socketio;
    } else {
        pub mod local;
    }
}

pub trait SweeperSocket {
    fn send(&mut self, message: ClientMessage);
    fn next_message(&mut self) -> Option<ServerMessage>;
    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk>;
    fn world(&mut self) -> &mut World;
}

