use crate::{Position, Tile, UpdatedRect, UpdatedTile};
use serde::{Deserialize, Serialize};
use std::i32;
use quickcheck::{Arbitrary, Gen};
use crate::player::Player;

#[derive(Debug, Clone, PartialEq)]
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
            Event::Clicked { player_id, at, updated } |
            Event::DoubleClicked { player_id, at, updated } => {
                ("c", player_id, at, Some(updated))
            }
            Event::Flag { player_id, at } => {
                ("f", player_id, at, None)
            }
            Event::Unflag { player_id, at } => {
                ("u", player_id, at, None)
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
        
        if header == "c" {
            match UpdatedRect::from_compressed(&compressed[index..]) {
                None => None,
                Some(updated) => {
                    Some(Event::Clicked {
                        player_id,
                        at,
                        updated
                    })
                }
            }
        } else if header == "f" {
            Some(Event::Flag {
                player_id,
                at
            })
        } else if header == "u" {
            Some(Event::Unflag {
                player_id,
                at
            })
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
    use quickcheck::quickcheck;
    use crate::Event;

    quickcheck! {
        fn event_compression_then_decompression(event: Event) -> bool {
            let compressed = event.compress();
            let decompressed = Event::from_compressed(&*compressed);
            event.updated_rect() == decompressed.unwrap().updated_rect()
        }
    }
}

#[test]
fn compression_test() -> Result<(), String> {
    let p = "Gary";
    let pos = Position(-50, 300);
    let flag = Event::Flag {
        player_id: p.to_string(),
        at: pos
    };
    let decompressed = Event::from_compressed(&*flag.compress()).ok_or("uh oh")?;
    match decompressed {
        Event::Flag { player_id, at } => {
            assert_eq!(player_id, p);
            assert_eq!(at, pos);
        }
        _ => panic!("Wrong event type")
    }

    let to_update = vec![
        UpdatedTile { position: Position(1, 1), tile: Tile::empty().with_revealed() },
        UpdatedTile { position: Position(3, 1), tile: Tile::empty().with_revealed() },
        UpdatedTile { position: Position(3, 3), tile: Tile::mine().with_revealed() },
        UpdatedTile { position: Position(2, 3), tile: Tile::mine().with_flag() },
    ];
    let clicked = Event::Clicked {
        player_id: p.to_string(),
        at: pos,
        updated: UpdatedRect::new(to_update.clone())
    };
    let decompressed = Event::from_compressed(&*clicked.compress()).ok_or("uh oh")?;
    match decompressed {
        Event::Clicked { player_id, at, updated } => {
            assert_eq!(player_id, p);
            assert_eq!(at, pos);
            assert_eq!(updated.public_tiles(), UpdatedRect::new(to_update).public_tiles());
        }
        _ => panic!("Wrong event type")
    }

    Ok(())
}