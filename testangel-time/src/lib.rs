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
        #[instruction(id = "time-wait", name = "Wait", lua_name = "Wait")]
        /// Wait for a specified number of milliseconds.
        fn time_wait(
            #[arg(name = "Duration (ms)")] duration: i32,
        ) {
            if duration < 0 {
                return Err(Box::new(EngineError::CantWaitNegative));
            }
            sleep(Duration::from_millis(duration as u64));
        }
    }
}
