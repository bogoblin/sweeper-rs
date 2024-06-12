use std::time::Instant;
use crate::Position;

#[derive(Debug)]
pub struct Player {
    pub position: Position,
    pub stats_revealed: [u32; 9],
    pub stats_flags_correct: i32,
    pub stats_flags_incorrect: i32,
    pub deaths: Vec<Instant>
}

impl Player {
    pub fn new() -> Self {
        Self {
            position: Position::origin(),
            stats_revealed: [0; 9],
            stats_flags_correct: 0,
            stats_flags_incorrect: 0,
            deaths: vec![],
        }
    }

    pub fn kill(&mut self) {
        self.deaths.push(Instant::now())
    }
}