use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use regex::Regex;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_VALIDATE: Instruction = Instruction::new(
        "regex-validate",
        "Check with Regular Expression",
        "Checks that input text matches a regular expression. This will cause the test flow to error if the text doesn't match.",
    )
    .with_parameter("regex", "Regular Expression", ParameterKind::String)
    .with_parameter("input", "Input", ParameterKind::String)
    .with_parameter("error", "Error Message", ParameterKind::String);
}

#[no_mangle]
pub unsafe extern "C" fn ta_call(input: *const c_char) -> *mut c_char {
    let input = CStr::from_ptr(input);
    let response = call_internal(String::from_utf8_lossy(input.to_bytes()).to_string());
    let c_response = CString::new(response).expect("valid response");
    c_response.into_raw()
}

#[no_mangle]
pub unsafe extern "C" fn ta_release(input: *mut c_char) {
    if !input.is_null() {
        drop(CString::from_raw(input));
    }
}

fn call_internal(request: String) -> String {
    match Request::try_from(request) {
        Err(e) => Response::Error {
            kind: ErrorKind::FailedToParseIPCJson,
            reason: format!("The IPC message was invalid. ({:?})", e),
        }
        .to_json(),
        Ok(request) => process_request(request).to_json(),
    }
}

fn process_request(request: Request) -> Response {
    match request {
        Request::ResetState => {
            // Nothing to do
            Response::StateReset
        }
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            Response::Instructions {
                friendly_name: "Regular Expressions".to_owned(),
                instructions: vec![INSTRUCTION_VALIDATE.clone()],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_VALIDATE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_VALIDATE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let regex = &i.parameters["regex"];
                    let input = &i.parameters["input"];
                    let error = &i.parameters["error"];

                    match Regex::new(&regex.value_string()) {
                        Ok(regex) => {
                            if !regex.is_match(&input.value_string()) {
                                return Response::Error {
                                    kind: ErrorKind::EngineProcessingError,
                                    reason: error.value_string(),
                                };
                            }
                        }
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: format!("Invalid regex in action: {e}"),
                            }
                        }
                    }

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else {
                    return Response::Error {
                        kind: ErrorKind::InvalidInstruction,
                        reason: format!(
                            "The requested instruction {} could not be handled by this engine.",
                            i.instruction
                        ),
                    };
                }
            }

            Response::ExecutionOutput { output, evidence }
        }
    }
}
