use std::{collections::HashMap, env, fmt, fs, path::PathBuf, sync::Arc};

use iced::{
    theme,
    widget::{
        column, combo_box, row, Button, Checkbox, Column, ComboBox, Container, PickList, Rule,
        Scrollable, Space, Text, TextInput,
    },
    Length,
};
use testangel::{
    ipc::EngineList,
    types::{Action, InstructionConfiguration, InstructionParameterSource, VersionedFile},
};
use testangel_ipc::prelude::{Instruction, ParameterKind, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum ActionEditorMessage {
    RunAction,
    WriteFileToDisk(PathBuf, SaveActionThen),
    SaveAction(SaveActionThen),
    SaveAsAction(SaveActionThen),
    DoPostSaveActions(SaveActionThen),
    CloseAction,

    NameChanged(String),
    GroupChanged(String),
    DescriptionChanged(String),
    AuthorChanged(String),
    VisibleChanged(bool),

    ParameterCreate,
    ParameterNameChange(usize, String),
    ParameterTypeChange(usize, ParameterKind),
    ParameterMoveUp(usize),
    ParameterMoveDown(usize),
    ParameterDelete(usize),

    StepCreate(AvailableInstruction),
    StepChangeComment(usize, String),
    StepChangeRunIf(usize, InstructionParameterSource),
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
pub enum SaveActionThen {
    DoNothing,
    Close,
}

#[derive(Clone, Debug)]
pub enum ActionEditorMessageOut {
    RunAction(Action),
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

#[derive(Clone, Debug)]
pub struct ComboInstructionParameterSource {
    /// The friendly label of this source
    friendly_label: String,
    /// The actual source held by this entry
    source: InstructionParameterSource,
    /// The kind of parameters
    kind: ParameterKind,
}

impl fmt::Display for ComboInstructionParameterSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.friendly_label)
    }
}

#[allow(clippy::from_over_into)]
impl Into<InstructionParameterSource> for ComboInstructionParameterSource {
    fn into(self) -> InstructionParameterSource {
        self.source
    }
}

pub struct ActionEditor {
    engines_list: Arc<EngineList>,
    output_list: Vec<(isize, ParameterKind, InstructionParameterSource)>,
    add_instruction_combo: combo_box::State<AvailableInstruction>,

    run_if_combo: Vec<combo_box::State<ComboInstructionParameterSource>>,
    parameter_source_combo: Vec<HashMap<String, combo_box::State<ComboInstructionParameterSource>>>,
    output_combo: Vec<combo_box::State<ComboInstructionParameterSource>>,
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
            run_if_combo: vec![],
            parameter_source_combo: vec![],
            output_combo: vec![],
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
        // Sort by engine name, then friendly name
        available_instructions.sort_by(|a, b| match a.engine_name.cmp(&b.engine_name) {
            std::cmp::Ordering::Equal => a.friendly_name.cmp(&b.friendly_name),
            ord => ord,
        });

