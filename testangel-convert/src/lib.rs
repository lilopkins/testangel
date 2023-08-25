use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_INT_STR: Instruction = Instruction::new(
        "convert-int-string",
        "Integer to String",
        "Convert an integer into a string.",
    )
    .with_parameter("val1", "Integer input", ParameterKind::Integer)
    .with_output("result", "String output", ParameterKind::String);
    static ref INSTRUCTION_DEC_STR: Instruction = Instruction::new(
        "convert-decimal-string",
        "Decimal to String",
        "Convert a decimal into a string.",
    )
    .with_parameter("val1", "Decimal input", ParameterKind::Decimal)
    .with_output("result", "String output", ParameterKind::String);
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
            // nothing to do
            Response::StateReset
        }
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            Response::Instructions {
                friendly_name: "Convert".to_owned(),
                instructions: vec![INSTRUCTION_INT_STR.clone(), INSTRUCTION_DEC_STR.clone()],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_INT_STR.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_INT_STR.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];

                    // Produce output and evidence
                    let result = val1.value_i32().to_string();
                    evidence.push(vec![]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::String(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_DEC_STR.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_DEC_STR.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];

                    // Produce output and evidence
                    let result = val1.value_f32().to_string();
                    evidence.push(vec![]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::String(result));
                    output.push(map);
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
