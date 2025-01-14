use crate::routes::game::create_router;
use crate::state::state::GameManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

mod models;
mod state;
mod handlers;
mod routes;

#[tokio::main]
async fn main() {

    tracing_subscriber::fmt::init();
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));

    let game_state = Arc::new(RwLock::new(GameManager::new()));
    let router = create_router(game_state);

    axum::Server::bind(&addr).serve(router.into_make_service()).await.unwrap();
}
