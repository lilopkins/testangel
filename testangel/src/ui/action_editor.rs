use std::{env, fmt, fs, path::PathBuf, sync::Arc};

use iced::widget::{
    column, combo_box, row, scrollable, Button, Column, Container, PickList, Row, Rule, Scrollable,
    Space, Text, TextInput,
};
use iced_aw::Card;
use testangel::{
    ipc::EngineList,
    types::{Action, InstructionConfiguration, InstructionParameterSource},
};
use testangel_ipc::prelude::{Instruction, ParameterKind, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum ActionEditorMessage {
    SaveAction,
    SaveAsAction,
    CloseAction,

    NameChanged(String),
    GroupChanged(String),
    DescriptionChanged(String),

    ParameterCreate,
    ParameterNameChange(usize, String),
    ParameterTypeChange(usize, ParameterKind),
    ParameterMoveUp(usize),
    ParameterMoveDown(usize),
    ParameterDelete(usize),

    StepCreate(AvailableInstruction),
    StepParameterSourceChange(usize, String, InstructionParameterSource),
    StepParameterValueChange(usize, String, String),
    StepMoveUp(usize),
    StepMoveDown(usize),
    StepDelete(usize),

    OutputCreate,
    OutputNameChange(usize, String),
    OutputMoveUp(usize),
    OutputMoveDown(usize),
    OutputDelete(usize),
    OutputSourceChange(usize, ParameterKind, InstructionParameterSource),
}

#[derive(Clone, Debug)]
pub enum ActionEditorMessageOut {
    CloseActionEditor,
}

pub enum SaveOrOpenActionError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    ActionNotVersionCompatible,
    MissingInstruction(String),
}

impl fmt::Display for SaveOrOpenActionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O Error: {e}"),
            Self::ParsingError(e) => write!(f, "Parsing Error: {e}"),
            Self::SerializingError(e) => write!(f, "Serializing error: {e}"),
            Self::ActionNotVersionCompatible => write!(
                f,
                "This action is not compatible with this version of TestAngel."
            ),
            Self::MissingInstruction(instruction_id) => write!(
                f,
                "The action contains an instruction ({instruction_id}) which could not be loaded."
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AvailableInstruction {
    /// The friendly name of this instruction.
    engine_name: String,
    /// The friendly name of this instruction.
    friendly_name: String,
    /// The instruction this is based on.
    base_instruction: Instruction,
}

impl fmt::Display for AvailableInstruction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.engine_name, self.friendly_name)
    }
}

pub struct ActionEditor {
    engines_list: Arc<EngineList>,
    output_list: Vec<(isize, ParameterKind, InstructionParameterSource)>,
    add_instruction_combo: combo_box::State<AvailableInstruction>,

    currently_open: Option<Action>,
    current_path: Option<PathBuf>,
    needs_saving: bool,
}

impl Default for ActionEditor {
    fn default() -> Self {
        Self {
            engines_list: Arc::new(EngineList::default()),
            output_list: vec![],
            add_instruction_combo: combo_box::State::new(Vec::new()),
            currently_open: None,
            current_path: None,
            needs_saving: false,
        }
    }
}

impl ActionEditor {
    /// Initialise a new ActionEditor with the provided [`ActionMap`].
    pub(crate) fn new(engines_list: Arc<EngineList>) -> Self {
        let mut available_instructions = vec![];
        for engine in engines_list.inner() {
            for instruction in &engine.instructions {
                available_instructions.push(AvailableInstruction {
                    engine_name: engine.name.clone(),
                    friendly_name: instruction.friendly_name().clone(),
                    base_instruction: instruction.clone(),
                });
            }
        }

        Self {
            engines_list,
            add_instruction_combo: combo_box::State::new(available_instructions),
            ..Default::default()
        }
    }

    /// Create a new action and open it
    pub(crate) fn new_action(&mut self) {
        self.offer_to_save_default_error_handling();
        self.currently_open = Some(Action::default());
        self.current_path = None;
        self.needs_saving = true;
    }

    /// Open an action
    pub(crate) fn open_action(&mut self, file: PathBuf) -> Result<(), SaveOrOpenActionError> {
        self.offer_to_save_default_error_handling();
        let action: Action =
            ron::from_str(&fs::read_to_string(&file).map_err(SaveOrOpenActionError::IoError)?)
                .map_err(SaveOrOpenActionError::ParsingError)?;
        if action.version() != 1 {
            return Err(SaveOrOpenActionError::ActionNotVersionCompatible);
        }
        for ic in &action.instructions {
            if self
                .engines_list
                .get_instruction_by_id(&ic.instruction_id)
                .is_none()
            {
                return Err(SaveOrOpenActionError::MissingInstruction(
                    ic.instruction_id.clone(),
                ));
            }
        }
        self.currently_open = Some(action);
        self.current_path = Some(file);
        self.needs_saving = false;
        self.update_outputs();
        Ok(())
    }

