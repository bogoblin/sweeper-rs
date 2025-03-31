use world::ClientMessage;
use world::ServerMessage;
use world::World;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        pub mod js_websocket;
    } else {
        pub mod local;
    }
}

pub trait SweeperSocket {
    fn send(&mut self, message: ClientMessage);
    fn next_message(&mut self) -> Option<ServerMessage>;
    fn world(&mut self) -> &mut World;
}

