use std::collections::HashMap;

use clap::Parser;
use testangel_ipc::prelude::*;

#[derive(Parser)]
#[command(author, about, version)]
struct Cli {
    #[arg(index = 1)]
    ipc_request: String,
}

fn main() {
    let cli = Cli::parse();

    // Parse the request
    let request = Request::try_from(cli.ipc_request);
    if let Err(e) = request {
        // Return a well-formatted error if the request couldn't be parsed.
        println!(
            "{}",
            Response::Error {
                kind: ErrorKind::FailedToParseIPCJson,
                reason: format!("The IPC message was invalid. ({:?})", e)
            }
            .to_json()
        );
        return;
    }

    let add_instruction = Instruction::new("dummy-add", "Add", "Add together two input integers")
        .with_parameter("val1", "Value 1", ParameterKind::Integer)
        .with_parameter("val2", "Value 2", ParameterKind::Integer)
        .with_output("result", "Result", ParameterKind::Integer);
    let sub_instruction = Instruction::new("dummy-sub", "Subtract", "Subtract two input integers")
        .with_parameter("val1", "Value 1", ParameterKind::Integer)
        .with_parameter("val2", "Value 2", ParameterKind::Integer)
        .with_output("result", "Result", ParameterKind::Integer);

    let request = request.unwrap();
    match request {
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            println!(
                "{}",
                Response::Instructions {
                    instructions: vec![add_instruction.clone(), sub_instruction.clone(),],
                }
                .to_json()
            );
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            for i in instructions {
                if i.instruction == "dummy-add" {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(val1.value_i32() + val2.value_i32()),
                    );
                    output.push(map);
                } else if i.instruction == "dummy-sub" {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(val1.value_i32() - val2.value_i32()),
                    );
                    output.push(map);
                } else {
                    println!(
                        "{}",
                        Response::Error {
                            kind: ErrorKind::InvalidInstruction,
                            reason: format!(
                                "The requested instruction {} could not be handled by this engine.",
                                i.instruction
                            ),
                        }
                        .to_json()
                    )
                }
            }
            // Print output
            println!("{}", Response::ExecutionOutput { output }.to_json());
        }
    }
}
