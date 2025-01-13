use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::models::card::{Card, Rank, Suit};
use rand::rng;
use rand::seq::SliceRandom;

pub enum GameError {
    InvalidPlayer,
    InvalidTurn,
    InvalidCard,
    InvalidMove,
    GameFinished,
}


#[derive(Deserialize, Serialize, Debug)]
pub struct Game {
    pub id: Uuid,
    pub players: Vec<Player>,
    pub deck: Vec<Card>,
    pub current_turn: usize,
}

impl Game {
    pub fn new( players_uuid: Vec<Uuid>) -> Game {
        Game {
            id: Uuid::new_v4(),
            players: players_uuid.iter().map(|&uuid| {
                Player {
                    id: uuid,
                    hand: vec![],
                    bin: vec![],
                }
            }).collect(),
            deck: Self::create_deck(),
            current_turn: 0,
        }
    }

    pub fn draw(&mut self, player_uuid: Uuid) -> Result<Ok, GameError>  {
        if self.players[self.current_turn].id != player_uuid {
            return Err(GameError::InvalidPlayer);
        }

        let card = match self.deck.pop() {
            Some(card) => card,
            None => return Err(GameError::InvalidMove),
        };

        if let Some(current_player) = self.players.get_mut(self.current_turn) {
            current_player.hand.push(card);
            Ok(())
        } else {
            self.deck.push(card);
            Err(GameError::InvalidTurn)
        }
    }

    fn create_deck() -> Vec<Card> {
        let mut cards = Vec::with_capacity(52);

        for suit in [Suit::Hearts, Suit::Diamonds, Suit::Clubs, Suit::Spades].iter() {
            for rank in [
                Rank::Ace, Rank::Two, Rank::Three, Rank::Four, Rank::Five,
                Rank::Six, Rank::Seven, Rank::Eight, Rank::Nine, Rank::Ten,
                Rank::Jack, Rank::Queen, Rank::King
            ].iter() {
                cards.push(Card {
                    suit: suit.clone(),
                    rank: rank.clone(),
                })
            }
        }
        cards.shuffle(&mut rng());
        cards
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: Uuid,
    pub hand: Vec<Card>,
    pub bin: Vec<Card>,
}


