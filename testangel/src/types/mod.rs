use std::{collections::HashMap, fmt, sync::Arc};

use mlua::{Lua, ObjectLike};
use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

use crate::{
    action_loader::ActionMap,
    action_syntax::{Descriptor, DescriptorKind},
    ipc::{self, EngineList, IpcError},
};

pub mod action_v1;

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub struct VersionedFile {
    version: usize,
}

impl VersionedFile {
    /// Get the version of the file
    pub fn version(&self) -> usize {
        self.version
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    /// The data version of this action.
    version: usize,
    /// The internal ID of this action. Must be unique.
    pub id: String,
    /// The friendly name of this action.
    pub friendly_name: String,
    /// A description of this action.
    pub description: String,
    /// A group this action belongs to.
    pub group: String,
    /// The author of this action.
    pub author: String,
    /// Whether this action should be visible in the flow editor.
    pub visible: bool,
    /// The Lua code driving this action.
    pub script: String,
    /// A vector of required instruction IDs for this action to work.
    pub required_instructions: Vec<String>,
}

impl Default for Action {
    fn default() -> Self {
        Self {
            version: 2,
            id: uuid::Uuid::new_v4().to_string(),
            friendly_name: String::new(),
            description: String::new(),
            author: String::new(),
            visible: true,
            group: String::new(),
            script: "--: param Integer Example Parameter\n--: return Text Some value to return\nfunction run_action(x)\n  \n  return 'Hello, world!'\nend\n".to_string(),
            required_instructions: Vec::new(),
        }
    }
}

impl Action {
    /// Get the version of this action.
    pub fn version(&self) -> usize {
        self.version
    }

    /// Generate a new ID for this action.
    pub fn new_id(&mut self) {
        self.id = uuid::Uuid::new_v4().to_string();
    }

    /// Check that all the instructions this action uses are available. Returns
    /// Ok if all instructions are available, otherwise returns a list of
    /// missing instructions.
    pub fn check_instructions_available(
        &self,
        engine_list: Arc<EngineList>,
    ) -> Result<(), Vec<String>> {
        let mut missing = vec![];
        for instruction in &self.required_instructions {
            if engine_list.get_instruction_by_id(instruction).is_none()
                && !missing.contains(instruction)
            {
                missing.push(instruction.clone());
            }
        }
        if missing.is_empty() {
            Ok(())
        } else {
            Err(missing)
        }
    }

    /// Get a list of parameters that need to be provided to this action.
    pub fn parameters(&self) -> Vec<(String, ParameterKind)> {
        let descriptors = Descriptor::parse_all(&self.script);
        let mut params = vec![];
        for d in descriptors {
            if d.descriptor_kind == DescriptorKind::Parameter {
                params.push((d.name.clone(), d.kind));
            }
        }
        params
    }

