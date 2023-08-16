use std::{collections::HashMap, fs, path::PathBuf, sync::Arc};

use clap::{arg, Parser};
use testangel::{types::AutomationFlow, *};
use testangel_ipc::prelude::*;

#[derive(Parser)]

struct Cli {
    /// The output for the report.
    #[arg(short, long, default_value = "report.pdf")]
    report: PathBuf,

    /// The flow file to execute.
    #[arg(index = 1)]
    flow: PathBuf,
}

fn main() {
    let cli = Cli::parse();

    let flow: AutomationFlow =
        ron::from_str(&fs::read_to_string(cli.flow).expect("Failed to read flow."))
            .expect("Failed to parse flow.");
    let action_map = Arc::new(action_loader::get_actions());
    let engine_map = Arc::new(ipc::get_engines());

    let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
    let mut evidence = Vec::new();

    for engine in engine_map.inner() {
        if let Err(_) = engine.reset_state() {
            evidence.push(Evidence {
                label: String::from("WARNING: State Warning"),
                content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
            });
        }
    }

    for action_config in flow.actions {
        let (output, ev) = action_config
            .execute(action_map.clone(), engine_map.clone(), outputs.clone())
            .expect("Failed to execute.");
        outputs.push(output);
        evidence = vec![evidence, ev].concat();
    }

    report_generation::save_report(cli.report, evidence);
}