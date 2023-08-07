use std::{collections::HashMap, path::PathBuf, env, fs};

use crate::action::types::Action;

#[derive(Default)]
pub struct ActionMap(HashMap<PathBuf, Action>);

impl ActionMap {
    /// Get an action from an action ID by iterating through available actions.
    pub fn get_action_by_id(&self, action_id: String) -> Option<Action> {
        for (_path, action) in &self.0 {
            if action.id == action_id {
                return Some(action.clone());
            }
        }
        return None;
    }

    /// Get actions grouped by action group
    pub fn get_by_group(&self) -> HashMap<String, Vec<Action>> {
        let mut map = HashMap::new();
        for (_path, action) in &self.0 {
            map.entry(action.group.clone()).or_default();
            map.entry(action.group.clone()).and_modify(|vec: &mut Vec<Action>| vec.push(action.clone()));
        }
        map
    }
}

/// Get the list of available engines.
pub fn get_actions() -> ActionMap {
    let mut actions = HashMap::new();
    let action_dir = env::var("ACTION_DIR").unwrap_or("./actions".to_owned());
    fs::create_dir_all(action_dir.clone()).unwrap();
    for path in fs::read_dir(action_dir).unwrap() {
        let path = path.unwrap();
        let filename = path.file_name();
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                continue;
            }
        }

        if let Ok(str) = filename.into_string() {
            if str.ends_with(".taaction") {
                log::debug!("Detected possible action {str}");
                if let Ok(res) = fs::read_to_string(path.path()) {
                    if let Ok(action) = ron::from_str::<Action>(&res) {
                        log::info!("Discovered action {} at {:?}", action.friendly_name, path.path());
                        actions.insert(
                            path.path(),
                            action,
                        );
                    }
                }
            }
        }
    }
    ActionMap(actions)
}
