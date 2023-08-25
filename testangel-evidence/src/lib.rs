use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_ADD_EVIDENCE: Instruction = Instruction::new(
        "evidence-add-text",
        "Add Text-based Evidence",
        "Add text based evidence to the report.",
    )
    .with_parameter("label", "Label", ParameterKind::String)
    .with_parameter("content", "Content", ParameterKind::String);
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
                friendly_name: "Evidence".to_owned(),
                instructions: vec![INSTRUCTION_ADD_EVIDENCE.clone()],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_ADD_EVIDENCE.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD_EVIDENCE.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let label = &i.parameters["label"];
                    let content = &i.parameters["content"];

                    // Produce output and evidence
                    evidence.push(vec![Evidence {
                        label: label.value_string(),
                        content: EvidenceContent::Textual(content.value_string()),
                    }]);
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
