use std::{collections::HashMap, env, fs, path::PathBuf, sync::Arc};

use crate::ipc::EngineList;
use crate::types::{Action, VersionedFile};

#[derive(Debug, Default)]
pub struct ActionMap(HashMap<PathBuf, Action>);

impl ActionMap {
    /// Get an action from an action ID by iterating through available actions.
    #[must_use]
    pub fn get_action_by_id(&self, action_id: &String) -> Option<Action> {
        for action in self.0.values() {
            if action.id == *action_id {
                return Some(action.clone());
            }
        }
        None
    }

    /// Get actions grouped by action group
    #[must_use]
    pub fn get_by_group(&self) -> HashMap<String, Vec<Action>> {
        let mut map = HashMap::new();
        for action in self.0.values() {
            map.entry(action.group.clone()).or_default();
            map.entry(action.group.clone())
                .and_modify(|vec: &mut Vec<Action>| vec.push(action.clone()));
        }
        map
    }
}

#[must_use]
pub fn get_action_directory() -> PathBuf {
    if let Ok(env_path) = env::var("TA_ACTION_DIR") {
        PathBuf::from(env_path)
    } else {
        if let Ok(exe_path) = env::current_exe() {
            exe_path.join("actions")
        } else {
            PathBuf::from(".").join("actions")
        }
    }
}

/// Get the list of available engines.
pub fn get_actions(engine_list: &Arc<EngineList>) -> ActionMap {
    let mut actions = HashMap::new();
    let action_dir = get_action_directory();
    if let Ok(exists) = fs::exists(&action_dir) {
        if !exists {
            fs::create_dir_all(action_dir.clone()).unwrap();
            let mut path = PathBuf::from(&action_dir);
            path.push("example.taaction");
            let _ = fs::write(path, include_str!("demo_action.taaction"));
        }
    }
    'action_loop: for path in fs::read_dir(action_dir).unwrap() {
        let path = path.unwrap();
        let filename = path.file_name();
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                continue;
            }
        }

        if let Ok(str) = filename.into_string() {
            if str.ends_with(".taaction") {
                tracing::debug!("Detected possible action {str}");
                if let Ok(res) = fs::read_to_string(path.path()) {
                    if let Ok(versioned_file) = ron::from_str::<VersionedFile>(&res) {
                        if versioned_file.version() != 2 {
                            tracing::warn!("Action {str} uses an incompatible file version.");
                            continue 'action_loop;
                        }
                    }

                    if let Ok(action) = ron::from_str::<Action>(&res) {
                        // Validate that all instructions are available for this action before loading
                        if let Err(missing) = action.check_instructions_available(engine_list) {
                            tracing::warn!(
                                "Couldn't load action {} because instructions {:?} aren't available.",
                                action.friendly_name,
                                missing,
                            );
                            continue 'action_loop;
                        }

                        tracing::info!(
                            "Discovered action {} ({}) at {:?}",
                            action.friendly_name,
                            action.id,
                            path.path(),
                        );

                        actions.insert(path.path(), action);
                    } else {
                        tracing::warn!("Couldn't parse action");
                    }
                } else {
                    tracing::warn!("Couldn't read action");
                }
            }
        }
    }
    ActionMap(actions)
}