    /// Get a list of outputs provided by this action.
    pub fn outputs(&self) -> Vec<(String, ParameterKind)> {
        let descriptors = Descriptor::parse_all(&self.script);
        let mut outputs = vec![];
        for d in descriptors {
            if d.descriptor_kind == DescriptorKind::Return {
                outputs.push((d.name.clone(), d.kind));
            }
        }
        outputs
    }
}

#[derive(Debug)]
pub enum FlowError {
    FromInstruction {
        error_kind: ErrorKind,
        reason: String,
    },
    Lua(String),
    IPCFailure(IpcError),
    ActionDidntReturnCorrectArgumentCount,
    ActionDidntReturnValidArguments,
    InstructionCalledWithWrongNumberOfParams,
    InstructionCalledWithInvalidParamType,
}

impl std::error::Error for FlowError {}

impl fmt::Display for FlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IPCFailure(e) => write!(f, "An IPC call failed ({e:?})."),
            Self::Lua(e) => write!(f, "An action script error occurred:\n{}", e),
            Self::FromInstruction { error_kind, reason } => write!(
                f,
                "An instruction returned an error: {error_kind:?}: {reason}"
            ),
            Self::ActionDidntReturnCorrectArgumentCount => {
                write!(f, "The action didn't return the correct amount of values.")
            }
            Self::ActionDidntReturnValidArguments => {
                write!(f, "The action didn't return valid values.")
            }
            Self::InstructionCalledWithWrongNumberOfParams => write!(
                f,
                "An instruction was called with the wrong number of parameters."
            ),
            Self::InstructionCalledWithInvalidParamType => write!(
                f,
                "An instruction was called with the wrong parameter type."
            ),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomationFlow {
    /// The version of this automation flow file
    version: usize,
    /// The actions called by this flow
    pub actions: Vec<ActionConfiguration>,
}

impl Default for AutomationFlow {
    fn default() -> Self {
        Self {
            version: 1,
            actions: vec![],
        }
    }
}

impl AutomationFlow {
    /// Get the version of this flow.
    pub fn version(&self) -> usize {
        self.version
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct ActionConfiguration {
    pub action_id: String,
    pub parameter_sources: HashMap<usize, ActionParameterSource>,
    pub parameter_values: HashMap<usize, ParameterValue>,
}
impl ActionConfiguration {
    /// Execute this action
    pub fn execute(
        &self,
        action_map: Arc<ActionMap>,
        engine_map: Arc<EngineList>,
        previous_action_outputs: Vec<HashMap<usize, ParameterValue>>,
    ) -> Result<(HashMap<usize, ParameterValue>, Vec<Evidence>), FlowError> {
        // Find action by ID
        let action = action_map.get_action_by_id(&self.action_id).unwrap();
        // Build action parameters
        let mut action_parameters = HashMap::new();
        for (id, src) in &self.parameter_sources {
            let value = match src {
                ActionParameterSource::Literal => self.parameter_values.get(id).unwrap().clone(),
                ActionParameterSource::FromOutput(step, id) => previous_action_outputs
                    .get(*step)
                    .unwrap()
                    .get(id)
                    .unwrap()
                    .clone(),
            };
            action_parameters.insert(*id, value);
        }
        let mut param_vec = vec![];
        for i in 0..action_parameters.len() {
            param_vec.push(action_parameters[&i].clone());
        }
        Self::execute_directly(engine_map, &action, param_vec)
    }

    #[allow(clippy::type_complexity)]
    /// Directly execute an action with a set of parameters.
    pub fn execute_directly(
        engine_map: Arc<EngineList>,
        action: &Action,
        action_parameters: Vec<ParameterValue>,
    ) -> Result<(HashMap<usize, ParameterValue>, Vec<Evidence>), FlowError> {
        let mut output = HashMap::new();

        // Prepare Lua environment
        let lua_env = Lua::new();
        lua_env.set_app_data::<Vec<Evidence>>(vec![]);

        // unwrap rationale: this will only fail under memory issues
        for engine in engine_map.inner().clone() {
            let engine_lua_name = engine.lua_name.clone();
            let engine_tbl = lua_env.create_table().unwrap();
            for instruction in engine.instructions.clone() {
                let instruction_lua_name = instruction.lua_name().clone();
                let engine = engine.clone();
                let instruction_fn = lua_env
                    .create_function(move |lua, args: mlua::MultiValue| {
                        // Check we have the correct number of parameters.
                        if args.len() != instruction.parameters().len() {
                            return Err(mlua::Error::external(
                                FlowError::InstructionCalledWithWrongNumberOfParams,
                            ));
                        }

                        // Check we have the correct parameter types and convert to parameter map
                        let mut param_map = HashMap::new();
                        for (idx, param_id) in instruction.parameter_order().iter().enumerate() {
                            if let Some((_name, kind)) = instruction.parameters().get(param_id) {
                                // Get argument and coerce
                                let arg = args[idx].clone();
                                match kind {
                                    ParameterKind::Boolean => {
                                        if let mlua::Value::Boolean(b) = arg {
                                            param_map.insert(
                                                param_id.clone(),
                                                ParameterValue::Boolean(b),
                                            );
                                        } else {
                                            return Err(mlua::Error::external(
                                                FlowError::InstructionCalledWithInvalidParamType,
                                            ));
                                        }
                                    }
                                    ParameterKind::String => {
                                        let maybe_str = lua.coerce_string(arg)?;
                                        if let Some(s) = maybe_str {
                                            param_map.insert(
                                                param_id.clone(),
                                                ParameterValue::String(s.to_str()?.to_string()),
                                            );
                                        } else {
                                            return Err(mlua::Error::external(
                                                FlowError::InstructionCalledWithInvalidParamType,
                                            ));
                                        }
                                    }
                                    ParameterKind::Decimal => {
                                        let maybe_dec = lua.coerce_number(arg)?;
                                        if let Some(d) = maybe_dec {
                                            param_map.insert(
                                                param_id.clone(),
                                                ParameterValue::Decimal(d),
                                            );
                                        } else {
                                            return Err(mlua::Error::external(
                                                FlowError::InstructionCalledWithInvalidParamType,
                                            ));
                                        }
                                    }
                                    ParameterKind::Integer => {
                                        let maybe_int = lua.coerce_integer(arg)?;
                                        if let Some(i) = maybe_int {
                                            param_map.insert(
                                                param_id.clone(),
                                                ParameterValue::Integer(i),
                                            );
                                        } else {
                                            return Err(mlua::Error::external(
                                                FlowError::InstructionCalledWithInvalidParamType,
                                            ));
                                        }
                                    }
                                }
                            }
                        }

                        // Trigger instruction behaviour
                        let response = ipc::ipc_call(
                            &engine,
                            Request::RunInstructions {
                                instructions: vec![InstructionWithParameters {
                                    instruction: instruction.id().clone(),
                                    parameters: param_map,
                                }],
                            },
                        )
                        .map_err(|e| mlua::Error::external(FlowError::IPCFailure(e)))?;

                        match response {
                            Response::ExecutionOutput { output, evidence } => {
                                // Add evidence
                                let mut ev = lua.app_data_mut::<Vec<Evidence>>().unwrap();
                                for item in &evidence[0] {
                                    ev.push(item.clone());
                                }

                                // Convert output back to Lua values
                                let mut outputs = vec![];
                                for output_id in instruction.output_order() {
                                    let o = output[0][output_id].clone();
                                    match o {
                                        ParameterValue::Boolean(b) => {
                                            log::debug!("Boolean {b} returned to Lua");
                                            outputs.push(mlua::Value::Boolean(b))
                                        }
                                        ParameterValue::String(s) => {
                                            log::debug!("String {s:?} returned to Lua");
                                            outputs.push(mlua::Value::String(lua.create_string(s)?))
                                        }
                                        ParameterValue::Integer(i) => {
                                            log::debug!("Integer {i} returned to Lua");
                                            outputs.push(mlua::Value::Integer(i))
                                        }
                                        ParameterValue::Decimal(n) => {
                                            log::debug!("Decimal {n} returned to Lua");
                                            outputs.push(mlua::Value::Number(n))
                                        }
                                    }
                                }

                                Ok(mlua::MultiValue::from_vec(outputs))
                            }
                            Response::Error { kind, reason } => {
                                Err(mlua::Error::external(FlowError::FromInstruction {
                                    error_kind: kind,
                                    reason,
                                }))
                            }
                            _ => unreachable!(),
                        }
                    })
                    .unwrap();
                engine_tbl
                    .set(instruction_lua_name.as_str(), instruction_fn)
                    .unwrap();
            }
            lua_env
                .globals()
                .set(engine_lua_name.as_str(), engine_tbl)
                .unwrap();
        }

        // Execute Lua script
        // Add parameters and get results
        let mut params = vec![];
        for param in action_parameters {
            match param {
                ParameterValue::Boolean(b) => params.push(mlua::Value::Boolean(b)),
                ParameterValue::String(s) => params.push(mlua::Value::String(
                    lua_env
                        .create_string(s)
                        .map_err(|e| FlowError::Lua(e.to_string()))?,
                )),
                ParameterValue::Integer(i) => params.push(mlua::Value::Integer(i)),
                ParameterValue::Decimal(n) => params.push(mlua::Value::Number(n)),
            }
        }

        lua_env
            .load(&action.script)
            .set_name(action.friendly_name.clone())
            .exec()
            .map_err(|e| FlowError::Lua(e.to_string()))?;

        let res: mlua::MultiValue = lua_env
            .globals()
            .call_function("run_action", mlua::MultiValue::from_vec(params))
            .map_err(|e| FlowError::Lua(e.to_string()))?;
        let res = res.into_vec();

        // Process return values
        let ao = action.outputs();
        if ao.len() != res.len() {
            return Err(FlowError::ActionDidntReturnCorrectArgumentCount);
        }
        for i in 0..ao.len() {
            let (_name, kind) = ao[i].clone();
            let out = res[i].clone();
            let ta_out = match out {
                mlua::Value::Boolean(b) => ParameterValue::Boolean(b),
                mlua::Value::String(s) => ParameterValue::String(s.to_str().unwrap().to_owned()),
                mlua::Value::Integer(i) => ParameterValue::Integer(i),
                mlua::Value::Number(n) => ParameterValue::Decimal(n),
                _ => return Err(FlowError::ActionDidntReturnValidArguments),
            };
            if ta_out.kind() != kind {
                return Err(FlowError::ActionDidntReturnValidArguments);
            }
            output.insert(i, ta_out);
        }

        let evidence = lua_env.app_data_ref::<Vec<Evidence>>().unwrap().clone();

        Ok((output, evidence))
    }

    /// Update this action configuration to match the inputs and outputs of the provided action.
    /// This will panic if the action's ID doesn't match the ID of this configuration already set.
    /// Return true if this configuration has changed.
    pub fn update(&mut self, action: Action) -> bool {
        if self.action_id != action.id {
            panic!("ActionConfiguration tried to be updated with a different action!");
        }

        // If number of parameters has changed
        if self.parameter_sources.len() != action.parameters().len() {
            *self = Self::from(action);
            return true;
        }

        for (n, value) in &self.parameter_values {
            let (_, action_param_kind) = &action.parameters()[*n];
            if value.kind() != *action_param_kind {
                // Reset parameters
                *self = Self::from(action);
                return true;
            }
        }

        false
    }
}

impl From<Action> for ActionConfiguration {
    fn from(value: Action) -> Self {
        let mut parameter_sources = HashMap::new();
        let mut parameter_values = HashMap::new();
        for (id, (_friendly_name, kind)) in value.parameters().iter().enumerate() {
            parameter_sources.insert(id, ActionParameterSource::Literal);
            parameter_values.insert(id, kind.default_value());
        }
        Self {
            action_id: value.id.clone(),
            parameter_sources,
            parameter_values,
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ActionParameterSource {
    #[default]
    Literal,
    FromOutput(usize, usize),
}

impl fmt::Display for ActionParameterSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FromOutput(step, id) => {
                write!(f, "From Step {}: Output {}", step + 1, id + 1)
            }
            Self::Literal => write!(f, "Literal"),
        }
    }
}
