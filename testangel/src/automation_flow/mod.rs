use std::{
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
    sync::Arc,
};

use egui_file::FileDialog;
use testangel_ipc::prelude::ParameterKind;
use itertools::Itertools;

use crate::{action_loader::ActionMap, UiComponent};
use types::{ActionConfiguration, AutomationFlow, ParameterSource};

pub mod types;

#[derive(Clone)]
struct PossibleOutput {
    step: usize,
    kind: ParameterKind,
    id: usize,
    friendly_name: String,
}

impl Into<ParameterSource> for PossibleOutput {
    fn into(self) -> ParameterSource {
        ParameterSource::FromOutput(self.step, self.id, self.friendly_name)
    }
}

#[derive(Default)]
pub(crate) struct AutomationFlowState {
    action_map: Arc<ActionMap>,
    target: Option<AutomationFlow>,
    error: String,
    trigger_error: bool,
    all_instructions_available: bool,
    save_path: Option<PathBuf>,
    open_dialog: Option<FileDialog>,
    save_dialog: Option<FileDialog>,
}

impl AutomationFlowState {
    pub fn new(action_map: Arc<ActionMap>) -> Self {
        Self {
            action_map,
            all_instructions_available: true,
            ..Default::default()
        }
    }

    pub fn add_action_menu(&mut self, ui: &mut egui::Ui, index: usize) {
        let groups = self.action_map.get_by_group();
        for group in groups.keys().sorted() {
            ui.menu_button(group.clone(), |ui| {
                let mut actions = groups[group].clone();
                actions.sort_by(|a, b| a.friendly_name.cmp(&b.friendly_name));
                for action in &actions {
                    if ui.button(action.friendly_name.clone()).clicked() {
                        // add action
                        ui.close_menu();
                        self.target
                            .as_mut()
                            .unwrap()
                            .actions
                            .insert(index, ActionConfiguration::from(action.clone()));
                    }
                }
            });
        }
    }

    fn delete_action_menu(&mut self, ui: &mut egui::Ui) {
        let instructions = &mut self.target.as_mut().unwrap().actions;
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

    fn save(&mut self) -> Result<(), ()> {
        if self.save_path.is_none() {
            panic!("Save path not set");
        }
        let save_path = self.save_path.as_mut().unwrap();
        save_path.set_extension("taflow");
        if let Ok(data) = ron::to_string(&self.target.as_ref().unwrap()) {
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

    /// Get a copy of the currently opened test flow
    pub(crate) fn test_flow(&self) -> AutomationFlow {
        self.target.as_ref().unwrap().clone()
    }

    /// Update the action map in this state.
    pub(crate) fn update_actions(&mut self, new_action_map: Arc<ActionMap>) {
        self.action_map = new_action_map;
    }
}

impl UiComponent for AutomationFlowState {
    fn menu_bar(&mut self, ui: &mut egui::Ui) -> Option<crate::State> {
        let mut next_state = None;
        ui.menu_button("Automation Flow", |ui| {
            if ui.button("New").clicked() {
                ui.close_menu();
                self.target = Some(AutomationFlow::default());
                next_state = Some(crate::State::AutomationFlowEditor);
            }
            if ui.button("Open...").clicked() {
                ui.close_menu();
                let mut dialog = FileDialog::open_file(None);
                dialog.open();
                self.open_dialog = Some(dialog);
            }
            ui.add_enabled_ui(self.target.is_some(), |ui| {
                if ui.button("Run Flow").clicked() {
                    ui.close_menu();
                    next_state = Some(crate::State::AutomationFlowRunning);
                }
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
        let error_modal =
            crate::modals::error_modal(ctx, "test_flow_editor_error_modal", &self.error);
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
                        use std::io::Read;
                        let mut buf = String::new();
                        let mut r = BufReader::new(file);
                        let res = r.read_to_string(&mut buf);
                        if let Ok(_) = res {
                            let res = ron::from_str(&buf);
                            if let Ok(action) = res {
                                self.save_path = Some(path.to_path_buf());
                                self.target = Some(action);
                                next_state = Some(crate::State::AutomationFlowEditor);
                            } else {
                                self.error =
                                    format!("Failed to parse action. ({:?})", res.unwrap_err());
                                error_modal.open();
                            }
                        } else {
                            self.error = format!("Failed to read action. ({:?})", res.unwrap_err());
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
                panic!("AutomationFlowEditor target is null, but AutomationFlowEditor is open!")
            }
            let target = self.target.as_mut().unwrap();

            self.all_instructions_available = true;
            let mut index = 0;
            let mut possible_outputs: Vec<PossibleOutput> = Vec::new();
            for action_config in &mut target.actions {
                let instruction = self.action_map.get_action_by_id(&action_config.action_id);
                if let None = instruction {
                    self.all_instructions_available = false;
                    continue;
                }
                let action = instruction.unwrap();

                ui.group(|ui| {
                    ui.heading(format!("Step {}: {}", index + 1, action.friendly_name));

                    ui.separator();
                    ui.label("Parameters:");
                    let mut param_id = 0;
                    for (param_name, param_kind) in action.parameters {
                        ui.horizontal_wrapped(|ui| {
                            ui.label(format!("{param_name} ({param_kind})"));

                            let param_source =
                                action_config.parameter_sources.get_mut(&param_id).unwrap();
                            egui::ComboBox::from_id_source(format!("{index}_param_{param_id}"))
                                .selected_text(param_source.text_repr())
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        param_source,
                                        ParameterSource::Literal,
                                        ParameterSource::Literal.text_repr(),
                                    );

                                    // Filter possible_outputs by same ParameterKind.
                                    for po in &possible_outputs {
                                        if po.kind == param_kind {
                                            let ps: ParameterSource = po.clone().into();
                                            ui.selectable_value(
                                                param_source,
                                                ps.clone(),
                                                ps.text_repr(),
                                            );
                                        }
                                    }
                                });

                            if let ParameterSource::Literal = param_source {
                                // Literal
                                let param_value =
                                    action_config.parameter_values.get_mut(&param_id).unwrap();

                                match param_kind {
                                    ParameterKind::Integer => {
                                        ui.add(
                                            egui::DragValue::new(param_value.int_mut()).speed(1),
                                        );
                                    }
                                    ParameterKind::Decimal => {
                                        ui.add(
                                            egui::DragValue::new(param_value.f32_mut()).speed(0.1),
                                        );
                                    }
                                    _ => {
                                        ui.text_edit_singleline(param_value.string_mut());
                                    }
                                }
                            }
                        });
                        param_id += 1;
                    }

                    ui.add_space(8.);
                    ui.label("Outputs:");
                    for (output_name, output_kind, _output_src) in &action.outputs {
                        ui.label(format!("{output_name} ({output_kind})"));
                    }
                });

                let mut output_id = 0;
                for (output_name, output_kind, _output_src) in action.outputs {
                    possible_outputs.push(PossibleOutput {
                        step: index,
                        kind: output_kind.clone(),
                        id: output_id,
                        friendly_name: output_name.clone(),
                    });
                    output_id += 1;
                }
                index += 1;
            }
            let last_index = target.actions.len();
            ui.horizontal_wrapped(|ui| {
                ui.menu_button("+ Add action", |ui| self.add_action_menu(ui, last_index));
                ui.menu_button("× Delete action", |ui| self.delete_action_menu(ui));
            });
        });

        None
    }
}
