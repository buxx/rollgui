use crate::tile::world::Tiles as WorldTiles;
use crate::util;
use crate::world;
use std::error::Error;
use std::fmt;

pub mod client;

#[derive(Clone, Debug)]
pub struct ServerAddress {
    pub host: String,
    pub port: u16,
    pub secure: bool,
}

impl ServerAddress {
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            secure: true,
        }
    }

    pub fn unsecure(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            secure: false,
        }
    }

    pub fn with_credentials(&self, login: &str, password: &str) -> String {
        let protocol = if self.secure { "https" } else { "http" };
        format!(
            "{}://{}:{}@{}:{}",
            protocol, login, password, self.host, self.port
        )
    }
}

impl fmt::Display for ServerAddress {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let protocol = if self.secure { "https" } else { "http" };
        write!(f, "{}://{}:{}", protocol, self.host, self.port)
    }
}

#[derive(Clone)]
pub struct Server {
    pub address: ServerAddress,
    pub character_id: Option<String>,
    pub client: client::Client,
    pub world: world::World,
    pub world_tiles: WorldTiles,
}

impl fmt::Debug for Server {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Server")
            .field("address", &self.address)
            .finish()
    }
}

impl Server {
    pub fn new(
        client: client::Client,
        address: ServerAddress,
        character_id: Option<String>,
    ) -> Result<Self, Box<dyn Error>> {
        // TODO grab possible moves, etc from server

        let world_source = client.get_world_source()?;
        let legend = util::extract_block_from_source("LEGEND", world_source.as_str())?;
        let world_raw = util::extract_block_from_source("GEO", world_source.as_str())?;

        let world_tiles = WorldTiles::new(legend.as_str())?;
        let world = world::World::new(world_raw.as_str(), &world_tiles)?;

        Ok(Self {
            address,
            character_id,
            client,
            world,
            world_tiles,
        })
    }
}
