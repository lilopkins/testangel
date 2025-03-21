use std::ffi::c_char;

#[repr(C)]
pub struct ta_result {
    /// A result code
    pub code: ta_result_code,
    /// If `code` is anything other than `TESTANGEL_OK`, a pointer to additional reason string.
    pub szReason: *const c_char,
}

#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
pub enum ta_result_code {
    /// The action completed successfully.
    TESTANGEL_OK = 0,
    /// You have asked this engine to run an instruction that it is not able to run.
    TESTANGEL_ERROR_INVALID_INSTRUCTION = 1,
    /// You are missing a parameter needed to execute.
    TESTANGEL_ERROR_MISSING_PARAMETER = 2,
    /// You have supplied a parameter which is unexpected.
    TESTANGEL_ERROR_INVALID_PARAMETER = 3,
    /// You have submitted a parameter with an invalid type.
    TESTANGEL_ERROR_INVALID_PARAMETER_TYPE = 4,
    /// An error occurred within the engine whilst processing the request.
    TESTANGEL_ERROR_ENGINE_PROCESSING = 5,
}
