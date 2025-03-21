use std::ffi::c_char;

use super::{
    evidence::ta_evidence, instruction::ta_instruction_metadata, result::ta_result,
    ta_engine_metadata, value::ta_named_value,
};

/// Return a list of instructions this engine supports
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_request_instructions(
    pOutputEngineMetadata: *mut ta_engine_metadata,
    parpOutputInstructions: *mut *const *const ta_instruction_metadata,
) -> *mut ta_result {
    todo!()
}

/// Execute an instruction
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_execute(
    szInstructionId: *const c_char,
    arpParameterList: *const *const ta_named_value,
    nParameterCount: u32,
    parpOutputList: *mut *mut *mut ta_named_value,
    parpOutputEvidenceList: *mut *mut *mut ta_evidence,
) -> *mut ta_result {
    todo!()
}

/// Reset engine state
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_reset_state() -> *mut ta_result {
    todo!()
}

/// Free a result struct
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_free_result(pTarget: *const ta_result) {
    todo!()
}

/// Free an engine metadata struct
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_free_engine_metadata(pTarget: *const ta_engine_metadata) {
    todo!()
}

/// Free an array of instruction metadata structs
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_free_instruction_metadata_array(
    arpTarget: *const *const ta_instruction_metadata,
) {
    todo!()
}

/// Free an array of named value structs
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_free_named_value_array(arpTarget: *const *const ta_named_value) {
    todo!()
}

/// Free an array of evidence structs
/// WARNING! This function shows the signature you need to match ONLY (without the prefixed `_`). DO NOT CALL THIS FUNCTION!
#[no_mangle]
extern "C" fn _ta_free_evidence_array(arpTarget: *const *const ta_evidence) {
    todo!()
}
