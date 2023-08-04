use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct TestFlow {
    /// The internal ID of this instruction. Must be unique.
    id: String,
    /// The friendly name of this instruction.
    friendly_name: String,
    /// A description of this instruction.
    description: String,
    /// The parameters this instruction takes, with a friendly name.
    parameters: HashMap<String, (String, ParameterKind)>,
    /// The order of the parameters in the editor.
    parameter_order: Vec<String>,
    /// The outputs this instruction produces, with a friendly name
    outputs: HashMap<String, (String, ParameterKind)>,
    /// The instructions called by this action
    pub instructions: Vec<ActionConfiguration>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfiguration {
    pub action_id: String,
    pub parameter_sources: HashMap<String, ParameterSource>,
    pub parameter_values: HashMap<String, ParameterValue>,
}

impl From<Instruction> for ActionConfiguration {
    fn from(value: Instruction) -> Self {
        let mut parameter_sources = HashMap::new();
        let mut parameter_values = HashMap::new();
        for (id, (_friendly_name, kind)) in value.parameters() {
            parameter_sources.insert(id.clone(), ParameterSource::Literal);
            parameter_values.insert(id.clone(), kind.default_value());
        }
        Self {
            action_id: value.id().clone(),
            parameter_sources,
            parameter_values,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterSource {
    #[default]
    Literal,
    FromOutput(usize, String, String),
}

impl ParameterSource {
    pub(crate) fn text_repr(&self) -> String {
        match self {
            ParameterSource::FromOutput(step, _id, friendly_name) => format!("From Step {}: {}", step + 1, friendly_name),
            ParameterSource::Literal => "Literal".to_owned(),
        }
    }
}
