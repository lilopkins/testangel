use std::ffi::c_char;

use super::value::ta_parameter_kind;

#[repr(C)]
pub struct ta_instruction_metadata {
    pub szId: *const c_char,
    pub szFriendlyName: *const c_char,
    pub szLuaName: *const c_char,
    pub szDescription: *const c_char,
    pub arpParameterList: *mut *mut ta_instruction_named_kind,
    pub arpOutputList: *mut *mut ta_instruction_named_kind,
}

#[repr(C)]
pub struct ta_instruction_named_kind {
    pub szId: *const c_char,
    pub szName: *const c_char,
    pub kind: ta_parameter_kind,
}
