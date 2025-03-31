use serde::{Deserialize, Serialize};
use derive_more::{Div, Mul};
use std::cmp::{max, min};
use crate::{ChunkPosition, Position};

#[repr(C)]
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq)]
#[derive(bytemuck::Pod, bytemuck::Zeroable)]
#[derive(Deserialize, Serialize)]
#[derive(Mul, Div)]
pub struct Rect {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

impl Rect {
    pub fn from_corners(top_left: Position, bottom_right: Position) -> Rect {
        let Position(left, top) = top_left;
        let Position(right, bottom) = bottom_right;
        Self { left, top, right, bottom }
    }

    pub fn from_top_left_and_size(top_left: Position, width: i32, height: i32) -> Rect {
        let bottom_right = &top_left + (width, height);
        Self::from_corners(top_left, bottom_right)
    }

    pub fn positions(&self) -> Vec<Position> {
        if self.right <= self.left || self.bottom <= self.top {
            return vec![];
        }
        let mut result = vec![];
        for x in self.left..self.right {
            for y in self.top..self.bottom {
                result.push(Position(x, y))
            }
        }
        result
    }

    pub fn expand_to_contain(&mut self, rect: Rect) {
        self.left = min(self.left, rect.left);
        self.top = min(self.top, rect.top);
        self.right = max(self.right, rect.right);
        self.bottom = max(self.bottom, rect.bottom);
    }

    pub fn split_x(&self, split: i32) -> Vec<Rect> {
        if split > self.left && split < self.right {
            let mut r1 = *self;
            let mut r2 = *self;
            r1.right = split;
            r2.left = split;
            vec![r1, r2]
        } else {
            vec![*self]
        }
    }

    pub fn split_y(&self, split: i32) -> Vec<Rect> {
        if split > self.top && split < self.bottom {
            let mut r1 = *self;
            let mut r2 = *self;
            r1.bottom = split;
            r2.top = split;
            vec![r1, r2]
        } else {
            vec![*self]
        }
    }

    pub fn shift(&mut self, x: i32, y: i32) {
        self.left += x;
        self.right += x;
        self.top += y;
        self.bottom += y;
    }

    pub fn modulo(&self, modulo: i32) -> Rect {
        let mut result = *self;
        result.left %= modulo;
        result.top %= modulo;
        result.right = result.left + self.width();
        result.bottom = result.top + self.height();
        result
    }

    pub fn from_center_and_size(center: Position, width: i32, height: i32) -> Self {
        let left = center.0 - width/2;
        let top = center.1 - height/2;
        Self {
            left, top,
            right: left + width,
            bottom: top + height,
        }
    }

    pub fn contains(&self, Position(x, y): Position) -> bool {
        x > self.left &&
            x < self.right &&
            y > self.top &&
            y < self.bottom
    }

    pub fn intersection(&self, other: &Rect) -> Option<Self> {
        let left = max(self.left, other.left);
        let right = min(self.right, other.right);
        let top = max(self.top, other.top);
        let bottom = min(self.bottom, other.bottom);
        if left <= right && top <= bottom {
            Some(Self { left, right, top, bottom })
        } else {
            None
        }
    }

    pub fn top_left(&self) -> Position {
        Position(self.left, self.top)
    }

    pub fn bottom_right(&self) -> Position {
        Position(self.right, self.bottom)
    }

    pub fn top_right(&self) -> Position {
        Position(self.right, self.top)
    }

    pub fn bottom_left(&self) -> Position {
        Position(self.left, self.bottom)
    }

    pub fn area(&self) -> i64 {
        self.width() as i64 * self.height() as i64
    }

    pub fn width(&self) -> i32 {
        self.right - self.left
    }

    pub fn height(&self) -> i32 {
        self.bottom - self.top
    }

    pub fn chunks_contained(&self) -> Vec<ChunkPosition> {
        // Adding 15, 15 here so that when it's rounded down, it will always be the first chunk that's totally in it
        let top_left = (self.top_left() + Position(15, 15)).chunk_position();
        let bottom_right = self.bottom_right().chunk_position();

        let mut x = top_left.0;

        let mut chunks = vec![];
        while x < bottom_right.0 {
            let mut y = top_left.1;
            while y < bottom_right.1 {
                chunks.push(ChunkPosition(x, y));
                y += 16;
            }
            x += 16;
        }

        chunks
    }

    pub fn chunks_containing(&self) -> Vec<ChunkPosition> {
        let mut new_rect = *self;
        new_rect.left -= 15;
        new_rect.top -= 15;
        new_rect.right += 15;
        new_rect.bottom += 15;
        new_rect.chunks_contained()
    }
}