#[cfg(test)]
#[allow(warnings)]
mod tests {
    use crate::engine::game::{Game, GameStatus};
    use uuid::Uuid;
    use crate::engine::card::{Card, Rank, Suit};

    fn test_create_game() {
        let game = Game::new(vec![Uuid::new_v4(), Uuid::new_v4(), Uuid::new_v4()]);
        println!("Created game: {:?}", game);
    }

    #[test]
    fn test_card() {
        let card = match Card::from_string("H2") {
            None => {panic!("Unable initiate card")}
            Some(card) => {card}
        };

        assert_eq!(card.to_string(), "H2");
        assert_eq!(card.points(),2);
    }


    #[test]
    fn test_game_flow() {
        let player1_id = Uuid::new_v4();
        let player2_id = Uuid::new_v4();
        let mut game = Game::new(vec![player1_id.clone(), player2_id.clone()]);
        let mut i = 0;
        loop {
            let current_player = game.current_player();
            let res_p1 = match i {
                i if i%2 == 1 && current_player.bin.len() > 0 => game.take_bin(&current_player.id),
                i if i%2 == 0 => game.draw(&current_player.id),
                _ => game.draw(&current_player.id)
            };


            let p1 = match res_p1 {
                Ok(p1) => p1,
                Err(e) => panic!("Error while drawing player: {:?}", e)
            };

            let card_to_discard = current_player.hand[3].clone();
            let res_p2 = game.discard(&current_player.id, card_to_discard);

            let p2 =match res_p2 {
                Ok(p2) => p2,
                Err(e) => panic!("Error while discard card: {:?}", e)
            };

            if p2.status == Option::from(GameStatus::Ended) {
                break;
            }
            i += 1;

        }

    }

    #[test]
    fn test_game_step() {
        let player1_id = Uuid::new_v4();
        let player2_id = Uuid::new_v4();
        let mut game = Game::new(vec![player1_id.clone(), player2_id.clone()]);

        let current_player = game.current_player();
        game.draw(&current_player.id).unwrap();
        let card_to_discard = current_player.hand[3].clone();
        let res = game.discard(&current_player.id, card_to_discard);

        match res {
            Ok(p2) => {},
            Err(e) => panic!("Error while discard card: {:?}", e)
        };

        let current_player = game.current_player();

        assert_eq!(current_player.hand.len(),4);
    }



}