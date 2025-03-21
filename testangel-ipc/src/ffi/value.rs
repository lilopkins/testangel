use std::ffi::c_char;

#[repr(C)]
pub struct ta_named_value {
    pub szName: *const c_char,
    pub value: ta_value,
}

#[repr(C)]
pub struct ta_value {
    pub kind: ta_parameter_kind,
    pub value: ta_inner_value,
}

#[repr(C)]
pub union ta_inner_value {
    pub szValue: *const c_char,
    pub iValue: *const i32,
    pub fValue: *const f64,
    pub bValue: *const bool,
}

#[derive(Copy, Clone)]
#[repr(C)]
pub enum ta_parameter_kind {
    TA_PARAMETER_STRING = 0,
    TA_PARAMETER_INTEGER = 1,
    TA_PARAMETER_DECIMAL = 2,
    TA_PARAMETER_BOOLEAN = 3,
}
