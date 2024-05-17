use serde::{Serialize, Serializer};
use serde_json::{json, Value};
use crate::world::{Chunk, Position};

pub fn chunk_message(chunk: &Chunk) -> (&'static str, Value) {
    let coords = chunk.position;
    let mut tiles = Vec::new();
    for y in 0..16 {
        for x in 0..16 {
            tiles.push(chunk.get_tile(Position(x, y)))
        }
    }
    ("chunk", json!({
                    "coords": [coords.0, coords.1],
                    "tiles": tiles,
                }))
}
