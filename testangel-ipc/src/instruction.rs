use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::{prelude::*, value::ParameterValue};

/// An instruction that this engine is capable of providing.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct Instruction {
    /// The internal ID of this instruction. Must be unique.
    id: String,
    /// The friendly name of this instruction.
    friendly_name: String,
    /// A description of this instruction.
    description: String,
    /// The parameters this instruction takes, with a friendly name.
    parameters: HashMap<String, (String, ParameterKind)>,
    /// The outputs this instruction produces, with a friendly name
    outputs: HashMap<String, (String, ParameterKind)>,
}

impl Instruction {
    /// Build a new instruction
    pub fn new<S>(id: S, friendly_name: S, description: S) -> Self
    where S: Into<String>
    {
        Self {
            id: id.into(),
            friendly_name: friendly_name.into(),
            description: description.into(),
            parameters: HashMap::new(),
            outputs: HashMap::new(),
        }
    }

    /// Add a parameter to this instruction.
    pub fn with_parameter<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where S: Into<String>
    {
        self.parameters.insert(id.into(), (friendly_name.into(), kind));
        self
    }

    /// Add a output to this instruction.
    pub fn with_output<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where S: Into<String>
    {
        self.outputs.insert(id.into(), (friendly_name.into(), kind));
        self
    }

    pub fn validate(&self, iwp: &InstructionWithParameters) -> Result<(), (ErrorKind, String)> {
        for (id, (_, kind)) in &self.parameters {
            if !iwp.parameters.contains_key(id) {
                return Err((ErrorKind::MissingParameter, format!("Missing parameter {id} from call to {}", iwp.instruction)));
            }

            if iwp.parameters[id].kind() != *kind {
                return Err((ErrorKind::InvalidParameterType, format!("Invalid kind of parameter {id} from call to {}", iwp.instruction)));
            }
        }

        Ok(())
    }
}

/// An instruction with it's parameters.
#[derive(Serialize, Deserialize, Clone, PartialEq)]
pub struct InstructionWithParameters {
    /// The ID of the instruction to run.
    pub instruction: String,
    /// The parameters for the instruction.
    pub parameters: HashMap<String, ParameterValue>,
}
