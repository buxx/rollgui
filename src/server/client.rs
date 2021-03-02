use reqwest;
use reqwest::blocking::Response;
use url::Url;

use crate::entity::build::Build;
use crate::entity::character::Character;
use crate::entity::corpse::AnimatedCorpse;
use crate::entity::player::{ApiCharacter, Player};
use crate::entity::resource::Resource;
use crate::entity::stuff::Stuff;
use crate::gui::lang::model::Description;
use crate::gui::lang::model::ErrorResponse;
use crate::server::ServerAddress;
use crate::util;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Number, Value};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::{fmt, fs, io};

#[derive(Debug)]
pub enum ClientError {
    NotFound { message: String },
    Unauthorized,
    PlayerNotFound { message: String },
    ClientSideError { message: String },
    ServerSideError { message: String },
    UnknownError { message: String },
}

impl Error for ClientError {}

impl ClientError {
    pub fn get_message(client_error: &ClientError) -> String {
        return match client_error {
            ClientError::NotFound { message } => format!("Not found: {}", message).to_string(),
            ClientError::PlayerNotFound { message } => {
                format!("Player not found: {}", message).to_string()
            }
            ClientError::ClientSideError { message } => {
                format!("Client side error: {}", message).to_string()
            }
            ClientError::ServerSideError { message } => {
                format!("Server side error: {}", message).to_string()
            }
            ClientError::UnknownError { message } => {
                format!("Unknown error: {}", message).to_string()
            }
            ClientError::Unauthorized => "Unauthorized".to_string(),
        };
    }
}

impl fmt::Display for ClientError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Use `self.number` to refer to each positional data point.
        write!(f, "{}", ClientError::get_message(&self))
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ItemModel {
    pub name: String,
    pub value_is_str: bool,
    pub value_is_float: bool,
    pub value_str: Option<String>,
    pub value_float: Option<f32>,
    pub url: Option<String>,
    pub classes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ListOfItemModel {
    pub items: Vec<ItemModel>,
}

#[derive(Clone)]
pub struct Client {
    pub address: ServerAddress,
    client: reqwest::blocking::Client,
    pub credentials: (String, String),
}

impl fmt::Debug for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Client")
            .field("address", &self.address)
            .finish()
    }
}

impl Client {
    pub fn new(address: ServerAddress, credentials: (String, String)) -> Self {
        Self {
            address,
            client: reqwest::blocking::Client::new(),
            credentials,
        }
    }

    fn get_base_path(&self) -> String {
        self.address.to_string()
    }

    fn check_response(&self, response: Response) -> Result<Response, ClientError> {
        if response.status().as_u16() == 404 {
            return Err(ClientError::NotFound {
                message: "Not Found".to_string(),
            });
        }

        if response.status().as_u16() == 401 {
            return Err(ClientError::Unauthorized);
        }

        if response.status().is_client_error() {
            let error: ErrorResponse = response.json().unwrap();
            return Err(ClientError::ClientSideError {
                message: error.message,
            });
        }

        if !response.status().is_success() {
            let error: ErrorResponse = response.json().unwrap();
            return Err(ClientError::ServerSideError {
                message: error.message,
            });
        }

        Ok(response)
    }

    pub fn get_current_character_id(
        &self,
        credentials: (String, String),
    ) -> Result<String, ClientError> {
        println!("Retrieve current character from server");
        let url = format!("{}/account/current_character_id", self.get_base_path());
        // TODO manage error
        let mut response: Response = self
            .client
            .get(url.as_str())
            .basic_auth(credentials.0, Some(credentials.1))
            .send()
            .unwrap();

        match self.check_response(response) {
            Err(ClientError::NotFound { message }) => {
                return Err(ClientError::PlayerNotFound { message })
            }
            Err(client_error) => return Err(client_error),
            Ok(resp) => return Ok(resp.text().unwrap().clone()),
        }
    }

