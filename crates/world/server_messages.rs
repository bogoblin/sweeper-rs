use serde_json::{json, Value};
use crate::{Chunk};
use crate::player::Player;

pub fn chunk_message(chunk: &Chunk) -> (&'static str, Value) {
    let coords = chunk.position;
    let tiles = chunk.tiles.to_vec();
    ("chunk", json!({
                    "coords": [coords.0, coords.1],
                    "tiles": tiles,
                }))
}

pub fn player_message(player: &Player) -> (&'static str, Value) {
    ("player", json!({
        "username": player.username,
        "lastClick": player.last_clicked
    }))
}