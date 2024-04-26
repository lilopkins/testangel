use std::collections::HashMap;

use convert_case::{Case, Casing};
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
        let mut script = String::new();

        // Add descriptors
        for (name, kind) in &self.parameters {
            script.push_str(&format!("--: param {} {}\n", kind, name));
        }
        for (name, kind, _src) in self.outputs {
            script.push_str(&format!("--: return {} {}\n", kind, name));
        }

        // Add function signature
        let mut params = String::new();
        for (name, _kind) in self.parameters {
            params.push_str(&format!("{}, ", name.to_case(Case::Snake)));
        }
        // remove the last ", "
        let _ = params.pop();
        let _ = params.pop();
        script.push_str(&format!("function run_action({})\n", params));

        // add steps
        for step in self.instructions {
            if !step.comment.is_empty() {
                script.push_str(&format!(
                    "  -- {}\n",
                    step.comment,
                ));
            }
            script.push_str(&format!(
                "  -- instr: {} | runif: {:?} | srcs: {:?} | vals: {:?}\n",
                step.instruction_id,
                step.run_if,
                step.parameter_sources,
                step.parameter_values,
            ));
        }

        // end function
        script.push_str("end\n");

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
