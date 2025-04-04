pub mod interface;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "wasm32")] {
        mod js_websocket;
        pub use js_websocket::WebSocketWorld as SocketWorld;
    } else {
        pub mod local;
        pub use local::LocalWorld as SocketWorld;
    }
}