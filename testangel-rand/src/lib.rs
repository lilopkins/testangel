use std::sync::Mutex;

use lazy_static::lazy_static;
use rand::Rng;
use testangel_engine::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Couldn't build expression.")]
    CouldntBuildExpression(#[from] rand_regex::Error),
}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(
        Engine::new("Random", "Random", env!("CARGO_PKG_VERSION")).with_instruction(
            Instruction::new(
                "rand-string",
                "StringByRegex",
                "Random String by Regex",
                "Generate a random string given the regular expression-like format you provide.",
            )
            .with_parameter("regex", "Regular Expression", ParameterKind::String)
            .with_output("result", "Result", ParameterKind::String),
            |_state, params, output, _evidence| {
                let regex = params["regex"].value_string();

                let expr = rand_regex::Regex::compile(&regex, 32)
                    .map_err(EngineError::CouldntBuildExpression)?;
                output.insert(
                    "result".to_string(),
                    ParameterValue::String(rand::rng().sample(&expr)),
                );

                Ok(())
            }
        )
    );
}

expose_engine!(ENGINE);
