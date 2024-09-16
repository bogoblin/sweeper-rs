use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use web_sys::js_sys::{Object, Uint8Array};
use world::events::Event;

#[wasm_bindgen]
pub fn decompress(compressed: Uint8Array) -> JsValue {
    if let Some(event) = Event::from_compressed(compressed.to_vec()) {
        match serde_json::to_string(&event) {
            Ok(json) => JsValue::from_str(&json),
            Err(err) => JsValue::from_str(&err.to_string()),
        }
    } else {
        Object::new().into()
    }
}
