use world::{Chunk, Position, Rect, World};
use world::events::Event;

pub trait SweeperSocket {
    fn click(&mut self, position: Position);
    fn double_click(&mut self, position: Position);
    fn flag(&mut self, position: Position);
    fn next_event(&mut self) -> Option<&Event>;
    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk>;
}

pub struct LocalWorld {
    world: World,
    next_event: usize,
}

impl LocalWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            next_event: 0,
        }
    }
}

impl SweeperSocket for LocalWorld {
    fn click(&mut self, position: Position) {
        self.world.click(position, "")
    }

    fn double_click(&mut self, position: Position) {
        self.world.double_click(position, "")
    }

    fn flag(&mut self, position: Position) {
        self.world.flag(position, "")
    }

    fn next_event(&mut self) -> Option<&Event> {
        if let Some(event) = self.world.events.get(self.next_event) {
            self.next_event += 1;
            Some(event)
        } else {
            None
        }
    }

    fn get_chunks(&self, rect: Rect) -> Vec<&Chunk> {
        let chunks_to_get = rect.chunks_contained();
        chunks_to_get.iter()
            .map(|chunk_position| self.world.get_chunk(chunk_position.position()))
            .filter_map(|v| v)
            .collect()
    }
}