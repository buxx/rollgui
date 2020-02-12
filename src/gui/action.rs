use crate::gui::lang::model::Description;
use doryen_rs::InputApi;
use std::time::{Duration, SystemTime};

#[derive(Debug, Clone)]
pub enum Action {
    NewCharacterId { character_id: String },
    StartupToZone { server_ip: String, server_port: u16 },
    ZoneToWorld,
    WorldToZone,
    ZoneToStartup,
    DescriptionToDescription { description: Description },
    DescriptionToDescriptionGet { url: String },
    ZoneToDescription { url: String },
    DescriptionToZone,
    ExitGame,
}

pub struct ActionManager {
    conditions: Vec<ActionCondition>,
    waiting_key: Option<String>,
    waiting_key_seen: bool,
    wait_since: Option<SystemTime>,
}

pub struct ActionCondition {
    pub keys: Vec<String>,
    pub engine_id: String,
    pub to: Action,
    pub wait_while_key: Option<String>,
}

impl ActionManager {
    pub fn new(conditions: Vec<ActionCondition>) -> Self {
        Self { conditions, waiting_key: None, wait_since: None, waiting_key_seen: false }
    }

    pub fn resolve(&mut self, input: &mut dyn InputApi, engine_id: &str) -> Option<Action> {
        // Protect from to fast interogation (doryen can continue to show pressed input.key)
        if !self.waiting_key_seen {
            if let Some(waiting_key) = self.waiting_key.as_ref() {
                if input.key(waiting_key) {
                    self.waiting_key_seen = true;
                    return None
                }
            }
        }
        else {
            if let Some(waiting_key) = self.waiting_key.as_ref() {
                let max_duration = Duration::new(3, 0);
                if !input.key(waiting_key.as_str()) || self.wait_since.unwrap().elapsed().unwrap() > max_duration {
                    self.waiting_key = None;
                    self.wait_since = None;
                    self.waiting_key_seen = false;
                } else {
                    return None
                }
            }
        }

        'conditions: for condition in self.conditions.iter() {
            if engine_id != condition.engine_id {
                continue 'conditions;
            }

            for key in condition.keys.iter() {
                if !input.key(key.as_str()) {
                    continue 'conditions;
                }
            }

            // all requirement passed
            self.waiting_key = condition.wait_while_key.clone();
            self.wait_since = Some(SystemTime::now());
            return Some(condition.to.clone());
        }

        None
    }
}
