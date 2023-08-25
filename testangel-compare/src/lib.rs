use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_INT_EQ: Instruction = Instruction::new(
        "compare-eq-ints",
        "Equal (Integer)",
        "Compare the value of two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A = B", ParameterKind::Boolean);
    static ref INSTRUCTION_STR_EQ: Instruction = Instruction::new(
        "compare-eq-str",
        "Equal (String)",
        "Compare the value of two strings.",
    )
    .with_parameter("val1", "A", ParameterKind::String)
    .with_parameter("val2", "B", ParameterKind::String)
    .with_output("result", "A = B", ParameterKind::Boolean);
    static ref INSTRUCTION_NOT: Instruction = Instruction::new(
        "compare-not",
        "Not (Boolean)",
        "If fed true, returns false, if fed false, returns true.",
    )
    .with_parameter("val1", "A", ParameterKind::Boolean)
    .with_output("result", "not A", ParameterKind::Boolean);
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
                friendly_name: "Compare".to_owned(),
                instructions: vec![
                    INSTRUCTION_INT_EQ.clone(),
                    INSTRUCTION_STR_EQ.clone(),
                    INSTRUCTION_NOT.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_INT_EQ.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_INT_EQ.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() == val2.value_i32();
                    evidence.push(vec![]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Boolean(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_STR_EQ.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_STR_EQ.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_string() == val2.value_string();
                    evidence.push(vec![]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Boolean(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_NOT.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_NOT.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];

                    // Produce output and evidence
                    let result = !val1.value_bool();
                    evidence.push(vec![]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Boolean(result));
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
