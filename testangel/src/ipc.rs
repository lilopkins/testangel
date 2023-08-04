use std::{collections::HashMap, ffi::OsStr, fs, path::PathBuf, process::Command, env};

use testangel_ipc::prelude::*;

pub fn ipc_call<P>(engine: P, request: Request) -> Result<Response, ()>
where
    P: AsRef<OsStr>,
{
    log::debug!(
        "Sending request {:?} to engine {:?}",
        request,
        engine.as_ref()
    );
    if let Ok(output) = Command::new(&engine).arg(request.to_json()).output() {
        let output_string = String::from_utf8(output.stdout).unwrap();
        let res = Response::try_from(output_string);
        if res.is_err() {
            log::error!(
                "Failed to parse subprocess response ({}) from engine {:?}.",
                res.unwrap_err(),
                engine.as_ref(),
            );
            return Err(());
        }
        let res = res.unwrap();
        log::debug!("Got response {res:?}");
        return Ok(res);
    }
    log::error!("Failed to run subprocess for engine {:?}.", engine.as_ref());
    return Err(());
}

#[derive(Clone, Debug)]
pub struct Engine {
    pub name: String,
    pub instructions: Vec<Instruction>,
}

#[derive(Default)]
pub struct EngineMap(HashMap<PathBuf, Engine>);

impl EngineMap {
    /// Get an instruction from an instruction ID by iterating through available engines.
    pub fn get_instruction_by_id(&self, instruction_id: String) -> Option<Instruction> {
        for (_path, engine) in &self.0 {
            for inst in &engine.instructions {
                if *inst.id() == instruction_id {
                    return Some(inst.clone());
                }
            }
        }
        return None;
    }

    pub fn inner(&self) -> &HashMap<PathBuf, Engine> {
        &self.0
    }
}

/// Get the list of available engines.
pub fn get_engines() -> EngineMap {
    let mut engines = HashMap::new();
    let engine_dir = env::var("ENGINE_DIR").unwrap_or("./engines".to_owned());
    for path in fs::read_dir(engine_dir).unwrap() {
        let path = path.unwrap();
        let basename = path.file_name();
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                continue;
            }
        }

        if let Ok(str) = basename.into_string() {
            if str.starts_with("testangel-") {
                log::debug!("Detected possible engine {str}");
                if let Ok(res) = ipc_call(path.path(), Request::Instructions) {
                    match res {
                        Response::Instructions {
                            friendly_name,
                            instructions,
                        } => {
                            log::info!("Discovered engine {friendly_name} at {:?}", path.path());
                            engines.insert(
                                path.path(),
                                Engine {
                                    name: friendly_name,
                                    instructions,
                                },
                            );
                        }
                        _ => log::error!("Invalid response from engine {str}"),
                    }
                }
            }
        }
    }
    EngineMap(engines)
}
