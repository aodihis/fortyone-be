use crate::engine::game::Game;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;
use crate::utils::generate_short_uuid;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum GameStateStatus {
    Lobby,
    InProgress,
    Finished,
}
#[derive(Clone)]
pub struct GameState {
    pub id: String,
    pub num_player: u8,
    pub status: GameStateStatus,
    pub game: Option<Game>,
    pub date_created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub players: HashMap<Uuid, (String,tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>)>,
}

pub struct GameManager {
    pub games: HashMap<String, GameState>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }

    pub fn create_game(&mut self) -> GameState {
        let game = GameState {
            id: generate_short_uuid(),
            num_player: 0,
            status: GameStateStatus::Lobby,
            game: None,
            date_created: Utc::now(),
            last_updated: Utc::now(),
            players: HashMap::new(),
        };
        self.games.insert(game.id.clone(), game.clone());
        game
    }
}