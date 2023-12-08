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
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Random", env!("CARGO_PKG_VERSION"))
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
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "rand-decimal",
            "Random Decimal",
            "Generates a random decimal between zero and the maximum you specify.",
        )
        .with_parameter("max", "Maximum", ParameterKind::Decimal)
        .with_output("result", "Result", ParameterKind::Decimal),
        |_state, params, output, _evidence| {
            let max = params["max"].value_f32();

            output.insert(
                "result".to_string(),
                ParameterValue::Decimal(rand::thread_rng().gen_range(0.0..max)),
            );
            Ok(())
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

            let expr = rand_regex::Regex::compile(&regex, 32).map_err(EngineError::CouldntBuildExpression)?;
            output.insert(
                "result".to_string(),
                ParameterValue::String(rand::thread_rng().sample(&expr)),
            );

            Ok(())
        })
    );
}

expose_engine!(ENGINE);
