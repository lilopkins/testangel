#![warn(clippy::pedantic)]

use std::collections::HashMap;

/// A prelude module to quickly import common imports.
pub mod prelude {
    pub use crate::evidence::{Evidence, EvidenceContent};
    pub use crate::instruction::{
        Instruction, InstructionFlags, InstructionNamedKind, InstructionWithParameters,
    };
    pub use crate::value::{ParameterKind, ParameterValue};
    pub use crate::{ErrorKind, Request, Response};
}

mod evidence;
mod instruction;
mod value;

#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused)]
pub mod ffi;

use prelude::*;

/// The possible request messages that could be sent over the JSON IPC channel.
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    /// Request the list of available instructions from this engine plugin.
    Instructions,
    /// Run the instruction with the provided parameters.
    RunInstruction {
        instruction: InstructionWithParameters,
    },
    /// Reset the state of this engine to the default.
    ResetState,
}

impl Request {
    /// Convert this request to JSON
    ///
    /// # Panics
    ///
    /// Theoretically panics if JSON cannot be produced. This should never occur.
    #[must_use]
    #[deprecated]
    pub fn to_json(&self) -> String {
        panic!()
    }
}

/// The possible response messages that could be sent over the JSON IPC channel.
#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    /// The list of instructions this engine is capable of.
    Instructions {
        /// The friendly name of the engine.
        friendly_name: String,
        /// The description of the engine.
        description: String,
        /// The semver version of this engine
        engine_version: String,
        /// The name of this engine in code.
        engine_lua_name: String,
        /// The version of IPC language that this engine talks, currently must be 1.
        ipc_version: usize,
        /// The list of instructions this engine is capable of.
        instructions: Vec<Instruction>,
    },
    /// Execution finished with the output provided.
    ExecutionOutput {
        /// The execution output.
        output: HashMap<String, ParameterValue>,
        /// The evidence output.
        evidence: Vec<Evidence>,
    },
    /// The state of this engine has been reset.
    StateReset,
    /// An error occured.
    Error { kind: ErrorKind, reason: String },
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ErrorKind {
    /// You have asked this engine to run an instruction that it is not able to run.
    InvalidInstruction,
    /// You are missing a parameter needed to execute.
    MissingParameter,
    /// You have submitted a parameter with an invalid type.
    InvalidParameterType,
    /// An error occurred within the engine whilst processing the request.
    EngineProcessingError,
}