    /// Offer to save if it is needed
    fn offer_to_save(&mut self) -> Result<(), SaveOrOpenActionError> {
        if self.needs_saving
            && rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Do you want to save this action?")
                .set_description("This action has been modified. Do you want to save it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
        {
            self.save_action(false)?;
        }
        Ok(())
    }

    /// Offer to save if it is needed with default error handling
    fn offer_to_save_default_error_handling(&mut self) {
        if let Err(e) = self.offer_to_save() {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to save action")
                .set_description(&format!("{e}"))
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
    }

    /// Save the currently opened action
    fn save_action(&mut self, always_prompt_where: bool) -> Result<(), SaveOrOpenActionError> {
        self.needs_saving = false;
        if always_prompt_where || self.current_path.is_none() {
            // Populate save path
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("TestAngel Actions", &["taaction"])
                .set_title("Save Action")
                .set_directory(
                    env::current_dir()
                        .expect("Failed to read current directory")
                        .join("actions"),
                )
                .save_file()
            {
                self.current_path = Some(file);
            } else {
                return Ok(());
            }
        }

        // Save
        let save_path = self.current_path.as_ref().unwrap();
        let data = ron::to_string(self.currently_open.as_ref().unwrap())
            .map_err(SaveOrOpenActionError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenActionError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Close the currently opened action
    fn close_action(&mut self) {
        self.offer_to_save_default_error_handling();
        self.currently_open = None;
        self.current_path = None;
        self.needs_saving = false;
    }

    /// Hint that the open action has been modified.
    fn modified(&mut self) {
        self.needs_saving = true;
    }

    /// Generate the UI for the action parameters
    fn ui_parameters(&self) -> iced::Element<'_, ActionEditorMessage> {
        let action = self.currently_open.as_ref().unwrap();

        action
            .parameters
            .iter()
            .enumerate()
            .fold(Column::new().spacing(4), |col, (idx, (name, kind))| {
                col.push(
                    row![
                        Button::new("×").on_press(ActionEditorMessage::ParameterDelete(idx)),
                        Button::new("ʌ").on_press_maybe(if idx == 0 {
                            None
                        } else {
                            Some(ActionEditorMessage::ParameterMoveUp(idx))
                        }),
                        Button::new("v").on_press_maybe(if (idx + 1) == action.parameters.len() {
                            None
                        } else {
                            Some(ActionEditorMessage::ParameterMoveDown(idx))
                        }),
                        TextInput::new("Parameter Name", name)
                            .on_input(move |s| ActionEditorMessage::ParameterNameChange(idx, s)),
                        PickList::new(
                            &[
                                ParameterKind::String,
                                ParameterKind::Integer,
                                ParameterKind::Decimal,
                            ][..],
                            Some(kind.clone()),
                            move |k| ActionEditorMessage::ParameterTypeChange(idx, k)
                        )
                        .placeholder("Parameter Kind"),
                    ]
                    .spacing(4)
                    .align_items(iced::Alignment::Center),
                )
            })
            .into()
    }

    fn ui_instruction_inputs(
        &self,
        step_idx: usize,
        instruction_config: InstructionConfiguration,
        instruction: Instruction,
    ) -> iced::Element<'_, ActionEditorMessage> {
        instruction
            .parameter_order()
            .iter()
            .enumerate()
            .fold(Column::new().spacing(4), |col, (_param_idx, id)| {
                let (name, kind) = &instruction.parameters()[id];
                let param_source = &instruction_config.parameter_sources[id];
                let param_value = &instruction_config.parameter_values[id];
                let id = id.clone();

                let source_opts = self
                    .output_list
                    .iter()
                    .filter(|(from_step, param_kind, _)| {
                        *from_step < (step_idx as isize) && param_kind == kind
                    })
                    .fold(
                        Row::new()
                            .spacing(2)
                            .push(Button::new(Text::new("Use Literal")).on_press(
                                ActionEditorMessage::StepParameterSourceChange(
                                    step_idx,
                                    id.clone(),
                                    InstructionParameterSource::Literal,
                                ),
                            )),
                        |row, (_, _, src)| {
                            row.push(
                                Button::new(Text::new(format!(
                                    "Set {}",
                                    self.friendly_source_string(src)
                                )))
                                .on_press(
                                    ActionEditorMessage::StepParameterSourceChange(
                                        step_idx,
                                        id.clone(),
                                        src.clone(),
                                    ),
                                ),
                            )
                        },
                    );

                let literal_input: iced::Element<'_, _> = match param_source {
                    InstructionParameterSource::Literal => {
                        TextInput::new("Literal value", &param_value.to_string())
                            .on_input(move |new_val| {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    id.clone(),
                                    new_val,
                                )
                            })
                            .into()
                    }
                    _ => Space::new(0, 0).into(),
                };

                col.push(
                    row![
                        Text::new(format!(
                            "{name} ({kind}) from {}",
                            self.friendly_source_string(param_source)
                        )),
                        literal_input,
                        Scrollable::new(source_opts).direction(scrollable::Direction::Horizontal(
                            scrollable::Properties::new()
                        )),
                    ]
                    .spacing(4)
                    .align_items(iced::Alignment::Center),
                )
            })
            .into()
    }

