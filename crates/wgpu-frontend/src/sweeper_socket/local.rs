use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use world::{Chunk, Rect, World};
use crate::sweeper_socket::SweeperSocket;

pub struct LocalWorld {
    world: World,
}

impl LocalWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
        }
    }
}

impl SweeperSocket for LocalWorld {
    fn send(&mut self, message: ClientMessage) {
        match message {
            ClientMessage::Connected => {}
            ClientMessage::Click(position) => { self.world.click(position, "") }
            ClientMessage::Flag(position) => { self.world.flag(position, "") }
            ClientMessage::DoubleClick(position) => { self.world.double_click(position, "") }
            ClientMessage::QueryChunks(_) => {}
        }
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        if let Some(event) = self.world.events.pop_front() {
            Some(ServerMessage::Event(event))
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

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}
