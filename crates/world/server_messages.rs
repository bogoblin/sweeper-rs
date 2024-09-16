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
    let binary = unsafe {String::from_utf8_unchecked(event.compress())};
    match event {
        Event::Clicked { player_id, at, updated } => {
            ("event", binary.into())
        }
        Event::DoubleClicked { player_id, at, updated } => {
            ("event", binary.into())
        }
        Event::Flag { player_id, at } => {
            ("event", binary.into())
        }
        Event::Unflag { player_id, at } => {
            ("event", binary.into())
        }
    }
}