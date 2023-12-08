use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

#[derive(Default)]
struct State {
    counter: i32,
}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, State>> = Mutex::new(Engine::new("Arithmetic", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "arithmetic-int-add",
            "Add (Integer)",
            "Add together two integers.",
        )
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A + B", ParameterKind::Integer),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();
            let val2 = params["val2"].value_i32();

            // Produce output and evidence
            let result = val1 + val2;
            output.insert("result".to_owned(), ParameterValue::Integer(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-int-sub",
            "Subtract (Integer)",
            "Subtract two integers.",
        )
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A - B", ParameterKind::Integer),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();
            let val2 = params["val2"].value_i32();

            // Produce output and evidence
            let result = val1 - val2;
            output.insert("result".to_owned(), ParameterValue::Integer(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-int-mul",
            "Multiply (Integer)",
            "Multiply two integers.",
        )
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A × B", ParameterKind::Integer),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();
            let val2 = params["val2"].value_i32();

            // Produce output and evidence
            let result = val1 * val2;
            output.insert("result".to_owned(), ParameterValue::Integer(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-int-div",
            "Divide (Integer)",
            "Divide two integers, returning the floored result.",
        )
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A ÷ B", ParameterKind::Integer),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_i32();
            let val2 = params["val2"].value_i32();

            // Produce output and evidence
            let result = val1 / val2;
            output.insert("result".to_owned(), ParameterValue::Integer(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-dec-add",
            "Add (Decimal)",
            "Add together two decimals.",
        )
        .with_parameter("val1", "A", ParameterKind::Decimal)
        .with_parameter("val2", "B", ParameterKind::Decimal)
        .with_output("result", "A + B", ParameterKind::Decimal),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_f32();
            let val2 = params["val2"].value_f32();

            // Produce output and evidence
            let result = val1 + val2;
            output.insert("result".to_owned(), ParameterValue::Decimal(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-dec-sub",
            "Subtract (Decimal)",
            "Subtract two decimals.",
        )
        .with_parameter("val1", "A", ParameterKind::Decimal)
        .with_parameter("val2", "B", ParameterKind::Decimal)
        .with_output("result", "A - B", ParameterKind::Decimal),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_f32();
            let val2 = params["val2"].value_f32();

            // Produce output and evidence
            let result = val1 - val2;
            output.insert("result".to_owned(), ParameterValue::Decimal(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-dec-mul",
            "Multiply (Decimal)",
            "Multiply two decimals.",
        )
        .with_parameter("val1", "A", ParameterKind::Decimal)
        .with_parameter("val2", "B", ParameterKind::Decimal)
        .with_output("result", "A × B", ParameterKind::Decimal),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_f32();
            let val2 = params["val2"].value_f32();

            // Produce output and evidence
            let result = val1 * val2;
            output.insert("result".to_owned(), ParameterValue::Decimal(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-dec-div",
            "Divide (Decimal)",
            "Divide two decimals, returning the result.",
        )
        .with_parameter("val1", "A", ParameterKind::Decimal)
        .with_parameter("val2", "B", ParameterKind::Decimal)
        .with_output("result", "A ÷ B", ParameterKind::Decimal),
        |_state, params, output, _evidence| {
            let val1 = params["val1"].value_f32();
            let val2 = params["val2"].value_f32();

            // Produce output and evidence
            let result = val1 / val2;
            output.insert("result".to_owned(), ParameterValue::Decimal(result));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-counter-inc",
            "Increase Counter",
            "Increase a counter.",
        )
        .with_output("value", "Counter Value", ParameterKind::Integer),
        |state: &mut State, _params, output, _evidence| {
            // Produce output and evidence
            state.counter += 1;

            output.insert("value".to_owned(), ParameterValue::Integer(state.counter));
            Ok(())
        })
    .with_instruction(
        Instruction::new(
            "arithmetic-counter-dec",
            "Decrease Counter",
            "Decrease a counter.",
        )
        .with_output("value", "Counter Value", ParameterKind::Integer),
        |state: &mut State, _params, output, _evidence| {
            // Produce output and evidence
            state.counter -= 1;

            output.insert("value".to_owned(), ParameterValue::Integer(state.counter));
            Ok(())
        })
    );
}

expose_engine!(ENGINE);