    fn ui_steps(&self) -> iced::Element<'_, ActionEditorMessage> {
        let action = self.currently_open.as_ref().unwrap();

        action
            .instructions
            .iter()
            .enumerate()
            .fold(
                Column::new().spacing(4),
                |col, (idx, instruction_config)| {
                    let instruction = self
                        .engines_list
                        .get_instruction_by_id(&instruction_config.instruction_id)
                        .unwrap();
                    let mut outputs_text = String::new();
                    for (name, kind) in instruction.outputs().values() {
                        outputs_text.push_str(&format!("{name}: {kind}"));
                    }
                    col.push(Card::new(
                        Text::new(format!("Step {}: {}", idx + 1, instruction.friendly_name())),
                        column![
                            row![
                                Button::new("×").on_press(ActionEditorMessage::StepDelete(idx)),
                                Button::new("ʌ").on_press_maybe(if idx == 0 {
                                    None
                                } else {
                                    Some(ActionEditorMessage::StepMoveUp(idx))
                                }),
                                Button::new("v").on_press_maybe(
                                    if (idx + 1) == action.instructions.len() {
                                        None
                                    } else {
                                        Some(ActionEditorMessage::StepMoveDown(idx))
                                    }
                                ),
                                Text::new(instruction.description().clone()),
                            ]
                            .spacing(4),
                            Text::new("Inputs"),
                            self.ui_instruction_inputs(
                                idx,
                                instruction_config.clone(),
                                instruction.clone()
                            ),
                            Text::new("Outputs"),
                            Text::new(outputs_text),
                        ]
                        .spacing(4),
                    ))
                },
            )
            .into()
    }

    fn ui_outputs(&self) -> iced::Element<'_, ActionEditorMessage> {
        let action = self.currently_open.as_ref().unwrap();

        action
            .outputs
            .iter()
            .enumerate()
            .fold(
                Column::new().spacing(4),
                |col, (idx, (name, kind, source))| {
                    let source_opts = self.output_list.iter().fold(
                        Row::new().spacing(2),
                        |row, (_, kind, src)| {
                            row.push(
                                Button::new(Text::new(format!(
                                    "Set {}",
                                    self.friendly_source_string(src)
                                )))
                                .on_press(
                                    ActionEditorMessage::OutputSourceChange(
                                        idx,
                                        kind.clone(),
                                        src.clone(),
                                    ),
                                ),
                            )
                        },
                    );

                    col.push(
                        row![
                            Button::new("×").on_press(ActionEditorMessage::OutputDelete(idx)),
                            Button::new("ʌ").on_press_maybe(if idx == 0 {
                                None
                            } else {
                                Some(ActionEditorMessage::OutputMoveUp(idx))
                            }),
                            Button::new("v").on_press_maybe(if (idx + 1) == action.outputs.len() {
                                None
                            } else {
                                Some(ActionEditorMessage::OutputMoveDown(idx))
                            }),
                            TextInput::new("Output Name", name)
                                .on_input(move |s| ActionEditorMessage::OutputNameChange(idx, s)),
                            Text::new(format!("({kind}) {}", self.friendly_source_string(source))),
                            Scrollable::new(source_opts).direction(
                                scrollable::Direction::Horizontal(scrollable::Properties::new())
                            ),
                        ]
                        .spacing(4)
                        .align_items(iced::Alignment::Center),
                    )
                },
            )
            .into()
    }

    /// Update the possible outputs
    fn update_outputs(&mut self) {
        self.output_list.clear();

        if let Some(action) = &self.currently_open {
            for (index, (_name, kind)) in action.parameters.iter().enumerate() {
                self.output_list.push((
                    -1,
                    kind.clone(),
                    InstructionParameterSource::FromParameter(index),
                ));
            }

            for (step, instruction_config) in action.instructions.iter().enumerate() {
                let instruction = self
                    .engines_list
                    .get_instruction_by_id(&instruction_config.instruction_id)
                    .unwrap();
                for (id, (_name, kind)) in instruction.outputs() {
                    self.output_list.push((
                        step as isize,
                        kind.clone(),
                        InstructionParameterSource::FromOutput(step, id.clone()),
                    ));
                }
            }
        }
    }

    /// Convert an [`InstructionParameterSource`] to a friendly String by fetching names of parameters
    /// and results from the currently opened action.
    fn friendly_source_string(&self, source: &InstructionParameterSource) -> String {
        if let Some(action) = &self.currently_open {
            return match source {
                InstructionParameterSource::Literal => "Literal value".to_owned(),
                InstructionParameterSource::FromParameter(idx) => {
                    format!("From Parameter {}", action.parameters[*idx].0)
                }
                InstructionParameterSource::FromOutput(step, id) => {
                    let ic = &action.instructions[*step];
                    let instruction = self
                        .engines_list
                        .get_instruction_by_id(&ic.instruction_id)
                        .unwrap();
                    format!("From Step {}: {}", step + 1, instruction.outputs()[id].0)
                }
            };
        }
        source.to_string()
    }
}

