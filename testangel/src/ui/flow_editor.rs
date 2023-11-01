use std::{env, fmt, fs, path::PathBuf, sync::Arc};

use iced::{
    theme,
    widget::{
        column, combo_box, row, Button, Checkbox, Column, ComboBox, Container, Scrollable, Space,
        Text, TextInput,
    },
    Length,
};
use testangel::{
    action_loader::ActionMap,
    types::{Action, ActionConfiguration, ActionParameterSource, AutomationFlow, VersionedFile},
};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum FlowEditorMessage {
    RunFlow,
    WriteFileToDisk(PathBuf, SaveFlowThen),
    SaveFlow(SaveFlowThen),
    SaveAsFlow(SaveFlowThen),
    DoPostSaveActions(SaveFlowThen),
    CloseFlow,

    StepCreate(AvailableAction),
    StepParameterSourceChange(usize, usize, ActionParameterSource),
    StepParameterValueChange(usize, usize, String),
    StepMoveUp(usize),
    StepMoveDown(usize),
    StepDelete(usize),
}

#[derive(Clone, Debug)]
pub enum SaveFlowThen {
    DoNothing,
    Close,
}

#[derive(Clone, Debug)]
pub enum FlowEditorMessageOut {
    RunFlow(AutomationFlow),
}

pub enum SaveOrOpenFlowError {
    IoError(std::io::Error),
    ParsingError(ron::error::SpannedError),
    SerializingError(ron::Error),
    FlowNotVersionCompatible,
    MissingAction(String),
}

impl fmt::Display for SaveOrOpenFlowError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "I/O Error: {e}"),
            Self::ParsingError(e) => write!(f, "Parsing Error: {e}"),
            Self::SerializingError(e) => write!(f, "Serializing error: {e}"),
            Self::FlowNotVersionCompatible => write!(
                f,
                "This flow is not compatible with this version of TestAngel."
            ),
            Self::MissingAction(action_id) => write!(
                f,
                "The flow contains an action ({action_id}) which could not be loaded. This may be because that action refers to an instruction that is not available."
            ),
        }
    }
}

#[derive(Clone, Debug)]
pub struct AvailableAction {
    /// The friendly name of this action.
    friendly_name: String,
    /// The action this is based on.
    base_action: Action,
}

impl fmt::Display for AvailableAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.friendly_name)
    }
}

#[derive(Clone, Debug)]
pub struct ComboActionParameterSource {
    /// The friendly label of this source
    friendly_label: String,
    /// The actual source held by this entry
    source: ActionParameterSource,
}

impl fmt::Display for ComboActionParameterSource {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.friendly_label)
    }
}

#[allow(clippy::from_over_into)]
impl Into<ActionParameterSource> for ComboActionParameterSource {
    fn into(self) -> ActionParameterSource {
        self.source
    }
}

pub struct FlowEditor {
    actions_list: Arc<ActionMap>,
    output_list: Vec<(isize, ParameterKind, ActionParameterSource)>,
    add_action_combo: combo_box::State<AvailableAction>,

    // a vector of steps<parameters<sources>>>
    parameter_source_combo: Vec<Vec<combo_box::State<ComboActionParameterSource>>>,
    currently_open: Option<AutomationFlow>,
    current_path: Option<PathBuf>,
    needs_saving: bool,
}

impl Default for FlowEditor {
    fn default() -> Self {
        Self {
            actions_list: Arc::new(ActionMap::default()),
            output_list: vec![],
            add_action_combo: combo_box::State::new(Vec::new()),
            parameter_source_combo: vec![],
            currently_open: None,
            current_path: None,
            needs_saving: false,
        }
    }
}

impl FlowEditor {
    /// Initialise a new FlowEditor with the provided [`ActionMap`].
    pub(crate) fn new(actions_list: Arc<ActionMap>) -> Self {
        let mut available_actions = vec![];
        for (group_name, actions) in actions_list.get_by_group() {
            for action in actions {
                if action.visible {
                    available_actions.push(AvailableAction {
                        friendly_name: format!("{group_name}: {}", action.friendly_name),
                        base_action: action.clone(),
                    });
                }
            }
        }

        Self {
            actions_list,
            add_action_combo: combo_box::State::new(available_actions),
            ..Default::default()
        }
    }

