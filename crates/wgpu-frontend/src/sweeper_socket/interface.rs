use world::ClientMessage;
use world::ServerMessage;
use world::World;

pub trait SweeperSocket {
    fn send(&mut self, message: ClientMessage);
    fn next_message(&mut self) -> Option<ServerMessage>;
    fn world(&mut self) -> &mut World;
}