impl UiComponent for ActionEditor {
    type Message = ActionEditorMessage;
    type MessageOut = ActionEditorMessageOut;

    fn title(&self) -> Option<&str> {
        Some("Action Editor")
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        let action = self
            .currently_open
            .as_ref()
            .expect("the action editor can't be open with no open action");

        Scrollable::new(
            Container::new(
                column![
                    // Toolbar
                    row![
                        Button::new("Save").on_press(ActionEditorMessage::SaveAction),
                        Button::new("Save as").on_press(ActionEditorMessage::SaveAsAction),
                        Button::new("Close Action").on_press(ActionEditorMessage::CloseAction),
                    ]
                    .spacing(8),
                    // Metadata
                    TextInput::new("Action Name", &action.friendly_name)
                        .on_input(ActionEditorMessage::NameChanged),
                    TextInput::new("Action Group", &action.group)
                        .on_input(ActionEditorMessage::GroupChanged),
                    TextInput::new("Description", &action.description)
                        .on_input(ActionEditorMessage::DescriptionChanged),
                    Rule::horizontal(2),
                    // Parameters
                    Text::new("Action Parameters"),
                    self.ui_parameters(),
                    Button::new("+ New parameter").on_press(ActionEditorMessage::ParameterCreate),
                    Rule::horizontal(2),
                    // Instructions
                    self.ui_steps(),
                    combo_box(
                        &self.add_instruction_combo,
                        "+ Add a step...",
                        None,
                        ActionEditorMessage::StepCreate
                    ),
                    Rule::horizontal(2),
                    // Outputs
                    Text::new("Action Outputs"),
                    self.ui_outputs(),
                    Button::new("+ New output").on_press(ActionEditorMessage::OutputCreate),
                ]
                .spacing(8),
            )
            .padding(16),
        )
        .into()
    }

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        match message {
            ActionEditorMessage::SaveAction => {
                if let Err(e) = self.save_action(false) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            ActionEditorMessage::SaveAsAction => {
                if let Err(e) = self.save_action(true) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            ActionEditorMessage::CloseAction => {
                self.close_action();
                return Some(ActionEditorMessageOut::CloseActionEditor);
            }

            ActionEditorMessage::NameChanged(new_name) => {
                self.currently_open.as_mut().unwrap().friendly_name = new_name;
                self.modified();
            }
            ActionEditorMessage::GroupChanged(new_group) => {
                self.currently_open.as_mut().unwrap().group = new_group;
                self.modified();
            }
            ActionEditorMessage::DescriptionChanged(new_description) => {
                self.currently_open.as_mut().unwrap().description = new_description;
                self.modified();
            }
            ActionEditorMessage::ParameterNameChange(idx, new_name) => {
                let (_, kind) = &self.currently_open.as_mut().unwrap().parameters[idx];
                self.currently_open.as_mut().unwrap().parameters[idx] = (new_name, kind.clone());
                self.modified();
            }
            ActionEditorMessage::ParameterTypeChange(idx, new_type) => {
                let action = self.currently_open.as_mut().unwrap();
                let (name, _) = &action.parameters[idx];
                action.parameters[idx] = (name.clone(), new_type.clone());

                // Updated any instruction parameters to literals
                for instruction_config in action.instructions.iter_mut() {
                    for (_, parameter_source) in instruction_config.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromParameter(p_idx) = parameter_source {
                            if idx == *p_idx {
                                // Change to literal
                                *parameter_source = InstructionParameterSource::Literal;
                            }
                        }
                    }
                }

                // Update any output kinds
                for (_, kind, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromParameter(p_idx) = src {
                        if idx == *p_idx {
                            // Change output type.
                            *kind = new_type.clone();
                        }
                    }
                }

                self.modified();
            }
            ActionEditorMessage::ParameterCreate => {
                self.currently_open
                    .as_mut()
                    .unwrap()
                    .parameters
                    .push((String::new(), ParameterKind::String));
                self.modified();
            }
            ActionEditorMessage::ParameterMoveUp(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                let params = &mut action.parameters;
                let val = params.remove(idx);
                params.insert((idx - 1).max(0), val);

                // Swap idx and (idx - 1)
                for instruction_config in action.instructions.iter_mut() {
                    for (_, src) in instruction_config.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromParameter(p_idx) = src {
                            if *p_idx == idx {
                                *p_idx = idx - 1;
                            } else if *p_idx == (idx - 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromParameter(p_idx) = src {
                        if *p_idx == idx {
                            *p_idx = idx - 1;
                        } else if *p_idx == (idx - 1) {
                            *p_idx = idx;
                        }
                    }
                }

                self.modified();
            }
            ActionEditorMessage::ParameterMoveDown(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                let params = &mut action.parameters;
                let val = params.remove(idx);
                params.insert((idx + 1).min(params.len()), val);

                // Swap idx and (idx + 1)
                for instruction_config in action.instructions.iter_mut() {
                    for (_, src) in instruction_config.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromParameter(p_idx) = src {
                            if *p_idx == idx {
                                *p_idx = idx + 1;
                            } else if *p_idx == (idx + 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromParameter(p_idx) = src {
                        if *p_idx == idx {
                            *p_idx = idx + 1;
                        } else if *p_idx == (idx + 1) {
                            *p_idx = idx;
                        }
                    }
                }

                self.modified();
            }
            ActionEditorMessage::ParameterDelete(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                action.parameters.remove(idx);

                // Reset delete outputs referring to this parameter.
                action.outputs.retain_mut(|(_, _, src)| match src {
                    InstructionParameterSource::FromParameter(p_idx) => *p_idx != idx,
                    _ => true,
                });

                // Renumber outputs referring to parameters afterwards
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromParameter(p_idx) = src {
                        if *p_idx > idx {
                            *p_idx -= 1
                        }
                    }
                }

                // Reset instruction parameters that referred to idx to Literal
                // Renumber all items after idx to (idx - 1).
                for instruction_config in action.instructions.iter_mut() {
                    for src in instruction_config.parameter_sources.values_mut() {
                        if let InstructionParameterSource::FromParameter(p_idx) = src {
                            match (*p_idx).cmp(&idx) {
                                std::cmp::Ordering::Equal => {
                                    *src = InstructionParameterSource::Literal
                                }
                                std::cmp::Ordering::Greater => *p_idx -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                self.modified();
            }

            ActionEditorMessage::StepCreate(instruction) => {
                self.currently_open
                    .as_mut()
                    .unwrap()
                    .instructions
                    .push(InstructionConfiguration::from(instruction.base_instruction));
                self.modified();
                self.add_instruction_combo.unfocus();
            }
            ActionEditorMessage::StepParameterSourceChange(idx, id, new_source) => {
                self.currently_open.as_mut().unwrap().instructions[idx]
                    .parameter_sources
                    .entry(id)
                    .and_modify(|v| {
                        *v = new_source;
                    });
                self.modified();
            }
            ActionEditorMessage::StepParameterValueChange(idx, id, new_value) => {
                self.currently_open.as_mut().unwrap().instructions[idx]
                    .parameter_values
                    .entry(id)
                    .and_modify(|v| match v {
                        ParameterValue::String(v) => *v = new_value,
                        ParameterValue::SpecialType { id: _, value } => *value = new_value,
                        ParameterValue::Integer(v) => *v = new_value.parse().unwrap_or(*v),
                        ParameterValue::Decimal(v) => *v = new_value.parse().unwrap_or(*v),
                    });
                self.modified();
            }
            ActionEditorMessage::StepMoveUp(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                let steps = &mut action.instructions;
                let val = steps.remove(idx);
                steps.insert((idx - 1).max(0), val);

                // Swap idx and (idx - 1)
                for instruction_config in action.instructions.iter_mut() {
                    for (_, src) in instruction_config.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                            if *p_idx == idx {
                                *p_idx = idx - 1;
                            } else if *p_idx == (idx - 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                        if *p_idx == idx {
                            *p_idx = idx - 1;
                        } else if *p_idx == (idx - 1) {
                            *p_idx = idx;
                        }
                    }
                }

                self.modified();
            }
            ActionEditorMessage::StepMoveDown(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                let steps = &mut action.instructions;
                let val = steps.remove(idx);
                steps.insert((idx + 1).min(steps.len()), val);

                // Swap idx and (idx + 1)
                for instruction_config in action.instructions.iter_mut() {
                    for (_, src) in instruction_config.parameter_sources.iter_mut() {
                        if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                            if *p_idx == idx {
                                *p_idx = idx + 1;
                            } else if *p_idx == (idx + 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                        if *p_idx == idx {
                            *p_idx = idx + 1;
                        } else if *p_idx == (idx + 1) {
                            *p_idx = idx;
                        }
                    }
                }

                self.modified();
            }
            ActionEditorMessage::StepDelete(idx) => {
                let action = self.currently_open.as_mut().unwrap();
                action.instructions.remove(idx);

                // Reset delete outputs referring to this step.
                action.outputs.retain_mut(|(_, _, src)| match src {
                    InstructionParameterSource::FromOutput(p_idx, _) => *p_idx != idx,
                    _ => true,
                });

                // Renumber outputs referring to steps afterwards
                for (_, _, src) in action.outputs.iter_mut() {
                    if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                        if *p_idx > idx {
                            *p_idx -= 1
                        }
                    }
                }

                // Reset instruction parameters that referred to idx to Literal
                // Renumber all items after idx to (idx - 1).
                for instruction_config in action.instructions.iter_mut() {
                    for src in instruction_config.parameter_sources.values_mut() {
                        if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                            match (*p_idx).cmp(&idx) {
                                std::cmp::Ordering::Equal => {
                                    *src = InstructionParameterSource::Literal
                                }
                                std::cmp::Ordering::Greater => *p_idx -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                self.modified();
            }

            ActionEditorMessage::OutputNameChange(idx, new_name) => {
                let (_, kind, src) = &self.currently_open.as_mut().unwrap().outputs[idx];
                self.currently_open.as_mut().unwrap().outputs[idx] =
                    (new_name, kind.clone(), src.clone());
                self.modified();
            }
            ActionEditorMessage::OutputSourceChange(idx, new_type, new_source) => {
                let (name, _, _) = &self.currently_open.as_mut().unwrap().outputs[idx];
                self.currently_open.as_mut().unwrap().outputs[idx] =
                    (name.clone(), new_type, new_source);
                self.modified();
            }
            ActionEditorMessage::OutputCreate => {
                if let Some((_, kind, default_output)) = self.output_list.get(0) {
                    self.currently_open.as_mut().unwrap().outputs.push((
                        String::new(),
                        kind.clone(),
                        default_output.clone(),
                    ));
                    self.modified();
                } else {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Warning)
                        .set_buttons(rfd::MessageButtons::Ok)
                        .set_title("No source")
                        .set_description("No source for output data. Add a parameter or a step.")
                        .show();
                }
            }
            ActionEditorMessage::OutputMoveUp(idx) => {
                let outputs = &mut self.currently_open.as_mut().unwrap().outputs;
                let val = outputs.remove(idx);
                outputs.insert((idx - 1).max(0), val);
                self.modified();
            }
            ActionEditorMessage::OutputMoveDown(idx) => {
                let outputs = &mut self.currently_open.as_mut().unwrap().outputs;
                let val = outputs.remove(idx);
                outputs.insert((idx + 1).min(outputs.len()), val);
                self.modified();
            }
            ActionEditorMessage::OutputDelete(idx) => {
                self.currently_open.as_mut().unwrap().outputs.remove(idx);
                self.modified();
            }
        };
        self.update_outputs();
        None
    }
}
