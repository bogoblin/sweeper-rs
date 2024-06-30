use serde_json::{json, Value};
use crate::{Chunk, Position, UpdatedRect};
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

pub fn updated_rect_message(updated_rect: &UpdatedRect) -> (&'static str, Value) {
    ("updated_rect", json!({
        "topLeft": updated_rect.top_left,
        "updated": updated_rect.updated,
    }))
}

pub fn flag_message(position: &Position) -> (&'static str, Value) {
    ("flag", json!({
        "position": position
    }))
}

pub fn unflag_message(position: &Position) -> (&'static str, Value) {
    ("unflag", json!({
        "position": position
    }))
}
