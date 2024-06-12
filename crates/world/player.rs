use crate::Position;

pub struct Player {
    pub position: Position,
    pub stats_revealed: [u32; 9],
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Position::origin(),
            stats_revealed: [0; 9],
        }
    }
}