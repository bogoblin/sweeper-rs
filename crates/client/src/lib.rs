pub mod chunk_quadtree;

use crate::chunk_quadtree::ChunkQuadtree;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;
use web_sys::js_sys::{Array, JsString, Number, Object, Uint8Array};
use world::server_messages::ServerMessage;
use world::ChunkPosition;

#[wasm_bindgen]
pub fn decompress(compressed: Uint8Array) -> JsValue {
    match ServerMessage::from_compressed(compressed.to_vec()) {
        Err(_) => Object::new().into(),
        Ok(ServerMessage::Event(event)) => {
            match serde_json::to_string(&event) {
                Ok(json) => JsValue::from_str(&json),
                Err(err) => JsValue::from_str(&err.to_string()),
            }
        }
        Ok(ServerMessage::Chunk(chunk)) => {
            match serde_json::to_string(&chunk) {
                Ok(json) => JsValue::from_str(&json),
                Err(err) => JsValue::from_str(&err.to_string()),
            }
        }
    }
}

#[wasm_bindgen]
pub fn quadtree_create() -> ChunkQuadtree {
    ChunkQuadtree::new()
}

#[wasm_bindgen]
pub fn quadtree_insert(quadtree: &mut ChunkQuadtree, x: Number, y: Number) {
    quadtree.insert(js_to_chunk_position(x, y));
}

/// Returns an array of strings, which is how we lookup chunks in the javascript
#[wasm_bindgen]
pub fn quadtree_query(quadtree: &ChunkQuadtree, x_min: Number, y_min: Number, x_max: Number, y_max: Number) -> Array {
    let x_min = x_min.as_f64().unwrap_or(0f64).floor() as i32;
    let y_min = y_min.as_f64().unwrap_or(0f64).floor() as i32;
    let x_max = x_max.as_f64().unwrap_or(0f64).floor() as i32;
    let y_max = y_max.as_f64().unwrap_or(0f64).floor() as i32;
    
    let (x_min, x_max) = if x_min < x_max {
        (x_min, x_max)
    } else {
        (x_max, x_min)
    };
    let (y_min, y_max) = if y_min < y_max {
        (y_min, y_max)
    } else {
        (y_max, y_min)
    };

    Array::from_iter(
        quadtree.query(ChunkPosition::new(x_min, y_min), ChunkPosition::new(x_max, y_max)).iter()
            .map(|ChunkPosition(x, y)| {
                JsString::from(format!("{x},{y}"))
            }))
}

#[wasm_bindgen_test]
fn quadtree_query_test() {
    let mut quadtree = quadtree_create();
    quadtree_insert(&mut quadtree, 16.into(), 32.into());
    let query = quadtree_query(&quadtree, 0.into(), 0.into(), 64.into(), 64.into());
    assert_eq!(query.get(0).as_string().unwrap(), "16,32");
}

fn js_to_chunk_position(x: Number, y: Number) -> ChunkPosition {
    let x = x.as_f64().unwrap_or(0f64).floor() as i32;
    let y = y.as_f64().unwrap_or(0f64).floor() as i32;
    ChunkPosition::new(x, y)
}

