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
- Second character: Value (A = Ace, 2-10, J = Jack, Q = Queen, K = King)
  Example: `"HA"` represents Ace of Hearts

### Server Response Payload

The server responds with game state updates in the following format:

```json
{
    "player_id": "1021968c-9133-475c-810c-70613c12010a",
    "status": "success",
    "player_pos": 1,
    "data": {
        "num_of_players": 2,
        "card_left": 44,
        "current_turn": 0,
        "current_phase": "p1",
        "event": {
            "event_type": "game_start",
            "from": null,
            "to": null
        },
        "message_type": "game_event",
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
- `player_id`: Unique identifier for the player
- `status`: Operation status ("success" or "error")
- `player_pos`: Player's position in the game
- `data`: Current game state including:
    - `num_of_players`: Total number of players
    - `card_left`: Remaining cards in deck
    - `current_turn`: Current player's turn
    - `current_phase`: Current game phase
    - `event`: Latest game event details
    - `players`: Array of player information including hands and discard bins
