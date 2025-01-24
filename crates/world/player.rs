use serde::{Deserialize, Serialize};
use crate::events::Event;
use crate::Position;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Player {
    pub player_id: String,
    pub position: Position,
}

impl Player {
    pub fn new(player_id: String) -> Self {
        Self {
            player_id,
            position: Position::origin(),
        }
    }
    
    pub fn update(&mut self, event: &Event) {
        match event {
            Event::Clicked { at, .. } |
            Event::DoubleClicked { at, .. } |
            Event::Flag { at, .. } |
            Event::Unflag { at, .. } => {
                self.position = at.clone();
            }
        }
    }
    
    pub fn numeric_hash(&self, max: usize) -> usize {
        let mut result: usize = 0;
        for (i, char) in self.player_id.chars().enumerate() {
            result += (char.to_digit(1 << 5).unwrap_or(0) * (1 << (5*i))) as usize;
            if result > max { break }
        }
        result % max
    }
    
    pub fn compress(&self, header: &str) -> Vec<u8> {
        let mut binary = vec![];
        binary.append(&mut header.as_bytes().to_vec());
        binary.append(&mut self.position.0.to_be_bytes().to_vec());
        binary.append(&mut self.position.1.to_be_bytes().to_vec());
        binary.append(&mut self.player_id.clone().into_bytes());
        binary
    }

    pub fn from_compressed(compressed: Vec<u8>) -> Option<Player> {
        let header = String::from_utf8_lossy(&compressed[0..=0]);
        if header != "p" && header != "w" {
            return None;
        }
        let mut index = 1;
        let position = {
            let x = i32::from_be_bytes(*compressed[index..].first_chunk()?);
            let y = i32::from_be_bytes(*compressed[index+4..].first_chunk()?);
            index += 8;
            Position(x, y)
        };
        let player_id = String::from_utf8_lossy(&compressed[index..]).to_string();

        Some(Player {
            player_id,
            position
        })
    }
}
