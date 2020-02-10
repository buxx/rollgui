use crate::config;
use crate::world;
use crate::util;
use crate::tile::world::Tiles as WorldTiles;
use std::error::Error;

pub mod client;

#[derive(Clone)]
pub struct Server {
    pub config: config::ServerConfig,
    pub client: client::Client,
    pub world: world::World,
    pub world_tiles: WorldTiles,
    // TODO: tiles, possible moves, etc
}

impl Server {
    pub fn new(
        client: client::Client,
        config: config::ServerConfig,
    ) -> Result<Self, Box<dyn Error>> {
        // TODO grab possible moves, etc from server

        let world_source = client.get_world_source()?;
        let legend = util::extract_block_from_source("LEGEND", world_source.as_str())?;
        let world_raw = util::extract_block_from_source("GEO", world_source.as_str())?;

        let tiles = WorldTiles::new(legend.as_str())?;
        let world = world::World::new(world_raw.as_str(), &tiles)?;

        Ok(Self { config, client, world, world_tiles: tiles })
    }
}