        Self {
            engines_list,
            add_instruction_combo: combo_box::State::new(available_instructions),
            ..Default::default()
        }
    }

    /// Create a new action and open it
    pub(crate) fn new_action(&mut self) {
        self.currently_open = Some(Action::default());
        self.current_path = None;
        self.needs_saving = true;
    }

    /// Open an action
    pub(crate) fn open_action(&mut self, file: PathBuf) -> Result<(), SaveOrOpenActionError> {
        let data = &fs::read_to_string(&file).map_err(SaveOrOpenActionError::IoError)?;

        let versioned_file: VersionedFile =
            ron::from_str(data).map_err(SaveOrOpenActionError::ParsingError)?;
        if versioned_file.version() != 1 {
            return Err(SaveOrOpenActionError::ActionNotVersionCompatible);
        }

        let action: Action = ron::from_str(data).map_err(SaveOrOpenActionError::ParsingError)?;
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

    /// Offer to save if it is needed with default error handling
    fn offer_to_save_default_error_handling(
        &mut self,
        then: SaveActionThen,
    ) -> iced::Command<super::AppMessage> {
        if self.needs_saving {
            iced::Command::perform(
                rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Info)
                    .set_title("Do you want to save this action?")
                    .set_description("This action has been modified. Do you want to save it?")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show(),
                |wants_to_save| {
                    if wants_to_save == rfd::MessageDialogResult::Yes {
                        super::AppMessage::ActionEditor(ActionEditorMessage::SaveAction(then))
                    } else {
                        super::AppMessage::ActionEditor(ActionEditorMessage::DoPostSaveActions(
                            then,
                        ))
                    }
                },
            )
        } else {
            self.do_then(then)
        }
    }

    fn write_to_disk(&mut self) -> Result<(), SaveOrOpenActionError> {
        let save_path = self
            .current_path
            .as_ref()
            .unwrap()
            .with_extension("taaction");
        let data = ron::to_string(self.currently_open.as_ref().unwrap())
            .map_err(SaveOrOpenActionError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenActionError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Save the currently opened action
    fn save_action(
        &mut self,
        always_prompt_where: bool,
        then: SaveActionThen,
    ) -> iced::Command<super::AppMessage> {
        self.needs_saving = false;

        if always_prompt_where && self.current_path.is_some() {
            // If this is a true 'save as' scenario, we need to generate a new UUID
            self.currently_open.as_mut().unwrap().new_id();
        }

        if always_prompt_where || self.current_path.is_none() {
            // Populate save path
            return iced::Command::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("TestAngel Actions", &["taaction"])
                    .set_title("Save Action")
                    .set_directory(env::var("TA_ACTION_DIR").unwrap_or("./actions".to_owned()))
                    .save_file(),
                |f| {
                    if let Some(file) = f {
                        return super::AppMessage::ActionEditor(
                            ActionEditorMessage::WriteFileToDisk(file.path().to_path_buf(), then),
                        );
                    }
                    super::AppMessage::NoOp
                },
            );
        }

        if let Err(e) = self.write_to_disk() {
            return iced::Command::perform(
                rfd::AsyncMessageDialog::new()
                    .set_title("Failed to save")
                    .set_description(&format!("Failed to save file: {e}"))
                    .show(),
                |_| super::AppMessage::NoOp,
            );
        }

        self.do_then(then)
    }

    fn do_then(&mut self, then: SaveActionThen) -> iced::Command<super::AppMessage> {
        match then {
            SaveActionThen::DoNothing => iced::Command::none(),
            SaveActionThen::Close => {
                self.close_action();
                iced::Command::perform(async {}, |_| super::AppMessage::CloseEditor)
            }
        }
    }

    /// Close the currently opened action
    fn close_action(&mut self) {
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
                                ParameterKind::Boolean,
                            ][..],
                            Some(*kind),
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

                let literal_input_id = id.clone();
                let literal_input: iced::Element<'_, _> = match param_source {
                    InstructionParameterSource::Literal => match kind {
                        ParameterKind::Boolean => {
                            Checkbox::new("Value", param_value.value_bool(), move |new_val| {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    if new_val { "yes" } else { "no" }.to_string(),
                                )
                            })
                            .into()
                        }
                        ParameterKind::Integer => TextInput::new(
                            "Literal number",
                            &if param_value.value_i32() == i32::MIN {
                                String::new()
                            } else {
                                param_value.to_string()
                            },
                        )
                        .on_input(move |new_val| {
                            if new_val.trim().is_empty() {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    i32::MIN.to_string(),
                                )
                            } else {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    new_val,
                                )
                            }
                        })
                        .width(250)
                        .into(),
                        ParameterKind::Decimal => TextInput::new(
                            "Literal decimal",
                            &if param_value.value_f32() == f32::MIN {
                                String::new()
                            } else {
                                param_value.to_string()
                            },
                        )
                        .on_input(move |new_val| {
                            if new_val.trim().is_empty() {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    f32::MIN.to_string(),
                                )
                            } else {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    new_val,
                                )
                            }
                        })
                        .width(250)
                        .into(),
                        _ => TextInput::new("Literal value", &param_value.to_string())
                            .on_input(move |new_val| {
                                ActionEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    literal_input_id.clone(),
                                    new_val,
                                )
                            })
                            .width(250)
                            .into(),
                    },
                    _ => Space::new(0, 0).into(),
                };

                let id = id.clone();
                col.push(
                    row![
                        Text::new(format!("{name} ({kind}) use")),
                        ComboBox::new(
                            &self.parameter_source_combo[step_idx][&id.clone()],
                            "Source",
                            Some(&ComboInstructionParameterSource {
                                friendly_label: self.friendly_source_string(param_source),
                                kind: *kind,
                                source: param_source.clone(),
                            }),
                            move |src: ComboInstructionParameterSource| {
                                ActionEditorMessage::StepParameterSourceChange(
                                    step_idx,
                                    id.clone(),
                                    src.into(),
                                )
                            }
                        ),
                        literal_input,
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
                    outputs_text = outputs_text.trim_end().to_string();
                    col.push(
                        Container::new(
                            column![
                                Text::new(format!(
                                    "Step {}: {}",
                                    idx + 1,
                                    instruction.friendly_name()
                                ))
                                .size(20),
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
                                TextInput::new("Comment", &instruction_config.comment).on_input(
                                    move |new_comment| ActionEditorMessage::StepChangeComment(
                                        idx,
                                        new_comment
                                    )
                                ),
                                ComboBox::new(
                                    &self.run_if_combo[idx],
                                    "Run if...",
                                    Some(&ComboInstructionParameterSource {
                                        friendly_label: if instruction_config.run_if
                                            == InstructionParameterSource::Literal
                                        {
                                            "Always run".to_string()
                                        } else {
                                            self.friendly_source_string(&instruction_config.run_if)
                                        },
                                        kind: ParameterKind::Boolean,
                                        source: instruction_config.run_if.clone(),
                                    }),
                                    move |src: ComboInstructionParameterSource| {
                                        ActionEditorMessage::StepChangeRunIf(idx, src.into())
                                    }
                                ),
                                Space::with_height(4),
                                Text::new("Inputs").size(18),
                                self.ui_instruction_inputs(
                                    idx,
                                    instruction_config.clone(),
                                    instruction.clone()
                                ),
                                Space::with_height(4),
                                Text::new("Outputs").size(18),
                                Text::new(outputs_text),
                            ]
                            .spacing(4),
                        )
                        .padding(8)
                        .width(Length::Fill)
                        .style(theme::Container::Box),
                    )
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
                            Text::new(format!("({kind}) from")),
                            ComboBox::new(
                                &self.output_combo[idx],
                                "Output Source",
                                Some(&ComboInstructionParameterSource {
                                    friendly_label: self.friendly_source_string(source),
                                    kind: ParameterKind::String, // ! This doesn't matter as it isn't used for matching.
                                    source: source.clone(),
                                }),
                                move |src: ComboInstructionParameterSource| {
                                    ActionEditorMessage::OutputSourceChange(
                                        idx,
                                        src.kind,
                                        src.into(),
                                    )
                                }
                            )
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
        self.parameter_source_combo.clear();
        self.run_if_combo.clear();
        self.output_combo.clear();

        if let Some(action) = &self.currently_open {
            for (index, (_name, kind)) in action.parameters.iter().enumerate() {
                self.output_list.push((
                    -1,
                    *kind,
                    InstructionParameterSource::FromParameter(index),
                ));
            }

            for (step, instruction_config) in action.instructions.iter().enumerate() {
                let instruction = self
                    .engines_list
                    .get_instruction_by_id(&instruction_config.instruction_id)
                    .unwrap();

                // Build Run If source list
                let run_if_options = self.output_list.iter().fold(
                    vec![ComboInstructionParameterSource {
                        friendly_label: "Always run".to_string(),
                        kind: ParameterKind::Boolean,
                        source: InstructionParameterSource::Literal,
                    }],
                    |mut list, (_step, kind, source)| {
                        if *kind == ParameterKind::Boolean {
                            list.push(ComboInstructionParameterSource {
                                friendly_label: self.friendly_source_string(source),
                                kind: *kind,
                                source: source.clone(),
                            });
                        }
                        list
                    },
                );
                self.run_if_combo
                    .push(combo_box::State::new(run_if_options));

                // Build parameter source list
                let mut source_opts = HashMap::new();
                for (param_idx, (_, param_kind)) in instruction.parameters() {
                    let mut sources = vec![ComboInstructionParameterSource {
                        friendly_label: self
                            .friendly_source_string(&InstructionParameterSource::Literal),
                        kind: *param_kind,
                        source: InstructionParameterSource::Literal,
                    }];

                    for (_step, kind, source) in &self.output_list {
                        if kind == param_kind {
                            sources.push(ComboInstructionParameterSource {
                                friendly_label: self.friendly_source_string(source),
                                kind: *kind,
                                source: source.clone(),
                            });
                        }
                    }

                    source_opts.insert(param_idx.clone(), combo_box::State::new(sources));
                }
                self.parameter_source_combo.push(source_opts);

                // Determine possible outputs from this step
                for (id, (_name, kind)) in instruction.outputs() {
                    self.output_list.push((
                        step as isize,
                        *kind,
                        InstructionParameterSource::FromOutput(step, id.clone()),
                    ));
                }
            }

            // Build output source list
            let output_options: Vec<_> = self
                .output_list
                .iter()
                .map(|(_step, kind, source)| ComboInstructionParameterSource {
                    friendly_label: self.friendly_source_string(source),
                    kind: *kind,
                    source: source.clone(),
                })
                .collect();
            for _ in 0..action.outputs.len() {
                self.output_combo
                    .push(combo_box::State::new(output_options.clone()));
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
                    format!("Parameter {}", action.parameters[*idx].0)
                }
                InstructionParameterSource::FromOutput(step, id) => {
                    let ic = &action.instructions[*step];
                    let instruction = self
                        .engines_list
                        .get_instruction_by_id(&ic.instruction_id)
                        .unwrap();
                    format!("Step {} Output: {}", step + 1, instruction.outputs()[id].0)
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
        let action = self.currently_open.as_ref();
        if action.is_none() {
            return Text::new("No action loaded.").into();
        }
        let action = action.unwrap();

        Scrollable::new(
            Container::new(
                column![
                    // Toolbar
                    row![
                        Button::new("Run Action").on_press(ActionEditorMessage::RunAction),
                        Button::new("Save")
                            .on_press(ActionEditorMessage::SaveAction(SaveActionThen::DoNothing)),
                        Button::new("Save as")
                            .on_press(ActionEditorMessage::SaveAsAction(SaveActionThen::DoNothing)),
                        Button::new("Close Action").on_press(ActionEditorMessage::CloseAction),
                    ]
                    .spacing(8),
                    // Metadata
                    TextInput::new("Action Name", &action.friendly_name)
                        .on_input(ActionEditorMessage::NameChanged),
                    TextInput::new("Action Group", &action.group)
                        .on_input(ActionEditorMessage::GroupChanged),
                    TextInput::new("Author", &action.author)
                        .on_input(ActionEditorMessage::AuthorChanged),
                    TextInput::new("Description", &action.description)
                        .on_input(ActionEditorMessage::DescriptionChanged),
                    Checkbox::new(
                        "Visible in Flow Editor",
                        action.visible,
                        ActionEditorMessage::VisibleChanged
                    ),
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

    fn update(
        &mut self,
        message: Self::Message,
    ) -> (
        Option<Self::MessageOut>,
        Option<iced::Command<super::AppMessage>>,
    ) {
        match message {
            ActionEditorMessage::WriteFileToDisk(path, then) => {
                self.current_path = Some(path.with_extension("taaction"));

                if let Err(e) = self.write_to_disk() {
                    return (
                        None,
                        Some(iced::Command::perform(
                            rfd::AsyncMessageDialog::new()
                                .set_title("Failed to save")
                                .set_description(&format!("Failed to save file: {e}"))
                                .show(),
                            |_| super::AppMessage::NoOp,
                        )),
                    );
                }

                return (None, Some(self.do_then(then)));
            }
            ActionEditorMessage::DoPostSaveActions(then) => {
                return (None, Some(self.do_then(then)));
            }
            ActionEditorMessage::SaveAction(then) => {
                return (None, Some(self.save_action(false, then)));
            }
            ActionEditorMessage::SaveAsAction(then) => {
                return (None, Some(self.save_action(true, then)));
            }
            ActionEditorMessage::CloseAction => {
                return (
                    None,
                    Some(self.offer_to_save_default_error_handling(SaveActionThen::Close)),
                );
            }

            ActionEditorMessage::RunAction => {
                return (
                    Some(ActionEditorMessageOut::RunAction(
                        self.currently_open.clone().unwrap(),
                    )),
                    None,
                );
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
            ActionEditorMessage::AuthorChanged(new_author) => {
                self.currently_open.as_mut().unwrap().author = new_author;
                self.modified();
            }
            ActionEditorMessage::VisibleChanged(now_visible) => {
                self.currently_open.as_mut().unwrap().visible = now_visible;
                self.modified();
            }
            ActionEditorMessage::ParameterNameChange(idx, new_name) => {
                let (_, kind) = &self.currently_open.as_mut().unwrap().parameters[idx];
                self.currently_open.as_mut().unwrap().parameters[idx] = (new_name, *kind);
                self.modified();
            }
            ActionEditorMessage::ParameterTypeChange(idx, new_type) => {
                let action = self.currently_open.as_mut().unwrap();
                let (name, _) = &action.parameters[idx];
                action.parameters[idx] = (name.clone(), new_type);

                for instruction_config in action.instructions.iter_mut() {
                    // Update any run ifs to literals
                    if let InstructionParameterSource::FromParameter(p_idx) =
                        instruction_config.run_if
                    {
                        if idx == p_idx {
                            // Change to literal
                            instruction_config.run_if = InstructionParameterSource::Literal;
                        }
                    }

                    // Updated any instruction parameters to literals
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
                            *kind = new_type;
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
                    if let InstructionParameterSource::FromParameter(p_idx) =
                        &mut instruction_config.run_if
                    {
                        if *p_idx == idx {
                            *p_idx = idx - 1;
                        } else if *p_idx == (idx - 1) {
                            *p_idx = idx;
                        }
                    }

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
                    if let InstructionParameterSource::FromParameter(p_idx) =
                        &mut instruction_config.run_if
                    {
                        if *p_idx == idx {
                            *p_idx = idx + 1;
                        } else if *p_idx == (idx + 1) {
                            *p_idx = idx;
                        }
                    }

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
                    if let InstructionParameterSource::FromParameter(p_idx) =
                        &mut instruction_config.run_if
                    {
                        match (*p_idx).cmp(&idx) {
                            std::cmp::Ordering::Equal => {
                                instruction_config.run_if = InstructionParameterSource::Literal
                            }
                            std::cmp::Ordering::Greater => *p_idx -= 1,
                            _ => (),
                        }
                    }

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
            ActionEditorMessage::StepChangeComment(step, new_comment) => {
                self.currently_open.as_mut().unwrap().instructions[step].comment = new_comment;
                self.modified();
            }
            ActionEditorMessage::StepChangeRunIf(step, new_run_if) => {
                self.currently_open.as_mut().unwrap().instructions[step].run_if = new_run_if;
                self.modified();
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
                        ParameterValue::Integer(v) => *v = new_value.parse().unwrap_or(*v),
                        ParameterValue::Decimal(v) => *v = new_value.parse().unwrap_or(*v),
                        ParameterValue::Boolean(v) => *v = new_value.to_ascii_lowercase() == "yes",
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
                    if let InstructionParameterSource::FromOutput(p_idx, _) =
                        &mut instruction_config.run_if
                    {
                        if *p_idx == idx {
                            *p_idx = idx - 1;
                        } else if *p_idx == (idx - 1) {
                            *p_idx = idx;
                        }
                    }

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
                    if let InstructionParameterSource::FromOutput(p_idx, _) =
                        &mut instruction_config.run_if
                    {
                        if *p_idx == idx {
                            *p_idx = idx + 1;
                        } else if *p_idx == (idx + 1) {
                            *p_idx = idx;
                        }
                    }

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

                for instruction_config in action.instructions.iter_mut() {
                    // Reset instruction parameters that referred to idx to Literal
                    if let InstructionParameterSource::FromOutput(p_idx, _) =
                        &mut instruction_config.run_if
                    {
                        match (*p_idx).cmp(&idx) {
                            std::cmp::Ordering::Equal => {
                                instruction_config.run_if = InstructionParameterSource::Literal
                            }
                            std::cmp::Ordering::Greater => *p_idx -= 1,
                            _ => (),
                        }
                    }

                    // Renumber all items after idx to (idx - 1).
                    for src in instruction_config.parameter_sources.values_mut() {
                        if let InstructionParameterSource::FromOutput(p_idx, _) = src {
                            match (*p_idx).cmp(&idx) {
                                std::cmp::Ordering::Equal => {
                                    log::debug!("Updated runif");
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
                self.currently_open.as_mut().unwrap().outputs[idx] = (new_name, *kind, src.clone());
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
                        *kind,
                        default_output.clone(),
                    ));
                    self.modified();
                } else {
                    return (
                        None,
                        Some(iced::Command::perform(
                            rfd::AsyncMessageDialog::new()
                                .set_level(rfd::MessageLevel::Warning)
                                .set_buttons(rfd::MessageButtons::Ok)
                                .set_title("No source")
                                .set_description(
                                    "No source for output data. Add a parameter or a step.",
                                )
                                .show(),
                            |_| super::AppMessage::NoOp,
                        )),
                    );
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
        (None, None)
    }
}
