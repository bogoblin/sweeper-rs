use crate::{Position, Tile, UpdatedRect, UpdatedTile};
use serde::{Deserialize, Serialize};
use std::i32;
use quickcheck::{Arbitrary, Gen};
use crate::player::Player;

#[derive(Debug, Clone)]
#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
pub enum Event {
    Clicked {
        player_id: String,
        at: Position,
        updated: UpdatedRect,
    },
    DoubleClicked {
        player_id: String,
        at: Position,
        updated: UpdatedRect,
    },
    Flag {
        player_id: String,
        at: Position,
    },
    Unflag {
        player_id: String,
        at: Position,
    },
}

impl Event {
    pub fn updated_rect(&self) -> UpdatedRect {
        match self {
            Event::Clicked { updated, .. } |
            Event::DoubleClicked { updated, .. } => {
                updated.clone()
            }
            Event::Flag { at, .. } => {
                UpdatedRect::new(vec![UpdatedTile {
                    position: at.clone(),
                    tile: Tile::empty().with_flag()
                }])
            }
            Event::Unflag { at, .. } => {
                UpdatedRect::new(vec![UpdatedTile {
                    position: at.clone(),
                    tile: Tile::empty()
                }])
            }
        }
    }
    
    pub fn player(&self) -> Player {
        match self {
            Event::Clicked { player_id, at, .. } |
            Event::DoubleClicked { player_id, at, .. } |
            Event::Flag { player_id, at, .. } |
            Event::Unflag { player_id, at, .. } => {
                Player {
                    player_id: player_id.clone(),
                    position: at.clone()
                }
            }
        }
    }
    
    pub fn should_send(&self) -> bool {
        true
    }
}
impl Event {
    pub fn compress(&self) -> Vec<u8> {
        let mut binary = vec![];
        let (header, player_id, at, updated) = match self {
            Event::Clicked { player_id, at, updated } => {
                ("C", player_id, at, Some(updated))
            }
            Event::DoubleClicked { player_id, at, updated } => {
                ("D", player_id, at, Some(updated))
            }
            Event::Flag { player_id, at } => {
                ("F", player_id, at, None)
            }
            Event::Unflag { player_id, at } => {
                ("U", player_id, at, None)
            }
        };

        binary.append(&mut header.as_bytes().to_vec());
        binary.append(&mut player_id.as_bytes().to_vec());
        binary.push(0);
        binary.append(&mut at.0.to_be_bytes().to_vec());
        binary.append(&mut at.1.to_be_bytes().to_vec());
        if let Some(updated) = updated {
            binary.append(&mut updated.into());
        }
        
        binary
    }
    
    pub fn from_compressed(compressed: &[u8]) -> Option<Event> {
        let header = String::from_utf8_lossy(&compressed[0..=0]);
        let mut index = 1;
        loop {
            let char = compressed.get(index)?;
            index += 1;
            if *char == 0 {
                break
            }
        }
        let player_id = String::from_utf8_lossy(&compressed[1..index-1]).to_string();
        let at = {
            let x = i32::from_be_bytes(*compressed[index..].first_chunk()?);
            let y = i32::from_be_bytes(*compressed[index+4..].first_chunk()?);
            index += 8;
            Position(x, y)
        };

        if header == "C" {
            UpdatedRect::from_compressed(&compressed[index..])
                .map(|updated| {
                    Event::Clicked { player_id, at, updated }
                })
        } else if header == "D" {
            UpdatedRect::from_compressed(&compressed[index..])
                .map(|updated| {
                    Event::DoubleClicked { player_id, at, updated }
                })
        } else if header == "F" {
            Some(Event::Flag { player_id, at })
        } else if header == "U" {
            Some(Event::Unflag { player_id, at })
        } else {
            None
        }
    }
}

impl Arbitrary for Event {
    fn arbitrary(g: &mut Gen) -> Self {
        let player_id = String::from("alfie");
        let at = Position::arbitrary(g);
        match u8::arbitrary(g) % 4 {
            0 => Event::Clicked {
                player_id,
                at,
                updated: UpdatedRect::arbitrary(g)
            },
            1 => Event::DoubleClicked {
                player_id,
                at,
                updated: UpdatedRect::arbitrary(g)
            },
            2 => Event::Flag { player_id, at },
            _ => Event::Unflag { player_id, at }
        }
    }
}

#[cfg(test)]
mod tests {
    use quickcheck_macros::quickcheck;
    use crate::Event;

    #[quickcheck]
    fn event_compression_then_decompression(event: Event) -> bool {
        let compressed = event.compress();
        let decompressed = Event::from_compressed(&*compressed).unwrap();
        event == decompressed
    }
}