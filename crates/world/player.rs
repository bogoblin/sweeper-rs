use serde::{Deserialize, Serialize};

use crate::Position;

#[derive(Debug)]
#[derive(Serialize, Deserialize)]
pub struct Player {
    pub position: Position,
    pub username: String,
    pub last_clicked: Position,
}

impl Player {
    pub fn new(username: String) -> Self {
        Self {
            position: Position::origin(),
            username,
            last_clicked: Position::origin(),
        }
    }

    pub fn kill(&mut self) {
    }
}