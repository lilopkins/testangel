use std::{collections::HashMap, sync::Mutex};

use interpolator::{format, Formattable};
use lazy_static::lazy_static;
use testangel_engine::*;

#[derive(Default)]
struct State {
    interpolation_values: HashMap<String, String>,
}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, State>> = Mutex::new(Engine::new("Convert", env!("CARGO_PKG_VERSION"))
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
    .with_instruction(
        Instruction::new(
            "convert-add-interpolate-value",
            "Interpolate Strings: Set Value",
            "Set a value to use during future string interpolation.",
        )
        .with_parameter("key", "Key", ParameterKind::String)
        .with_parameter("value", "Value", ParameterKind::String),
        |state: &mut State, params, _output, _evidence| {
            let key = params["key"].value_string();
            let value = params["value"].value_string();

            state.interpolation_values.insert(key, value);
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "convert-interpolate-strings",
            "Interpolate Strings",
            "Interpolate values into a string. Uses values set previously by 'Interpolate Strings: Set Value'. For formatting guidance, see https://hpkns.uk/tafmt.",
        )
        .with_parameter("template", "Template String", ParameterKind::String)
        .with_output("result", "Interpolated String", ParameterKind::String),
        |state, params, output, _evidence| {
            let template = params["template"].value_string();

            let mut values: HashMap<&str, Formattable<'_>> = HashMap::new();
            for (k, v) in &state.interpolation_values {
                values.insert(k, Formattable::display(v));
            }

            // Produce output and evidence
            let result = format(&template, &values)?;
            output.insert("result".to_owned(), ParameterValue::String(result));
            Ok(())
        })
    );
}

expose_engine!(ENGINE);
