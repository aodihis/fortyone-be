use std::cmp::PartialEq;
use std::collections::HashMap;
use axum::debug_handler;
use crate::handlers::error::GameError;
use crate::state::state::{GameManager, GameState, GameStateStatus};
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::IntoResponse
    ,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use axum::extract::Query;
use axum::http::StatusCode;
use tokio::sync::{Mutex, RwLock};
use uuid::{uuid, Uuid};
use crate::models::game::MAX_PLAYER;

#[derive(Debug, Deserialize)]
struct CreateGameRequest {
    player_name: String,
}

#[derive(Debug, Serialize)]
pub struct CreateGameResponse {
    game_id: Uuid,
    num_of_players: usize,
}

#[derive(Debug, Deserialize)]
struct JoinGameRequest {
    player_name: String,
}

pub async fn create_game(State(state): State<Arc<RwLock<GameManager>>>) -> Result<Json<CreateGameResponse>, GameError>{
    let game = state.write().await.create_game();
    Ok(Json(CreateGameResponse{
        game_id: game.id.clone(),
        num_of_players: game.players.len(),
    }))
}


#[debug_handler]
pub async fn game(ws: WebSocketUpgrade, Path(game_id): Path<String>, Query(params): Query<HashMap<String, String>>, State(state): State<Arc<RwLock<GameManager>>>) -> impl IntoResponse {

    println!("Game join");

    let player_id = Uuid::new_v4();
    {
        let mut game_manager = state.write().await;
        let game_id = {
            match Uuid::parse_str(&game_id) {
                Ok(id) => id,
                Err(_) => {
                    return Err((StatusCode::BAD_REQUEST, "Invalid game ID").into_response())
                }
            }
        };
        if !game_manager.games.contains_key(&game_id) {
            return Err((StatusCode::BAD_REQUEST, "Game not found.").into_response());
        }

        if game_manager.games[&game_id].players.len() >= MAX_PLAYER {
            return Err((StatusCode::BAD_REQUEST, "Max player has been reached.").into_response());
        }

        if game_manager.games[&game_id].status != GameStateStatus::Lobby {
            return Err((StatusCode::BAD_REQUEST, "Game already started.").into_response());
        }

        let name = {
            match params.get("player_name") {
                Some(name) => name.to_string(),
                None => {format!("Player {}",game_manager.games[&game_id].players.len() )}
            }
        };

        let (tx, _) = tokio::sync::mpsc::unbounded_channel();

        if let Some(game) = game_manager.games.get_mut(&game_id) {
            game.players.insert(player_id.clone(), (name, tx));
        } else {
            eprintln!("Game with ID {} not found", game_id);
        }
    }
    println!("Game created");
    Ok(ws.on_upgrade(move |socket| handle_game_connection(socket, state, player_id)))
}


async fn handle_game_connection(mut socket: WebSocket, state: Arc<RwLock<GameManager>>, player_id: Uuid) {
    while let Some(Ok(message)) = socket.recv().await {
        if let Message::Text(text) = message {
            if let Err(e) = socket.send(Message::Text("hello".to_string())).await {
                eprintln!("Error sending message: {:?}", e);
            }
        }
    }
}