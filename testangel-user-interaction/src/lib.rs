use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_WAIT_FOR_USER: Instruction = Instruction::new(
        "user-interaction-wait",
        "Wait for OK",
        "Display a message dialog and don't continue running the test flow until the user presses 'OK'.",
    )
    .with_parameter("message", "Message", ParameterKind::String);
    static ref INSTRUCTION_ASK: Instruction = Instruction::new(
        "user-interaction-ask",
        "Yes/No Question",
        "Returns a boolean if the input text matches a regular expression.",
    )
    .with_parameter("message", "Message", ParameterKind::String)
    .with_output("response", "Response", ParameterKind::Boolean);
    static ref INSTRUCTION_ASK_CONTINUE: Instruction = Instruction::new(
        "user-interaction-ask-continue",
        "Ask to Continue Flow",
        "Ask the user if they want to continue the automation flow.",
    )
    .with_parameter("message", "Message", ParameterKind::String);
    static ref INSTRUCTION_TERMINATE_FLOW: Instruction = Instruction::new(
        "user-interaction-terminate-flow",
        "Terminate Flow",
        "Let the user know that the flow has been stopped for a reason.",
    )
    .with_parameter("message", "Message", ParameterKind::String);
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
                friendly_name: "User Interaction".to_owned(),
                instructions: vec![
                    INSTRUCTION_WAIT_FOR_USER.clone(),
                    INSTRUCTION_ASK.clone(),
                    INSTRUCTION_ASK_CONTINUE.clone(),
                    INSTRUCTION_TERMINATE_FLOW.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_WAIT_FOR_USER.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_WAIT_FOR_USER.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let message = i.parameters["message"].value_string();

                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("Information")
                        .set_description(&message)
                        .show();

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_ASK.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ASK.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let message = i.parameters["message"].value_string();

                    let mut o = HashMap::new();
                    o.insert(
                        "response".to_string(),
                        ParameterValue::Boolean(
                            rfd::MessageDialog::new()
                                .set_level(rfd::MessageLevel::Info)
                                .set_buttons(rfd::MessageButtons::YesNo)
                                .set_title("Question")
                                .set_description(&message)
                                .show(),
                        ),
                    );

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(o);
                } else if i.instruction == *INSTRUCTION_ASK_CONTINUE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ASK_CONTINUE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let message = i.parameters["message"].value_string();

                    if !rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::YesNo)
                        .set_title("Continue flow?")
                        .set_description(&message)
                        .show()
                    {
                        return Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: "The user terminated the flow.".to_string(),
                        };
                    }

                    // Produce output and evidence
                    evidence.push(vec![]);
                    output.push(HashMap::new());
                } else if i.instruction == *INSTRUCTION_TERMINATE_FLOW.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_TERMINATE_FLOW.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let message = i.parameters["message"].value_string();

                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Info)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("Flow Terminating")
                        .set_description(&message)
                        .show();

                    return Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: "The flow was terminated by a step.".to_string(),
                    };
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
