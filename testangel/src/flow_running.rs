use std::{collections::HashMap, sync::Arc, thread};

use testangel_ipc::prelude::*;

use crate::{
    action_loader::ActionMap, automation_flow::types::AutomationFlow, ipc::EngineMap, UiComponent,
};

#[derive(Default)]
pub struct FlowRunningState {
    action_map: Arc<ActionMap>,
    engine_map: Arc<EngineMap>,
    pub flow: Option<AutomationFlow>,
    running: bool,
    num_dots_ellipsis: f32,
    worker_thread: Option<thread::JoinHandle<Result<FlowExecutionResult, FlowError>>>,
}

impl FlowRunningState {
    pub fn new(action_map: Arc<ActionMap>) -> Self {
        Self {
            action_map,
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
}

impl UiComponent for FlowRunningState {
    fn menu_bar(&mut self, _ui: &mut egui::Ui) -> Option<crate::State> {
        None
    }

    fn always_ui(&mut self, _ctx: &egui::Context) -> Option<crate::State> {
        None
    }

    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<crate::State> {
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
            // TODO show file save dialog for evidence PDF. On save, set the worker thread to None,
            // TODO the flow to None, and return a new state of the editor.
        }

        None
    }
}

struct FlowExecutionResult {
    evidence: Vec<Evidence>,
}

pub enum FlowError {
    FromInstruction {
        error_kind: ErrorKind,
        reason: String,
    },
    IPCFailure,
}
