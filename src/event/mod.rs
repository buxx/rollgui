use crate::entity::build::Build;
use crate::error::RollingError;
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde_derive::{Deserialize as DeserializeDerive, Serialize as SerializeDerive};
use serde_json::Value;
use std::collections::HashMap;

pub const PLAYER_MOVE: &str = "PLAYER_MOVE";
pub const CLIENT_WANT_CLOSE: &str = "CLIENT_WANT_CLOSE";
pub const SERVER_PERMIT_CLOSE: &str = "SERVER_PERMIT_CLOSE";
pub const CHARACTER_ENTER_ZONE: &str = "CHARACTER_ENTER_ZONE";
pub const CHARACTER_EXIT_ZONE: &str = "CHARACTER_EXIT_ZONE";
pub const CLIENT_REQUIRE_AROUND: &str = "CLIENT_REQUIRE_AROUND";
pub const THERE_IS_AROUND: &str = "THERE_IS_AROUND";
pub const CLICK_ACTION_EVENT: &str = "CLICK_ACTION_EVENT";
pub const NEW_RESUME_TEXT: &str = "NEW_RESUME_TEXT";
pub const NEW_BUILD: &str = "NEW_BUILD";

#[derive(SerializeDerive, DeserializeDerive, Debug)]
#[serde(untagged)]
pub enum ZoneEventType {
    // FIXME rename into ClientClosing
    ClientWantClose,
    // FIXME rename into ClientClosingAcknowledge
    ServerPermitClose,
    PlayerMove {
        to_row_i: i32,
        to_col_i: i32,
        character_id: String,
    },
    CharacterEnter {
        zone_row_i: i32,
        zone_col_i: i32,
        character_id: String,
    },
    CharacterExit {
        character_id: String,
    },
    ClientRequireAround {
        zone_row_i: i32,
        zone_col_i: i32,
        character_id: String,
    },
    ThereIsAround {
        items: Vec<(String, Option<String>)>,
    },
    ClickActionEvent {
        base_url: String,
        row_i: i16,
        col_i: i16,
    },
    NewResumeText {
        resume: Vec<(String, Option<String>)>,
    },
    NewBuild {
        build: Build,
    },
}

#[derive(Debug)]
pub struct ZoneEvent {
    pub event_type: ZoneEventType,
    pub event_type_name: String,
}

impl ZoneEvent {
    // TODO: by hand for now ... how to do automatic ?
    pub fn from_value(value: Value) -> Result<Self, RollingError> {
        let type_ = value["type"].as_str().unwrap();
        let data = value.get("data").unwrap();

        match &type_ {
            &PLAYER_MOVE => Ok(ZoneEvent {
                event_type_name: String::from(PLAYER_MOVE),
                event_type: ZoneEventType::PlayerMove {
                    to_row_i: data["to_row_i"].as_i64().unwrap() as i32,
                    to_col_i: data["to_col_i"].as_i64().unwrap() as i32,
                    character_id: String::from(data["character_id"].as_str().unwrap()),
                },
            }),
            &CLIENT_WANT_CLOSE => Ok(ZoneEvent {
                event_type_name: String::from(CLIENT_WANT_CLOSE),
                event_type: ZoneEventType::ClientWantClose,
            }),
            &SERVER_PERMIT_CLOSE => Ok(ZoneEvent {
                event_type_name: String::from(SERVER_PERMIT_CLOSE),
                event_type: ZoneEventType::ServerPermitClose,
            }),
            &CHARACTER_ENTER_ZONE => Ok(ZoneEvent {
                event_type_name: String::from(CHARACTER_ENTER_ZONE),
                event_type: ZoneEventType::CharacterEnter {
                    zone_row_i: data["zone_row_i"].as_i64().unwrap() as i32,
                    zone_col_i: data["zone_col_i"].as_i64().unwrap() as i32,
                    character_id: String::from(data["character_id"].as_str().unwrap()),
                },
            }),
            &CHARACTER_EXIT_ZONE => Ok(ZoneEvent {
                event_type_name: String::from(CHARACTER_EXIT_ZONE),
                event_type: ZoneEventType::CharacterExit {
                    character_id: String::from(data["character_id"].as_str().unwrap()),
                },
            }),
            &THERE_IS_AROUND => {
                let mut items: Vec<(String, Option<String>)> = vec![];
                for value in data["items"].as_array().unwrap() {
                    let vector = value.as_array().unwrap();
                    let text = vector.get(0).unwrap().as_str().unwrap().to_string();
                    let mut url: Option<String> = None;
                    if let Some(txt) = vector.get(1).unwrap().as_str() {
                        url = Some(txt.to_string());
                    }
                    items.push((text, url));
                }

                Ok(ZoneEvent {
                    event_type_name: String::from(THERE_IS_AROUND),
                    event_type: ZoneEventType::ThereIsAround { items },
                })
            }
            &NEW_RESUME_TEXT => {
                let mut items: Vec<(String, Option<String>)> = vec![];
                for value in data["resume"].as_array().unwrap() {
                    let vector = value.as_array().unwrap();
                    let text = vector.get(0).unwrap().as_str().unwrap().to_string();
                    let mut url: Option<String> = None;
                    if let Some(txt) = vector.get(1).unwrap().as_str() {
                        url = Some(txt.to_string());
                    }
                    items.push((text, url));
                }

                Ok(ZoneEvent {
                    event_type_name: String::from(NEW_RESUME_TEXT),
                    event_type: ZoneEventType::NewResumeText { resume: items },
                })
            }
            &NEW_BUILD => {
                let build_data = data["build"].as_object().unwrap();
                let mut classes: Vec<String> = vec![];
                for value in build_data["classes"].as_array().unwrap() {
                    let class = value.as_str().unwrap();
                    classes.push(class.to_string());
                }
                let mut traversable: HashMap<String, bool> = HashMap::new();
                traversable.insert(
                    "WALKING".to_string(),
                    build_data["traversable"]
                        .as_object()
                        .unwrap()
                        .get("WALKING")
                        .unwrap()
                        .as_bool()
                        .unwrap(),
                );

                Ok(ZoneEvent {
                    event_type_name: String::from(NEW_BUILD),
                    event_type: ZoneEventType::NewBuild {
                        build: Build {
                            id: build_data["id"].as_i64().unwrap() as i32,
                            build_id: build_data["build_id"].as_str().unwrap().to_string(),
                            row_i: build_data["row_i"].as_i64().unwrap() as i32,
                            col_i: build_data["col_i"].as_i64().unwrap() as i32,
                            classes,
                            traversable,
                        },
                    },
                })
            }
            _ => Err(RollingError {
                message: format!("Unknown event {}", &type_),
            }),
        }
    }
}

impl Serialize for ZoneEvent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("ZoneEvent", 2)?;
        state.serialize_field("type", &self.event_type_name)?;
        state.serialize_field("data", &self.event_type)?;
        state.end()
    }
}
