use std::{env, path::PathBuf};

fn main() {
    let bindings = cbindgen::generate(env::var("CARGO_MANIFEST_DIR").unwrap())
        .expect("failed to generate C bindings");
    bindings.write_to_file("testangel.h");

    println!("cargo::rerun-if-changed=demo_c_engine.c");
    let path = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let path = path.parent().unwrap().parent().unwrap().parent().unwrap();
    cc::Build::new()
        .pic(true)
        .shared_flag(true)
        .file("demo_c_engine.c")
        .out_dir(path)
        .compile("testangel-demo-c-engine");
}
