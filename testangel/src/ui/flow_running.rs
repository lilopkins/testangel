use std::{collections::HashMap, sync::Arc, thread};

use egui_file::FileDialog;
use testangel::{types::{AutomationFlow, FlowError}, report_generation};
use testangel_ipc::prelude::*;

use super::{action_loader::ActionMap, ipc::EngineList, UiComponent};

#[derive(Default)]
pub struct FlowRunningState {
    action_map: Arc<ActionMap>,
    engine_map: Arc<EngineList>,
    pub flow: Option<AutomationFlow>,
    running: bool,
    num_dots_ellipsis: f32,
    save_dialog: Option<FileDialog>,
    worker_thread: Option<thread::JoinHandle<Result<FlowExecutionResult, FlowError>>>,
}

impl FlowRunningState {
    pub fn new(action_map: Arc<ActionMap>, engine_map: Arc<EngineList>) -> Self {
        Self {
            action_map,
            engine_map,
            ..Default::default()
        }
    }

    /// Start execution of this automation flow.
    pub fn start_flow(&mut self) {
        self.running = true;
        let flow = self.flow.as_ref().unwrap().clone();
        let action_map = self.action_map.clone();
        let engine_map = self.engine_map.clone();

        self.worker_thread = Some(thread::spawn(move || {
            let flow = flow;
            let mut outputs: Vec<HashMap<usize, ParameterValue>> = Vec::new();
            let mut evidence = Vec::new();

            for engine in engine_map.inner() {
                if let Err(_) = engine.reset_state() {
                    evidence.push(Evidence {
                        label: String::from("WARNING: State Warning"),
                        content: EvidenceContent::Textual(String::from("For this test execution, the state couldn't be correctly reset. Some results may not be accurate."))
                    });
                }
            }

            for action_config in flow.actions {
                let (output, ev) = action_config.execute(
                    action_map.clone(),
                    engine_map.clone(),
                    outputs.clone(),
                )?;
                outputs.push(output);
                evidence = vec![evidence, ev].concat();
            }
            Ok(FlowExecutionResult { evidence })
        }));
    }

    /// Update the action map in this state.
    pub(crate) fn update_actions(&mut self, new_action_map: Arc<ActionMap>) {
        self.action_map = new_action_map;
    }
}

impl UiComponent for FlowRunningState {
    fn menu_bar(&mut self, _ui: &mut egui::Ui) -> Option<super::State> {
        None
    }

    fn always_ui(&mut self, ctx: &egui::Context) -> Option<super::State> {
        if let Some(dialog) = &mut self.save_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    // generate pdf and save
                    let res = self.worker_thread.take().unwrap().join().unwrap();
                    if let Ok(fer) = res {
                        report_generation::save_report(path, fer.evidence);
                    }

                    // Set the worker thread to None, the flow to None, and return a new state of the editor.
                    self.worker_thread = None;
                    self.flow = None;
                    self.save_dialog = None;
                    return Some(super::State::AutomationFlowEditor);
                }
            }
        }

        None
    }

    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<super::State> {
        if self.running {
            self.num_dots_ellipsis =
                ctx.animate_value_with_time(egui::Id::new("flowrunning-ellipses"), 3.99, 3.0);
            if self.num_dots_ellipsis == 3.99 {
                self.num_dots_ellipsis =
                    ctx.animate_value_with_time(egui::Id::new("flowrunning-ellipses"), 0.0, 0.0);
            }

            let mut ellipses = String::new();
            let num_dots = self.num_dots_ellipsis.floor() as i32;
            for _ in 0..num_dots {
                ellipses.push('.');
            }
            ui.heading(format!("Automation flow running{ellipses}"));

            if let Some(handle) = &self.worker_thread {
                if handle.is_finished() {
                    self.running = false;
                }
            }
        } else {
            ui.heading("Saving Automation Flow execution report.");
            if let None = self.save_dialog {
                let mut dialog = FileDialog::save_file(None);
                dialog.open();
                self.save_dialog = Some(dialog);
            }
        }

        None
    }
}

struct FlowExecutionResult {
    evidence: Vec<Evidence>,
}