    pub fn get_player(&self, id: &str) -> Result<Player, ClientError> {
        println!("Retrieve character '{}' from server", id);
        let url = format!("{}/character/{}", self.get_base_path(), id);
        // TODO manage error
        let mut response: Response = self
            .client
            .get(url.as_str())
            .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
            .send()
            .unwrap();

        match self.check_response(response) {
            Err(ClientError::NotFound { message }) => {
                return Err(ClientError::PlayerNotFound { message })
            }
            Err(client_error) => return Err(client_error),
            Ok(resp) => response = resp,
        }

        let character: ApiCharacter = response.json::<ApiCharacter>().unwrap();

        Ok(Player::new(
            character.id.as_str(),
            character.name.as_str(),
            (character.zone_row_i, character.zone_col_i),
            (character.world_row_i, character.world_col_i),
            character.max_life_comp,
            character.life_points,
            character.action_points,
            character.thirst,
            character.hunger,
            character.unread_event,
            character.unread_zone_message,
            character.unread_conversation,
            character.unvote_affinity_relation,
            character.unread_transactions,
            character.pending_actions,
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
        let response: Response = self.check_response(
            self.client
                .post(url.as_str())
                .json(&data)
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        let character: ApiCharacter = response.json::<ApiCharacter>().unwrap();

        Ok(Player::new(
            character.id.as_str(),
            character.name.as_str(),
            (character.zone_row_i, character.zone_col_i),
            (character.world_row_i, character.world_col_i),
            character.max_life_comp,
            character.life_points,
            character.action_points,
            character.thirst,
            character.hunger,
            character.unread_event,
            character.unread_zone_message,
            character.unread_conversation,
            character.unvote_affinity_relation,
            character.unread_transactions,
            character.pending_actions,
        ))
    }

    pub fn get_tiles_data(&self) -> Result<Value, ClientError> {
        println!("Retrieve tiles from server");
        let url = format!("{}/zones/tiles", self.get_base_path());
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

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
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

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
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

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
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        Ok(response.json::<Vec<Stuff>>().unwrap())
    }

    pub fn get_zone_resources(
        &self,
        world_row_i: i32,
        world_col_i: i32,
    ) -> Result<Vec<Resource>, ClientError> {
        println!("Retrieve resources from server");
        let url = format!(
            "{}/zones/{}/{}/resources",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        Ok(response.json::<Vec<Resource>>().unwrap())
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
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        Ok(response.json::<Vec<Build>>().unwrap())
    }

    pub fn get_world_source(&self) -> Result<String, ClientError> {
        println!("Retrieve world source from server");
        let url = format!("{}/world/source", self.get_base_path());
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        Ok(response.text().unwrap().to_string())
    }

    pub fn describe(
        &self,
        url: &str,
        data: Option<Map<String, Value>>,
        query: Option<Map<String, Value>>,
    ) -> Result<Description, ClientError> {
        let url = if !url.starts_with("http") {
            format!("{}{}", self.get_base_path(), url)
        } else {
            url.to_string()
        };
        let url = self.url_with_query(url, query);
        println!("Describe with url {}", url);

        let mut request = self.client.post(url.as_str());
        if let Some(data_) = data {
            request = request.json(&data_);
        }

        let response: Response = self.check_response(
            request
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;
        let mut description = response.json::<Description>().unwrap();
        description.origin_url = Some(url);

        if let Some(illustration_name) = &description.illustration_name {
            match self.cache_media(&illustration_name) {
                Ok(_) => {}
                Err(error) => {
                    let message = &format!(
                        "Error when retrieve media {}: {}",
                        illustration_name,
                        ClientError::get_message(&error),
                    );
                    eprintln!("{}", message);
                    sentry::capture_message(message, sentry::Level::Error);
                }
            };
            match self.cache_media_bg(&illustration_name) {
                Ok(_) => {}
                Err(error) => {
                    let message = &format!(
                        "Error when retrieve media bg {}: {}",
                        illustration_name,
                        ClientError::get_message(&error),
                    );
                    eprintln!("{}", message);
                    sentry::capture_message(message, sentry::Level::Error);
                }
            }
        }

        Ok(description)
    }

    pub fn get_character_resume_texts(
        &self,
        character_id: &str,
    ) -> Result<Vec<ItemModel>, ClientError> {
        let url = format!(
            "{}/character/{}/resume_texts",
            self.get_base_path(),
            character_id
        );
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        Ok(response.json::<ListOfItemModel>().unwrap().items)
    }

    fn url_with_query(&self, url: String, query: Option<Map<String, Value>>) -> String {
        match query {
            Some(query_) => {
                let mut params: Vec<(String, String)> = Vec::new();
                for (key, value) in query_.iter() {
                    match value {
                        Value::Number(number) => {
                            params.push((key.to_string(), number.to_string()));
                        }
                        Value::String(str_) => {
                            params.push((key.to_string(), str_.to_string()));
                        }
                        Value::Bool(bool_) => {
                            params.push((key.to_string(), bool_.to_string()));
                        }
                        Value::Null => {}
                        _ => {}
                    }
                }

                let url = Url::parse_with_params(url.as_str(), &params).unwrap();
                return url.into_string();
            }
            None => return url,
        }
    }

    pub fn player_is_dead(&self, character_id: &str) -> Result<bool, ClientError> {
        let url = format!("{}/character/{}/dead", self.get_base_path(), character_id);
        let result = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        );
        if let Err(ClientError::NotFound { message: _ }) = result {
            return Ok(false);
        }
        let response: Response = result?;

        if response.text().unwrap() == "1" {
            return Ok(true);
        }
        Ok(false)
    }

    pub fn download_image(&self, image_id: i32, image_extension: &str) -> Result<(), ClientError> {
        let url = format!("{}/image/{}", self.get_base_path(), image_id);
        let mut response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;
        // FIXME BS: user dir or (MUST BE IN STATIC)
        let file_path = format!("static/cache/{}{}", image_id, image_extension);
        fs::create_dir_all("static/cache").unwrap();
        let mut out = File::create(Path::new(&file_path))
            .expect(format!("failed to create file {}", &file_path).as_str());
        io::copy(&mut response, &mut out)
            .expect(format!("failed to copy content into {}", &file_path).as_str());

        Ok(())
    }

    pub fn get_version(&self) -> Result<(u8, u8, u8), ClientError> {
        let url = format!("{}/system/version", self.get_base_path());
        let response: Response =
            self.check_response(self.client.get(url.as_str()).send().unwrap())?;
        let text_response: String = response.text().unwrap();
        Ok(util::str_version_to_tuple(&text_response))
    }

    pub fn get_animated_corpses(
        &self,
        world_row_i: i32,
        world_col_i: i32,
    ) -> Result<Vec<AnimatedCorpse>, ClientError> {
        let url = format!(
            "{}/ac/?world_row_i={}&world_col_i={}",
            self.get_base_path(),
            world_row_i,
            world_col_i
        );
        let response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;

        let value = response.json::<Value>().unwrap();
        let mut animated_corpses: Vec<AnimatedCorpse> = vec![];
        for item in value.as_array().unwrap().iter() {
            let animated_corpse: AnimatedCorpse = serde_json::from_value(item.clone()).unwrap();
            animated_corpses.push(animated_corpse);
        }
        Ok(animated_corpses)
    }

    pub fn cache_media(&self, media_name: &str) -> Result<(), ClientError> {
        if Path::new(&format!("cache/{}", media_name)).exists() {
            println!("Media {} already in cache", media_name);
            return Ok(());
        }
        println!("Download media {}", media_name);
        let url = format!("{}/media/{}", self.get_base_path(), media_name);
        let mut response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;
        let mut buf: Vec<u8> = vec![];
        response.copy_to(&mut buf).unwrap();
        fs::create_dir_all(Path::new("cache")).unwrap();
        File::create(Path::new(&format!("cache/{}", media_name)))
            .unwrap()
            .write(&buf)
            .unwrap();
        Ok(())
    }

    pub fn cache_media_bg(&self, media_name: &str) -> Result<(), ClientError> {
        if Path::new(&format!("cache/bg/{}", media_name)).exists() {
            println!("Media bg {} already in cache", media_name);
            return Ok(());
        }
        println!("Download media bg {}", media_name);

        let url = format!("{}/media_bg/{}", self.get_base_path(), media_name);
        let mut response: Response = self.check_response(
            self.client
                .get(url.as_str())
                .basic_auth(self.credentials.0.clone(), Some(self.credentials.1.clone()))
                .send()
                .unwrap(),
        )?;
        let mut buf: Vec<u8> = vec![];
        response.copy_to(&mut buf).unwrap();
        fs::create_dir_all(Path::new("cache/bg")).unwrap();
        File::create(Path::new(&format!("cache/bg/{}", media_name)))
            .unwrap()
            .write(&buf)
            .unwrap();
        Ok(())
    }
}
