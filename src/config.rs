use serde::{Serialize, Deserialize};


#[derive(Serialize, Deserialize, Debug)]
pub struct ServerConfig {
    ip: String,
    port: u16,
    character_id: Option<String>,
}
