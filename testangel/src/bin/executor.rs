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
    pretty_env_logger::init();

    let cli = Cli::parse();

    let flow: AutomationFlow =
        ron::from_str(&fs::read_to_string(cli.flow).expect("Failed to read flow."))
            .expect("Failed to parse flow.");
    let engine_map = Arc::new(ipc::get_engines());
    let action_map = Arc::new(action_loader::get_actions(engine_map.clone()));

    // Check flow for actions that aren't available.
    for action_config in &flow.actions {
        if action_map
            .get_action_by_id(&action_config.action_id)
            .is_none()
        {
            eprintln!("This flow cannot be executed because an action isn't available or wasn't loaded. Maybe an engine is missing?");
            std::process::exit(1);
        }
    }

    let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
    let mut evidence = Vec::new();

    for engine in engine_map.inner() {
        if engine.reset_state().is_err() {
            evidence.push(Evidence {
                label: String::from("WARNING: State Warning"),
                content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
            });
        }
    }

    for action_config in flow.actions {
        match action_config.execute(action_map.clone(), engine_map.clone(), outputs.clone()) {
            Ok((output, ev)) => {
                outputs.push(output);
                evidence = vec![evidence, ev].concat();
            }
            Err(e) => {
                panic!("Failed to execute: {e}");
            }
        }
    }

    report_generation::save_report(cli.report, evidence);
}
