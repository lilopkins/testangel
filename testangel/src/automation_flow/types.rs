use std::{collections::HashMap, sync::Arc};

use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

use crate::{
    action::{self, types::Action},
    action_loader::ActionMap,
    flow_running::FlowError,
    ipc::EngineMap,
};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct AutomationFlow {
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
    /// The actions called by this flow
    pub actions: Vec<ActionConfiguration>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfiguration {
    pub action_id: String,
    pub parameter_sources: HashMap<usize, ParameterSource>,
    pub parameter_values: HashMap<usize, ParameterValue>,
}
impl ActionConfiguration {
    /// Execute this action
    pub fn execute(
        &self,
        action_map: Arc<ActionMap>,
        engine_map: Arc<EngineMap>,
        previous_action_outputs: Vec<HashMap<usize, ParameterValue>>,
    ) -> Result<(HashMap<usize, ParameterValue>, Vec<Evidence>), FlowError> {
        // Find action by ID
        let action = action_map.get_action_by_id(&self.action_id).unwrap();

        // Build action parameters
        let mut action_parameters = HashMap::new();
        for (id, src) in &self.parameter_sources {
            let value = match src {
                ParameterSource::Literal => self.parameter_values.get(id).unwrap().clone(),
                ParameterSource::FromOutput(step, id, _friendly_name) => previous_action_outputs
                    .get(*step)
                    .unwrap()
                    .get(id)
                    .unwrap()
                    .clone(),
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
                action::types::ParameterSource::Literal => panic!("Output set to literal."),
                action::types::ParameterSource::FromOutput(step, id, _friendly_name) => {
                    instruction_outputs
                        .get(*step)
                        .unwrap()
                        .get(id)
                        .unwrap()
                        .clone()
                }
                action::types::ParameterSource::FromParameter(id, _friendly_name) => {
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
            parameter_sources.insert(id, ParameterSource::Literal);
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
pub enum ParameterSource {
    #[default]
    Literal,
    FromOutput(usize, usize, String),
}

impl ParameterSource {
    pub(crate) fn text_repr(&self) -> String {
        match self {
            ParameterSource::FromOutput(step, _id, friendly_name) => {
                format!("From Step {}: {}", step + 1, friendly_name)
            }
            ParameterSource::Literal => "Literal".to_owned(),
        }
    }
}
