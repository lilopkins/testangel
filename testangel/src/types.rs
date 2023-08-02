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
}

impl From<Instruction> for InstructionConfiguration {
    fn from(value: Instruction) -> Self {
        let mut parameter_sources = HashMap::new();
        for (id, (_friendly_name, _kind)) in value.parameters() {
            parameter_sources.insert(id.clone(), ParameterSource::Literal);
        }
        Self {
            instruction_id: value.id().clone(),
            parameter_sources,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ParameterSource {
    #[default]
    Literal,
    FromOutput,
}
