use std::collections::HashMap;

#[cfg(feature = "schemas")]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{prelude::*, value::ParameterValue};

/// An instruction that this engine is capable of providing.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct Instruction {
    /// The internal ID of this instruction. Must be unique.
    id: String,
    /// The lua name of this instruction. Must be a valid lua function name.
    lua_name: String,
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
    /// The order of the outputs in the editor.
    output_order: Vec<String>,
}

impl Instruction {
    /// Build a new instruction
    pub fn new<S>(id: S, lua_name: S, friendly_name: S, description: S) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            lua_name: lua_name.into(),
            friendly_name: friendly_name.into(),
            description: description.into(),
            parameters: HashMap::new(),
            parameter_order: Vec::new(),
            outputs: HashMap::new(),
            output_order: Vec::new(),
        }
    }

    /// Get the lua name for this instruction
    pub fn lua_name(&self) -> &String {
        &self.lua_name
    }

    /// Get the friendly name of this instruction
    pub fn friendly_name(&self) -> &String {
        &self.friendly_name
    }

    /// Add a parameter to this instruction.
    pub fn with_parameter<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where
        S: Into<String>,
    {
        let id = id.into();
        self.parameters
            .insert(id.clone(), (friendly_name.into(), kind));
        self.parameter_order.push(id.clone());
        self
    }

    /// Add a output to this instruction.
    pub fn with_output<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where
        S: Into<String>,
    {
        let id = id.into();
        self.outputs
            .insert(id.clone(), (friendly_name.into(), kind));
        self.output_order.push(id.clone());
        self
    }

    pub fn validate(&self, iwp: &InstructionWithParameters) -> Result<(), (ErrorKind, String)> {
        for (id, (_, kind)) in &self.parameters {
            if !iwp.parameters.contains_key(id) {
                return Err((
                    ErrorKind::MissingParameter,
                    format!("Missing parameter {id} from call to {}", iwp.instruction),
                ));
            }

            if iwp.parameters[id].kind() != *kind {
                return Err((
                    ErrorKind::InvalidParameterType,
                    format!(
                        "Invalid kind of parameter {id} from call to {}",
                        iwp.instruction
                    ),
                ));
            }
        }

        Ok(())
    }

    /// Get the ID of this instruction
    pub fn id(&self) -> &String {
        &self.id
    }

    /// Get the description of this instruction
    pub fn description(&self) -> &String {
        &self.description
    }

    /// Get the parameters of this instruction
    pub fn parameters(&self) -> &HashMap<String, (String, ParameterKind)> {
        &self.parameters
    }

    /// Get the order of parameters of this instruction
    pub fn parameter_order(&self) -> &Vec<String> {
        &self.parameter_order
    }

    /// Get the outputs of this instruction
    pub fn outputs(&self) -> &HashMap<String, (String, ParameterKind)> {
        &self.outputs
    }

    /// Get the order of outputs of this instruction
    pub fn output_order(&self) -> &Vec<String> {
        &self.output_order
    }
}

/// An instruction with it's parameters.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub struct InstructionWithParameters {
    /// The ID of the instruction to run.
    pub instruction: String,
    /// The parameters for the instruction.
    pub parameters: HashMap<String, ParameterValue>,
}
