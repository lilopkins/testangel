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

    let add_instruction = Instruction::new("arithmetic-int-add", "Add (Integer)", "Add together two integers.")
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A + B", ParameterKind::Integer);
    let sub_instruction = Instruction::new("arithmetic-int-sub", "Subtract (Integer)", "Subtract two integers.")
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A - B", ParameterKind::Integer);
    let mul_instruction = Instruction::new("arithmetic-int-mul", "Multiply (Integer)", "Multiply two integers.")
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A × B", ParameterKind::Integer);
    let div_instruction = Instruction::new("arithmetic-int-div", "Divide (Integer)", "Divide two integers, returning the floored result.")
        .with_parameter("val1", "A", ParameterKind::Integer)
        .with_parameter("val2", "B", ParameterKind::Integer)
        .with_output("result", "A ÷ B", ParameterKind::Integer);

    let request = request.unwrap();
    match request {
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            println!(
                "{}",
                Response::Instructions {
                    friendly_name: "Arithmetic".to_owned(),
                    instructions: vec![add_instruction.clone(), sub_instruction.clone(), mul_instruction.clone(), div_instruction.clone()],
                }
                .to_json()
            );
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *add_instruction.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() + val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} + {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(result),
                    );
                    output.push(map);
                } else if i.instruction == *sub_instruction.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() - val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} - {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(result),
                    );
                    output.push(map);
                } else if i.instruction == *mul_instruction.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() * val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} × {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(result),
                    );
                    output.push(map);
                } else if i.instruction == *div_instruction.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = add_instruction.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Get parameters
                    let val1 = &i.parameters["val1"];
                    let val2 = &i.parameters["val2"];

                    // Produce output and evidence
                    let result = val1.value_i32() / val2.value_i32();
                    evidence.push(vec![Evidence {
                        label: "Arithmetic Operation".to_owned(),
                        content: EvidenceContent::Textual(format!("{val1} ÷ {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert(
                        "result".to_owned(),
                        ParameterValue::Integer(result),
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
            println!("{}", Response::ExecutionOutput { output, evidence }.to_json());
        }
    }
}
