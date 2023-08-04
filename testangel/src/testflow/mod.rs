use std::{rc::Rc, path::PathBuf, fs::{self, File}, io::BufReader};

use egui_file::FileDialog;
use testangel_ipc::prelude::ParameterKind;

use crate::{ipc::EngineMap, UiComponent};
use types::{TestFlow, ActionConfiguration, ParameterSource};

mod types;

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
pub(crate) struct TestFlowState {
    engine_map: Rc<EngineMap>,
    target: Option<TestFlow>,
    error: String,
    trigger_error: bool,
    all_instructions_available: bool,
    save_path: Option<PathBuf>,
    open_dialog: Option<FileDialog>,
    save_dialog: Option<FileDialog>,
}

impl TestFlowState {
    pub fn new(engine_map: Rc<EngineMap>) -> Self {
        Self {
            engine_map,
            all_instructions_available: true,
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
                        self.target.as_mut().unwrap().instructions.insert(index, ActionConfiguration::from(instruction.clone()));
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

    fn save(&self) -> Result<(), ()> {
        if self.save_path.is_none() {
            panic!("Save path not set");
        }
        if let Ok(data) = rmp_serde::to_vec(&self.target.as_ref().unwrap()) {
            if let Ok(_) = fs::write(self.save_path.as_ref().unwrap(), data) {
                return Ok(());
            }
        }
        Err(())
    }

    pub(crate) fn close(&mut self) {
        self.save_path = None;
        self.target = None;
    }
}

impl UiComponent for TestFlowState {
    fn menu_bar(&mut self, ui: &mut egui::Ui) -> Option<crate::State> {
        let mut next_state = None;
        ui.menu_button("Test Flow", |ui| {
            if ui.button("New").clicked() {
                ui.close_menu();
                self.target = Some(TestFlow::default());
                next_state = Some(crate::State::TestFlowEditor);
            }
            if ui.button("Open...").clicked() {
                ui.close_menu();
                let mut dialog = FileDialog::open_file(None);
                dialog.open();
                self.open_dialog = Some(dialog);
            }
            ui.add_enabled_ui(self.target.is_some(), |ui| {
                if ui.button("Save").clicked() {
                    ui.close_menu();
                    if let None = self.save_path {
                        let mut dialog = FileDialog::save_file(None);
                        dialog.open();
                        self.save_dialog = Some(dialog);
                    } else {
                        if let Err(_) = self.save() {
                            self.error = "Failed to save.".to_owned();
                            self.trigger_error = true;
                        }
                    }
                }
                if ui.button("Save as...").clicked() {
                    ui.close_menu();
                    let mut dialog = FileDialog::save_file(None);
                    dialog.open();
                    self.save_dialog = Some(dialog);
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

    fn always_ui(&mut self, ctx: &egui::Context) -> Option<crate::State> {
        let mut next_state = None;

        // handle error modal
        let error_modal = crate::modals::error_modal(ctx, "test_flow_editor_error_modal", &self.error);
        if self.trigger_error {
            error_modal.open();
            self.trigger_error = false;
        }

        // handle open dialog
        if let Some(dialog) = &mut self.open_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    let res = File::open(path);
                    if let Ok(file) = res {
                        let res = rmp_serde::from_read(BufReader::new(file));
                        if let Ok(action) = res {
                            self.save_path = Some(path.to_path_buf());
                            self.target = Some(action);
                            next_state = Some(crate::State::TestFlowEditor);
                        } else {
                            self.error = format!("Failed to parse action. ({:?})", res.unwrap_err());
                            error_modal.open();
                        }
                    } else {
                        self.error = format!("Failed to open action. ({:?})", res.unwrap_err());
                        error_modal.open();
                    }
                }
            }
        }

        // handle save dialog
        if let Some(dialog) = &mut self.save_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    self.save_path = Some(path.to_path_buf());
                    if let Err(_) = self.save() {
                        self.error = "Failed to save.".to_owned();
                        error_modal.open();
                    }
                }
            }
        }

        next_state
    }

    fn ui(&mut self, _ctx: &egui::Context, ui: &mut egui::Ui) -> Option<crate::State> {
        // produce UI for action editor
        egui::ScrollArea::vertical().show(ui, |ui| {
            if let None = self.target {
                panic!("TestFlowEditor target is null, but TestFlowEditor is open!")
            }
            let target = self.target.as_mut().unwrap();

            self.all_instructions_available = true;
            let mut index = 0;
            let mut possible_outputs: Vec<PossibleOutput> = Vec::new();
            for instruction_config in &mut target.instructions {
                let instruction = self.engine_map.get_instruction_by_id(instruction_config.action_id.clone());
                if let None = instruction {
                    self.all_instructions_available = false;
                    continue;
                }
                let instruction = instruction.unwrap();

                ui.group(|ui| {
                    ui.heading(format!("Step {}: {}", index + 1, instruction.friendly_name()));

                    ui.separator();
                    ui.label("Parameters:");
                    for param_id in instruction.parameter_order() {
                        let (param_name, param_kind) = instruction.parameters().get(param_id).unwrap();
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
                ui.menu_button("+ Add action", |ui| self.add_instruction_menu(ui, last_index));
                ui.menu_button("× Delete step", |ui| self.delete_step_menu(ui));
            });
        });

        None
    }
}
