use serde_json::{json, Value};
use crate::{Chunk};
use crate::events::Event;

pub fn chunk_message(chunk: &Chunk) -> (&'static str, Value) {
    let coords = chunk.position;
    let tiles = chunk.tiles.0.to_vec();
    ("chunk", json!({
                    "coords": [coords.0, coords.1],
                    "tiles": tiles,
                }))
}

pub fn from_event(event: &Event) -> (&'static str, Value) {
    match event {
        Event::Clicked { player_id, at, updated } => {
            ("click", json!({
                "position": at,
                "player_id": player_id,
                "updated_rect": updated,
            }))
        }
        Event::DoubleClicked { player_id, at, updated } => {
            ("click", json!({
                "position": at,
                "player_id": player_id,
                "updated_rect": updated,
            }))
        }
        Event::Flag { player_id, at } => {
            ("flag", json!({
                "position": at,
                "player_id": player_id,
            }))
        }
        Event::Unflag { player_id, at } => {
            ("unflag", json!({
                "position": at,
                "player_id": player_id,
            }))
        }
    }
}