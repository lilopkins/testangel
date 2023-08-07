use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

use crate::{
    flow_running::FlowError,
    ipc::{self, EngineMap},
};

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
impl InstructionConfiguration {
    pub fn execute(
        &self,
        engine_map: Arc<EngineMap>,
        action_parameters: &HashMap<usize, ParameterValue>,
        previous_outputs: Vec<HashMap<String, ParameterValue>>,
    ) -> Result<(HashMap<String, ParameterValue>, Vec<Evidence>), FlowError> {
        // Get instruction
        let (_instruction, engine_path) = engine_map
            .get_instruction_and_engine_path_by_id(&self.instruction_id)
            .unwrap();

        // Build input parameters
        let mut parameters = HashMap::new();
        for (id, src) in &self.parameter_sources {
            let value = match src {
                ParameterSource::Literal => self.parameter_values.get(id).unwrap().clone(),
                ParameterSource::FromOutput(step, id, _friendly_name) => previous_outputs
                    .get(*step)
                    .unwrap()
                    .get(id)
                    .unwrap()
                    .clone(),
                ParameterSource::FromParameter(id, _friendly_name) => {
                    action_parameters.get(id).unwrap().clone()
                }
            };
            parameters.insert(id.clone(), value);
        }

        // Make IPC call
        let response = ipc::ipc_call(
            engine_path,
            Request::RunInstructions {
                instructions: vec![InstructionWithParameters {
                    instruction: self.instruction_id.clone(),
                    parameters,
                }],
            },
        );
        if response.is_err() {
            return Err(FlowError::IPCFailure);
        }
        let response = response.unwrap();

        // Generate output table and return
        match response {
            Response::ExecutionOutput { output, evidence } => {
                return Ok((output[0].clone(), evidence[0].clone()));
            }
            Response::Error { kind, reason } => {
                return Err(FlowError::FromInstruction {
                    error_kind: kind,
                    reason,
                })
            }
            _ => unreachable!(),
        }
    }
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
