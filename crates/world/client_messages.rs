use crate::{AuthKey, Position};

#[derive(Debug)]
pub enum ClientMessage {
    Welcome,
    Click(Position),
    Flag(Position),
    DoubleClick(Position),
    Move(Position),
    Register(String),
    Login(AuthKey)
}