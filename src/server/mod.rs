use crate::config;
use std::error::Error;

pub mod client;

#[derive(Clone)]
pub struct Server {
    pub config: config::ServerConfig,
    pub client: client::Client,
    // TODO: tiles, possible moves, etc
}

impl Server {
    pub fn new(
        client: client::Client,
        config: config::ServerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        // TODO grab tiles, possible moves, etc from server

        Ok(Self { config, client })
    }
}
