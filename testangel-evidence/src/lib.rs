use std::sync::Mutex;

use lazy_static::lazy_static;
use testangel_engine::*;

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(Engine::new("Evidence", env!("CARGO_PKG_VERSION"))
    .with_instruction(
        Instruction::new(
            "evidence-add-text",
            "Add Text-based Evidence",
            "Add text based evidence to the report.",
        )
        .with_parameter("label", "Label", ParameterKind::String)
        .with_parameter("content", "Content", ParameterKind::String),
        |_state, params, _output, evidence| {
            let label = params["label"].value_string();
            let content = params["content"].value_string();

            // Produce output and evidence
            evidence.push(Evidence {
                label,
                content: EvidenceContent::Textual(content),
            });
            Ok(())
        })
    );
}

expose_engine!(ENGINE);
