#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use crate::models::game::Game;

    fn test_create_game() {
        let game = Game::new(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);
        println!("Created game: {:?}", game);
    }


}