    pub(crate) fn update_action_map(&mut self, actions_list: Arc<ActionMap>) {
        self.actions_list = actions_list.clone();
        let mut available_actions = vec![];
        let show_anyway = env::var("TA_SHOW_HIDDEN_ACTIONS")
            .unwrap_or("no".to_string())
            .eq_ignore_ascii_case("yes");
        for (group_name, actions) in actions_list.get_by_group() {
            for action in actions {
                if action.visible || show_anyway {
                    available_actions.push(AvailableAction {
                        friendly_name: format!(
                            "{group_name}: {}{}",
                            action.friendly_name,
                            if !action.visible { " (Hidden)" } else { "" }
                        ),
                        base_action: action.clone(),
                    });
                }
            }
        }
        available_actions.sort_by(|a, b| a.friendly_name.cmp(&b.friendly_name));
        self.add_action_combo = combo_box::State::new(available_actions);
    }

    /// Create a new flow and open it
    pub(crate) fn new_flow(&mut self) {
        self.currently_open = Some(AutomationFlow::default());
        self.current_path = None;
        self.needs_saving = true;
    }

    /// Open a flow
    pub(crate) fn open_flow(&mut self, file: PathBuf) -> Result<Vec<usize>, SaveOrOpenFlowError> {
        let data = &fs::read_to_string(&file).map_err(SaveOrOpenFlowError::IoError)?;

        let versioned_file: VersionedFile =
            ron::from_str(data).map_err(SaveOrOpenFlowError::ParsingError)?;
        if versioned_file.version() != 1 {
            return Err(SaveOrOpenFlowError::FlowNotVersionCompatible);
        }

        let mut flow: AutomationFlow =
            ron::from_str(data).map_err(SaveOrOpenFlowError::ParsingError)?;
        if flow.version() != 1 {
            return Err(SaveOrOpenFlowError::FlowNotVersionCompatible);
        }
        let mut steps_reset = vec![];
        for (step, ac) in flow.actions.iter_mut().enumerate() {
            // Check for missing action
            match self.actions_list.get_action_by_id(&ac.action_id) {
                None => return Err(SaveOrOpenFlowError::MissingAction(ac.action_id.clone())),
                Some(action) => {
                    // Check that action parameters haven't changed. If they have, reset values.
                    if ac.update(action) {
                        steps_reset.push(step + 1);
                    }
                }
            }
        }
        self.currently_open = Some(flow);
        self.current_path = Some(file);
        self.needs_saving = false;
        self.update_outputs();
        Ok(steps_reset)
    }

    /// Offer to save if it is needed with default error handling
    fn offer_to_save_default_error_handling(
        &mut self,
        then: SaveFlowThen,
    ) -> iced::Command<super::AppMessage> {
        if self.needs_saving {
            iced::Command::perform(
                rfd::AsyncMessageDialog::new()
                    .set_level(rfd::MessageLevel::Info)
                    .set_title("Do you want to save this flow?")
                    .set_description("This flow has been modified. Do you want to save it?")
                    .set_buttons(rfd::MessageButtons::YesNo)
                    .show(),
                |wants_to_save| {
                    if wants_to_save == rfd::MessageDialogResult::Yes {
                        super::AppMessage::FlowEditor(FlowEditorMessage::SaveFlow(then))
                    } else {
                        super::AppMessage::FlowEditor(FlowEditorMessage::DoPostSaveActions(then))
                    }
                },
            )
        } else {
            self.do_then(then)
        }
    }

    fn do_then(&mut self, then: SaveFlowThen) -> iced::Command<super::AppMessage> {
        match then {
            SaveFlowThen::DoNothing => iced::Command::none(),
            SaveFlowThen::Close => {
                self.close_flow();
                iced::Command::perform(async {}, |_| super::AppMessage::CloseEditor)
            }
        }
    }

