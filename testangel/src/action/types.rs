use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The internal ID of this action. Must be unique.
    pub id: String,
    /// The friendly name of this action.
    pub friendly_name: String,
    /// A description of this action.
    pub description: String,
    /// A group this action belongs to.
    pub group: String,
    /// The parameters this action takes, with a friendly name.
    pub parameters: Vec<(String, ParameterKind)>,
    /// The outputs this action produces, with a friendly name
    pub outputs: Vec<(String, ParameterKind, ParameterSource)>,
    /// The instructions called by this action
    pub instructions: Vec<InstructionConfiguration>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InstructionConfiguration {
    pub instruction_id: String,
    pub parameter_sources: HashMap<String, ParameterSource>,
    pub parameter_values: HashMap<String, ParameterValue>,
}

impl From<Instruction> for InstructionConfiguration {
    fn from(value: Instruction) -> Self {
        let mut parameter_sources = HashMap::new();
        let mut parameter_values = HashMap::new();
        for (id, (_friendly_name, kind)) in value.parameters() {
            parameter_sources.insert(id.clone(), ParameterSource::Literal);
            parameter_values.insert(id.clone(), kind.default_value());
        }
        Self {
            instruction_id: value.id().clone(),
            parameter_sources,
            parameter_values,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterSource {
    #[default]
    Literal,
    FromParameter(usize, String),
    FromOutput(usize, String, String),
}

impl ParameterSource {
    pub(crate) fn text_repr(&self) -> String {
        match self {
            ParameterSource::FromOutput(step, _id, friendly_name) => {
                format!("From Step {}: {}", step + 1, friendly_name)
            }
            ParameterSource::FromParameter(_id, friendly_name) => {
                format!("Parameter: {friendly_name}")
            }
            ParameterSource::Literal => "Literal".to_owned(),
        }
    }
}
