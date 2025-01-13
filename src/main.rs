use uuid::Uuid;
use crate::models::game::Game;

mod models;

fn main() {
    let game = Game::new(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);
    println!("Created game: {:?}", game);
}
