use std::{env, fmt, fs, path::PathBuf, sync::Arc};

use iced::widget::{
    column, row, Button, Column, Container, PickList, Rule, Scrollable, Text, TextInput,
};
use iced_aw::Card;
use testangel::{
    ipc::EngineList,
    types::{Action, InstructionConfiguration, InstructionParameterSource},
};
use testangel_ipc::prelude::{Instruction, ParameterKind};

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

    StepParameterSourceChange(usize, String, StepParameterSourceOptions),
    StepMoveUp(usize),
    StepMoveDown(usize),
    StepDelete(usize),

    OutputCreate,
    OutputNameChange(usize, String),
    OutputTypeChange(usize, ParameterKind),
    OutputMoveUp(usize),
    OutputMoveDown(usize),
    OutputDelete(usize),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum StepParameterSourceOptions {
    Literal,
    FromOutput,
    FromParameter,
}

impl ToString for StepParameterSourceOptions {
    fn to_string(&self) -> String {
        match self {
            Self::Literal => "Literal",
            Self::FromOutput => "From the output of a previous step",
            Self::FromParameter => "From an action parameter",
        }
        .to_owned()
    }
}

impl From<InstructionParameterSource> for StepParameterSourceOptions {
    fn from(value: InstructionParameterSource) -> Self {
        match value {
            InstructionParameterSource::Literal => StepParameterSourceOptions::Literal,
            InstructionParameterSource::FromOutput(_, _) => StepParameterSourceOptions::FromOutput,
            InstructionParameterSource::FromParameter(_) => {
                StepParameterSourceOptions::FromParameter
            }
        }
    }
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

#[derive(Default)]
pub struct ActionEditor {
    engines_list: Arc<EngineList>,

