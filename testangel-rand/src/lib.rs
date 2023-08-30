use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::Rng;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Random")
    .with_instruction(
        Instruction::new(
            "rand-number",
            "Random Integer",
            "Generates a random integer between zero and the maximum you specify.",
        )
        .with_parameter("max", "Maximum", ParameterKind::Integer)
        .with_output("result", "Result", ParameterKind::Integer),
        |_state, params, output, _evidence| {
            let max = params["max"].value_i32();

            output.insert(
                "result".to_string(),
                ParameterValue::Integer(rand::thread_rng().gen_range(0..max)),
            );
            None
        })
    .with_instruction(
        Instruction::new(
            "rand-string",
            "Random String",
            "Generate a random string given the regular expression-like format you provide.",
        )
        .with_parameter("regex", "Regular Expression", ParameterKind::String)
        .with_output("result", "Result", ParameterKind::String),
        |_state, params, output, _evidence| {
            let regex = params["regex"].value_string();

            match rand_regex::Regex::compile(&regex, 32) {
                Err(e) => {
                    return Some(Response::Error {
                        kind: ErrorKind::EngineProcessingError,
                        reason: format!("Couldn't build expression: {e}"),
                    })
                }
                Ok(expr) => {
                    output.insert(
                        "result".to_string(),
                        ParameterValue::String(rand::thread_rng().sample(&expr)),
                    );
                }
            }
            None
        })
    );
}

expose_engine!(ENGINE);
