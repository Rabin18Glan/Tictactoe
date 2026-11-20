use crate::game::Game;
use serde::{Deserialize, Serialize};
#[derive(Serialize, Deserialize)]
pub enum Request {
    Join {
        invitation: String,
        identity: String,
    },
    Move {
        cell: usize,
        revision: usize,
    },
    Rematch,
}
#[derive(Serialize, Deserialize)]
pub struct Snapshot {
    pub game: Game,
    pub connected: bool,
    pub message: String,
}