    currently_open: Option<Action>,
    current_path: Option<PathBuf>,
    needs_saving: bool,
}

impl ActionEditor {
    /// Initialise a new ActionEditor with the provided [`ActionMap`].
    pub(crate) fn new(engines_list: Arc<EngineList>) -> Self {
        Self {
            engines_list,
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
        let action: Action = ron::from_str(
            &fs::read_to_string(&file).map_err(|e| SaveOrOpenActionError::IoError(e))?,
        )
        .map_err(|e| SaveOrOpenActionError::ParsingError(e))?;
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
        Ok(())
    }

    /// Offer to save if it is needed
    fn offer_to_save(&mut self) -> Result<(), SaveOrOpenActionError> {
        if self.needs_saving {
            if rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Do you want to save this action?")
                .set_description("This action has been modified. Do you want to save it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
            {
                self.save_action(false)?;
            }
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
            .map_err(|e| SaveOrOpenActionError::SerializingError(e))?;
        fs::write(save_path, data).map_err(|e| SaveOrOpenActionError::IoError(e))?;
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
        idx: usize,
        instruction_config: &InstructionConfiguration,
        instruction: Instruction,
    ) -> iced::Element<'_, ActionEditorMessage> {
        instruction
            .parameter_order()
            .iter()
            .fold(Column::new().spacing(4), |col, id| {
                let (name, kind) = &instruction.parameters()[id];
                let param_source = &instruction_config.parameter_sources[id];
                let param_value = &instruction_config.parameter_values[id];
                let param_source_val = param_source.clone().into();

                col.push(
                    row![
                        Text::new(format!("{name} ({kind})")),
                        PickList::new(
                            &[
                                StepParameterSourceOptions::Literal,
                                StepParameterSourceOptions::FromOutput,
                                StepParameterSourceOptions::FromParameter,
                            ][..],
                            Some(param_source_val),
                            move |k| ActionEditorMessage::StepParameterSourceChange(
                                idx,
                                id.clone(),
                                k.clone()
                            )
                        )
                        .placeholder("Parameter Kind"),
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
                    for (_id, (name, kind)) in instruction.outputs() {
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
                                instruction_config,
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
        let mut col = Column::new().spacing(4);
        let action = self.currently_open.as_ref().unwrap();

        for (idx, (name, kind, source)) in action.outputs.iter().enumerate() {
            col = col.push(
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
                    PickList::new(
                        &[
                            ParameterKind::String,
                            ParameterKind::Integer,
                            ParameterKind::Decimal,
                        ][..],
                        Some(kind.clone()),
                        move |k| ActionEditorMessage::OutputTypeChange(idx, k)
                    )
                    .placeholder("Output Kind"),
                    // TODO Output source
                ]
                .spacing(4)
                .align_items(iced::Alignment::Center),
            );
        }

        col.into()
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
                    Button::new("+ Add step"),
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
                let (name, _) = &self.currently_open.as_mut().unwrap().parameters[idx];
                self.currently_open.as_mut().unwrap().parameters[idx] = (name.clone(), new_type);
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
                let params = &mut self.currently_open.as_mut().unwrap().parameters;
                let val = params.remove(idx);
                params.insert((idx - 1).max(0), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::ParameterMoveDown(idx) => {
                let params = &mut self.currently_open.as_mut().unwrap().parameters;
                let val = params.remove(idx);
                params.insert((idx + 1).min(params.len()), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::ParameterDelete(idx) => {
                self.currently_open.as_mut().unwrap().parameters.remove(idx);
                // TODO Renumber
                self.modified();
            }

            ActionEditorMessage::StepParameterSourceChange(idx, id, new_source) => {
                self.currently_open.as_mut().unwrap().instructions[idx]
                    .parameter_sources
                    .entry(id)
                    .and_modify(|v| {
                        *v = match new_source {
                            StepParameterSourceOptions::Literal => {
                                InstructionParameterSource::Literal
                            }
                            StepParameterSourceOptions::FromParameter => {
                                InstructionParameterSource::FromParameter(0)
                            }
                            StepParameterSourceOptions::FromOutput => {
                                InstructionParameterSource::FromOutput(0, String::new())
                            }
                        };
                    });
                self.modified();
            }
            ActionEditorMessage::StepMoveUp(idx) => {
                let steps = &mut self.currently_open.as_mut().unwrap().instructions;
                let val = steps.remove(idx);
                steps.insert((idx - 1).max(0), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::StepMoveDown(idx) => {
                let steps = &mut self.currently_open.as_mut().unwrap().instructions;
                let val = steps.remove(idx);
                steps.insert((idx + 1).min(steps.len()), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::StepDelete(idx) => {
                self.currently_open
                    .as_mut()
                    .unwrap()
                    .instructions
                    .remove(idx);
                // TODO Renumber
                self.modified();
            }

            ActionEditorMessage::OutputNameChange(idx, new_name) => {
                let (_, kind, src) = &self.currently_open.as_mut().unwrap().outputs[idx];
                self.currently_open.as_mut().unwrap().outputs[idx] =
                    (new_name, kind.clone(), src.clone());
                self.modified();
            }
            ActionEditorMessage::OutputTypeChange(idx, new_type) => {
                let (name, _, src) = &self.currently_open.as_mut().unwrap().outputs[idx];
                self.currently_open.as_mut().unwrap().outputs[idx] =
                    (name.clone(), new_type, src.clone());
                self.modified();
            }
            ActionEditorMessage::OutputCreate => {
                self.currently_open.as_mut().unwrap().outputs.push((
                    String::new(),
                    ParameterKind::String,
                    InstructionParameterSource::Literal,
                ));
                self.modified();
            }
            ActionEditorMessage::OutputMoveUp(idx) => {
                let outputs = &mut self.currently_open.as_mut().unwrap().outputs;
                let val = outputs.remove(idx);
                outputs.insert((idx - 1).max(0), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::OutputMoveDown(idx) => {
                let outputs = &mut self.currently_open.as_mut().unwrap().outputs;
                let val = outputs.remove(idx);
                outputs.insert((idx + 1).min(outputs.len()), val);
                // TODO Renumber
                self.modified();
            }
            ActionEditorMessage::OutputDelete(idx) => {
                self.currently_open.as_mut().unwrap().outputs.remove(idx);
                // TODO Renumber
                self.modified();
            }
        };
        None
    }
}
