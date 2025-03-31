use serde::{Deserialize, Serialize};
use crate::Event;
use crate::Position;

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
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

    pub fn numeric_hash(player_id: &str, max: usize) -> usize {
        let mut result: usize = 0;
        let mut buf: [u8; 4] = Default::default();
        for char in player_id.chars() {
            result <<= 5;
            result += (char.encode_utf8(&mut buf).as_bytes()[0] & 0b11111) as usize;
            if result > max { break }
        }
        result % max
    }

    pub fn compress(&self, header: u8) -> Vec<u8> {
        let mut binary = vec![header];
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
