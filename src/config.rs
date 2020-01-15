use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    pub ip: String,
    pub port: u16,
    pub character_id: Option<String>,
}
