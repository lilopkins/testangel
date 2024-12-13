use std::{sync::Mutex, thread::sleep, time::Duration};

use lazy_static::lazy_static;
use testangel_engine::*;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Duration cannot be negative.")]
    CantWaitNegative,
}

lazy_static! {
    static ref ENGINE: Mutex<Engine<'static, ()>> = Mutex::new(
        Engine::new("Time", "Time", env!("CARGO_PKG_VERSION")).with_instruction(
            Instruction::new(
                "time-wait",
                "Wait",
                "Wait",
                "Wait for a specified number of milliseconds.",
            )
            .with_parameter("duration", "Duration (ms)", ParameterKind::Integer),
            |_state, params, _output, _evidence| {
                let duration = params["duration"].value_i32();
                if duration < 0 {
                    return Err(Box::new(EngineError::CantWaitNegative));
                }
                sleep(Duration::from_millis(duration as u64));
                Ok(())
            }
        )
    );
}

expose_engine!(ENGINE);
