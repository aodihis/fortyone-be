use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Suit {
    Hearts,
    Diamonds,
    Clubs,
    Spades,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Rank {
    Ace,
    Two,
    Three,
    Four,
    Five,
    Six,
    Seven,
    Eight,
    Nine,
    Ten,
    Jack,
    Queen,
    King,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Card {
    pub(crate) suit: Suit,
    pub(crate) rank: Rank,
}

impl Card {
    pub fn points(&self) -> u16 {
        match self.rank {
            Rank::Ace => {11}
            Rank::Two => {2}
            Rank::Three => {3}
            Rank::Four => {4}
            Rank::Five => {5}
            Rank::Six => {6}
            Rank::Seven => {7}
            Rank::Eight => {8}
            Rank::Nine => {9}
            Rank::Ten => {10}
            Rank::Jack => {10}
            Rank::Queen => {10}
            Rank::King => {10}
        }
    }
}