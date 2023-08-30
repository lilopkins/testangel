use std::{
    collections::HashMap,
    ffi::{c_char, CStr, CString},
};

use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

lazy_static! {
    static ref INSTRUCTION_NOW_FORMATTED: Instruction = Instruction::new(
        "date-now-formatted",
        "Format the date and time now.",
        "Format the current date and time. To see what options are available for formatting, see https://hpkns.uk/dateformatting.",
    )
    .with_parameter("format", "Format", ParameterKind::String)
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
                    INSTRUCTION_NOW_FORMATTED.clone(),
                ],
            }
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_NOW_FORMATTED.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_NOW_FORMATTED.validate(&i) {
                        return Response::Error { kind, reason };
                    }

                    // Get parameters
                    let format_string = i.parameters["format"].value_string();
                    let mut o = HashMap::new();

                    o.insert(
                        "result".to_string(),
                        ParameterValue::String(chrono::Local::now().format(&format_string).to_string()),
                    );

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
