use std::ffi::c_char;

pub mod evidence;
pub mod example_prototypes;
pub mod instruction;
pub mod result;
pub mod value;

#[repr(C)]
pub struct ta_engine_metadata {
    pub iSupportsIpcVersion: u32,
    pub szFriendlyName: *const c_char,
    pub szVersion: *const c_char,
    pub szLuaName: *const c_char,
    pub szDescription: *const c_char,
}
