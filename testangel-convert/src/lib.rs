use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Convert", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "convert-int-string",
            "Integer to String",
            "Convert an integer into a string.",
        )
        .with_parameter("val1", "Integer input", ParameterKind::Integer)
        .with_output("result", "String output", ParameterKind::String),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();

            // Produce output and evidence
            let result = val1.to_string();
            output.insert("result".to_owned(), ParameterValue::String(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "convert-decimal-string",
            "Decimal to String",
            "Convert a decimal into a string.",
        )
        .with_parameter("val1", "Decimal input", ParameterKind::Decimal)
        .with_output("result", "String output", ParameterKind::String),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_f32();

            // Produce output and evidence
            let result = val1.to_string();
            output.insert("result".to_owned(), ParameterValue::String(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "convert-concat-strings",
            "Concatenate Strings",
            "Concatenate two strings into one.",
        )
        .with_parameter("val1", "StringA", ParameterKind::String)
        .with_parameter("val2", "StringB", ParameterKind::String)
        .with_output("result", "StringAStringB", ParameterKind::String),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_string();
            let val2 = params["val2"].value_string();

            // Produce output and evidence
            let result = format!("{val1}{val2}");
            output.insert("result".to_owned(), ParameterValue::String(result));
            Ok(())
        })
    );
}

expose_engine!(ENGINE);
