use serde_json::{json, Value};
use crate::{Chunk, ChunkPosition, Position};
use crate::events::Event;

pub fn chunk_message(chunk: &Chunk) -> (&'static str, Value) {
    let coords = chunk.position;
    let tiles = chunk.tiles.0.to_vec();
    ("chunk", json!({
                    "coords": [coords.0, coords.1],
                    "tiles": tiles,
                }))
}

// ServerMessage is anything the server sends that gets compressed to bytes
pub enum ServerMessage<'a> {
    Event(&'a Event),
    Chunk(&'a Chunk)
}

impl <'a> From<ServerMessage<'a>> for Vec<u8> {
    fn from(value: ServerMessage<'a>) -> Self {
        match value {
            ServerMessage::Event(event) => {
                event.compress()
            }
            ServerMessage::Chunk(chunk) => {
                chunk.compress()
            }
        }
    }
}

impl Chunk {
    pub fn compress(&self) -> Vec<u8> {
        let mut result = vec![];
        result.append(&mut "h".as_bytes().to_vec());
        result.append(&mut self.position.to_bytes());
        result
    }
}

impl ChunkPosition {
    /// Compresses a chunk position down to 7 bytes, because chunks are always aligned to the 16x16
    /// grid, so the last 4 bits are always 0, so we don't need to send those!
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut result = vec![];
        let (x_bytes, y_bytes) = (self.0.to_be_bytes(), self.1.to_be_bytes());
        // Okay to unwrap() here because self.* will always be 4 bytes
        let (&last_x, first_3_x) = x_bytes.split_last().unwrap();
        let (&last_y, first_3_y) = y_bytes.split_last().unwrap();
        let last_byte = last_x + (last_y >> 4);
        result.extend_from_slice(first_3_x);
        result.extend_from_slice(first_3_y);
        result.push(last_byte);
        result
    }

    /// ```
    /// use world::ChunkPosition;
    /// let cp = ChunkPosition::new(1600, -3264);
    /// assert_eq!(ChunkPosition::from_bytes(cp.to_bytes()), Some(cp));
    /// ```
    pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
        let mut sections = bytes.chunks(3);
        let first_3_x = sections.next()?;
        let first_3_y = sections.next()?;
        let &last_byte = sections.next()?.first()?;
        let last_x = last_byte & 0b11110000;
        let last_y = last_byte << 4;

        let mut x_bytes = vec![];
        x_bytes.extend_from_slice(first_3_x);
        x_bytes.push(last_x);
        let mut y_bytes = vec![];
        y_bytes.extend_from_slice(first_3_y);
        y_bytes.push(last_y);

        Some(Self(
            i32::from_be_bytes(x_bytes.try_into().ok()?),
            i32::from_be_bytes(y_bytes.try_into().ok()?)
        ))
    }
}