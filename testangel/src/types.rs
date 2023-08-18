use std::{collections::HashMap, fmt, sync::Arc};

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

use crate::{
    action_loader::ActionMap,
    ipc::{self, EngineList, IpcError},
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
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
    /// The parameters this action takes, with a friendly name.
    pub parameters: Vec<(String, ParameterKind)>,
    /// The outputs this action produces, with a friendly name
    pub outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    /// The instructions called by this action
    pub instructions: Vec<InstructionConfiguration>,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            version: 1,
            id: uuid::Uuid::new_v4().to_string(),
            friendly_name: String::new(),
            description: String::new(),
            group: String::new(),
            parameters: Vec::new(),
            outputs: Vec::new(),
            instructions: Vec::new(),
        }
    }
}

impl Action {
    /// Get the version of this action.
    pub fn version(&self) -> usize {
        self.version
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct InstructionConfiguration {
    pub instruction_id: String,
    pub parameter_sources: HashMap<String, InstructionParameterSource>,
    pub parameter_values: HashMap<String, ParameterValue>,
}
impl InstructionConfiguration {
    pub fn execute(
        &self,
        engine_map: Arc<EngineList>,
        action_parameters: &HashMap<usize, ParameterValue>,
        previous_outputs: Vec<HashMap<String, ParameterValue>>,
    ) -> Result<(HashMap<String, ParameterValue>, Vec<Evidence>), FlowError> {
        // Get instruction
        let engine = engine_map
            .get_engine_by_instruction_id(&self.instruction_id)
            .unwrap();

        // Build input parameters
        let mut parameters = HashMap::new();
        for (id, src) in &self.parameter_sources {
            let value = match src {
                InstructionParameterSource::Literal => {
                    self.parameter_values.get(id).unwrap().clone()
                }
                InstructionParameterSource::FromOutput(step, id, _friendly_name) => {
                    previous_outputs
                        .get(*step)
                        .unwrap()
                        .get(id)
                        .unwrap()
                        .clone()
                }
                InstructionParameterSource::FromParameter(id, _friendly_name) => {
                    action_parameters.get(id).unwrap().clone()
                }
            };
            parameters.insert(id.clone(), value);
        }

        // Make IPC call
        let response = ipc::ipc_call(
            engine,
            Request::RunInstructions {
                instructions: vec![InstructionWithParameters {
                    instruction: self.instruction_id.clone(),
                    parameters,
                }],
            },
        )
        .map_err(|e| FlowError::IPCFailure(e))?;

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
            parameter_sources.insert(id.clone(), InstructionParameterSource::Literal);
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
pub enum InstructionParameterSource {
    #[default]
    Literal,
    FromParameter(usize, String),
    FromOutput(usize, String, String),
}

impl InstructionParameterSource {
    /// Get a text representation of this [`ParameterSource`]
    pub fn text_repr(&self) -> String {
        match self {
            Self::FromOutput(step, _id, friendly_name) => {
                format!("From Step {}: {}", step + 1, friendly_name)
            }
            Self::FromParameter(_id, friendly_name) => {
                format!("Parameter: {friendly_name}")
            }
            Self::Literal => "Literal".to_owned(),
        }
    }
}

#[derive(Debug)]
pub enum FlowError {
    FromInstruction {
        error_kind: ErrorKind,
        reason: String,
    },
    IPCFailure(IpcError),
}

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IPCFailure(e) => write!(f, "An IPC call failed ({e:?})."),
            Self::FromInstruction { error_kind, reason } => write!(
                f,
                "An instruction returned an error: {error_kind:?}: {reason}"
            ),
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AutomationFlow {
    /// The actions called by this flow
    pub actions: Vec<ActionConfiguration>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfiguration {
    pub action_id: String,
    pub parameter_sources: HashMap<usize, ActionParameterSource>,
    pub parameter_values: HashMap<usize, ParameterValue>,
}
impl ActionConfiguration {
    /// Execute this action
    pub fn execute(
        &self,
        action_map: Arc<ActionMap>,
        engine_map: Arc<EngineList>,
        previous_action_outputs: Vec<HashMap<usize, ParameterValue>>,
    ) -> Result<(HashMap<usize, ParameterValue>, Vec<Evidence>), FlowError> {
        // Find action by ID
        let action = action_map.get_action_by_id(&self.action_id).unwrap();

        // Build action parameters
        let mut action_parameters = HashMap::new();
        for (id, src) in &self.parameter_sources {
            let value = match src {
                ActionParameterSource::Literal => self.parameter_values.get(id).unwrap().clone(),
                ActionParameterSource::FromOutput(step, id, _friendly_name) => {
                    previous_action_outputs
                        .get(*step)
                        .unwrap()
                        .get(id)
                        .unwrap()
                        .clone()
                }
            };
            action_parameters.insert(id.clone(), value);
        }

        // Iterate through instructions
        let mut instruction_outputs = Vec::new();
        let mut evidence = Vec::new();
        for instruction_config in &action.instructions {
            // Execute instruction
            let (outputs, ev) = instruction_config.execute(
                engine_map.clone(),
                &action_parameters,
                instruction_outputs.clone(),
            )?;
            instruction_outputs.push(outputs);
            evidence = vec![evidence, ev].concat();
        }

        // Generate output map
        let mut output = HashMap::new();
        let mut index = 0;
        for (_friendly_name, _kind, src) in &action.outputs {
            let value = match src {
                InstructionParameterSource::Literal => panic!("Output set to literal."),
                InstructionParameterSource::FromOutput(step, id, _friendly_name) => {
                    instruction_outputs
                        .get(*step)
                        .unwrap()
                        .get(id)
                        .unwrap()
                        .clone()
                }
                InstructionParameterSource::FromParameter(id, _friendly_name) => {
                    action_parameters.get(id).unwrap().clone()
                }
            };
            output.insert(index, value);
            index += 1;
        }

        Ok((output, evidence))
    }
}

impl From<Action> for ActionConfiguration {
    fn from(value: Action) -> Self {
        let mut parameter_sources = HashMap::new();
        let mut parameter_values = HashMap::new();
        let mut id = 0;
        for (_friendly_name, kind) in value.parameters {
            parameter_sources.insert(id, ActionParameterSource::Literal);
            parameter_values.insert(id, kind.default_value());
            id += 1;
        }
        Self {
            action_id: value.id.clone(),
            parameter_sources,
            parameter_values,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionParameterSource {
    #[default]
    Literal,
    FromOutput(usize, usize, String),
}

impl ActionParameterSource {
    /// Get a text representation of this [`ActionParameterSource`].
    pub fn text_repr(&self) -> String {
        match self {
            Self::FromOutput(step, _id, friendly_name) => {
                format!("From Step {}: {}", step + 1, friendly_name)
            }
            Self::Literal => "Literal".to_owned(),
        }
    }
}
