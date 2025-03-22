use std::{thread::sleep, time::Duration};

use testangel_engine::engine;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Duration cannot be negative.")]
    CantWaitNegative,
}

engine! {
    /// Work with time.
    #[engine(
        version = env!("CARGO_PKG_VERSION"),
    )]
    struct Time;

    impl Time {
        #[instruction(
            id = "time-wait",
            name = "Wait",
            lua_name = "Wait",
            flags = InstructionFlags::AUTOMATIC,
        )]
        /// Wait for a specified number of milliseconds.
        fn time_wait(
            #[arg(name = "Duration (ms)")] duration: i32,
        ) {
            if duration < 0 {
                return Err(Box::new(EngineError::CantWaitNegative));
            }

            if !dry_run {
                sleep(Duration::from_millis(duration as u64));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use testangel_engine::iwp;

    use super::*;

    #[test]
    fn test_time_wait() {
        let mut engine = TIME_ENGINE.lock().unwrap();
        let (_output, _evidence) = engine.run_instruction(iwp!("time-wait", false, "duration" => 300))
            .expect("Failed to trigger instruction");
    }
}
