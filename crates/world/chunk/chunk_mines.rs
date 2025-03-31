use rand::prelude::{IteratorRandom, StdRng};
use std::ops::{Deref, DerefMut};
use bitvec::order::Lsb0;
use serde::{Deserialize, Serialize};
use bitvec::prelude::BitSlice;
use bitvec::view::BitView;
use crate::{Chunk, ChunkPosition, PositionInChunk};
use crate::tile::Tile;

#[derive(Serialize, Deserialize)]
#[derive(Default, Clone, Debug)]
pub struct ChunkMines ([u8; 32]);

impl ChunkMines {
    pub fn random(number_of_mines: u8, mut rng: StdRng) -> Self {
        let mut result = Self::default();
        for mine_index in (0..255).choose_multiple(&mut rng, number_of_mines as usize) {
            result.set(mine_index, true);
        }
        result
    }

    pub fn positions(&self) -> Vec<PositionInChunk> {
        let n_ones = self.count_ones();
        let mut result = Vec::with_capacity(n_ones);
        for index in self.iter_ones() {
            result.push(PositionInChunk::from_index(index as u8))
        }
        result
    }

    pub fn to_chunk(&self, position: ChunkPosition) -> Chunk {
        let mut new_chunk = Chunk::empty(position);
        for index in self.iter_ones() {
            new_chunk.tiles.0[index] = Tile::mine();
        }
        new_chunk
    }
}

impl Deref for ChunkMines {
    type Target = BitSlice<u8>;

    fn deref(&self) -> &Self::Target {
        self.0.view_bits::<Lsb0>()
    }
}

impl DerefMut for ChunkMines {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.view_bits_mut::<Lsb0>()
    }
}

impl TryFrom<Vec<u8>> for ChunkMines {
    type Error = ();

    fn try_from(value: Vec<u8>) -> Result<Self, Self::Error> {
        let mines: [u8; 32] = *value.first_chunk().ok_or(())?;
        Ok(Self(mines))
    }
}

impl AsRef<[u8]> for ChunkMines {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}