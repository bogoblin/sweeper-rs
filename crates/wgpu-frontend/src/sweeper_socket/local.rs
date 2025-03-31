use std::collections::VecDeque;
use crate::sweeper_socket::SweeperSocket;
use world::ClientMessage;
use world::ServerMessage;
use world::World;

pub struct LocalWorld {
    world: World,
    message_queue: VecDeque<ServerMessage>
}

impl LocalWorld {
    pub fn new() -> Self {
        Self {
            world: World::new(),
            message_queue: Default::default(),
        }
    }
}

impl SweeperSocket for LocalWorld {
    fn send(&mut self, message: ClientMessage) {
        let event = match message {
            ClientMessage::Connected => { None }
            ClientMessage::Click(position) => { self.world.click(position, "") }
            ClientMessage::Flag(position) => { self.world.flag(position, "") }
            ClientMessage::DoubleClick(position) => { self.world.double_click(position, "") }
            ClientMessage::Query(_) => { None }
            ClientMessage::Disconnected(player_id) => { self.world.players.remove(&player_id); None }
        };
        if let Some(event) = event {
            self.message_queue.push_back(ServerMessage::Event(event));
        }
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        self.message_queue.pop_front()
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}
