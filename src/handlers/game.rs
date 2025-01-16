use crate::engine::game::{Game, MAX_PLAYER};
use crate::handlers::error::GameError;
use crate::state::state::{GameManager, GameStateStatus};
use axum::debug_handler;
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::IntoResponse
    ,
    Json,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

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

#[derive(Debug, Deserialize)]
struct GameRequest  {
    action: String,
    card: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct GameResponse {
    status: String,
}
#[derive(Debug, Serialize, Deserialize)]
struct GameMessage {
    player_id: Uuid,
    status: String,
    player_pos: u8,
    data: Option<GameData>
}
#[derive(Debug, Serialize, Deserialize)]
struct GameData {
    num_of_players: u8,
    current_turn: u8,
    current_phase: String,
    event: GameEvent,
    players: Vec<PlayerData>,

}
#[derive(Debug, Serialize, Deserialize)]
struct PlayerData {
    name: String,
    hand: Vec<String>,
    bin: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize)]
struct GameEvent {
    event_type: String,
    from: Option<u8>,
    to: Option<u8>,
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

    let game_uuid = {
        match Uuid::parse_str(&game_id) {
            Ok(id) => id,
            Err(_) => {
                return Err((StatusCode::BAD_REQUEST, "Invalid game ID").into_response())
            }
        }
    };

    let player_id = Uuid::new_v4();
    {
        let mut game_manager = state.write().await;

        if !game_manager.games.contains_key(&game_uuid) {
            return Err((StatusCode::BAD_REQUEST, "Game not found.").into_response());
        }

        if game_manager.games[&game_uuid].players.len() >= MAX_PLAYER {
            return Err((StatusCode::BAD_REQUEST, "Max player has been reached.").into_response());
        }

        if game_manager.games[&game_uuid].status != GameStateStatus::Lobby {
            return Err((StatusCode::BAD_REQUEST, "Game already started.").into_response());
        }

        let name = {
            match params.get("player_name") {
                Some(name) => name.to_string(),
                None => {format!("Player {}",game_manager.games[&game_uuid].players.len() )}
            }
        };

        let (tx, _) = tokio::sync::mpsc::unbounded_channel();

        if let Some(game) = game_manager.games.get_mut(&game_uuid) {
            game.players.insert(player_id.clone(), (name, tx));
        } else {
            eprintln!("Game with ID {} not found", game_uuid);
        }
    }
    println!("Game created");
    Ok(ws.on_upgrade(move |socket| handle_game_connection(socket, state, player_id, game_uuid)))
}


async fn handle_game_connection(mut socket: WebSocket, state: Arc<RwLock<GameManager>>, player_id: Uuid, game_id: Uuid) {


    while let Some(Ok(message)) = socket.recv().await {

        match message {
            Message::Text(msg) => {
                match serde_json::from_str::<GameRequest>(&msg) {
                    Ok(data) => {
                        handle_game_data(&mut socket, &state, player_id, game_id, Json::from(data)).await;
                    }
                    Err(_) => {}
                }
            }
            (_) => {}
        }
    }
}

async fn handle_game_data(socket: &mut WebSocket, state: &Arc<RwLock<GameManager>>, player_id: Uuid, game_id : Uuid, data: Json<GameRequest>) -> impl IntoResponse {
    let mut write_state = state.write().await;
    let game_state = write_state.games.get_mut(&game_id).unwrap();
    let game_res: &Option<Game> = &game_state.game;
    let action = data.action.as_str();

    if action == "start_game" {
        match game_res {
            None => {
                let game = Game::new(game_state.players.keys().cloned().collect());
                game_state.status = GameStateStatus::InProgress;

                for (id, (_name, con)) in game_state.players.iter() {

                    let player_pos = match game.player_pos(&player_id){
                        None => {panic!("Player {} not found", id)}
                        Some(i) => {i as u8}
                    };

                    let game_event = GameEvent {
                        event_type: "game_start".to_string(),
                        from: None,
                        to: None,
                    };

                    let mut players = vec![];

                    for i in [player_pos, 0, 1, 2, 3] {
                        if i >= game.players.len() as u8 {
                            break
                        }
                        if i == player_pos {
                            continue;
                        }
                        let id = game.players[i as usize].id;
                        let (name,_) = game_state.players.get(&id).unwrap();
                        players.push(PlayerData {
                            name: name.to_string(),
                            hand: {
                                game.players[i as usize].hand.iter().map(|card| card.to_string()).collect()
                            },
                            bin: vec![],
                        })
                    }


                    let game_data = GameData{
                        num_of_players: game_state.players.len() as u8,
                        current_turn: game.current_turn as u8,
                        current_phase: "".to_string(),
                        event: game_event,
                        players,
                    };
                    let msg = GameMessage{
                        player_id,
                        status: "success".to_string(),
                        player_pos,
                        data: Some(game_data),
                    };

                    if let Err(e) = con.send(Message::Text(serde_json::to_string(&msg).unwrap())) {
                        eprintln!("Error sending message: {:?}", e);
                    }
                }

            }
            Some(_game) => {
                let res = GameResponse{ status: "failed".to_string() };
                if let Err(e) = socket.send(Message::Text(serde_json::to_string(&res).unwrap())).await {
                    eprintln!("Error sending message: {:?}", e);
                }
            }
        }
    }

    let _game = &game_state.game;
    match data.action.as_str() {
        "draw" => {
        },
        "take_bin" => {},
        "discard" => {},
        _ => {}
    }
}