use crate::entity::build::Build;
use crate::error::RollingError;
use crate::server::client::{ItemModel, ListOfItemModel};
use serde::ser::{Serialize, SerializeStruct, Serializer};
use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};
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
pub const REQUEST_CHAT: &str = "REQUEST_CHAT";
pub const NEW_CHAT_MESSAGE: &str = "NEW_CHAT_MESSAGE";
pub const ANIMATED_CORPSE_MOVE: &str = "ANIMATED_CORPSE_MOVE";
pub const TOP_BAR_MESSAGE: &str = "TOP_BAR_MESSAGE";

#[derive(SerializeDerive, DeserializeDerive, Debug)]
#[serde(untagged)]
pub enum TopBarMessageType {
    NORMAL,
    ERROR,
}

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
        stuff_count: i32,
        resource_count: i32,
        build_count: i32,
        character_count: i32,
    },
    ClickActionEvent {
        base_url: String,
        row_i: i16,
        col_i: i16,
    },
    NewResumeText {
        resume: Vec<ItemModel>,
    },
    NewBuild {
        build: Build,
    },
    RequestChat {
        character_id: String,
        previous_conversation_id: Option<i32>,
        message_count: i32,
        next: bool,
        previous: bool,
    },
    NewChatMessage {
        character_id: String,
        conversation_id: Option<i32>,
        conversation_title: Option<String>,
        message: String,
    },
    AnimatedCorpseMove {
        to_row_i: i32,
        to_col_i: i32,
        animated_corpse_id: i32,
    },
    TopBarMessage {
        message: String,
        type_: TopBarMessageType,
    },
}

#[derive(SerdeSerialize, SerdeDeserialize, Debug)]
pub struct NewChatMessage {
    pub conversation_id: Option<i32>,
    pub conversation_title: Option<String>,
    pub message: String,
    pub character_id: String,
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
                let stuff_count: i32 = data["stuff_count"].as_i64().unwrap() as i32;
                let resource_count: i32 = data["resource_count"].as_i64().unwrap() as i32;
                let build_count: i32 = data["build_count"].as_i64().unwrap() as i32;
                let character_count: i32 = data["character_count"].as_i64().unwrap() as i32;

                Ok(ZoneEvent {
                    event_type_name: String::from(THERE_IS_AROUND),
                    event_type: ZoneEventType::ThereIsAround {
                        stuff_count,
                        resource_count,
                        build_count,
                        character_count,
                    },
                })
            }
            &NEW_RESUME_TEXT => {
                let list_of_items: ListOfItemModel =
                    serde_json::from_value(data.get("resume").unwrap().clone()).unwrap();
                Ok(ZoneEvent {
                    event_type_name: String::from(NEW_RESUME_TEXT),
                    // FIXME BS NOW
                    event_type: ZoneEventType::NewResumeText {
                        resume: list_of_items.items,
                    },
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
            &NEW_CHAT_MESSAGE => {
                let new_chat_message: NewChatMessage =
                    serde_json::from_value(data.clone()).unwrap();
                Ok(ZoneEvent {
                    event_type_name: String::from(NEW_CHAT_MESSAGE),
                    event_type: ZoneEventType::NewChatMessage {
                        conversation_id: new_chat_message.conversation_id,
                        conversation_title: new_chat_message.conversation_title,
                        message: new_chat_message.message,
                        character_id: new_chat_message.character_id,
                    },
                })
            }
            &ANIMATED_CORPSE_MOVE => Ok(ZoneEvent {
                event_type_name: String::from(ANIMATED_CORPSE_MOVE),
                event_type: ZoneEventType::AnimatedCorpseMove {
                    to_row_i: data["to_row_i"].as_i64().unwrap() as i32,
                    to_col_i: data["to_col_i"].as_i64().unwrap() as i32,
                    animated_corpse_id: data["animated_corpse_id"].as_i64().unwrap() as i32,
                },
            }),
            &TOP_BAR_MESSAGE => Ok(ZoneEvent {
                event_type_name: String::from(TOP_BAR_MESSAGE),
                event_type: ZoneEventType::TopBarMessage {
                    message: data["message"].as_str().unwrap().to_string(),
                    type_: match data["type_"].as_str().unwrap() {
                        "ERROR" => TopBarMessageType::ERROR,
                        _ => TopBarMessageType::NORMAL,
                    },
                },
            }),
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
