use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Compare", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "compare-eq-ints",
            "Equal (Integer)",
            "Compare the value of two integers.",
        )
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A = B", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();
            let val2 = params["val2"].value_i32();

            // Produce output and evidence
            let result = val1 == val2;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "compare-eq-str",
            "Equal (String)",
            "Compare the value of two strings.",
        )
        .with_parameter("val1", "A", ParameterKind::String)
        .with_parameter("val2", "B", ParameterKind::String)
        .with_output("result", "A = B", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_string();
            let val2 = params["val2"].value_string();

            // Produce output and evidence
            let result = val1 == val2;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "compare-eq-bool",
            "Equal (Boolean)",
            "Compare the value of two Booleans.",
        )
        .with_parameter("val1", "A", ParameterKind::Boolean)
        .with_parameter("val2", "B", ParameterKind::Boolean)
        .with_output("result", "A = B", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_bool();
            let val2 = params["val2"].value_bool();

            // Produce output and evidence
            let result = val1 == val2;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "compare-not",
            "Not (Boolean)",
            "If fed true, returns false, if fed false, returns true.",
        )
        .with_parameter("val1", "A", ParameterKind::Boolean)
        .with_output("result", "not A", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_bool();

            // Produce output and evidence
            let result = !val1;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "compare-and",
            "And (Boolean)",
            "Returns true if both A and B are true.",
        )
        .with_parameter("val1", "A", ParameterKind::Boolean)
        .with_parameter("val2", "B", ParameterKind::Boolean)
        .with_output("result", "A and B", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_bool();
            let val2 = params["val2"].value_bool();

            // Produce output and evidence
            let result = val1 && val2;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "compare-or",
            "Or (Boolean)",
            "Returns true if A or B is true.",
        )
        .with_parameter("val1", "A", ParameterKind::Boolean)
        .with_parameter("val2", "B", ParameterKind::Boolean)
        .with_output("result", "not A", ParameterKind::Boolean),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_bool();
            let val2 = params["val2"].value_bool();

            // Produce output and evidence
            let result = val1 || val2;
            output.insert("result".to_owned(), ParameterValue::Boolean(result));
            Ok(())
        })
    );
}

expose_engine!(ENGINE);
