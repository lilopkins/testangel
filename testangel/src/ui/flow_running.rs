use std::{
    collections::HashMap,
    env,
    sync::Arc,
    thread::{self, JoinHandle},
    time::Duration,
};

use iced::widget::{Container, Text};
use testangel::{
    action_loader::ActionMap, ipc::EngineList, report_generation, types::AutomationFlow,
};
use testangel_ipc::prelude::{Evidence, EvidenceContent, ParameterValue};

use super::UiComponent;

#[derive(Clone, Debug)]
pub enum FlowRunningMessage {
    Tick,
}

#[derive(Clone, Debug)]
pub enum FlowRunningMessageOut {
    BackToEditor,
}

#[derive(Default)]
pub struct FlowRunning {
    actions_list: Arc<ActionMap>,
    engines_list: Arc<EngineList>,

    thread: Option<JoinHandle<Option<Vec<Evidence>>>>,
    is_saving: bool,
}

impl FlowRunning {
    pub fn new(actions_list: Arc<ActionMap>, engines_list: Arc<EngineList>) -> Self {
        Self {
            actions_list,
            engines_list,
            thread: None,
            is_saving: false,
        }
    }

    pub(crate) fn start_flow(&mut self, flow: AutomationFlow) {
        self.is_saving = false;
        let actions_list = self.actions_list.clone();
        let engines_list = self.engines_list.clone();
        self.thread = Some(thread::spawn(move || {
            let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
            let mut evidence = Vec::new();

            for engine in engines_list.inner() {
                if engine.reset_state().is_err() {
                    evidence.push(Evidence {
                        label: String::from("WARNING: State Warning"),
                        content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
                    });
                }
            }

            for (step, action_config) in flow.actions.iter().enumerate() {
                let mut proceed = true;
                match action_config.execute(
                    actions_list.clone(),
                    engines_list.clone(),
                    outputs.clone(),
                ) {
                    Ok((output, ev)) => {
                        outputs.push(output);
                        evidence = vec![evidence, ev].concat();
                    }
                    Err(e) => {
                        rfd::MessageDialog::new()
                            .set_level(rfd::MessageLevel::Error)
                            .set_title("Failed to execute")
                            .set_description(&format!(
                                "Failed to execute flow at step {}: {e}",
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

            Some(evidence)
        }));
    }

    pub(crate) fn update_action_map(&mut self, actions_list: Arc<ActionMap>) {
        self.actions_list = actions_list;
    }
}

impl UiComponent for FlowRunning {
    type Message = FlowRunningMessage;
    type MessageOut = FlowRunningMessageOut;

    fn title(&self) -> Option<&str> {
        Some("Flow Running")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::time::every(Duration::from_millis(500)).map(|_| FlowRunningMessage::Tick)
    }

    fn update(&mut self, message: Self::Message) -> Option<Self::MessageOut> {
        match message {
            FlowRunningMessage::Tick => {
                if let Some(thread) = &self.thread {
                    if thread.is_finished() {
                        self.is_saving = true;
                        if let Some(evidence) = self.thread.take().unwrap().join().unwrap() {
                            if let Some(path) = rfd::FileDialog::new()
                                .add_filter("Portable Document Format", &["pdf"])
                                .set_file_name("report.pdf")
                                .set_title("Save Report")
                                .set_directory(env::current_dir().expect("Failed to get cwd"))
                                .save_file()
                            {
                                report_generation::save_report(
                                    path.with_extension("pdf"),
                                    evidence,
                                );
                            }
                        }
                        return Some(FlowRunningMessageOut::BackToEditor);
                    }
                }
            }
        }
        None
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
        Container::new(Text::new(if self.is_saving {
            "Saving report..."
        } else {
            "Flow running..."
        }))
        .padding(32)
        .into()
    }
}
