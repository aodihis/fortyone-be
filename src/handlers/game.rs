use crate::engine::card::Card;
use crate::engine::game::{Game, GamePhase, MAX_PLAYER};
use crate::handlers::error::GameError;
use crate::state::state::{GameManager, GameState, GameStateStatus};
use axum::extract::Query;
use axum::http::StatusCode;
use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State},
    response::IntoResponse
    ,
    Json,
};
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CreateGameResponse {
    game_id: String,
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
    message_type: String,
    players: Vec<PlayerData>,

}
#[derive(Debug, Serialize, Deserialize)]
struct PlayerData {
    name: String,
    hand: Vec<String>,
    bin: Vec<String>,
}
#[derive(Debug, Serialize, Deserialize, Clone)]
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


pub async fn game(ws: WebSocketUpgrade, Path(game_id): Path<String>, Query(params): Query<HashMap<String, String>>, State(state): State<Arc<RwLock<GameManager>>>) -> impl IntoResponse {

    let player_id = Uuid::new_v4();
    let mut player_name = "Player".to_string();
    {
        let mut game_manager = state.write().await;

        if !game_manager.games.contains_key(&game_id) {
            return Err((StatusCode::BAD_REQUEST, "Game not found.").into_response());
        }

        if game_manager.games[&game_id].players.len() >= MAX_PLAYER {
            return Err((StatusCode::BAD_REQUEST, "Max player has been reached.").into_response());
        }

        if game_manager.games[&game_id].status != GameStateStatus::Lobby {
            return Err((StatusCode::BAD_REQUEST, "Game already started.").into_response());
        }

         player_name = {
            match params.get("player_name") {
                Some(name) => name.to_string(),
                None => {format!("Player {}",game_manager.games[&game_id].players.len() )}
            }
        };

        if game_manager.games.get(&game_id).is_none() {
            eprintln!("Game with ID {} not found", game_id);
        }
    }
    Ok(ws.on_upgrade(move |socket| handle_game_connection(socket, state, player_id,player_name, game_id)))
}


async fn handle_game_connection(mut socket: WebSocket, state: Arc<RwLock<GameManager>>, player_id: Uuid, player_name: String, game_id: String) {


    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<Message>();

    let mut send_task = tokio::spawn(async move {
       while let Some(message) = rx.recv().await {
           if sender.send(message).await.is_err() {
               continue;
           }
       }
    });

    {
        let mut write_state = state.write().await;
        let mut game_state = write_state.games.get_mut(&game_id).unwrap();
        let join_message = format!("{} joined game", player_name);
        let join_json = serde_json::json!({
                "status": "success",
                "message_type": "player_join",
                "message": join_message,
            });
        broadcast_message(join_json.to_string(), game_state).await;


        game_state.players.insert(player_id, (player_name.clone(), tx));
    }

    while let Some(Ok(message)) = receiver.next().await {

        match message {
            Message::Text(msg) => {
                match serde_json::from_str::<GameRequest>(&msg) {
                    Ok(data) => {
                        handle_game_data(&state, player_id, &game_id, Json::from(data)).await;
                    }
                    Err(_) => {}
                }
            }
            _ => {}
        }
    }

    {
        let mut write_state = state.write().await;
        if let Some(game_state) = write_state.games.get_mut(&game_id) {
            game_state.players.remove(&player_id);
            if let Some(game) = &mut game_state.game {
                game.remove_player(&player_id).unwrap();
            }
            let leave_message = format!("{} left game", player_name);
            let leave_json = serde_json::json!({
                "status": "success",
                "message_type": "player_left",
                "message": leave_message,
            });
            broadcast_message(leave_json.to_string(), game_state).await;
        }
    }

    send_task.abort();
}

