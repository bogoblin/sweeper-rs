use std::collections::VecDeque;
use world::{Position, Rect};
use world::client_messages::ClientMessage;

#[derive(Debug)]
pub struct ChunkLoader {
    loaded: Rect,
    queries: VecDeque<Rect>,
}

impl ChunkLoader {
    pub fn new(visible_area: Rect) -> Self {
        let top_left = visible_area.top_left().chunk_position().position();
        let bottom_right = visible_area.bottom_right().chunk_position().bottom_right().position();
        Self {
            loaded: Rect::from_corners(top_left, bottom_right),
            queries: VecDeque::from([visible_area]),
        }
    }
    
    fn grow_right(&mut self, columns: u32) {
        let columns = (columns * 16) as i32;
        self.queries.push_back(Rect::from_top_left_and_size(
            self.loaded.top_right(),
            columns,
            self.loaded.height()
        ));
        self.loaded.right += columns;
    }
    fn grow_bottom(&mut self, rows: u32) {
        let rows = (rows * 16) as i32;
        self.queries.push_back(Rect::from_top_left_and_size(
            self.loaded.bottom_left(),
            self.loaded.width(),
            rows,
        ));
        self.loaded.bottom += rows;
    }
    fn grow_left(&mut self, columns: u32) {
        let columns = (columns * 16) as i32;
        self.queries.push_back(Rect::from_top_left_and_size(
            self.loaded.top_left() - Position(columns, 0),
            columns,
            self.loaded.height()
        ));
        self.loaded.left -= columns;
    }
    fn grow_top(&mut self, rows: u32) {
        let rows = (rows * 16) as i32;
        self.queries.push_back(Rect::from_top_left_and_size(
            self.loaded.top_left() - Position(0, rows),
            self.loaded.width(),
            rows,
        ));
        self.loaded.top -= rows;
    }

    const GROW_AMOUNT: u32 = 4;
    pub fn query(&mut self, target_area: Rect) {
        if target_area.right > self.loaded.right {
            self.grow_right(Self::GROW_AMOUNT);
        }
        if target_area.bottom > self.loaded.bottom {
            self.grow_bottom(Self::GROW_AMOUNT);
        }
        if target_area.left < self.loaded.left {
            self.grow_left(Self::GROW_AMOUNT);
        }
        if target_area.top < self.loaded.top {
            self.grow_top(Self::GROW_AMOUNT);
        }
    }
    
    pub fn next_query_message(&mut self) -> Option<ClientMessage> {
        match self.queries.pop_front() {
            None => None,
            Some(query) =>  Some(ClientMessage::Query(query))
        }
    }
}