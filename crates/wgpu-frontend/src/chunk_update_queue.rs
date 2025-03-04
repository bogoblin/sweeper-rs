use std::collections::{HashSet, VecDeque};

pub struct ChunkUpdateQueue {
    queue: VecDeque<usize>,
    set: HashSet<usize>,
}

impl ChunkUpdateQueue {
    pub fn new() -> Self {
        Self {
            queue: Default::default(),
            set: Default::default(),
        }
    }
    
    pub fn add_chunk_ids(&mut self, chunk_ids: Vec<usize>) {
        for chunk_id in chunk_ids {
            if !self.set.contains(&chunk_id) {
                self.set.insert(chunk_id);
                self.queue.push_back(chunk_id);
            }
        }
    }
    
    pub fn chunks_waiting(&self) -> usize {
        assert_eq!(self.queue.len(), self.set.len());
        self.queue.len()
    }
}

impl Iterator for ChunkUpdateQueue {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk_id = self.queue.pop_front()?;
        self.set.remove(&chunk_id);
        Some(chunk_id)
    }
}