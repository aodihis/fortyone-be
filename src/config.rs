
use dotenvy::dotenv;
use std::env;

#[derive(Debug)]
pub struct Config {
    pub server_address: String,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Self {
        // Load .env file into environment variables
        dotenv().ok();

        let server_address = env::var("SERVER_ADDRESS").expect("SERVER_ADDRESS is not set");

        Self { server_address }
    }
}
