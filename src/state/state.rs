use crate::models::game::Game;
use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Clone, Debug, Serialize, PartialEq)]
pub enum GameStateStatus {
    Lobby,
    InProgress,
    Finished,
}
#[derive(Clone)]
pub struct GameState {
    pub id: Uuid,
    pub num_player: u8,
    pub status: GameStateStatus,
    pub game: Option<Game>,
    pub date_created: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub players: HashMap<Uuid, (String,tokio::sync::mpsc::UnboundedSender<axum::extract::ws::Message>)>,
}

pub struct GameManager {
    pub games: HashMap<Uuid, GameState>,
}

impl GameManager {
    pub fn new() -> Self {
        Self {
            games: HashMap::new(),
        }
    }

    pub fn create_game(&mut self) -> GameState {
        let game = GameState {
            id: Uuid::new_v4(),
            num_player: 0,
            status: GameStateStatus::Lobby,
            game: None,
            date_created: Utc::now(),
            last_updated: Utc::now(),
            players: HashMap::new(),
        };
        self.games.insert(game.id, game.clone());
        game
    }
}