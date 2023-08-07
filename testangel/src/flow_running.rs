use crate::{UiComponent, automation_flow::types::AutomationFlow};

#[derive(Default)]
pub struct FlowRunningState {
    pub flow: Option<AutomationFlow>,
}

impl UiComponent for FlowRunningState {
    fn menu_bar(&mut self, _ui: &mut egui::Ui) -> Option<crate::State> {
        None
    }

    fn always_ui(&mut self, _ctx: &egui::Context) -> Option<crate::State> {
        None
    }

    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Option<crate::State> {
        ui.heading("Automation flow running.");
        ui.horizontal_wrapped(|ui| {
            ui.label("Progress:");
            let pb = egui::ProgressBar::new(0.);
            ui.add(pb);
        });
        None
    }
}
