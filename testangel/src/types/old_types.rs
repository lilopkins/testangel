use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionV1 {
    /// The data version of this action.
    version: usize,
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
    /// The parameters this action takes, with a friendly name.
    pub parameters: Vec<(String, ParameterKind)>,
    /// The outputs this action produces, with a friendly name
    pub outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    /// The instructions called by this action
    instructions: Vec<InstructionConfiguration>,
}

impl ActionV1 {
    pub fn upgrade_action(self) -> crate::types::Action {
        let mut instruction_script = String::new();
        for ins_cfg in self.instructions {
            if !ins_cfg.comment.is_empty() {
                instruction_script.push_str(&format!("  -- {}", ins_cfg.comment));
            }
            // TODO: need to load instruction and build lines here
            todo!();
        }

        let script = format!(
            "--[[\n  {}\n  Author: {}\n  Description: {}\n--]]\nfunction run_action({})\n{}\n{}\nend",
            self.friendly_name,
            self.author,
            self.description,
            "params",
            instruction_script,
            "outputs",
        );
        crate::types::Action {
            version: 2,
            id: self.id,
            friendly_name: self.friendly_name,
            description: self.description,
            group: self.group,
            author: self.author,
            visible: self.visible,
            script,
            required_instructions: Vec::new(), // this will be populated on save
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct InstructionConfiguration {
    pub instruction_id: String,
    pub comment: String,
    /// Run If can depend on any boolean parameter, or if set to 'Literal' will always run.
    pub run_if: InstructionParameterSource,
    pub parameter_sources: HashMap<String, InstructionParameterSource>,
    pub parameter_values: HashMap<String, ParameterValue>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstructionParameterSource {
    #[default]
    Literal,
    FromParameter(usize),
    FromOutput(usize, String),
}
