use std::rc::Rc;

use testangel_ipc::prelude::ParameterKind;

use crate::{ipc::EngineMap, types::{Action, InstructionConfiguration, ParameterSource}, UiComponent};

#[derive(Default)]
pub(crate) struct ActionState {
    engine_map: Rc<EngineMap>,
    target: Option<Action>,
    all_instructions_available: bool,
}

impl ActionState {
    pub fn new(engine_map: Rc<EngineMap>) -> Self {
        Self {
            engine_map,
            ..Default::default()
        }
    }

    pub fn add_instruction_menu(&mut self, ui: &mut egui::Ui, index: usize) {
        for (_path, engine) in self.engine_map.inner() {
            ui.menu_button(engine.name.clone(), |ui| {
                for instruction in &engine.instructions {
                    if ui.button(instruction.friendly_name()).clicked() {
                        // add instruction
                        ui.close_menu();
                        self.target.as_mut().unwrap().instructions.insert(index, InstructionConfiguration::from(instruction.clone()));
                    }
                }
            });
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
        let target = self.target.as_mut().unwrap();

        // produce UI for action editor
        self.all_instructions_available = true;
        let mut index = 0;
        for instruction_config in &mut target.instructions {
            let instruction = self.engine_map.get_instruction_by_id(instruction_config.instruction_id.clone());
            if let None = instruction {
                self.all_instructions_available = false;
                continue;
            }
            let instruction = instruction.unwrap();

            ui.group(|ui| {
                ui.heading(format!("Step {}: {}", index + 1, instruction.friendly_name()));

                ui.separator();
                ui.label("Parameters:");
                for (param_id, (param_name, param_kind)) in instruction.parameters() {
                    ui.horizontal_wrapped(|ui| {
                        ui.label(format!("{param_name} ({param_kind})"));

                        let param_source = instruction_config.parameter_sources.get_mut(param_id).unwrap();
                        egui::ComboBox::from_id_source(format!("{index}_param_{param_id}"))
                            .selected_text(param_source.text_repr())
                            .show_ui(ui, |ui| {
                                ui.selectable_value(param_source, ParameterSource::Literal, ParameterSource::Literal.text_repr());
                                ui.selectable_value(param_source, ParameterSource::FromOutput, ParameterSource::FromOutput.text_repr());
                            });

                        if *param_source == ParameterSource::FromOutput {
                            // TODO: Show output selector.
                            // Get a list of previous step's outputs (or maintain one as we go).
                            // Filter by same ParameterKind.
                            // Show in ComboBox.
                        } else {
                            // Literal
                            let param_value = instruction_config.parameter_values.get_mut(param_id).unwrap();

                            match param_kind {
                                ParameterKind::Integer => {
                                    ui.add(egui::DragValue::new(param_value.int_mut()).speed(1));
                                }
                                ParameterKind::Decimal => {
                                    ui.add(egui::DragValue::new(param_value.f32_mut()).speed(0.1));
                                }
                                _ => {
                                    ui.text_edit_singleline(param_value.string_mut());
                                }
                            }
                        }
                    });
                }

                ui.add_space(8.);
                ui.label("Outputs:");
                for (_output_id, (output_name, output_kind)) in instruction.outputs() {
                    ui.label(format!("{output_name} ({output_kind})"));
                }
            });

            index += 1;
        }
        let last_index = target.instructions.len();
        ui.menu_button("+ Add instruction", |ui| self.add_instruction_menu(ui, last_index));


        None
    }
}
