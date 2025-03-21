use std::{env, process::Command};

fn main() {
    let bindings = cbindgen::generate(env::var("CARGO_MANIFEST_DIR").unwrap())
        .expect("failed to generate C bindings");
    bindings.write_to_file("testangel.h");

    if cfg!(target_os = "linux") {
        let _ = Command::new("gcc")
            .args([
                "-shared",
                "-o",
                "libtestangel-demo-c-engine.so",
                "-fPIC",
                "-Wall",
                "-g",
                "demo_c_engine.c",
            ])
            .output();
    }
}
