use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use rand::Rng;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_RAND_NUMBER: Instruction = Instruction::new(
        "rand-number",
        "Random Integer",
        "Generates a random integer between zero and the maximum you specify.",
    )
    .with_parameter("max", "Maximum", ParameterKind::Integer)
    .with_output("result", "Result", ParameterKind::Integer);
    static ref INSTRUCTION_RAND_STRING: Instruction = Instruction::new(
        "rand-string",
        "Random String",
        "Generate a random string given the regular expression-like format you provide.",
    )
    .with_parameter("regex", "Regular Expression", ParameterKind::String)
    .with_output("result", "Result", ParameterKind::String);
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
                friendly_name: "Random".to_owned(),
                instructions: vec![
                    INSTRUCTION_RAND_NUMBER.clone(),
                    INSTRUCTION_RAND_STRING.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_RAND_NUMBER.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_RAND_NUMBER.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let max = i.parameters["max"].value_i32();
                    let mut o = HashMap::new();

                    o.insert(
                        "result".to_string(),
                        ParameterValue::Integer(rand::thread_rng().gen_range(0..max)),
                    );

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_RAND_STRING.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_RAND_STRING.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let regex = i.parameters["regex"].value_string();
                    let mut o = HashMap::new();

                    match rand_regex::Regex::compile(&regex, 32) {
                        Err(e) => {
                            return Response::Error {
                                kind: ErrorKind::EngineProcessingError,
                                reason: format!("Couldn't build expression: {e}"),
                            }
                        }
                        Ok(expr) => {
                            o.insert(
                                "result".to_string(),
                                ParameterValue::String(rand::thread_rng().sample(&expr)),
                            );
                        }
                    }

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(o);
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
