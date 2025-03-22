use rand::Rng;
use testangel_engine::engine;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EngineError {
    #[error("Couldn't build expression.")]
    CouldntBuildExpression(#[from] rand_regex::Error),
}

engine! {
    /// Generate data with randomness.
    #[engine(
        version = env!("CARGO_PKG_VERSION"),
    )]
    struct Random;

    impl Random {
        #[instruction(id = "rand-string", name = "Random String by Regex")]
        /// Generate a random string given the regular expression-like format you provide.
        fn string_by_regex(
            #[arg(name = "Regular Expression")] regex: String,
        ) -> #[output(id = "result", name = "Result")] String {
            let expr = rand_regex::Regex::compile(&regex, 32)
                .map_err(EngineError::CouldntBuildExpression)?;
            rand::rng().sample(&expr)
        }
    }
}

#[cfg(test)]
mod tests {
    use testangel_engine::iwp;

    use super::*;

    #[test]
    fn test_string_by_regex() {
        let mut engine = RANDOM_ENGINE.lock().unwrap();
        let (output, _evidence) = engine.run_instruction(iwp!("rand-string", "regex" => "[a-z]{5}"))
            .expect("Failed to trigger instruction");
        let result = output["result"].value_string();
        for c in result.chars() {
            assert!(c.is_ascii_lowercase());
        }
    }
}
