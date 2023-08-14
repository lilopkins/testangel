use std::{collections::HashMap, io};

use clap::Parser;
use lazy_static::lazy_static;
use testangel_ipc::prelude::*;

#[derive(Parser)]
#[command(author, about, version)]
struct Cli {
    /// Rather than running as a REPL loop, process a single request
    #[arg(short, long)]
    request: Option<String>,
}

#[derive(Default)]
struct State {
    counter: i32,
}

lazy_static! {
    static ref INSTRUCTION_ADD: Instruction = Instruction::new(
        "arithmetic-int-add",
        "Add (Integer)",
        "Add together two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A + B", ParameterKind::Integer);
    static ref INSTRUCTION_SUB: Instruction = Instruction::new(
        "arithmetic-int-sub",
        "Subtract (Integer)",
        "Subtract two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A - B", ParameterKind::Integer);
    static ref INSTRUCTION_MUL: Instruction = Instruction::new(
        "arithmetic-int-mul",
        "Multiply (Integer)",
        "Multiply two integers.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A × B", ParameterKind::Integer);
    static ref INSTRUCTION_DIV: Instruction = Instruction::new(
        "arithmetic-int-div",
        "Divide (Integer)",
        "Divide two integers, returning the floored result.",
    )
    .with_parameter("val1", "A", ParameterKind::Integer)
    .with_parameter("val2", "B", ParameterKind::Integer)
    .with_output("result", "A ÷ B", ParameterKind::Integer);
    static ref INSTRUCTION_COUNTER_INC: Instruction = Instruction::new(
        "arithmetic-counter-inc",
        "Increase Counter",
        "Increase a counter.",
    )
    .with_output("value", "Counter Value", ParameterKind::Integer);
    static ref INSTRUCTION_COUNTER_DEC: Instruction = Instruction::new(
        "arithmetic-counter-dec",
        "Decrease Counter",
        "Decrease a counter.",
    )
    .with_output("value", "Counter Value", ParameterKind::Integer);
}

fn main() {
    let cli = Cli::parse();

    // Parse the request
    if let Some(request) = cli.request {
        let request = Request::try_from(request);
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
        let request = request.unwrap();
        process_request(&mut State::default(), request);
    } else {
        // Start REPL loop
        repl_loop().expect("io failure");
    }
}

fn repl_loop() -> io::Result<()> {
    let mut state = State::default();
    loop {
        let mut buf = String::new();
        io::stdin().read_line(&mut buf)?;
        let buf = buf.trim();

        if buf == "\x04" {
            // EOF
            return Ok(());
        }

        if buf.len() == 0 {
            continue;
        }

        let request = Request::try_from(buf.to_owned());
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
            continue;
        }
        let request = request.unwrap();
        process_request(&mut state, request);
    }
}

fn process_request(state: &mut State, request: Request) {
    match request {
        Request::ResetState => {
            // Reset the state.
            *state = State::default();
            println!("{}", Response::StateReset.to_json());
        }
        Request::Instructions => {
            // Provide a list of instructions this engine can run.
            println!(
                "{}",
                Response::Instructions {
                    friendly_name: "Arithmetic".to_owned(),
                    instructions: vec![
                        INSTRUCTION_ADD.clone(),
                        INSTRUCTION_SUB.clone(),
                        INSTRUCTION_MUL.clone(),
                        INSTRUCTION_DIV.clone(),
                        INSTRUCTION_COUNTER_INC.clone(),
                        INSTRUCTION_COUNTER_DEC.clone(),
                    ],
                }
                .to_json()
            );
        }
        Request::RunInstructions { instructions } => {
            let mut output = Vec::new();
            let mut evidence = Vec::new();
            for i in instructions {
                if i.instruction == *INSTRUCTION_ADD.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
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
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_SUB.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
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
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_MUL.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
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
                        content: EvidenceContent::Textual(format!("{val1} x {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_DIV.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_ADD.validate(&i) {
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
                        content: EvidenceContent::Textual(format!("{val1} / {val2} = {result}")),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("result".to_owned(), ParameterValue::Integer(result));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_COUNTER_INC.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_COUNTER_INC.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Produce output and evidence
                    state.counter += 1;

                    evidence.push(vec![Evidence {
                        label: "Counter Increased".to_owned(),
                        content: EvidenceContent::Textual(format!("New value: {}", state.counter)),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("value".to_owned(), ParameterValue::Integer(state.counter));
                    output.push(map);
                } else if i.instruction == *INSTRUCTION_COUNTER_DEC.id() {
                    // Validate parameters
                    if let Err((kind, reason)) = INSTRUCTION_COUNTER_DEC.validate(&i) {
                        println!("{}", Response::Error { kind, reason }.to_json());
                        return;
                    }

                    // Produce output and evidence
                    state.counter -= 1;

                    evidence.push(vec![Evidence {
                        label: "Counter decreased".to_owned(),
                        content: EvidenceContent::Textual(format!("New value: {}", state.counter)),
                    }]);
                    let mut map = HashMap::new();
                    map.insert("value".to_owned(), ParameterValue::Integer(state.counter));
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
            println!(
                "{}",
                Response::ExecutionOutput { output, evidence }.to_json()
            );
        }
    }
}
