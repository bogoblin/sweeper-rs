use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Uint8Array};
use world::server_messages::ServerMessage;

#[wasm_bindgen]
pub fn decompress(compressed: Uint8Array) -> JsValue {
    match ServerMessage::from_compressed(compressed.to_vec()) {
        None => Object::new().into(),
        Some(ServerMessage::Event(event)) => {
            match serde_json::to_string(&event) {
                Ok(json) => JsValue::from_str(&json),
                Err(err) => JsValue::from_str(&err.to_string()),
            }
        }
        Some(ServerMessage::Chunk(chunk)) => {
            match serde_json::to_string(&chunk) {
                Ok(json) => JsValue::from_str(&json),
                Err(err) => JsValue::from_str(&err.to_string()),
            }
        }
    }
}
