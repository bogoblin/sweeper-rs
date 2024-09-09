use serde::{Deserialize, Serialize};
use crate::{Position, UpdatedRect};

#[derive(Debug)]
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
    }
}