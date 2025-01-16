use crate::config::Config;
use crate::routes::game::create_router;
use crate::state::state::GameManager;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;

mod engine;
mod state;
mod handlers;
mod routes;
mod config;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let config = Config::from_env();
    let addr: SocketAddr = config
        .server_address
        .parse()
        .expect("Invalid server address format");

    let game_state = Arc::new(RwLock::new(GameManager::new()));
    let router = create_router(game_state);
    println!("Listening on {}", addr);
    axum::Server::bind(&addr).serve(router.into_make_service()).await.unwrap();
}
