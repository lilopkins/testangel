#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use egui::{Ui, ComboBox};
use egui_file::FileDialog;
use egui_modal::{Modal, Icon};
use ipc::Engine;

mod ipc;
mod types;
use testangel_ipc::prelude::*;
use types::{Action, InstructionConfiguration, ParameterSource};

fn main() {
    pretty_env_logger::init();

    let native_options = eframe::NativeOptions::default();
    // TODO: Create icon for app
    // native_options.icon_data = Some(IconData::try_from_png_bytes(include_bytes!("icon.png")).unwrap());
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
    engines: HashMap<PathBuf, Engine>,
    error: String,
    file_dialog: Option<(FileDialogAction, FileDialog)>,
    open_path: Option<PathBuf>,
    open_action: Option<Action>,
}

enum FileDialogAction {
    Open,
    SaveAs,
}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            engines: ipc::get_engines(),
            ..Default::default()
        }
    }

    fn save_file(&self) -> Result<(), ()> {
        if self.open_path.is_none() {
            panic!("path not set");
        }
        if let Ok(data) = rmp_serde::to_vec(&self.open_action.as_ref().unwrap()) {
            if let Ok(_) = fs::write(self.open_path.as_ref().unwrap(), data) {
                return Ok(());
            }
        }
        Err(())
    }

    fn close_file(&mut self) {
        self.open_action = None;
        self.open_path = None;
    }

    fn add_instruction_context(&mut self, ui: &mut Ui, index: usize) {
        for (_path, engine) in &self.engines {
            ui.menu_button(&engine.name, |ui| {
                for inst in &engine.instructions {
                    if ui.button(inst.friendly_name()).clicked() {
                        ui.close_menu();
                        self.open_action.as_mut().unwrap().instructions.insert(index, InstructionConfiguration::from(inst.clone()));
                    }
                }
            });
        }
    }

    fn get_instruction(&self, instruction_id: String) -> Option<Instruction> {
        for (_path, engine) in &self.engines {
            for inst in &engine.instructions {
                if *inst.id() == instruction_id {
                    return Some(inst.clone());
                }
            }
        }
        return None;
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle about modal
        let about_modal = Modal::new(ctx, "about_modal");
        about_modal.show(|ui| {
            about_modal.title(ui, "About TestAngel");
            about_modal.frame(ui, |ui| {
                about_modal.body(ui, "TestAngel automates testing across a number of tools by providing a standardised interface to communicate actions to perform.");
            });
            about_modal.buttons(ui, |ui| {
                let _ = about_modal.button(ui, "Close");
            });
        });

        // Handle error modal
        let error_modal = Modal::new(ctx, "file_error_modal");
        error_modal.show(|ui| {
            error_modal.title(ui, "Error");
            error_modal.frame(ui, |ui| {
                error_modal.body_and_icon(ui, &self.error, Icon::Error);
            });
            error_modal.buttons(ui, |ui| {
                let _ = error_modal.button(ui, "Close");
            });
        });

        // Handle file dialog
        if let Some((action, dialog)) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    match action {
                        FileDialogAction::Open => {
                            let res = File::open(path);
                            if let Ok(file) = res {
                                let res = rmp_serde::from_read(BufReader::new(file));
                                if let Ok(action) = res {
                                    self.open_path = Some(path.to_path_buf());
                                    self.open_action = action;
                                } else {
                                    self.error = format!("Failed to parse action. ({:?})", res.unwrap_err());
                                    error_modal.open();
                                }
                            } else {
                                self.error = format!("Failed to open action. ({:?})", res.unwrap_err());
                                error_modal.open();
                            }
                        }
                        FileDialogAction::SaveAs => {
                            self.open_path = Some(path.to_path_buf());
                            if let Err(()) = self.save_file() {
                                self.error = "Failed to save action.".to_owned();
                                error_modal.open();
                            }
                        }
                    }
                }
            }
        }

        // Render top menu
        egui::TopBottomPanel::top("ta_top").show(ctx, |ui| {
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;
                egui::widgets::global_dark_light_mode_switch(ui);
                if ui
                    .button("×")
                    .on_hover_text("Close the currently opened item")
                    .clicked()
                {
                    self.close_file();
                }

                ui.separator();

                ui.menu_button("Test Flows", |ui| {
                    if ui.button("New").clicked() {
                        ui.close_menu();
                        self.close_file();
                        self.open_action = Some(Action::default());
                    }
                    if ui.button("Open...").clicked() {
                        ui.close_menu();
                        self.close_file();
                        let mut dialog = FileDialog::open_file(None);
                        dialog.open();
                        self.file_dialog = Some((FileDialogAction::Open, dialog));
                    }
                    ui.add_enabled_ui(self.open_action.is_some(), |ui| {
                        if ui.button("Save").clicked() {
                            ui.close_menu();
                            self.close_file();
                            if self.open_path.is_none() {
                                let mut dialog = FileDialog::save_file(None);
                                dialog.open();
                                self.file_dialog = Some((FileDialogAction::SaveAs, dialog));
                            } else {
                                if let Err(()) = self.save_file() {
                                    self.error = "Failed to save action.".to_owned();
                                    error_modal.open();
                                }
                            }
                        }
                        if ui.button("Save as...").clicked() {
                            ui.close_menu();
                            let mut dialog = FileDialog::save_file(None);
                            dialog.open();
                            self.file_dialog = Some((FileDialogAction::SaveAs, dialog));
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                        self.close_file();
                        about_modal.open();
                    }
                    ui.hyperlink_to("GitHub", "https://github.com/lilopkins/testangel");
                });
            });
        });

        // Render content
        if self.open_action.is_some() {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.menu_button("+ Add instruction", |ui| self.add_instruction_context(ui, 0));
                let mut index = 0;
                for mut ic in self.open_action.as_ref().unwrap().instructions.clone() /* TODO: Don't clone here */ {
                    // add ui for instruction
                    let instruction = self.get_instruction(ic.instruction_id.clone());
                    let heading = if instruction.is_none() {
                        self.error = "An instruction contained within this action can no longer be found.".to_owned();
                        error_modal.open();
                        ic.instruction_id
                    } else {
                        instruction.as_ref().unwrap().friendly_name().clone()
                    };

                    ui.collapsing(heading, |ui| {
                        if instruction.is_none() { return; }
                        let inst = instruction.unwrap();

                        ui.label(inst.description());
                        ui.separator();
                        ui.label("Parameters:");
                        egui::Grid::new(format!("{}_{}_param_grid", index, inst.id()))
                            .num_columns(3)
                            .show(ui, |ui| {
                                for (param_id, (param_name, _param_kind)) in inst.parameters() {
                                    ui.label(param_name);
                                    ComboBox::new(format!("{index}_{}_{param_id}", inst.id()), "Source")
                                        .show_ui(ui, |ui| {
                                            ui.selectable_value(ic.parameter_sources.get_mut(param_id).unwrap(), ParameterSource::Literal, "Literal");
                                            ui.selectable_value(ic.parameter_sources.get_mut(param_id).unwrap(), ParameterSource::FromOutput, "From output of another instruction");
                                        });
                                    if *ic.parameter_sources.get(param_id).unwrap() == ParameterSource::Literal {
                                        // TODO: Match input type depending on type
                                        ui.text_edit_singleline(&mut "text");
                                    } else {
                                        // TODO: Combo box of possible sources
                                    }
                                    ui.end_row();
                                }
                            });

                        ui.label("Outputs:");
                    });
                    index += 1;
                    ui.menu_button("+ Add instruction", |ui| self.add_instruction_context(ui, index));
                }
            });
        }
    }
}
