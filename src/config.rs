use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
    pub character_id: Option<String>,
}
