use std::ffi::c_char;

use super::value::ta_parameter_kind;

#[repr(C)]
pub struct ta_instruction_metadata {
    pub szId: *const c_char,
    pub szFriendlyName: *const c_char,
    pub szLuaName: *const c_char,
    pub szDescription: *const c_char,
    /// Instruction behaviour flags. See `ta_instruction_flag` for options.
    pub iFlags: ta_instruction_flag,
    pub arpParameterList: *mut *mut ta_instruction_named_kind,
    pub arpOutputList: *mut *mut ta_instruction_named_kind,
}

#[repr(C)]
pub struct ta_instruction_named_kind {
    pub szId: *const c_char,
    pub szName: *const c_char,
    pub kind: ta_parameter_kind,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum ta_instruction_flag {
    /// No specific flags
    TA_INSTRUCTION_FLAG_NONE = 0b0000_0000_0000_0000,
    /// This instruction is pure, i.e. it has no side effects (it doesn't affect
    /// any other external systems and for each identical execution, provides
    /// identical output).
    TA_INSTRUCTION_FLAG_PURE = 0b0000_0000_0000_0001,
    /// This instruction is infallible.
    TA_INSTRUCTION_FLAG_INFALLIBLE = 0b0000_0000_0000_0010,
    /// This instruction is fully automatic, i.e. it requires no user input at
    /// any stage, regardless of behaviour.
    TA_INSTRUCTION_FLAG_AUTOMATIC = 0b0000_0000_0000_0100,
}
