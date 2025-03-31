use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::ops::{Index, IndexMut};
use serde::de::{Error, Visitor};
use std::fmt::Formatter;
use bytes_cast::BytesCast;
use crate::tile::Tile;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct ChunkTiles(pub [Tile; 256]);

impl ChunkTiles {
    pub fn from(bytes: [u8; 256]) -> Self {
        Self(bytes.map(Tile))
    }

    pub fn bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Index<u8> for ChunkTiles {
    type Output = Tile;

    fn index(&self, index: u8) -> &Self::Output {
        // This is safe because u8 is an int from 0-255, which is always in bounds
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<u8> for ChunkTiles {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

impl Serialize for ChunkTiles {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer
    {
        serializer.serialize_bytes(self.0.map(|t| t.0).as_slice())
    }
}

struct ChunkTileVisitor;

impl Visitor<'_> for ChunkTileVisitor {
    type Value = ChunkTiles;

    fn expecting(&self, _formatter: &mut Formatter) -> std::fmt::Result {
        todo!()
    }

    fn visit_bytes<E>(self, bytes: &[u8]) -> Result<Self::Value, E>
    where
        E: Error,
    {
        if let Some(&chunk_bytes) = bytes.first_chunk::<256>() {
            Ok(Self::Value::from(chunk_bytes))
        } else {
            Err(Error::invalid_length(bytes.len(), &self))
        }
    }
}

impl<'de> Deserialize<'de> for ChunkTiles {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>
    {
        let visitor = ChunkTileVisitor{};
        deserializer.deserialize_bytes(visitor)
    }
}