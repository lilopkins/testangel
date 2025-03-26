use std::{collections::HashMap, ffi::CStr};

use bitflags::bitflags;
use getset::Getters;

use crate::{
    ffi::{instruction::ta_instruction_metadata, value::ta_parameter_kind},
    prelude::*,
    value::ParameterValue,
};

/// An instruction that this engine is capable of providing.
#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct Instruction {
    /// The internal ID of this instruction. Must be unique.
    id: String,
    /// The lua name of this instruction. Must be a valid lua function name.
    lua_name: String,
    /// The friendly name of this instruction.
    friendly_name: String,
    /// A description of this instruction.
    description: String,
    /// Flags for this instruction
    flags: InstructionFlags,
    /// The parameters this instruction takes, with a friendly name.
    parameters: Vec<InstructionNamedKind>,
    /// The outputs this instruction produces, with a friendly name
    outputs: Vec<InstructionNamedKind>,
}

impl Instruction {
    /// Convert from the FFI type to this safe type.
    ///
    /// ## Errors
    ///
    /// Returns an error if some of the data in the instruction isn't valid UTF-8.
    ///
    /// ## Safety
    ///
    /// This function will be safe as long as the following are present on the instruction:
    /// - szId
    /// - szLuaName
    /// - szFriendlyName
    /// - szDescription
    /// - arpParameterList (not a null pointer, correctly NULL terminated)
    /// - arpOutputList (not a null pointer, correctly NULL terminated)
    pub unsafe fn from_ffi(metadata: *const ta_instruction_metadata) -> Result<Self, ()> {
        let id = {
            let cstr = unsafe { CStr::from_ptr((*metadata).szId) };
            let str_slice = cstr.to_str().map_err(|_| ())?;
            str_slice.to_owned()
        };
        let lua_name = {
            let cstr = unsafe { CStr::from_ptr((*metadata).szLuaName) };
            let str_slice = cstr.to_str().map_err(|_| ())?;
            str_slice.to_owned()
        };
        let friendly_name = {
            let cstr = unsafe { CStr::from_ptr((*metadata).szFriendlyName) };
            let str_slice = cstr.to_str().map_err(|_| ())?;
            str_slice.to_owned()
        };
        let description = {
            let cstr = unsafe { CStr::from_ptr((*metadata).szDescription) };
            let str_slice = cstr.to_str().map_err(|_| ())?;
            str_slice.to_owned()
        };

        let flags = InstructionFlags::from_bits_truncate((*metadata).iFlags as u16);

        let mut i = 0;
        let raw_parameters = (*metadata).arpParameterList;
        let mut parameters = vec![];
        loop {
            let parameter_raw = unsafe { *raw_parameters.add(i) };
            if parameter_raw.is_null() {
                break;
            }
            let id = {
                let cstr = unsafe { CStr::from_ptr((*parameter_raw).szId) };
                let str_slice = cstr.to_str().map_err(|_| ())?;
                str_slice.to_owned()
            };
            let friendly_name = {
                let cstr = unsafe { CStr::from_ptr((*parameter_raw).szName) };
                let str_slice = cstr.to_str().map_err(|_| ())?;
                str_slice.to_owned()
            };
            let kind = match (*parameter_raw).kind {
                ta_parameter_kind::TA_PARAMETER_STRING => ParameterKind::String,
                ta_parameter_kind::TA_PARAMETER_BOOLEAN => ParameterKind::Boolean,
                ta_parameter_kind::TA_PARAMETER_DECIMAL => ParameterKind::Decimal,
                ta_parameter_kind::TA_PARAMETER_INTEGER => ParameterKind::Integer,
            };
            let ink = InstructionNamedKind {
                id,
                friendly_name,
                kind,
            };
            parameters.push(ink);
            i += 1;
        }

        let mut i = 0;
        let raw_outputs = (*metadata).arpOutputList;
        let mut outputs = vec![];
        loop {
            let output_raw = unsafe { *raw_outputs.add(i) };
            if output_raw.is_null() {
                break;
            }
            let id = {
                let cstr = unsafe { CStr::from_ptr((*output_raw).szId) };
                let str_slice = cstr.to_str().map_err(|_| ())?;
                str_slice.to_owned()
            };
            let friendly_name = {
                let cstr = unsafe { CStr::from_ptr((*output_raw).szName) };
                let str_slice = cstr.to_str().map_err(|_| ())?;
                str_slice.to_owned()
            };
            let kind = match (*output_raw).kind {
                ta_parameter_kind::TA_PARAMETER_STRING => ParameterKind::String,
                ta_parameter_kind::TA_PARAMETER_BOOLEAN => ParameterKind::Boolean,
                ta_parameter_kind::TA_PARAMETER_DECIMAL => ParameterKind::Decimal,
                ta_parameter_kind::TA_PARAMETER_INTEGER => ParameterKind::Integer,
            };
            let ink = InstructionNamedKind {
                id,
                friendly_name,
                kind,
            };
            outputs.push(ink);
            i += 1;
        }

        Ok(Self {
            id,
            lua_name,
            friendly_name,
            description,
            flags,
            parameters,
            outputs,
        })
    }

