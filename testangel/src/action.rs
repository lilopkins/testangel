use std::rc::Rc;

use crate::{ipc::EngineMap, types::Action, UiComponent};

#[derive(Default)]
pub(crate) struct ActionState {
    engine_map: Rc<EngineMap>,
    target: Option<Action>,
}

impl ActionState {
    pub fn new(engine_map: Rc<EngineMap>) -> Self {
        Self {
            engine_map,
            ..Default::default()
        }
    }
}

impl UiComponent for ActionState {
    fn menu_bar(&mut self, ui: &mut egui::Ui) -> Option<crate::State> {
        let mut next_state = None;
        ui.menu_button("Actions", |ui| {
            if ui.button("New").clicked() {
                ui.close_menu();
                self.target = Some(Action::default());
                next_state = Some(crate::State::ActionEditor);
            }
            if ui.button("Open...").clicked() {
                ui.close_menu();
                // TODO: Open file dialog to open Action.
            }
            ui.add_enabled_ui(false /* TODO */, |ui| {
                if ui.button("Save").clicked() {
                    ui.close_menu();
                    // TODO: Open file dialog (if needed) to save Action.
                }
                if ui.button("Save as...").clicked() {
                    ui.close_menu();
                    // TODO: Open file dialog to save Action.
                }
                if ui.button("Close").clicked() {
                    ui.close_menu();
                    self.target = None;
                    next_state = Some(crate::State::Nothing);
                }
            });
        });
        next_state
    }

    fn ui(&mut self, ui: &mut egui::Ui) -> Option<crate::State> {
        if let None = self.target {
            panic!("ActionEditor target is null, but ActionEditor is open!")
        }
        let mut target = self.target.as_mut().unwrap();

        // TODO produce UI for action editor

        None
    }
}
