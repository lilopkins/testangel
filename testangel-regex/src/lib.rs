use std::sync::Mutex;

use lazy_static::lazy_static;
use regex::Regex;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Regular Expressions", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "regex-validate",
            "Validate with Regular Expression",
            "Checks that input text matches a regular expression. This will cause the test flow to error if the text doesn't match.",
        )
        .with_parameter("regex", "Regular Expression", ParameterKind::String)
        .with_parameter("input", "Input", ParameterKind::String)
        .with_parameter("error", "Error Message", ParameterKind::String),
        |_state, params, _output, _evidence| {
            let regex = params["regex"].value_string();
            let input = params["input"].value_string();
            let error = params["error"].value_string();

            match Regex::new(&regex) {
                Ok(regex) => {
                    if !regex.is_match(&input) {
                        return Some(Response::Error {
                            kind: ErrorKind::EngineProcessingError,
                            reason: error,
                        })
                    }
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: format!("Invalid regex in action: {e}"),
                    })
                }
            }
            None
        })
    .with_instruction(
        Instruction::new(
            "regex-match",
            "Match with Regular Expression",
            "Returns a boolean if the input text matches a regular expression.",
        )
        .with_parameter("regex", "Regular Expression", ParameterKind::String)
        .with_parameter("input", "Input", ParameterKind::String)
        .with_output("match", "Input matches?", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let regex = params["regex"].value_string();
            let input = params["input"].value_string();

            match Regex::new(&regex) {
                Ok(regex) => {
                    output.insert(
                        "match".to_string(),
                        ParameterValue::Boolean(regex.is_match(&input)),
                    );
                }
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: format!("Invalid regex in action: {e}"),
                    })
                }
            }
            None
        })
    );
}

expose_engine!(ENGINE);
