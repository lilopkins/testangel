use std::rc::Rc;

use testangel_ipc::prelude::ParameterKind;

use crate::{ipc::EngineMap, types::{Action, InstructionConfiguration, ParameterSource}, UiComponent};

#[derive(Clone)]
struct PossibleOutput {
    step: usize,
    kind: ParameterKind,
    id: String,
    friendly_name: String,
}

impl Into<ParameterSource> for PossibleOutput {
    fn into(self) -> ParameterSource {
        ParameterSource::FromOutput(self.step, self.id, self.friendly_name)
    }
}

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

    fn delete_step_menu(&mut self, ui: &mut egui::Ui) {
        let instructions = &mut self.target.as_mut().unwrap().instructions;
        for index in 0..instructions.len() {
            if ui.button(format!("Step {}", index + 1)).clicked() {
                ui.close_menu();
                instructions.remove(index);
                // reshuffle `FromOutput`s
                for step in instructions.iter_mut() {
                    for (_param_id, param_source) in &mut step.parameter_sources {
                        if let ParameterSource::FromOutput(step, _id, _name) = param_source {
                            if *step == index {
                                *param_source = ParameterSource::Literal;
                            } else if *step > index {
                                *step -= 1;
                            }
                        }
                    }
                }
            }
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
        // produce UI for action editor
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let None = self.target {
                panic!("ActionEditor target is null, but ActionEditor is open!")
            }
            let target = self.target.as_mut().unwrap();

            self.all_instructions_available = true;
            let mut index = 0;
            let mut possible_outputs: Vec<PossibleOutput> = Vec::new();
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

                                    // Filter possible_outputs by same ParameterKind.
                                    for po in &possible_outputs {
                                        if po.kind == *param_kind {
                                            let ps: ParameterSource = po.clone().into();
                                            ui.selectable_value(param_source, ps.clone(), ps.text_repr());
                                        }
                                    }
                                });

                            if let ParameterSource::Literal = param_source {
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

                for (output_id, (output_name, output_kind)) in instruction.outputs() {
                    possible_outputs.push(PossibleOutput {
                        step: index,
                        kind: output_kind.clone(),
                        id: output_id.clone(),
                        friendly_name: output_name.clone(),
                    });
                }
                index += 1;
            }
            let last_index = target.instructions.len();
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("+ Add instruction", |ui| self.add_instruction_menu(ui, last_index));
                ui.menu_button("× Delete step", |ui| self.delete_step_menu(ui));
            });
        });

        None
    }
}
