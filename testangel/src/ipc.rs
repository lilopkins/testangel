use std::{fs, path::PathBuf, process::Command, ffi::OsStr};

use testangel_ipc::prelude::*;

pub fn ipc_call<P>(engine: P, request: Request) -> Result<Response, ()>
where P: AsRef<OsStr>
{
    log::debug!("Sending request {:?} to engine {:?}", request, engine.as_ref());
    if let Ok(output) = Command::new(engine)
        .arg(request.to_json())
        .output()
    {
        let output_string = String::from_utf8(output.stdout).unwrap();
        let res = Response::try_from(output_string);
        if res.is_err() {
            log::error!("Failed to parse subprocess response.");
            return Err(());
        }
        let res = res.unwrap();
        log::debug!("Got response {res:?}");
        return Ok(res);
    }
    log::error!("Failed to run subprocess.");
    return Err(());
}

/// Get the list of available engines.
pub fn get_engines() -> Vec<(PathBuf, Vec<Instruction>)> {
    let mut engines = Vec::new();
    for path in fs::read_dir("./").unwrap() {
        let path = path.unwrap();
        let basename = path.file_name();
        if let Ok(str) = basename.into_string() {
            if str.starts_with("testangel-") {
                log::debug!("Detected engine {str}");
                if let Ok(res) = ipc_call(path.path(), Request::Instructions) {
                    match res {
                        Response::Instructions { instructions } => engines.push((path.path(), instructions)),
                        _ => log::error!("Invalid response from engine {str}"),
                    }
                }
            }
        }
    }
    engines
}