    /// Build a new instruction
    #[must_use]
    pub fn new<S>(
        id: S,
        lua_name: S,
        friendly_name: S,
        description: S,
        flags: InstructionFlags,
    ) -> Self
    where
        S: Into<String>,
    {
        Self {
            id: id.into(),
            lua_name: lua_name.into(),
            friendly_name: friendly_name.into(),
            description: description.into(),
            flags,
            parameters: vec![],
            outputs: vec![],
        }
    }

    /// Add a parameter to this instruction.
    #[must_use]
    pub fn with_parameter<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where
        S: Into<String>,
    {
        self.parameters.push(InstructionNamedKind {
            id: id.into(),
            friendly_name: friendly_name.into(),
            kind,
        });
        self
    }

    /// Add a output to this instruction.
    #[must_use]
    pub fn with_output<S>(mut self, id: S, friendly_name: S, kind: ParameterKind) -> Self
    where
        S: Into<String>,
    {
        self.outputs.push(InstructionNamedKind {
            id: id.into(),
            friendly_name: friendly_name.into(),
            kind,
        });
        self
    }

    /// Validate that the provided [`InstructionWithParameters`] matches the
    /// requirements for this instruction.
    ///
    /// # Errors
    ///
    /// - [`ErrorKind::MissingParameter`] if a parameter isn't provided
    /// - [`ErrorKind::InvalidParameterType`] is the type of a provided
    ///   parameter doesn't match
    pub fn validate(&self, iwp: &InstructionWithParameters) -> Result<(), (ErrorKind, String)> {
        for InstructionNamedKind {
            id,
            friendly_name: _,
            kind,
        } in &self.parameters
        {
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
}

#[derive(Clone, Debug, PartialEq, Getters)]
#[getset(get = "pub")]
pub struct InstructionNamedKind {
    pub id: String,
    pub friendly_name: String,
    pub kind: ParameterKind,
}

/// An instruction with it's parameters.
#[derive(Clone, Debug, PartialEq)]
pub struct InstructionWithParameters {
    /// The ID of the instruction to run.
    pub instruction: String,
    /// Should this instruction be triggered as a dry run?
    pub dry_run: bool,
    /// The parameters for the instruction.
    pub parameters: HashMap<String, ParameterValue>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct InstructionFlags: u16 {
        /// No specific flags
        const NONE = 0b0000_0000_0000_0000;
        /// This instruction is pure, i.e. it has no side effects (it doesn't affect
        /// any other external systems and for each identical execution, provides
        /// identical output).
        const PURE = 0b0000_0000_0000_0001;
        /// This instruction is infallible.
        const INFALLIBLE = 0b0000_0000_0000_0010;
        /// This instruction is fully automatic, i.e. it requires no user input at
        /// any stage, regardless of behaviour.
        const AUTOMATIC = 0b0000_0000_0000_0100;
    }
}