async fn broadcast_message(message: String, game_state: &mut GameState) {
    println!("broadcasting message: {}", message);
    for (_, (name, tx)) in game_state.players.iter() {
        if let Err(e) = tx.send(Message::Text(message.clone())) {
            eprintln!("Error sending message: {:?}", e.to_string());
        }
    }
}
async fn handle_game_data( state: &Arc<RwLock<GameManager>>, player_id: Uuid, game_id : &String, data: Json<GameRequest>) {
    let mut write_state = state.write().await;
    let game_state: &mut GameState = write_state.games.get_mut(game_id).unwrap();
    let mut game_res: &mut Option<Game> = &mut game_state.game;
    let action: &str = data.action.as_str();

    if action == "start_game" {
        match game_res {
            None => {
                let player_list = game_state.players.keys().cloned().collect();
                let game = Game::new(player_list);
                game_state.game = Some(game);
                game_state.status = GameStateStatus::InProgress;
                let game_event = GameEvent {
                    event_type: "game_start".to_string(),
                    from: None,
                    to: None,
                };
                broadcast_game_message(game_state, game_event);
            }

            Some(_game) => {
                let res = GameResponse { status: "failed".to_string() };
                let (_, rx) = game_state.players.get_mut(&player_id).unwrap();
                if let Err(e) = rx.send(Message::Text(serde_json::to_string(&res).unwrap())) {
                    eprintln!("Error sending message: {:?}", e);
                }
            }
        };
        return;
    }

    if game_res.is_none() {
        send_failed_message(game_state, &player_id);
        return;
    };
    let mut game = game_res.as_mut().unwrap();
    let player_pos = game.player_pos(&player_id).unwrap();
    let action = data.action.as_str();
    match action {
        "draw" => {
            match game.draw(&player_id) {
                Ok(_) => {
                    let game_event = GameEvent {
                        event_type: "draw".to_string(),
                        from: None,
                        to: Option::from(player_pos as u8),
                    };
                    broadcast_game_message(game_state, game_event);
                }
                Err(_) => {send_failed_message(game_state, &player_id);}
            }
        },
        "take_bin" => {
            match game.take_bin(&player_id) {
                Ok(_) => {
                    let game_event = GameEvent {
                        event_type: "take_bin".to_string(),
                        from: Option::from(player_pos as u8),
                        to: Option::from(player_pos as u8),
                    };
                    broadcast_game_message(game_state, game_event);
                }
                Err(_) => {send_failed_message(game_state, &player_id);}
            }
        },
        "discard" => {
            let card_data = match &data.card {
                Some(card_data) => card_data,
                None => {
                    send_failed_message(game_state, &player_id);
                return;
                }
            };
            let card = {
                match Card::from_string(card_data) {
                    Some(card) => card,
                    _ => {
                        send_failed_message(game_state, &player_id);
                        return;
                    }
                }
            };
            match game.discard(&player_id, card) {
                Ok(_) => {
                    let game_event = GameEvent {
                        event_type: "discard".to_string(),
                        from: Option::from(player_pos as u8),
                        to: Option::from(game.current_turn as u8),
                    };
                    broadcast_game_message(game_state, game_event);
                }
                Err(_) => {send_failed_message(game_state, &player_id);}
            }
        },
        _ => {}
    }
}


fn send_failed_message(game_state: &mut GameState, player_id: &Uuid) {
    let res = GameResponse { status: "failed".to_string() };
    let (_, rx) = game_state.players.get_mut(player_id).unwrap();
    if let Err(e) = rx.send(Message::Text(serde_json::to_string(&res).unwrap())) {
        eprintln!("Error sending message: {:?}", e);
    }
}
fn broadcast_game_message(game_state: &mut GameState, game_event: GameEvent) {
        let game = game_state.game.as_ref().unwrap();
        for (id, (_name, con)) in game_state.players.iter() {
            let msg = build_game_message(id, game, game_state, game_event.clone());

            if let Err(e) = con.send(Message::Text(serde_json::to_string(&msg).unwrap())) {
                eprintln!("Error sending message: {:?}", e);
            }
        }
}

fn build_game_message(id: &Uuid, game: &Game, game_state: &GameState, game_event: GameEvent) -> GameMessage {
    let player_pos = match game.player_pos(&id){
        None => {panic!("Player {} not found", id)}
        Some(i) => {i as u8}
    };

    let mut players = vec![];

    for i in 0..game.players.len() {
        let p_id = game.players[i].id;
        let (name,_) = game_state.players.get(&p_id).unwrap();
        players.push(PlayerData {
            name: name.to_string(),
            hand: {
                if p_id  == *id {
                    game.players[i].hand.iter().map(|card| card.to_string()).collect()
                } else {
                    vec!["".to_string();4]
                }

            },
            bin: game.players[i].bin.iter().map(|card| card.to_string()).collect(),
        })
    }


    let game_data =  GameData{
        num_of_players: game_state.players.len() as u8,
        current_turn: game.current_turn as u8,
        current_phase: match game.phase {
            GamePhase::GameEnded => {"ended"}
            GamePhase::P1 => {"p1"}
            GamePhase::P2 => {"p2"}
        }.to_string(),
        event: game_event,
        message_type: "game_event".to_string(),
        players,
    };
    let msg = GameMessage{
        player_id: id.clone(),
        status: "success".to_string(),
        player_pos,
        data: Some(game_data),
    };

    msg
}
