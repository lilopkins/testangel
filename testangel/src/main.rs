#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Arc;

use action::ActionState;
use automation_flow::AutomationFlowState;
use eframe::IconData;
use flow_running::FlowRunningState;

mod action;
mod action_loader;
mod automation_flow;
mod flow_running;
mod ipc;
mod modals;

fn main() {
    pretty_env_logger::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data =
        Some(IconData::try_from_png_bytes(include_bytes!("../../icon.png")).unwrap());
    if let Err(err) = eframe::run_native(
        "TestAngel",
        native_options,
        Box::new(|cc| Box::new(App::new(cc))),
    ) {
        log::error!("Error initialising window: {err}");
    }
}

#[derive(Default)]
struct App {
    state: State,
    action_state: action::ActionState,
    test_flow_state: automation_flow::AutomationFlowState,
    flow_running_state: flow_running::FlowRunningState,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum State {
    #[default]
    Nothing,
    AutomationFlowEditor,
    AutomationFlowRunning,
    ActionEditor,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let engines_rc = Arc::new(ipc::get_engines());
        let actions_rc = Arc::new(action_loader::get_actions());
        Self {
            action_state: ActionState::new(engines_rc.clone()),
            test_flow_state: AutomationFlowState::new(actions_rc.clone()),
            flow_running_state: FlowRunningState::new(actions_rc, engines_rc),
            ..Default::default()
        }
    }

    fn change_state(&mut self, next_state: State) {
        if next_state == State::AutomationFlowEditor {
            // reload actions
            let actions_rc = Arc::new(action_loader::get_actions());
            self.test_flow_state.update_actions(actions_rc.clone());
            self.flow_running_state.update_actions(actions_rc);
        }
        self.state = next_state;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle modals
        let about_modal = modals::about_modal(ctx);

        // Render top menu
        egui::TopBottomPanel::top("menu_bar").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                egui::widgets::global_dark_light_mode_switch(ui);

                ui.separator();

                if let Some(next_state) = self.test_flow_state.menu_bar(ui) {
                    if next_state != State::AutomationFlowEditor
                        && next_state != State::AutomationFlowRunning
                    {
                        self.test_flow_state.close();
                    } else if next_state == State::AutomationFlowRunning {
                        // Pass over flow
                        let flow = self.test_flow_state.test_flow();
                        self.flow_running_state.flow = Some(flow);
                        self.flow_running_state.start_flow();
                    }
                    self.change_state(next_state);
                }
                if let Some(next_state) = self.flow_running_state.menu_bar(ui) {
                    self.change_state(next_state);
                }
                if let Some(next_state) = self.action_state.menu_bar(ui) {
                    if next_state != State::ActionEditor {
                        self.action_state.close();
                    }
                    self.change_state(next_state);
                }

                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                        about_modal.open();
                    }
                    ui.hyperlink_to("GitHub", "https://github.com/lilopkins/testangel");
                });
            });
        });

        // Render always UI
        if let Some(next_state) = self.test_flow_state.always_ui(ctx) {
            if next_state != State::AutomationFlowEditor
                && next_state != State::AutomationFlowRunning
            {
                self.test_flow_state.close();
            }
            self.change_state(next_state);
        }
        if let Some(next_state) = self.flow_running_state.always_ui(ctx) {
            self.change_state(next_state);
        }
        if let Some(next_state) = self.action_state.always_ui(ctx) {
            if next_state != State::ActionEditor {
                self.action_state.close();
            }
            self.change_state(next_state);
        }

        // Render content
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            State::Nothing => {
                ui.label("Nothing to display.");
            }
            State::AutomationFlowEditor => {
                if let Some(next_state) = self.test_flow_state.ui(ctx, ui) {
                    if next_state != State::AutomationFlowEditor
                        && next_state != State::AutomationFlowRunning
                    {
                        self.test_flow_state.close();
                    }
                    self.change_state(next_state);
                }
            }
            State::AutomationFlowRunning => {
                if let Some(next_state) = self.flow_running_state.ui(ctx, ui) {
                    self.change_state(next_state);
                }
            }
            State::ActionEditor => {
                if let Some(next_state) = self.action_state.ui(ctx, ui) {
                    if next_state != State::ActionEditor {
                        self.action_state.close();
                    }
                    self.change_state(next_state);
                }
            }
        });
    }
}

trait UiComponent {
    /// Render the menu bar. Returns an optional next state to transition to.
    fn menu_bar(&mut self, ui: &mut egui::Ui) -> Option<State>;

    /// Render UI that runs regardless of current state. Returns an optional next state to transition to.
    fn always_ui(&mut self, ctx: &egui::Context) -> Option<State>;

    /// Render the central panel UI. Returns an optional next state to transition to.
    fn ui(&mut self, ctx: &egui::Context, ui: &mut egui::Ui) -> Option<State>;
}
