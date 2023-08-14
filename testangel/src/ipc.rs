use std::{
    env, fmt, fs,
    io::{self, Read, Write},
    path::PathBuf,
    process::{ChildStdin, ChildStdout, Command, Stdio},
    sync::{Arc, Mutex},
};

use testangel_ipc::prelude::*;

#[derive(Debug)]
pub enum IpcError {
    IoError(io::Error),
    EngineNotStarted,
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
    engine.write(request.to_json())?;

    let buf = engine.read_until_eol()?;
    let res = Response::try_from(buf).map_err(|e| {
        log::error!(
            "Failed to parse subprocess response ({}) from engine {}.",
            e,
            engine,
        );
        return IpcError::InvalidResponseFromEngine;
    })?;

    log::debug!("Got response {res:?}");
    return Ok(res);
}

#[derive(Clone, Debug, Default)]
pub struct Engine {
    path: PathBuf,
    pub name: String,
    pub instructions: Vec<Instruction>,
    stdin: Option<Arc<Mutex<ChildStdin>>>,
    stdout: Option<Arc<Mutex<ChildStdout>>>,
}

impl Engine {
    fn start(&mut self) -> Result<(), IpcError> {
        let proc = Command::new(&self.path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| IpcError::IoError(e))?;
        self.stdin = proc.stdin.map(|v| Arc::new(Mutex::new(v)));
        self.stdout = proc.stdout.map(|v| Arc::new(Mutex::new(v)));
        Ok(())
    }

    fn write<S: AsRef<str>>(&self, message: S) -> Result<(), IpcError> {
        let stdin = self.stdin.as_ref().ok_or(IpcError::EngineNotStarted)?;
        let mut stdin = stdin.lock().map_err(|_| IpcError::CantLockEngineIo)?;
        stdin
            .write_fmt(format_args!("{}\n", message.as_ref()))
            .map_err(|e| IpcError::IoError(e))
    }

    fn read_until_eol(&self) -> Result<String, IpcError> {
        let stdout = self.stdout.as_ref().ok_or(IpcError::EngineNotStarted)?;
        let mut stdout = stdout.lock().map_err(|_| IpcError::CantLockEngineIo)?;
        let mut buf = Vec::new();
        loop {
            let mut one_byte = [0u8];
            stdout
                .read_exact(&mut one_byte)
                .map_err(|e| IpcError::IoError(e))?;
            if one_byte[0] != ('\n' as u8) {
                buf.push(one_byte[0]);
            } else {
                return Ok(String::from_utf8(buf).map_err(|_| IpcError::InvalidResponseFromEngine)?);
            }
        }
    }

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
        return None;
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
        return None;
    }

    /// Return the inner list of engines
    pub fn inner(&self) -> &Vec<Engine> {
        &self.0
    }
}

/// Get the list of available engines.
pub fn get_engines() -> EngineList {
    let mut engines = Vec::new();
    let engine_dir = env::var("ENGINE_DIR").unwrap_or("./engines".to_owned());
    fs::create_dir_all(engine_dir.clone()).unwrap();
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
                let mut engine = Engine {
                    name: String::from("newly discovered engine"),
                    path: path.path(),
                    ..Default::default()
                };
                if let Err(_) = engine.start() {
                    log::error!("Failed to start engine.");
                    continue;
                }
                if let Ok(res) = ipc_call(&mut engine, Request::Instructions) {
                    match res {
                        Response::Instructions {
                            friendly_name,
                            instructions,
                        } => {
                            log::info!("Discovered engine {friendly_name} at {:?}", path.path());
                            engine.name = friendly_name.clone();
                            engine.instructions = instructions;
                            engines.push(engine);
                        }
                        _ => log::error!("Invalid response from engine {str}"),
                    }
                }
            }
        }
    }
    EngineList(engines)
}
