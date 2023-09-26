use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use iced::{
    widget::{column, row, Button, Checkbox, Column, Container, Space, Text, TextInput, Rule},
    Length,
};
use testangel::{ipc::EngineList, report_generation, types::Action};
use testangel_ipc::prelude::{Evidence, EvidenceContent, ParameterKind, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum ActionRunningMessage {
    Tick,
    RunAction,
    BackToEditor,
    Save(Option<PathBuf>, Vec<Evidence>),
    ParameterValueChange(usize, String),
}

#[derive(Clone, Debug)]
pub enum ActionRunningMessageOut {
    BackToEditor,
    SaveActionReport(Vec<Evidence>),
}

#[derive(Default)]
pub struct ActionRunning {
    engines_list: Arc<EngineList>,
    action: Option<Action>,
    parameters: HashMap<usize, ParameterValue>,

    thread: Option<JoinHandle<Option<Vec<Evidence>>>>,
    is_saving: bool,
}

impl ActionRunning {
    pub fn new(engines_list: Arc<EngineList>) -> Self {
        Self {
            engines_list,
            action: None,
            parameters: HashMap::new(),
            thread: None,
            is_saving: false,
        }
    }

    pub fn set_action(&mut self, action: Action) {
        self.parameters.clear();
        for (idx, (_name, kind)) in action.parameters.iter().enumerate() {
            self.parameters.insert(idx, kind.default_value());
        }
        self.action = Some(action);
    }

    fn start_flow(&mut self) {
        self.is_saving = false;
        let engines_list = self.engines_list.clone();
        let action = self.action.clone().unwrap();
        let parameters = self.parameters.clone();
        self.thread = Some(thread::spawn(move || {
            let mut outputs: Vec<HashMap<String, ParameterValue>> = Vec::new();
            let mut evidence = Vec::new();

            for engine in engines_list.inner() {
                if engine.reset_state().is_err() {
                    evidence.push(Evidence {
                        label: String::from("WARNING: State Warning"),
                        content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
                    });
                }
            }

            for (step, instruction_config) in action.instructions.iter().enumerate() {
                let mut proceed = true;
                match instruction_config.execute(
                    engines_list.clone(),
                    &parameters.clone(),
                    outputs.clone(),
                ) {
                    Ok((output, ev)) => {
                        outputs.push(output);
                        evidence = [evidence, ev].concat();
                    }
                    Err(e) => {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Failed to execute")
                            .set_description(&format!(
                                "Failed to execute action at step {}: {e}",
                                step + 1
                            ))
                            .show();
                        proceed = false;
                    }
                }
                if !proceed {
                    return None;
                }
            }

            // Add outputs as evidence
            for (name, kind, src) in action.outputs {
                let value = match src {
                    testangel::types::InstructionParameterSource::Literal => unimplemented!(),
                    testangel::types::InstructionParameterSource::FromParameter(param_id) => {
                        parameters[&param_id].clone()
                    }
                    testangel::types::InstructionParameterSource::FromOutput(step, id) => {
                        outputs[step][&id].clone()
                    }
                };
                evidence.push(Evidence {
                    label: format!("Output '{name}' of kind '{kind}'"),
                    content: EvidenceContent::Textual(value.to_string()),
                });
            }

            Some(evidence)
        }));
    }

    fn ui_action_inputs(&self, action: Action) -> iced::Element<'_, ActionRunningMessage> {
        action
            .parameters
            .iter()
            .enumerate()
            .fold(Column::new().spacing(4), |col, (id, (name, kind))| {
                let param_value = &self.parameters[&id];

                let literal_input: iced::Element<'_, _> = match kind {
                    ParameterKind::Boolean => {
                        Checkbox::new("Value", param_value.value_bool(), move |new_val| {
                            ActionRunningMessage::ParameterValueChange(
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
                            ActionRunningMessage::ParameterValueChange(id, i32::MIN.to_string())
                        } else {
                            ActionRunningMessage::ParameterValueChange(id, new_val)
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
                            ActionRunningMessage::ParameterValueChange(id, f32::MIN.to_string())
                        } else {
                            ActionRunningMessage::ParameterValueChange(id, new_val)
                        }
                    })
                    .width(250)
                    .into(),
                    _ => TextInput::new("Literal value", &param_value.to_string())
                        .on_input(move |new_val| {
                            ActionRunningMessage::ParameterValueChange(id, new_val)
                        })
                        .width(250)
                        .into(),
                };

                col.push(
                    row![Text::new(format!("{name} ({kind}) use")), literal_input,]
                        .spacing(4)
                        .align_items(iced::Alignment::Center),
                )
            })
            .into()
    }
}

impl UiComponent for ActionRunning {
    type Message = ActionRunningMessage;
    type MessageOut = ActionRunningMessageOut;

    fn title(&self) -> Option<&str> {
        Some("Action Running")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(500)).map(|_| ActionRunningMessage::Tick)
    }

    fn update(
        &mut self,
        message: Self::Message,
    ) -> (
        Option<Self::MessageOut>,
        Option<iced::Command<super::AppMessage>>,
    ) {
        match message {
            ActionRunningMessage::Tick => {
                if let Some(thread) = &self.thread {
                    if thread.is_finished() {
                        self.is_saving = true;
                        if let Some(evidence) = self.thread.take().unwrap().join().unwrap() {
                            return (
                                Some(ActionRunningMessageOut::SaveActionReport(evidence)),
                                None,
                            );
                        }
                        return (Some(ActionRunningMessageOut::BackToEditor), None);
                    }
                }
            }

            ActionRunningMessage::ParameterValueChange(id, val) => {
                self.parameters.insert(
                    id,
                    match self.parameters[&id].kind() {
                        ParameterKind::Boolean => ParameterValue::Boolean(val == "yes"),
                        ParameterKind::Integer => {
                            ParameterValue::Integer(val.parse().unwrap_or_default())
                        }
                        ParameterKind::Decimal => {
                            ParameterValue::Decimal(val.parse().unwrap_or_default())
                        }
                        ParameterKind::String => ParameterValue::String(val),
                    },
                );
            }

            ActionRunningMessage::RunAction => {
                self.start_flow();
            }

            ActionRunningMessage::BackToEditor => {
                return (Some(ActionRunningMessageOut::BackToEditor), None);
            }

            ActionRunningMessage::Save(to, evidence) => {
                if let Some(path) = to {
                    report_generation::save_report(path.with_extension("pdf"), evidence);
                    if let Err(e) = opener::open(path.with_extension("pdf")) {
                        log::warn!("Failed to open evidence: {e}");
                    }
                }
                return (Some(ActionRunningMessageOut::BackToEditor), None);
            }
        }
        (None, None)
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        if self.thread.is_none() {
            Container::new(
                column![
                    Text::new("Run Action").size(24),
                    Text::new("This will execute an action alone and produce a report with the evidence and the outputs of the action."),
                    Rule::horizontal(1),
                    Text::new("Inputs").size(18),
                    self.ui_action_inputs(self.action.clone().unwrap()),
                    Space::with_height(4),
                    Button::new("Run Action").on_press(ActionRunningMessage::RunAction),
                    Button::new("Back to Editor").on_press(ActionRunningMessage::BackToEditor),
                ]
                .spacing(4),
            )
            .padding(16)
            .width(Length::Fill)
            .into()
        } else {
            Container::new(Text::new(if self.is_saving {
                "Saving report..."
            } else {
                "Action running..."
            }))
            .padding(32)
            .into()
        }
    }
}
