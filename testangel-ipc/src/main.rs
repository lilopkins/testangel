#[cfg(feature = "schemas")]
fn main() {
    use schemars::schema_for;
    use testangel_ipc::{Request, Response};
    use std::fs;

    // Build request schema
    let request_schema = schema_for!(Request);
    fs::write("request.schema.json", serde_json::to_string_pretty(&request_schema).unwrap()).unwrap();

    // Build response schema
    let response_schema = schema_for!(Response);
    fs::write("response.schema.json", serde_json::to_string_pretty(&response_schema).unwrap()).unwrap();
}

#[cfg(not(feature = "schemas"))]
fn main() {
    println!("This must have been built with the `schemas` feature to be able to export JSON schemas.");
}
