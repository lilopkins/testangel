use std::{collections::HashMap, error::Error, ffi::c_char};

pub use dynamic_plugin::plugin_impl;
use dynamic_plugin::plugin_interface;
use getset::Getters;
pub use lazy_static::lazy_static;

pub use testangel_engine_macros::*;
pub use testangel_ipc::ffi::{
    evidence::ta_evidence,
    instruction::ta_instruction_metadata,
    result::{ta_result, ta_result_code},
    ta_engine_metadata,
    value::ta_named_value,
};
pub use testangel_ipc::prelude::*;

plugin_interface! {
    extern trait EngineInterface {
        /// Return a list of instructions this engine supports
        fn ta_request_instructions(
            p_output_engine_metadata: *mut ta_engine_metadata,
            parp_output_instructions: *mut *const *const ta_instruction_metadata,
        ) -> *mut ta_result;

        /// Execute an instruction
        fn ta_execute(
            sz_instruction_id: *const c_char,
            arp_parameter_list: *const *const ta_named_value,
            n_parameter_count: u32,
            parp_output_list: *mut *mut *mut ta_named_value,
            parp_output_evidence_list: *mut *mut *mut ta_evidence,
        ) -> *mut ta_result;

        /// Reset engine state
        fn ta_reset_state() -> *mut ta_result;

        /// Free a result struct
        fn ta_free_result(p_target: *const ta_result);

        /// Free an engine metadata struct
        fn ta_free_engine_metadata(p_target: *const ta_engine_metadata);

        /// Free an array of instruction metadata structs
        fn ta_free_instruction_metadata_array(arp_target: *const *const ta_instruction_metadata);

        /// Free an array of named value structs
        fn ta_free_named_value_array(arp_target: *const *const ta_named_value);

        /// Free an array of evidence structs
        fn ta_free_evidence_array(arp_target: *const *const ta_evidence);
    }
}

pub type ParameterMap = HashMap<String, ParameterValue>;
pub type OutputMap = HashMap<String, ParameterValue>;
pub type EvidenceList = Vec<Evidence>;

pub type FnEngineInstruction<'a, T> = dyn 'a
    + Send
    + Sync
    + Fn(&mut T, ParameterMap, &mut OutputMap, &mut EvidenceList) -> Result<(), Box<dyn Error>>;

#[derive(Getters)]
pub struct Engine<'a, T: Default + Send + Sync> {
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    version: String,
    #[getset(get = "pub")]
    lua_name: String,
    #[getset(get = "pub")]
    description: String,
    instructions: Vec<Instruction>,
    functions: HashMap<String, Box<FnEngineInstruction<'a, T>>>,
    state: T,
}

impl<'a, T: Default + Send + Sync> Engine<'a, T> {
    /// Create a new engine with the given name
    pub fn new<S: AsRef<str>>(name: S, lua_name: S, version: S, description: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            version: version.as_ref().to_string(),
            lua_name: lua_name.as_ref().to_string(),
            description: description.as_ref().to_string(),
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

    /// Reset the state of the engine
    pub fn reset_state(&mut self) {
        self.state = Default::default();
    }
}
