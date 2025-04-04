# Fortyone Rummy Backend

A WebSocket-based backend server for the Fortyone Rummy card game implemented in Rust.

## Description

This backend provides the core game logic and networking capabilities for playing Fortyone Rummy, handling game creation, player connections, and game state management.

## Getting Started

### Prerequisites

- Rust and Cargo installed on your system

### Installation

Clone the repository and run the following command in the project directory:

```bash
cargo run
```

## API Endpoints

### Create Game
- **Endpoint:** `/create`
- **Method:** GET
- **Description:** Creates a new game instance and returns a game ID

### Join Game
- **Endpoint:** `/game_id/join?player_name={name}`
- **Type:** WebSocket
- **Description:** Establishes a WebSocket connection to join an existing game

## WebSocket Communication

### Client Actions Payload

The client can send the following actions through the WebSocket connection:

```json
{
    "action": "<action_type>",
    "card": "<card_symbol>"
}
```

#### Action Types
- `start_game`: Begin the game
- `draw`: Draw a card from the deck
- `take_bin`: Take a card from the discard pile
- `discard`: Discard a card from hand

#### Card Symbols
Cards are represented using a two-character code:
- First character: Suit (H = Hearts, S = Spades, D = Diamonds, C = Clubs)
- Second character: Value (A = Ace, 2-9, X = 10, J = Jack, Q = Queen, K = King)
  Example: `"HA"` represents Ace of Hearts

### Server Response Payload during game process

The server responds with game state updates in the following format:

```json
{
    
    "status": "success",
    "message_type": "game_event",
    "message": "Message",
    "data": {
        "player_id": "1021968c-9133-475c-810c-70613c12010a",
        "player_pos": 1,
        "num_of_players": 2,
        "card_left": 44,
        "current_turn": 0,
        "current_phase": "p1",
        "event": {
            "event_type": "game_start",
            "from": null,
            "to": null
        },
        "players": [
            {
                "name": "Player 1",
                "hand": ["","","",""],
                "bin": []
            },
            {
                "name": "Player 0",
                "hand": ["SQ","S3","DJ","H9"],
                "bin": []
            }
        ]
    }
}
```

### Response Fields

- `status`: Operation status ("success" or "error")
- `message_type`: Message Type ("game_event", "player_joined", "player_left")
- `message`: A bunch of message, for now used when player joined or left only.
- `data`: Current game state including:
    - `player_id`: Unique identifier for the player
    - `num_of_players`: Total number of players
    - `player_pos`: Player's position in the game
    - `card_left`: Remaining cards in deck
    - `current_turn`: Current player's turn
    - `current_phase`: Current game phase
    - `event`: Latest game event details
    - `players`: Array of player information including hands and discard bins

### Server message when the game is over

```json
{
    
    "status": "success",
    "message_type": "end_game",
    "message": "Message",
    "data": {
        "winner_name": "name",
        "players": [
            {
                "name": "Player 1",
                "hand": ["HA","H3","HX","HK"],
                "score": 30
            },
            {
                "name": "Player 0",
                "hand": ["SQ","S3","DJ","H9"],
                "score": 30
            }
        ]
    }
}
```