use std::cmp::PartialEq;
use axum::debug_handler;
use crate::handlers::error::GameError;
use crate::state::state::{GameManager, GameState, GameStateStatus};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::IntoResponse
    ,
    Json,
};
use serde::Deserialize;
use std::sync::Arc;
use axum::http::StatusCode;
use tokio::sync::{Mutex, RwLock};
use uuid::{uuid, Uuid};
use crate::models::game::MAX_PLAYER;

#[derive(Debug, Deserialize)]
struct CreateGameRequest {
    player_name: String,
}

#[derive(Debug, Deserialize)]
struct JoinGameRequest {
    player_name: String,
}

pub async fn create_game(State(state): State<Arc<RwLock<GameManager>>>) -> Result<Json<GameState>, GameError>{
    let game = state.write().await.create_game();
    Ok(Json(game))
}


#[debug_handler]
pub async fn game(ws: WebSocketUpgrade, Path(game_id): Path<String>, State(state): State<Arc<RwLock<GameManager>>>) -> impl IntoResponse {
    {
        let game_manager = state.read().await;
        let game_id = Uuid::parse_str(&game_id).unwrap();
        if !game_manager.games.contains_key(&game_id) {
            return Err((StatusCode::BAD_REQUEST, "Game not found."));
        }

        if game_manager.games[&game_id].num_player >= MAX_PLAYER {
            return Err((StatusCode::BAD_REQUEST, "Max player has been reached."));
        }

        if game_manager.games[&game_id].status != GameStateStatus::Lobby {
            return Err((StatusCode::BAD_REQUEST, "Game already started."));
        }
    }
    Ok(ws.on_upgrade(move |socket| handle_game_connection(socket, state)))
}


async fn handle_game_connection(mut socket: WebSocket, state: Arc<RwLock<GameManager>>) {
    while let Some(Ok(message)) = socket.recv().await {
        if let Message::Text(text) = message {
            if let Err(e) = socket.send(Message::Text("hello".to_string())).await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }
}