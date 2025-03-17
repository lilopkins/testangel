use std::{
    env,
    ffi::{CStr, CString},
    fmt, fs, io,
    path::{Path, PathBuf},
    sync::Arc,
};

use testangel_engine::EngineInterface;
use testangel_ipc::prelude::*;

#[derive(Debug)]
pub enum IpcError {
    IoError(io::Error),
    EngineNotStarted,
    EngineNotCompliant,
    CantLockEngineIo,
    InvalidResponseFromEngine,
}

pub fn ipc_call(engine: &Engine, request: &Request) -> Result<Response, IpcError> {
    tracing::debug!(
        "Sending request {:?} to engine {} at {:?}.",
        request,
        engine,
        engine.path
    );

    let request = request.to_json();
    let c_request = CString::new(request).unwrap();
    let response = unsafe {
        let lib = engine.lib.clone().ok_or(IpcError::EngineNotStarted)?;

        let res = lib
            .ta_call(c_request.as_ptr())
            .map_err(|_| IpcError::EngineNotCompliant)?;
        let res = CStr::from_ptr(res);
        let string = String::from_utf8_lossy(res.to_bytes()).to_string();

        // release string
        lib.ta_release(res.as_ptr())
            .map_err(|_| IpcError::EngineNotCompliant)?;

        string
    };

    let res = Response::try_from(response).map_err(|e| {
        tracing::error!("Failed to parse response ({}) from engine {}.", e, engine,);
        IpcError::InvalidResponseFromEngine
    })?;

    tracing::debug!("Got response {res:?}");
    Ok(res)
}

#[derive(Clone, Default)]
pub struct Engine {
    path: PathBuf,
    pub name: String,
    pub lua_name: String,
    pub instructions: Vec<Instruction>,
    lib: Option<Arc<EngineInterface>>,
}

impl Engine {
    /// Ask the engine to reset it's state for test repeatability.
    pub fn reset_state(&self) -> Result<(), IpcError> {
        ipc_call(self, &Request::ResetState).map(|_| ())
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Debug for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self { name, lua_name, .. } = self;
        write!(f, "Engine {{ {name} ({lua_name}) }}")
    }
}

#[derive(Default, Debug)]
pub struct EngineList(Vec<Engine>);

impl EngineList {
    /// Get an instruction from an instruction ID by iterating through available engines.
    #[must_use]
    pub fn get_instruction_by_id(&self, instruction_id: &String) -> Option<Instruction> {
        for engine in &self.0 {
            for inst in &engine.instructions {
                if *inst.id() == *instruction_id {
                    return Some(inst.clone());
                }
            }
        }
        None
    }

    /// Get an instruction and engine from an instruction ID by iterating through available engines.
    #[must_use]
    pub fn get_engine_by_instruction_id(&self, instruction_id: &String) -> Option<&Engine> {
        for engine in &self.0 {
            for inst in &engine.instructions {
                if *inst.id() == *instruction_id {
                    return Some(engine);
                }
            }
        }
        None
    }

    /// Return the inner list of engines
    #[must_use]
    pub fn inner(&self) -> &Vec<Engine> {
        &self.0
    }
}

/// Get the list of available engines.
pub fn get_engines() -> EngineList {
    let mut engines = Vec::new();
    let engine_dir = env::var("TA_ENGINE_DIR").unwrap_or("./engines".to_owned());
    fs::create_dir_all(engine_dir.clone()).unwrap();
    tracing::info!("Searching for engines in {engine_dir:?}");
    let mut lua_names = vec![];
    search_engine_dir(engine_dir, &mut engines, &mut lua_names);
    EngineList(engines)
}

fn search_engine_dir(engine_dir: String, engines: &mut Vec<Engine>, lua_names: &mut Vec<String>) {
    for path in fs::read_dir(engine_dir).unwrap() {
        let path = path.unwrap();
        let basename = path.file_name();
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                // Search subdir
                search_engine_dir(
                    path.path()
                        .canonicalize()
                        .unwrap()
                        .as_os_str()
                        .to_os_string()
                        .into_string()
                        .unwrap(),
                    engines,
                    lua_names,
                );
                continue;
            }
        }

        if let Ok(str) = basename.into_string() {
            tracing::debug!("Found {:?}", path.path());
            if Path::new(&str).extension().is_some_and(|ext| {
                ext.eq_ignore_ascii_case("so")
                    || ext.eq_ignore_ascii_case("dll")
                    || ext.eq_ignore_ascii_case("dylib")
            }) {
                tracing::debug!("Detected possible engine {str}");
                match EngineInterface::load_plugin_and_check(path.path()) {
                    Ok(lib) => {
                        let mut engine = Engine {
                            name: String::from("newly discovered engine"),
                            path: path.path(),
                            lib: Some(Arc::new(lib)),
                            ..Default::default()
                        };

                        match ipc_call(&engine, &Request::Instructions) {
                            Ok(res) => {
                                if let Response::Instructions {
                                    friendly_name,
                                    engine_version,
                                    engine_lua_name,
                                    ipc_version,
                                    instructions,
                                } = res
                                {
                                    if ipc_version == 2 {
                                        if lua_names.contains(&engine_lua_name) {
                                            tracing::warn!(
                                                "Engine {friendly_name} (v{engine_version}) at {:?} uses a lua name that is already used by another engine!",
                                                path.path()
                                            );
                                            continue;
                                        }
                                        tracing::info!(
                                            "Discovered engine {friendly_name} (v{engine_version}) at {:?}",
                                            path.path()
                                        );
                                        engine.name.clone_from(&friendly_name);
                                        engine.lua_name.clone_from(&engine_lua_name);
                                        engine.instructions = instructions;
                                        engines.push(engine);
                                        lua_names.push(engine_lua_name);
                                    } else {
                                        tracing::warn!(
                                            "Engine {friendly_name} (v{engine_version}) at {:?} doesn't speak the right IPC version!",
                                            path.path()
                                        );
                                    }
                                } else {
                                    tracing::error!("Invalid response from engine {str}");
                                }
                            }
                            Err(e) => tracing::warn!("IPC error: {e:?}"),
                        }
                    }
                    Err(e) => tracing::warn!("Failed to load engine {str}: {e}"),
                }
            }
        }
    }
}
