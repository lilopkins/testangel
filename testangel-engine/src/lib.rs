use std::{collections::HashMap, error::Error};

pub use testangel_engine_macros::expose_engine;
pub use testangel_ipc::prelude::*;

pub type ParameterMap = HashMap<String, ParameterValue>;
pub type OutputMap = HashMap<String, ParameterValue>;
pub type EvidenceList = Vec<Evidence>;

pub type FnEngineInstruction<'a, T> = dyn 'a
    + Send
    + Sync
    + Fn(&mut T, ParameterMap, &mut OutputMap, &mut EvidenceList) -> Result<(), Box<dyn Error>>;

pub struct Engine<'a, T: Default + Send + Sync> {
    name: String,
    version: String,
    lua_name: String,
    instructions: Vec<Instruction>,
    functions: HashMap<String, Box<FnEngineInstruction<'a, T>>>,
    state: T,
}

impl<'a, T: Default + Send + Sync> Engine<'a, T> {
    /// Create a new engine with the given name
    pub fn new<S: AsRef<str>>(name: S, lua_name: S, version: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            version: version.as_ref().to_string(),
            lua_name: lua_name.as_ref().to_string(),
            instructions: vec![],
            state: Default::default(),
            functions: HashMap::new(),
        }
    }

    /// Add an instruction to this engine.
    pub fn with_instruction<F>(mut self, instruction: Instruction, execute: F) -> Self
    where
        F: 'a
            + Send
            + Sync
            + Fn(
                &mut T,
                ParameterMap,
                &mut OutputMap,
                &mut EvidenceList,
            ) -> Result<(), Box<dyn Error>>,
    {
        self.functions
            .insert(instruction.id().clone(), Box::new(execute));
        self.instructions.push(instruction);
        self
    }

    /// Process a request and produce a response
    pub fn process_request(&mut self, request: Request) -> Response {
        match request {
            Request::ResetState => {
                self.state = Default::default();
                Response::StateReset
            }

            Request::Instructions => {
                // Provide a list of instructions this engine can run.
                Response::Instructions {
                    friendly_name: self.name.clone(),
                    engine_version: self.version.clone(),
                    engine_lua_name: self.lua_name.clone(),
                    ipc_version: 2,
                    instructions: self.instructions.clone(),
                }
            }

            Request::RunInstructions {
                instructions: requested_instructions,
            } => {
                let mut output = Vec::new();
                let mut evidence = Vec::new();
                'request_loop: for requested_instruction_with_params in requested_instructions {
                    for instruction in &self.instructions {
                        if requested_instruction_with_params.instruction == *instruction.id() {
                            // run this instruction
                            // Validate parameters
                            if let Err((kind, reason)) =
                                instruction.validate(&requested_instruction_with_params)
                            {
                                return Response::Error { kind, reason };
                            }

                            let parameters = requested_instruction_with_params.parameters;

                            // Execute instruction
                            let f = &self.functions[instruction.id()];
                            let mut this_instruction_output = OutputMap::new();
                            let mut this_instruction_evidence = EvidenceList::new();
                            let instruction_result = f(
                                &mut self.state,
                                parameters,
                                &mut this_instruction_output,
                                &mut this_instruction_evidence,
                            );
                            if let Err(e) = instruction_result {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: format!("{e}"),
                                };
                            }

                            evidence.push(this_instruction_evidence);
                            output.push(this_instruction_output);

                            continue 'request_loop;
                        }
                    }

                    // If the requested instruction doesn't match:
                    return Response::Error {
                        kind: ErrorKind::InvalidInstruction,
                        reason: format!(
                            "The requested instruction {} could not be handled by this engine.",
                            requested_instruction_with_params.instruction
                        ),
                    };
                }

                Response::ExecutionOutput { output, evidence }
            }
        }
    }
}
