use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// A prelude module to quickly import common imports.
pub mod prelude {
    pub use crate::evidence::{Evidence, EvidenceContent};
    pub use crate::instruction::{Instruction, InstructionWithParameters};
    pub use crate::value::{ParameterKind, ParameterValue};
    pub use crate::{ErrorKind, Request, Response};
}

mod evidence;
mod instruction;
mod value;

use prelude::*;
#[cfg(feature = "schemas")]
use schemars::JsonSchema;

/// The possible request messages that could be sent over the JSON IPC channel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum Request {
    /// Request the list of available instructions from this engine plugin.
    Instructions,
    /// Run the list of instructions given in the order they are listed.
    RunInstructions {
        instructions: Vec<InstructionWithParameters>,
    },
    /// Reset the state of this engine to the default.
    ResetState,
}

impl Request {
    /// Convert this request to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl TryFrom<String> for Request {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        serde_json::from_str(&value)
    }
}

/// The possible response messages that could be sent over the JSON IPC channel.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
#[serde(tag = "type")]
pub enum Response {
    /// The list of instructions this engine is capable of.
    Instructions {
        /// The friendly name of the engine.
        friendly_name: String,
        /// The semver version of this engine
        engine_version: String,
        /// The version of IPC language that this engine talks, currently must be 1.
        ipc_version: usize,
        /// The list of instructions this engine is capable of.
        instructions: Vec<Instruction>,
    },
    /// Execution finished with the output provided.
    ExecutionOutput {
        /// TOrder matches the list of instructions sent originally.he execution output. Order matches the list of instructions sent originally.
        output: Vec<HashMap<String, ParameterValue>>,
        /// The evidence output. Order matches the list of instructions sent originally.
        evidence: Vec<Vec<Evidence>>,
    },
    /// The state of this engine has been reset.
    StateReset,
    /// An error occured.
    Error { kind: ErrorKind, reason: String },
}

impl Response {
    /// Convert this response to JSON
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap()
    }
}

impl TryFrom<String> for Response {
    type Error = serde_json::Error;

    fn try_from(value: String) -> Result<Self, <Self as TryFrom<String>>::Error> {
        serde_json::from_str(&value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "schemas", derive(JsonSchema))]
pub enum ErrorKind {
    /// The IPC JSON request couldn't be parsed.
    FailedToParseIPCJson,
    /// You have asked this engine to run an instruction that it is not able to run.
    InvalidInstruction,
    /// You are missing a parameter needed to execute.
    MissingParameter,
    /// You have submitted a parameter with an invalid type.
    InvalidParameterType,
    /// An error occurred within the engine whilst processing the request.
    EngineProcessingError,
}