    fn write_to_disk(&mut self) -> Result<(), SaveOrOpenFlowError> {
        // Save
        let save_path = self.current_path.as_ref().unwrap();
        let data = ron::to_string(self.currently_open.as_ref().unwrap())
            .map_err(SaveOrOpenFlowError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenFlowError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Save the currently opened flow
    fn save_flow(
        &mut self,
        always_prompt_where: bool,
        then: SaveFlowThen,
    ) -> iced::Command<super::AppMessage> {
        self.needs_saving = false;
        if always_prompt_where || self.current_path.is_none() {
            // Populate save path
            return iced::Command::perform(
                rfd::AsyncFileDialog::new()
                    .add_filter("TestAngel Flows", &["taflow"])
                    .set_title("Save Flow")
                    .set_directory(env::var("TA_FLOW_DIR").unwrap_or(".".to_owned()))
                    .save_file(),
                |f| {
                    if let Some(file) = f {
                        return super::AppMessage::FlowEditor(FlowEditorMessage::WriteFileToDisk(
                            file.path().to_path_buf(),
                            then,
                        ));
                    }
                    super::AppMessage::NoOp
                },
            );
        }

        if let Err(e) = self.write_to_disk() {
            return iced::Command::perform(
                rfd::AsyncMessageDialog::new()
                    .set_title("Failed to save")
                    .set_description(format!("Failed to save file: {e}"))
                    .show(),
                |_| super::AppMessage::NoOp,
            );
        }

        self.do_then(then)
    }

    /// Close the currently opened flow
    fn close_flow(&mut self) {
        self.currently_open = None;
        self.current_path = None;
        self.needs_saving = false;
    }

    /// Hint that the open flow has been modified.
    fn modified(&mut self) {
        self.needs_saving = true;
    }

    fn ui_action_inputs(
        &self,
        step_idx: usize,
        action_config: ActionConfiguration,
        action: Action,
    ) -> iced::Element<'_, FlowEditorMessage> {
        action
            .parameters
            .iter()
            .enumerate()
            .fold(Column::new().spacing(4), |col, (id, (name, kind))| {
                let param_source = &action_config.parameter_sources[&id];
                let param_value = &action_config.parameter_values[&id];

                let literal_input: iced::Element<'_, _> = match param_source {
                    ActionParameterSource::Literal => match kind {
                        ParameterKind::Boolean => {
                            Checkbox::new("Value", param_value.value_bool(), move |new_val| {
                                FlowEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    id,
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
                                FlowEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    id,
                                    i32::MIN.to_string(),
                                )
                            } else {
                                FlowEditorMessage::StepParameterValueChange(step_idx, id, new_val)
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
                                FlowEditorMessage::StepParameterValueChange(
                                    step_idx,
                                    id,
                                    f32::MIN.to_string(),
                                )
                            } else {
                                FlowEditorMessage::StepParameterValueChange(step_idx, id, new_val)
                            }
                        })
                        .width(250)
                        .into(),
                        _ => TextInput::new("Literal value", &param_value.to_string())
                            .on_input(move |new_val| {
                                FlowEditorMessage::StepParameterValueChange(step_idx, id, new_val)
                            })
                            .width(250)
                            .into(),
                    },
                    _ => Space::new(0, 0).into(),
                };

                col.push(
                    row![
                        Text::new(format!("{name} ({kind}) use")),
                        ComboBox::new(
                            &self.parameter_source_combo[step_idx][id],
                            "Source",
                            Some(&ComboActionParameterSource {
                                friendly_label: self.friendly_source_string(param_source),
                                source: param_source.clone(),
                            }),
                            move |src: ComboActionParameterSource| {
                                FlowEditorMessage::StepParameterSourceChange(
                                    step_idx,
                                    id,
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

    fn ui_steps(&self) -> iced::Element<'_, FlowEditorMessage> {
        let flow = self.currently_open.as_ref();
        if flow.is_none() {
            return Text::new("No flow loaded.").into();
        }
        let flow = flow.unwrap();

        flow.actions
            .iter()
            .enumerate()
            .fold(Column::new().spacing(4), |col, (idx, action_config)| {
                let action = self
                    .actions_list
                    .get_action_by_id(&action_config.action_id)
                    .unwrap();
                let mut outputs_text = String::new();
                for (name, kind, _) in &action.outputs {
                    outputs_text.push_str(&format!("{name}: {kind}\n"));
                }
                outputs_text = outputs_text.trim_end().to_string();
                col.push(
                    Container::new(
                        column![
                            Text::new(format!("Step {}: {}", idx + 1, action.friendly_name)),
                            row![
                                Button::new("×").on_press(FlowEditorMessage::StepDelete(idx)),
                                Button::new("ʌ").on_press_maybe(if idx == 0 {
                                    None
                                } else {
                                    Some(FlowEditorMessage::StepMoveUp(idx))
                                }),
                                Button::new("v").on_press_maybe(
                                    if (idx + 1) == flow.actions.len() {
                                        None
                                    } else {
                                        Some(FlowEditorMessage::StepMoveDown(idx))
                                    }
                                ),
                                Text::new(action.description.clone()),
                            ]
                            .spacing(4),
                            Space::with_height(4),
                            Text::new("Inputs").size(18),
                            self.ui_action_inputs(idx, action_config.clone(), action.clone()),
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
            })
            .into()
    }

    /// Update the possible outputs
    fn update_outputs(&mut self) {
        self.output_list.clear();
        self.parameter_source_combo.clear();

        if let Some(flow) = &self.currently_open {
            for (step, action_config) in flow.actions.iter().enumerate() {
                let action = self
                    .actions_list
                    .get_action_by_id(&action_config.action_id)
                    .unwrap();

                // Build parameter source list
                let mut source_opts = vec![];
                for (_, param_kind) in &action.parameters {
                    let mut sources = vec![ComboActionParameterSource {
                        friendly_label: self
                            .friendly_source_string(&ActionParameterSource::Literal),
                        source: ActionParameterSource::Literal,
                    }];

                    for (_step, kind, source) in &self.output_list {
                        if kind == param_kind {
                            sources.push(ComboActionParameterSource {
                                friendly_label: self.friendly_source_string(source),
                                source: source.clone(),
                            });
                        }
                    }

                    source_opts.push(combo_box::State::new(sources));
                }
                self.parameter_source_combo.push(source_opts);

                // Determine possible outputs from this step
                for (id, (_name, kind, _src)) in action.outputs.iter().enumerate() {
                    self.output_list.push((
                        step as isize,
                        *kind,
                        ActionParameterSource::FromOutput(step, id),
                    ));
                }
            }
        }
    }

    /// Convert an [`InstructionParameterSource`] to a friendly String by fetching names of parameters
    /// and results from the currently opened action.
    fn friendly_source_string(&self, source: &ActionParameterSource) -> String {
        if let Some(flow) = &self.currently_open {
            return match source {
                ActionParameterSource::Literal => "Literal value".to_owned(),
                ActionParameterSource::FromOutput(step, id) => {
                    let ac = &flow.actions[*step];
                    let instruction = self.actions_list.get_action_by_id(&ac.action_id).unwrap();
                    format!("Step {} Output: {}", step + 1, instruction.outputs[*id].0)
                }
            };
        }
        source.to_string()
    }
}

impl UiComponent for FlowEditor {
    type Message = FlowEditorMessage;
    type MessageOut = FlowEditorMessageOut;

    fn title(&self) -> Option<&str> {
        Some("Flow Editor")
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Scrollable::new(
            Container::new(
                column![
                    // Toolbar
                    row![
                        Button::new("Run Flow").on_press(FlowEditorMessage::RunFlow),
                        Button::new("Save")
                            .on_press(FlowEditorMessage::SaveFlow(SaveFlowThen::DoNothing)),
                        Button::new("Save as")
                            .on_press(FlowEditorMessage::SaveAsFlow(SaveFlowThen::DoNothing)),
                        Button::new("Close Flow").on_press(FlowEditorMessage::CloseFlow),
                    ]
                    .spacing(8),
                    // Actions
                    self.ui_steps(),
                    ComboBox::new(
                        &self.add_action_combo,
                        "+ Add a step...",
                        None,
                        FlowEditorMessage::StepCreate
                    ),
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
            FlowEditorMessage::RunFlow => {
                return (
                    Some(FlowEditorMessageOut::RunFlow(
                        self.currently_open.as_ref().unwrap().clone(),
                    )),
                    None,
                );
            }
            FlowEditorMessage::WriteFileToDisk(path, then) => {
                self.current_path = Some(path.with_extension("taflow"));

                if let Err(e) = self.write_to_disk() {
                    return (
                        None,
                        Some(iced::Command::perform(
                            rfd::AsyncMessageDialog::new()
                                .set_title("Failed to save")
                                .set_description(format!("Failed to save file: {e}"))
                                .show(),
                            |_| super::AppMessage::NoOp,
                        )),
                    );
                }

                return (None, Some(self.do_then(then)));
            }
            FlowEditorMessage::DoPostSaveActions(then) => {
                return (None, Some(self.do_then(then)));
            }
            FlowEditorMessage::SaveFlow(then) => {
                return (None, Some(self.save_flow(false, then)));
            }
            FlowEditorMessage::SaveAsFlow(then) => {
                return (None, Some(self.save_flow(true, then)));
            }
            FlowEditorMessage::CloseFlow => {
                return (
                    None,
                    Some(self.offer_to_save_default_error_handling(SaveFlowThen::Close)),
                );
            }

            FlowEditorMessage::StepCreate(action) => {
                self.currently_open
                    .as_mut()
                    .unwrap()
                    .actions
                    .push(ActionConfiguration::from(action.base_action));
                self.modified();
                self.add_action_combo.unfocus();
            }
            FlowEditorMessage::StepParameterSourceChange(idx, id, new_source) => {
                self.currently_open.as_mut().unwrap().actions[idx]
                    .parameter_sources
                    .entry(id)
                    .and_modify(|v| {
                        *v = new_source;
                    });
                self.modified();
            }
            FlowEditorMessage::StepParameterValueChange(idx, id, new_value) => {
                self.currently_open.as_mut().unwrap().actions[idx]
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
            FlowEditorMessage::StepMoveUp(idx) => {
                let flow = self.currently_open.as_mut().unwrap();
                let steps = &mut flow.actions;
                let val = steps.remove(idx);
                steps.insert((idx - 1).max(0), val);

                // Swap idx and (idx - 1)
                for action_config in flow.actions.iter_mut() {
                    for (_, src) in action_config.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(p_idx, _) = src {
                            if *p_idx == idx {
                                *p_idx = idx - 1;
                            } else if *p_idx == (idx - 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }

                self.modified();
            }
            FlowEditorMessage::StepMoveDown(idx) => {
                let flow = self.currently_open.as_mut().unwrap();
                let steps = &mut flow.actions;
                let val = steps.remove(idx);
                steps.insert((idx + 1).min(steps.len()), val);

                // Swap idx and (idx + 1)
                for action_config in flow.actions.iter_mut() {
                    for (_, src) in action_config.parameter_sources.iter_mut() {
                        if let ActionParameterSource::FromOutput(p_idx, _) = src {
                            if *p_idx == idx {
                                *p_idx = idx + 1;
                            } else if *p_idx == (idx + 1) {
                                *p_idx = idx;
                            }
                        }
                    }
                }

                self.modified();
            }
            FlowEditorMessage::StepDelete(idx) => {
                let flow = self.currently_open.as_mut().unwrap();
                flow.actions.remove(idx);

                // Reset instruction parameters that referred to idx to Literal
                // Renumber all items after idx to (idx - 1).
                for action_config in flow.actions.iter_mut() {
                    for src in action_config.parameter_sources.values_mut() {
                        if let ActionParameterSource::FromOutput(p_idx, _) = src {
                            match (*p_idx).cmp(&idx) {
                                std::cmp::Ordering::Equal => *src = ActionParameterSource::Literal,
                                std::cmp::Ordering::Greater => *p_idx -= 1,
                                _ => (),
                            }
                        }
                    }
                }

                self.modified();
            }
        };
        self.update_outputs();
        (None, None)
    }
}
