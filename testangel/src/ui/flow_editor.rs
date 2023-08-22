use std::{env, fmt, fs, path::PathBuf, sync::Arc};

use iced::widget::{
    column, combo_box, row, scrollable, Button, Column, Container, Row, Scrollable, Space, Text,
    TextInput,
};
use iced_aw::Card;
use testangel::{
    action_loader::ActionMap,
    types::{Action, ActionConfiguration, ActionParameterSource, AutomationFlow},
};
use testangel_ipc::prelude::{ParameterKind, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum FlowEditorMessage {
    SaveFlow,
    SaveAsFlow,
    CloseFlow,

    StepCreate(AvailableAction),
    StepParameterSourceChange(usize, usize, ActionParameterSource),
    StepParameterValueChange(usize, usize, String),
    StepMoveUp(usize),
    StepMoveDown(usize),
    StepDelete(usize),
}

#[derive(Clone, Debug)]
pub enum FlowEditorMessageOut {
    CloseFlowEditor,
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

pub struct FlowEditor {
    actions_list: Arc<ActionMap>,
    output_list: Vec<(isize, ParameterKind, ActionParameterSource)>,
    add_action_combo: combo_box::State<AvailableAction>,

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
                available_actions.push(AvailableAction {
                    friendly_name: format!("{group_name}: {}", action.friendly_name),
                    base_action: action.clone(),
                });
            }
        }

        Self {
            actions_list,
            add_action_combo: combo_box::State::new(available_actions),
            ..Default::default()
        }
    }

    pub(crate) fn update_action_map(&mut self, actions_list: Arc<ActionMap>) {
        self.actions_list = actions_list;
    }

    /// Create a new flow and open it
    pub(crate) fn new_flow(&mut self) {
        self.offer_to_save_default_error_handling();
        self.currently_open = Some(AutomationFlow::default());
        self.current_path = None;
        self.needs_saving = true;
    }

    /// Open a flow
    pub(crate) fn open_flow(&mut self, file: PathBuf) -> Result<(), SaveOrOpenFlowError> {
        self.offer_to_save_default_error_handling();
        let flow: AutomationFlow =
            ron::from_str(&fs::read_to_string(&file).map_err(SaveOrOpenFlowError::IoError)?)
                .map_err(SaveOrOpenFlowError::ParsingError)?;
        if flow.version() != 1 {
            return Err(SaveOrOpenFlowError::FlowNotVersionCompatible);
        }
        for ac in &flow.actions {
            if self.actions_list.get_action_by_id(&ac.action_id).is_none() {
                return Err(SaveOrOpenFlowError::MissingAction(ac.action_id.clone()));
            }
        }
        self.currently_open = Some(flow);
        self.current_path = Some(file);
        self.needs_saving = false;
        self.update_outputs();
        Ok(())
    }

    /// Offer to save if it is needed
    fn offer_to_save(&mut self) -> Result<(), SaveOrOpenFlowError> {
        if self.needs_saving
            && rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Info)
                .set_title("Do you want to save this flow?")
                .set_description("This flow has been modified. Do you want to save it?")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show()
        {
            self.save_flow(false)?;
        }
        Ok(())
    }

    /// Offer to save if it is needed with default error handling
    fn offer_to_save_default_error_handling(&mut self) {
        if let Err(e) = self.offer_to_save() {
            rfd::MessageDialog::new()
                .set_level(rfd::MessageLevel::Error)
                .set_title("Failed to save flow")
                .set_description(&format!("{e}"))
                .set_buttons(rfd::MessageButtons::Ok)
                .show();
        }
    }

    /// Save the currently opened flow
    fn save_flow(&mut self, always_prompt_where: bool) -> Result<(), SaveOrOpenFlowError> {
        self.needs_saving = false;
        if always_prompt_where || self.current_path.is_none() {
            // Populate save path
            if let Some(file) = rfd::FileDialog::new()
                .add_filter("TestAngel Flows", &["taflow"])
                .set_title("Save Flow")
                .set_directory(env::current_dir().expect("Failed to read current directory"))
                .save_file()
            {
                self.current_path = Some(file.with_extension("taflow"));
            } else {
                return Ok(());
            }
        }

        // Save
        let save_path = self.current_path.as_ref().unwrap();
        let data = ron::to_string(self.currently_open.as_ref().unwrap())
            .map_err(SaveOrOpenFlowError::SerializingError)?;
        fs::write(save_path, data).map_err(SaveOrOpenFlowError::IoError)?;
        self.needs_saving = false;
        Ok(())
    }

    /// Close the currently opened flow
    fn close_flow(&mut self) {
        self.offer_to_save_default_error_handling();
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
                                FlowEditorMessage::StepParameterSourceChange(
                                    step_idx,
                                    id,
                                    ActionParameterSource::Literal,
                                ),
                            )),
                        |row, (_, _, src)| {
                            row.push(
                                Button::new(Text::new(format!(
                                    "Set {}",
                                    self.friendly_source_string(src)
                                )))
                                .on_press(
                                    FlowEditorMessage::StepParameterSourceChange(
                                        step_idx,
                                        id,
                                        src.clone(),
                                    ),
                                ),
                            )
                        },
                    );

                let literal_input: iced::Element<'_, _> = match param_source {
                    ActionParameterSource::Literal => {
                        TextInput::new("Literal value", &param_value.to_string())
                            .on_input(move |new_val| {
                                FlowEditorMessage::StepParameterValueChange(step_idx, id, new_val)
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

    fn ui_steps(&self) -> iced::Element<'_, FlowEditorMessage> {
        let flow = self.currently_open.as_ref().unwrap();

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
                    outputs_text.push_str(&format!("{name}: {kind}"));
                }
                col.push(Card::new(
                    Text::new(format!("Step {}: {}", idx + 1, action.friendly_name)),
                    column![
                        row![
                            Button::new("×").on_press(FlowEditorMessage::StepDelete(idx)),
                            Button::new("ʌ").on_press_maybe(if idx == 0 {
                                None
                            } else {
                                Some(FlowEditorMessage::StepMoveUp(idx))
                            }),
                            Button::new("v").on_press_maybe(if (idx + 1) == flow.actions.len() {
                                None
                            } else {
                                Some(FlowEditorMessage::StepMoveDown(idx))
                            }),
                            Text::new(action.description.clone()),
                        ]
                        .spacing(4),
                        Text::new("Inputs"),
                        self.ui_action_inputs(idx, action_config.clone(), action.clone()),
                        Text::new("Outputs"),
                        Text::new(outputs_text),
                    ]
                    .spacing(4),
                ))
            })
            .into()
    }

    /// Update the possible outputs
    fn update_outputs(&mut self) {
        self.output_list.clear();

        if let Some(flow) = &self.currently_open {
            for (step, action_config) in flow.actions.iter().enumerate() {
                let instruction = self
                    .actions_list
                    .get_action_by_id(&action_config.action_id)
                    .unwrap();
                for (id, (_name, kind, _src)) in instruction.outputs.iter().enumerate() {
                    self.output_list.push((
                        step as isize,
                        kind.clone(),
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
                    format!("From Step {}: {}", step + 1, instruction.outputs[*id].0)
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
                        Button::new("Save").on_press(FlowEditorMessage::SaveFlow),
                        Button::new("Save as").on_press(FlowEditorMessage::SaveAsFlow),
                        Button::new("Close Flow").on_press(FlowEditorMessage::CloseFlow),
                    ]
                    .spacing(8),
                    // Actions
                    self.ui_steps(),
                    combo_box(
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

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        match message {
            FlowEditorMessage::SaveFlow => {
                if let Err(e) = self.save_flow(false) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            FlowEditorMessage::SaveAsFlow => {
                if let Err(e) = self.save_flow(true) {
                    rfd::MessageDialog::new()
                        .set_level(rfd::MessageLevel::Error)
                        .set_title("Failed to save action")
                        .set_description(&format!("{e}"))
                        .set_buttons(rfd::MessageButtons::Ok)
                        .show();
                }
            }
            FlowEditorMessage::CloseFlow => {
                self.close_flow();
                return Some(FlowEditorMessageOut::CloseFlowEditor);
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
                        ParameterValue::SpecialType { id: _, value } => *value = new_value,
                        ParameterValue::Integer(v) => *v = new_value.parse().unwrap_or(*v),
                        ParameterValue::Decimal(v) => *v = new_value.parse().unwrap_or(*v),
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
        None
    }
}
