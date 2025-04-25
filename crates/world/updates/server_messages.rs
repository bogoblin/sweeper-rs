use quickcheck::{Arbitrary, Gen};
use crate::PublicTile;
use crate::player::Player;
use crate::{Chunk, ChunkPosition, ChunkTiles, Event, Position, Tile, UpdatedRect};
use huffman::{BitWriter, HuffmanCode};
use serde::{Deserialize, Serialize};

// ServerMessage is anything the server sends that gets compressed to bytes
#[repr(u8)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
#[derive(PartialEq)]
pub enum ServerMessage {
    Event(Event) = b'e',
    Chunk(Chunk) = b'h',
    Rect(UpdatedRect) = b'r',
    Player(Player) = b'p',
    Welcome(Player) = b'w',
    Disconnected(String) = b'x',
    Connected = b'+',
}

impl ServerMessage {
    fn header(&self) -> u8 {
        // We're using the discriminant as the header, so this unsafe code gets that:
        unsafe { *<*const _>::from(self).cast::<u8>() }
    }
}

impl From<&ServerMessage> for Vec<u8> {
    fn from(value: &ServerMessage) -> Self {
        let header = value.header();
        match value {
            ServerMessage::Event(event) => {
                event.compress()
            }
            ServerMessage::Chunk(chunk) => {
                chunk.compress()
            }
            ServerMessage::Rect(rect) => {
                let mut result = vec![header];
                result.append(&mut rect.into());
                result
            }
            ServerMessage::Player(player) |
            ServerMessage::Welcome(player) => {
                player.compress(header)
            }
            ServerMessage::Disconnected(player_id) => {
                let mut result = vec![];
                result.append(&mut "x".as_bytes().to_vec());
                result.append(&mut player_id.as_bytes().to_vec());
                result
            },
            ServerMessage::Connected => vec![],
        }
    }
}

#[derive(Debug)]
pub enum ServerMessageError {
    BadChunk,
    BadEvent,
    BadPlayer,
    BadTile,
    BadRect,
}

impl ServerMessage {
    pub fn from_compressed(compressed: &[u8]) -> Result<ServerMessage, ServerMessageError> {
        if compressed.is_empty() {
            return Err(ServerMessageError::BadEvent)
        }
        let header = String::from_utf8_lossy(&compressed[0..=0]);
        if header == "h" {
            match Chunk::from_compressed(compressed) {
                Some(chunk) => Ok(ServerMessage::Chunk(chunk)),
                None => Err(ServerMessageError::BadChunk)
            }
        }
        else if header == "r" {
            match UpdatedRect::from_compressed(&compressed[1..]) {
                Some(rect) => Ok(ServerMessage::Rect(rect)),
                None => Err(ServerMessageError::BadRect)
            }
        }
        else if header == "p" {
            match Player::from_compressed(compressed) {
                Some(player) => Ok(ServerMessage::Player(player)),
                None => Err(ServerMessageError::BadPlayer)
            }
        }
        else if header == "w" {
            match Player::from_compressed(compressed) {
                Some(player) => Ok(ServerMessage::Welcome(player)),
                None => Err(ServerMessageError::BadPlayer)
            }
        }
        else if header == "x" { 
            let player_id = String::from_utf8_lossy(&compressed[1..]);
            Ok(ServerMessage::Disconnected(player_id.into()))
        }
        else {
            match Event::from_compressed(&compressed) {
                Some(event) => Ok(ServerMessage::Event(event)),
                None => Err(ServerMessageError::BadEvent)
            }
        }
    }
}

impl Arbitrary for ServerMessage {
    fn arbitrary(g: &mut Gen) -> Self {
        match u32::arbitrary(g)%5 {
            1 => Self::Disconnected(String::arbitrary(g)),
            2 => Self::Rect(UpdatedRect::arbitrary(g)),
            3 => Self::Connected,
            _ => Self::Chunk(Chunk::arbitrary(g))
        }
    }
}

impl Chunk {
    pub fn compress(&self) -> Vec<u8> {
        let mut result = vec![];
        result.append(&mut "h".as_bytes().to_vec());
        result.append(&mut self.position.to_bytes());
        let mut bw = BitWriter::new();
        for tile in self.public_tiles() {
            tile.encode(&mut bw);
        }
        result.append(&mut bw.to_bytes());
        result
    }

    pub fn public_tiles(&self) -> Vec<PublicTile> {
        self.tiles.0.iter().map(|t| {
            t.into()
        }).collect::<Vec<PublicTile>>()
    }

    /// ```
    /// use world::*;
    /// let mut world = World::new();
    /// let position = Position(16, 16);
    /// let chunk_id = world.generate_chunk(position.clone());
    /// world.generate_surrounding_chunks(position.clone());
    /// world.click(Position(17, 17), "player");
    /// let chunk = world.get_chunk(position).unwrap();
    /// let compressed = chunk.compress();
    /// let decompressed = Chunk::from_compressed(&compressed.clone()).unwrap();
    /// assert_eq!(decompressed.public_tiles().len(), 256);
    /// for (decompressed_tile, tile) in decompressed.public_tiles().iter().zip(chunk.public_tiles()) {
    ///     assert_eq!(decompressed_tile.clone(), tile);
    /// }
    /// ```
    pub fn from_compressed(compressed: &[u8]) -> Option<Self> {
        let position = ChunkPosition::from_bytes(compressed[1..8].to_vec())?;
        let tiles = PublicTile::from_huffman_bytes(compressed[8..].to_vec());
        Some(Chunk::from_position_and_tiles(position, tiles.into()))
    }
}

impl From<Vec<Box<PublicTile>>> for ChunkTiles {
    fn from(tiles: Vec<Box<PublicTile>>) -> Self {
        let mut result = Self([Tile::empty(); 256]);
        for i in 0..256 {
            if let Some(tile) = tiles.get(i) {
                result.0[i] = tile.as_ref().clone().into();
            }
        }
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

impl UpdatedRect {
    pub fn from_compressed(compressed: &[u8]) -> Option<Self> {
        let top_left = Position::from_compressed(compressed)?;
        let mut updated = UpdatedRect::empty_at(top_left);

        let index = 8;
        let tiles = PublicTile::from_huffman_bytes(compressed[index..].to_vec());

        for tile in tiles {
            match *tile {
                PublicTile::Newline => updated.push_newline(),
                tile => updated.push(tile.into()),
            }
        }
        
        Some(updated)
    }
}

#[cfg(test)]
mod tests {
    use quickcheck::quickcheck;
    use crate::ServerMessage;

    quickcheck! {
        fn compression_then_decompression(message: ServerMessage) -> bool {
            let compressed: Vec<u8> = (&message).into();
            if let Ok(decompressed) = ServerMessage::from_compressed(&compressed) {
                return decompressed == message;
            }
            false
        }
    }
}