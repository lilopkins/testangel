#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::rc::Rc;

use action::ActionState;
use eframe::IconData;

mod action;
mod ipc;
mod modals;
mod types;

fn main() {
    pretty_env_logger::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data = Some(IconData::try_from_png_bytes(include_bytes!("../../icon.png")).unwrap());
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
    engines: Rc<ipc::EngineMap>,
    state: State,
    action_state: action::ActionState,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
enum State {
    #[default]
    Nothing,
    ActionEditor,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        let engines_rc = Rc::new(ipc::get_engines());
        Self {
            engines: engines_rc.clone(),
            action_state: ActionState::new(engines_rc),
            ..Default::default()
        }
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

                if let Some(next_state) = self.action_state.menu_bar(ui) {
                    self.state = next_state;
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
        if let Some(next_state) = self.action_state.always_ui(ctx) {
            self.state = next_state;
        }

        // Render content
        egui::CentralPanel::default().show(ctx, |ui| match self.state {
            State::Nothing => {
                ui.label("Nothing to display.");
            }
            State::ActionEditor => {
                if let Some(next_state) = self.action_state.ui(ctx, ui) {
                    self.state = next_state;
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
