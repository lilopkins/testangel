use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("User Interaction", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "user-interaction-wait",
            "Wait for OK",
            "Display a message dialog and don't continue running the test flow until the user presses 'OK'.",
        )
        .with_parameter("message", "Message", ParameterKind::String),
        |_state, params, _output, _evidence| {
            let message = params["message"].value_string();

            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Information")
                .set_description(&message)
                .show();
            None
        })
    .with_instruction(
        Instruction::new(
            "user-interaction-ask",
            "Yes/No Question",
            "Returns a boolean if the input text matches a regular expression.",
        )
        .with_parameter("message", "Message", ParameterKind::String)
        .with_output("response", "Response", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let message = params["message"].value_string();

            output.insert(
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
            None
        })
    .with_instruction(
        Instruction::new(
            "user-interaction-ask-continue",
            "Ask to Continue Flow",
            "Ask the user if they want to continue the automation flow.",
        )
        .with_parameter("message", "Message", ParameterKind::String),
        |_state, params, _output, _evidence| {
            let message = params["message"].value_string();

            if !rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::YesNo)
                .set_title("Continue flow?")
                .set_description(&message)
                .show()
            {
                return Some(Response::Error {
                    kind: ErrorKind::EngineProcessingError,
                    reason: "The user terminated the flow.".to_string(),
                })
            }
            None
        })
    .with_instruction(
        Instruction::new(
            "user-interaction-terminate-flow",
            "Terminate Flow",
            "Let the user know that the flow has been stopped for a reason.",
        )
        .with_parameter("message", "Message", ParameterKind::String),
        |_state, params, _output, _evidence| {
            let message = params["message"].value_string();

            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_buttons(rfd::MessageButtons::Ok)
                .set_title("Flow Terminating")
                .set_description(&message)
                .show();

            Some(Response::Error {
                kind: ErrorKind::EngineProcessingError,
                reason: "The flow was terminated by a step.".to_string(),
            })
        })
    );
}

expose_engine!(ENGINE);
