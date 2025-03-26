use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionV2 {
    /// The data version of this action.
    pub version: usize,
    /// The internal ID of this action. Must be unique.
    pub id: String,
    /// The friendly name of this action.
    pub friendly_name: String,
    /// A description of this action.
    pub description: String,
    /// A group this action belongs to.
    pub group: String,
    /// The author of this action.
    pub author: String,
    /// Whether this action should be visible in the flow editor.
    pub visible: bool,
    /// The Lua code driving this action.
    pub script: String,
    /// A vector of required instruction IDs for this action to work.
    pub required_instructions: Vec<String>,
}

impl ActionV2 {
    #[must_use]
    pub fn upgrade_action(self) -> crate::types::Action {
        let mut script = String::new();

        // Convert metadata to descriptors and prefix to script.
        script.push_str(&format!("--: name {}\n", self.friendly_name));
        script.push_str(&format!("--: group {}\n", self.group));
        script.push_str(&format!("--: creator {}\n", self.author));
        script.push_str(&format!("--: description {}\n", self.description));
        if !self.visible {
            script.push_str("--: hide-in-flow-editor\n");
        }

        script.push_str(&self.script);

        crate::types::Action {
            version: 3,
            id: self.id,
            script,
            required_instructions: Vec::new(), // this will be populated on save
        }
    }
}
