use std::{collections::HashMap, sync::Arc};

use convert_case::{Case, Casing};
use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

use crate::ipc::EngineList;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionV1 {
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
    /// The parameters this action takes, with a friendly name.
    pub parameters: Vec<(String, ParameterKind)>,
    /// The outputs this action produces, with a friendly name
    pub outputs: Vec<(String, ParameterKind, InstructionParameterSource)>,
    /// The instructions called by this action
    instructions: Vec<InstructionConfiguration>,
}

impl ActionV1 {
    #[must_use]
    pub fn upgrade_action(self, engine_list: &Arc<EngineList>) -> crate::types::Action {
        let mut script = String::new();

        // Add descriptors
        for (name, kind) in &self.parameters {
            script.push_str(&format!("--: param {kind} {name}\n"));
        }
        for (name, kind, _src) in self.outputs {
            script.push_str(&format!("--: return {kind} {name}\n"));
        }

        // Add function signature
        let mut params = String::new();
        for (name, _kind) in &self.parameters {
            params.push_str(&format!("{}, ", name.to_case(Case::Snake)));
        }
        // remove the last ", "
        let _ = params.pop();
        let _ = params.pop();
        script.push_str(&format!("function run_action({params})\n"));

        // add steps
        for (idx, step) in self.instructions.iter().enumerate() {
            script.push_str(&format!(
                "  -- Step {}{}{}\n",
                idx + 1,
                if step.comment.is_empty() { "" } else { ": " },
                step.comment
            ));

            if let Some(instruction) = engine_list.get_instruction_by_id(&step.instruction_id) {
                let mut line = "  ".to_string();
                // Add conditional
                line.push_str(&match &step.run_if {
                    InstructionParameterSource::Literal => String::new(),
                    InstructionParameterSource::FromOutput(step, name) => {
                        format!("if s{}_{} then ", step + 1, name.to_case(Case::Snake))
                    }
                    InstructionParameterSource::FromParameter(param) => format!(
                        "if {} then ",
                        self.parameters[*param].clone().0.to_case(Case::Snake)
                    ),
                });

                // Add outputs with predicatable names
                if !instruction.output_order().is_empty() {
                    line.push_str("local ");
                }
                for output in instruction.output_order() {
                    let (output_name, _kind) = instruction.outputs()[output].clone();
                    line.push_str(&format!(
                        "s{}_{}, ",
                        idx + 1,
                        output_name.to_case(Case::Snake)
                    ));
                }
                if !instruction.output_order().is_empty() {
                    // Remove last ", "
                    let _ = line.pop();
                    let _ = line.pop();
                    line.push_str(" = ");
                }

                // Call instruction with parameters and literals as specified
                let mut inst_params = String::new();
                for param_id in instruction.parameter_order() {
                    let src = &step.parameter_sources[param_id];
                    inst_params.push_str(&match src {
                        InstructionParameterSource::Literal => {
                            match step.parameter_values.get(param_id).unwrap() {
                                ParameterValue::Boolean(b) => format!("{b}"),
                                ParameterValue::Decimal(d) => format!("{d}"),
                                ParameterValue::Integer(i) => format!("{i}"),
                                ParameterValue::String(s) => format!("'{s}'"),
                            }
                        }
                        InstructionParameterSource::FromOutput(step, name) => {
                            format!("s{}_{}", step + 1, name.to_case(Case::Snake))
                        }
                        InstructionParameterSource::FromParameter(param) => {
                            self.parameters[*param].clone().0.to_case(Case::Snake)
                        }
                    });
                    inst_params.push_str(", ");
                }
                // Remove last ", "
                let _ = inst_params.pop();
                let _ = inst_params.pop();

                let engine_lua_name = &engine_list
                    .get_engine_by_instruction_id(&step.instruction_id)
                    .unwrap()
                    .lua_name;
                let instruction_lua_name = instruction.lua_name();
                line.push_str(&format!(
                    "{engine_lua_name}.{instruction_lua_name}({inst_params})"
                ));

                line.push_str(match &step.run_if {
                    InstructionParameterSource::Literal => "",
                    _ => " end",
                });

                script.push_str(&line);
                script.push('\n');
            } else {
                // Improve parameter source and values output
                let mut new_params: HashMap<String, String> = HashMap::new();
                for (param_id, src) in &step.parameter_sources {
                    new_params.insert(
                        param_id.clone(),
                        match src {
                            InstructionParameterSource::Literal => {
                                format!("{}", step.parameter_values.get(param_id).unwrap())
                            }
                            InstructionParameterSource::FromOutput(step, name) => {
                                format!("s{}_{}", step + 1, name.to_case(Case::Snake))
                            }
                            InstructionParameterSource::FromParameter(param) => {
                                self.parameters[*param].clone().0.to_case(Case::Snake)
                            }
                        },
                    );
                }
                script.push_str(&format!(
                    "  -- Instr: {} | RunIf: {:?} | Params: {:?}\n",
                    step.instruction_id, step.run_if, new_params,
                ));
            }
        }

        // end function
        script.push_str("end\n");

        crate::types::Action {
            version: 2,
            id: self.id,
            friendly_name: self.friendly_name,
            description: self.description,
            group: self.group,
            author: self.author,
            visible: self.visible,
            script,
            required_instructions: Vec::new(), // this will be populated on save
        }
    }
}

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
struct InstructionConfiguration {
    pub instruction_id: String,
    pub comment: String,
    /// Run If can depend on any boolean parameter, or if set to 'Literal' will always run.
    pub run_if: InstructionParameterSource,
    pub parameter_sources: HashMap<String, InstructionParameterSource>,
    pub parameter_values: HashMap<String, ParameterValue>,
}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum InstructionParameterSource {
    #[default]
    Literal,
    FromParameter(usize),
    FromOutput(usize, String),
}
