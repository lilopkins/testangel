use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Date and Time", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "date-now-formatted",
            "Format the date and time now.",
            "Format the current date and time. To see what options are available for formatting, see https://hpkns.uk/dateformatting.",
        )
        .with_parameter("format", "Format", ParameterKind::String)
        .with_output("result", "Result", ParameterKind::String),
        |_state, params, output, _evidence| {
            let format_string = params["format"].value_string();

            output.insert(
                "result".to_string(),
                ParameterValue::String(
                    chrono::Local::now().format(&format_string).to_string(),
                ),
            );
            None
        })
    );
}

expose_engine!(ENGINE);
