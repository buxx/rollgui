use reqwest;
use reqwest::blocking::Response;

use crate::entity::character::Character;
use crate::entity::stuff::Stuff;
use crate::entity::build::Build;
use crate::entity::player::Player;
use serde_json::{Map, Number, Value};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum ClientError {
    NotFound { response: String },
    PlayerNotFound { response: String },
    ClientSideError { response: String },
    ServerSideError { response: String },
    UnknownError { response: String },
}

impl Error for ClientError {}

impl ClientError {
    fn get_message(&self, client_error: &ClientError) -> String {
        return match client_error {
            ClientError::NotFound { response }  => format!("Not found: {}", response.to_string()).to_string(),
            ClientError::PlayerNotFound { response }  => format!("Player not found: {}", response.to_string()).to_string(),
            ClientError::ClientSideError { response } => format!("Client side error: {}", response.to_string()).to_string(),
            ClientError::ServerSideError { response } => format!("Server side error: {}", response.to_string()).to_string(),
            ClientError::UnknownError { response } => format!("Unknown error: {}", response.to_string()).to_string(),
        };
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", self.get_message(self))
    }
}

pub struct Client {
    server_ip: String,
    server_port: u16,
    client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(server_ip: &str, server_port: u16) -> Self {
        Self {
            server_ip: String::from(server_ip),
            server_port,
            client: reqwest::blocking::Client::new(),
        }
    }

    fn get_base_path(&self) -> String {
        // TODO https
        return format!("http://{}:{}", self.server_ip, self.server_port);
    }

    fn check_response(&self, response: Response) -> Result<Response, ClientError> {
        if response.status().as_u16() == 404 {
            return Err(ClientError::NotFound{
                response: response.text().unwrap(),
            });
        }

        if response.status().is_client_error() {
            return Err(ClientError::ClientSideError {
                response: response.text().unwrap(),
            });
        }

        if !response.status().is_success() {
            return Err(ClientError::ServerSideError {
                response: response.text().unwrap(),
            });
        }

        Ok(response)
    }

    pub fn get_player(&self, id: &str) -> Result<Player, ClientError> {
        println!("Retrieve character '{}' from server", id);
        let url = format!("{}/character/{}", self.get_base_path(), id);
        // TODO manage error
        let mut response: Response = self.client.get(url.as_str()).send().unwrap();

        match self.check_response(response) {
            Err(ClientError::NotFound{response}) => return Err(ClientError::PlayerNotFound{response}),
            Err(client_error) => return Err(client_error),
            Ok(resp) => response = resp,
        }

        let player_value: Value = response.json::<Value>().unwrap();

        let player_zone_row_i = player_value["zone_row_i"].as_i64().unwrap() as i32;
        let player_zone_col_i = player_value["zone_col_i"].as_i64().unwrap() as i32;
        let player_world_row_i = player_value["world_row_i"].as_i64().unwrap() as i32;
        let player_world_col_i = player_value["world_col_i"].as_i64().unwrap() as i32;
        let player_id = player_value["id"].as_str().unwrap();

        Ok(Player::new(
            player_id,
            (player_zone_row_i, player_zone_col_i),
            (player_world_row_i, player_world_col_i),
        ))
    }

    pub fn create_player(&self, name: &str) -> Result<Player, ClientError> {
        let mut data = Map::new();
        data.insert("name".to_string(), Value::String(name.to_string()));
        data.insert(
            "background_story".to_string(),
            Value::String("".to_string()),
        );
        data.insert(
            "max_life_comp".to_string(),
            Value::Number(Number::from(0u64)),
        );
        data.insert(
            "hunting_and_collecting_comp".to_string(),
            Value::Number(Number::from(0u64)),
        );
        data.insert(
            "find_water_comp".to_string(),
            Value::Number(Number::from(0u64)),
        );

        let url = format!("{}/character", self.get_base_path());
        let response: Response =
            self.check_response(self.client.post(url.as_str()).json(&data).send().unwrap())?;

        let player_value = response.json::<Value>().unwrap();
        let player_zone_row_i = player_value["zone_row_i"].as_i64().unwrap() as i32;
        let player_zone_col_i = player_value["zone_col_i"].as_i64().unwrap() as i32;
        let player_world_row_i = player_value["world_row_i"].as_i64().unwrap() as i32;
        let player_world_col_i = player_value["world_col_i"].as_i64().unwrap() as i32;
        let player_id = player_value["id"].as_str().unwrap();

        Ok(Player::new(
            player_id,
            (player_zone_row_i, player_zone_col_i),
            (player_world_row_i, player_world_col_i),
        ))
    }

    pub fn get_tiles_data(&self) -> Result<Value, ClientError> {
        println!("Retrieve tiles from server");
        let url = format!("{}/zones/tiles", self.get_base_path());
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.json::<Value>().unwrap())
    }

    pub fn get_zone_data(&self, world_row_i: i32, world_col_i: i32) -> Result<Value, ClientError> {
        println!("Retrieve zone from server");
        let url = format!(
            "{}/zones/{}/{}",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.json::<Value>().unwrap())
    }

    pub fn get_zone_characters(
        &self,
        world_row_i: i32,
        world_col_i: i32,
    ) -> Result<Vec<Character>, ClientError> {
        println!("Retrieve characters from server");
        let url = format!(
            "{}/zones/{}/{}/characters",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.json::<Vec<Character>>().unwrap())
    }

    pub fn get_zone_stuffs(
        &self,
        world_row_i: i32,
        world_col_i: i32,
    ) -> Result<Vec<Stuff>, ClientError> {
        println!("Retrieve stuffs from server");
        let url = format!(
            "{}/zones/{}/{}/stuff",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.json::<Vec<Stuff>>().unwrap())
    }

    pub fn get_zone_builds(
        &self,
        world_row_i: i32,
        world_col_i: i32,
    ) -> Result<Vec<Build>, ClientError> {
        println!("Retrieve builds from server");
        let url = format!(
            "{}/zones/{}/{}/builds",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.json::<Vec<Build>>().unwrap())
    }

    pub fn get_world_source(&self) -> Result<String, ClientError> {
        println!("Retrieve world source from server");
        let url = format!("{}/world/source", self.get_base_path());
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;

        Ok(response.text().unwrap().to_string())
    }
}
