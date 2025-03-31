use std::ops::{Deref, DerefMut};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy)]
#[derive(Serialize, Deserialize)]
pub struct PositionInChunk(u8);

impl PositionInChunk {
    pub fn new(x: i32, y:i32) -> Self {
        Self(((x & 0b1111) + ((y & 0b1111) << 4)) as u8)
    }

    pub fn first() -> Self {
        Self(0)
    }

    pub fn from_index(index: u8) -> Self {
        Self(index)
    }

    pub fn x(&self) -> u8 {
        self.0 & 0b1111
    }

    pub fn y(&self) -> u8 {
        (self.0 >> 4) & 0b1111
    }

    pub fn next(&self) -> Option<Self> {
        Self(self.checked_add(1)?).into()
    }
}

impl Deref for PositionInChunk {
    type Target = u8;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for PositionInChunk {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}