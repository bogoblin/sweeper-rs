use crate::sweeper_socket::SweeperSocket;
use world::client_messages::ClientMessage;
use world::server_messages::ServerMessage;
use world::World;

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
            ClientMessage::Click(position) => { self.world.click(position, ""); }
            ClientMessage::Flag(position) => { self.world.flag(position, ""); }
            ClientMessage::DoubleClick(position) => { self.world.double_click(position, ""); }
            ClientMessage::Query(_) => {}
            ClientMessage::Disconnected(player_id) => { self.world.players.remove(&player_id); }
        }
    }

    fn next_message(&mut self) -> Option<ServerMessage> {
        if let Some(event) = self.world.events.pop_front() {
            Some(ServerMessage::Event(event))
        } else {
            None
        }
    }

    fn world(&mut self) -> &mut World {
        &mut self.world
    }
}
