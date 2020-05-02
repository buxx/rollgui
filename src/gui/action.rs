use crate::gui::lang::model::Description;
use doryen_rs::InputApi;

#[derive(Debug, Clone)]
pub enum Action {
    NewCharacterId {
        character_id: String,
    },
    StartupToZone {
        server_ip: String,
        server_port: u16,
    },
    ZoneToWorld,
    WorldToZone,
    ZoneToConfirmExit,
    ConfirmExitToZone,
    ToStartup,
    DescriptionToDescription {
        description: Description,
        back_url: Option<String>,
    },
    DescriptionToDescriptionGet {
        url: String,
        back_url: Option<String>,
    },
    ZoneToDescription {
        url: String,
    },
    DescriptionToZone,
    ExitGame,
}

pub struct ActionManager {
    conditions: Vec<ActionCondition>,
}

pub struct ActionCondition {
    pub keys: Vec<String>,
    pub engine_id: String,
    pub to: Action,
    pub wait_while_key: Option<String>,
}

impl ActionManager {
    pub fn new(conditions: Vec<ActionCondition>) -> Self {
        Self { conditions }
    }

    pub fn resolve(&mut self, input: &mut dyn InputApi, engine_id: &str) -> Option<Action> {
        'conditions: for condition in self.conditions.iter() {
            if engine_id != condition.engine_id {
                continue 'conditions;
            }

            for key in condition.keys.iter() {
                if !input.key_pressed(key.as_str()) {
                    continue 'conditions;
                }
            }
            return Some(condition.to.clone());
        }

        None
    }
}
