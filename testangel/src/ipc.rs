use std::{
    env,
    ffi::{c_char, CStr, CString},
    fmt, fs, io,
    path::PathBuf,
    sync::Arc,
};

use testangel_ipc::prelude::*;

#[derive(Debug)]
pub enum IpcError {
    IoError(io::Error),
    EngineNotStarted,
    EngineNotCompliant,
    CantLockEngineIo,
    InvalidResponseFromEngine,
}

pub fn ipc_call(engine: &Engine, request: Request) -> Result<Response, IpcError> {
    log::debug!(
        "Sending request {:?} to engine {} at {:?}.",
        request,
        engine,
        engine.path
    );

    let request = request.to_json();
    let c_request = CString::new(request).unwrap();
    let response = unsafe {
        let lib = engine.lib.clone().ok_or(IpcError::EngineNotStarted)?;

        let ta_call: libloading::Symbol<
            unsafe extern "C" fn(input: *const c_char) -> *const c_char,
        > = lib
            .get(b"ta_call")
            .map_err(|_| IpcError::EngineNotCompliant)?;
        let res = ta_call(c_request.as_ptr());
        let res = CStr::from_ptr(res);
        let string = String::from_utf8_lossy(res.to_bytes()).to_string();

        // release string
        let ta_release: libloading::Symbol<unsafe extern "C" fn(target: *const c_char)> = lib
            .get(b"ta_release")
            .map_err(|_| IpcError::EngineNotCompliant)?;
        ta_release(res.as_ptr());

        string
    };

    let res = Response::try_from(response).map_err(|e| {
        log::error!("Failed to parse response ({}) from engine {}.", e, engine,);
        IpcError::InvalidResponseFromEngine
    })?;

    log::debug!("Got response {res:?}");
    Ok(res)
}

#[derive(Clone, Debug, Default)]
pub struct Engine {
    path: PathBuf,
    pub name: String,
    pub instructions: Vec<Instruction>,
    lib: Option<Arc<libloading::Library>>,
}

impl Engine {
    /// Ask the engine to reset it's state for test repeatability.
    pub fn reset_state(&self) -> Result<(), IpcError> {
        ipc_call(self, Request::ResetState).map(|_| ())
    }
}

impl fmt::Display for Engine {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

#[derive(Default, Debug)]
pub struct EngineList(Vec<Engine>);

impl EngineList {
    /// Get an instruction from an instruction ID by iterating through available engines.
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
    pub fn inner(&self) -> &Vec<Engine> {
        &self.0
    }
}

/// Get the list of available engines.
pub fn get_engines() -> EngineList {
    let mut engines = Vec::new();
    let engine_dir = env::var("TA_ENGINE_DIR").unwrap_or("./engines".to_owned());
    fs::create_dir_all(engine_dir.clone()).unwrap();
    log::info!("Searching for engines in {engine_dir:?}");
    for path in fs::read_dir(engine_dir).unwrap() {
        let path = path.unwrap();
        let basename = path.file_name();
        if let Ok(meta) = path.metadata() {
            if meta.is_dir() {
                continue;
            }
        }

        if let Ok(str) = basename.into_string() {
            log::debug!("Found {str}");
            if str.ends_with(".so") || str.ends_with(".dll") || str.ends_with(".dylib") {
                log::debug!("Detected possible engine {str}");
                match unsafe { libloading::Library::new(path.path()) } {
                    Ok(lib) => {
                        let mut engine = Engine {
                            name: String::from("newly discovered engine"),
                            path: path.path(),
                            lib: Some(Arc::new(lib)),
                            ..Default::default()
                        };
                        match ipc_call(&engine, Request::Instructions) {
                            Ok(res) => match res {
                                Response::Instructions {
                                    friendly_name,
                                    engine_version,
                                    ipc_version,
                                    instructions,
                                } => {
                                    if ipc_version == 1 {
                                        log::info!(
                                            "Discovered engine {friendly_name} (v{engine_version}) at {:?}",
                                            path.path()
                                        );
                                        engine.name = friendly_name.clone();
                                        engine.instructions = instructions;
                                        engines.push(engine);
                                    } else {
                                        log::warn!(
                                            "Engine {friendly_name} (v{engine_version}) at {:?} doesn't speak the right IPC version!",
                                            path.path()
                                        );
                                    }
                                }
                                _ => log::error!("Invalid response from engine {str}"),
                            },
                            Err(e) => log::warn!("IPC error: {e:?}"),
                        }
                    }
                    Err(e) => log::warn!("Failed to load engine {str}: {e}"),
                }
            }
        }
    }
    EngineList(engines)
}
