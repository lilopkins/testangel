use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Action {
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
    FromOutput,
}

impl ParameterSource {
    pub(crate) fn text_repr(&self) -> &'static str {
        match self {
            ParameterSource::FromOutput => "From Output",
            ParameterSource::Literal => "Literal",
        }
    }
}
