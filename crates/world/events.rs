use crate::{Position, UpdatedRect};

#[derive(Debug)]
pub enum Event {
    Registered {
        player_id: usize
    },
    Clicked {
        player_id: usize,
        at: Position,
        updated: UpdatedRect,
    },
    DoubleClicked {
        player_id: usize,
        at: Position,
        updated: UpdatedRect,
    },
    Flag {
        player_id: usize,
        at: Position,
    },
    Unflag {
        player_id: usize,
        at: Position,
    }
}