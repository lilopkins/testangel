use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
    sync::Mutex,
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

#[derive(Default)]
struct State {
    counter: i32,
}

lazy_static! {
    static ref INSTRUCTION_ADD: Instruction = Instruction::new(
        "arithmetic-int-add",
        "Add (Integer)",
        "Add together two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A + B", ParameterKind::Integer);
    static ref INSTRUCTION_SUB: Instruction = Instruction::new(
        "arithmetic-int-sub",
        "Subtract (Integer)",
        "Subtract two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A - B", ParameterKind::Integer);
    static ref INSTRUCTION_MUL: Instruction = Instruction::new(
        "arithmetic-int-mul",
        "Multiply (Integer)",
        "Multiply two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A × B", ParameterKind::Integer);
    static ref INSTRUCTION_DIV: Instruction = Instruction::new(
        "arithmetic-int-div",
        "Divide (Integer)",
        "Divide two integers, returning the floored result.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A ÷ B", ParameterKind::Integer);
    static ref INSTRUCTION_COUNTER_INC: Instruction = Instruction::new(
        "arithmetic-counter-inc",
        "Increase Counter",
        "Increase a counter.",
    )
    .with_output("value", "Counter Value", ParameterKind::Integer);
    static ref INSTRUCTION_COUNTER_DEC: Instruction = Instruction::new(
        "arithmetic-counter-dec",
        "Decrease Counter",
        "Decrease a counter.",
    )
    .with_output("value", "Counter Value", ParameterKind::Integer);
    static ref STATE: Mutex<State> = Mutex::new(State::default());
}

#[no_mangle]
pub unsafe extern "C" fn ta_call(input: *const c_char, result: *mut *const c_char) -> isize {
    if result.is_null() || !(*result).is_null() {
        return 1; // result isn't null
    }

    let input = CStr::from_ptr(input);
    let response = call_internal(String::from_utf8_lossy(input.to_bytes()).to_string());
    let c_response = CString::new(response).expect("valid response");
    *result = c_response.as_ptr();

    0
}

fn call_internal(request: String) -> String {
    match Request::try_from(request) {
        Err(e) => Response::Error {
            kind: ErrorKind::FailedToParseIPCJson,
            reason: format!("The IPC message was invalid. ({:?})", e),
        }
        .to_json(),
        Ok(request) => process_request(STATE.lock().as_deref_mut().unwrap(), request).to_json(),
    }
}

fn process_request(state: &mut State, request: Request) -> Response {
    match request {
        Request::ResetState => {
            // Reset the state.
            *state = State::default();
            Response::StateReset
        }
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            Response::Instructions {
                friendly_name: "Arithmetic".to_owned(),
                instructions: vec![
                    INSTRUCTION_ADD.clone(),
                    INSTRUCTION_SUB.clone(),
                    INSTRUCTION_MUL.clone(),
                    INSTRUCTION_DIV.clone(),
                    INSTRUCTION_COUNTER_INC.clone(),
                    INSTRUCTION_COUNTER_DEC.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_ADD.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() + val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} + {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_SUB.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() - val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} - {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_MUL.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() * val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} x {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_DIV.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() / val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} / {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_COUNTER_INC.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_COUNTER_INC.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Produce output and evidence
                    state.counter += 1;

                    evidence.push(vec![Evidence {
                        label: "Counter Increased".to_owned(),
                        content: EvidenceContent::Textual(format!("New value: {}", state.counter)),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("value".to_owned(), ParameterValue::Integer(state.counter));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_COUNTER_DEC.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_COUNTER_DEC.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Produce output and evidence
                    state.counter -= 1;

                    evidence.push(vec![Evidence {
                        label: "Counter decreased".to_owned(),
                        content: EvidenceContent::Textual(format!("New value: {}", state.counter)),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("value".to_owned(), ParameterValue::Integer(state.counter));
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
