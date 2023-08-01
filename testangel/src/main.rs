#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::PathBuf,
};

use egui::Ui;
use egui_file::FileDialog;
use egui_modal::Modal;
use ipc::Engine;
use serde::{Deserialize, Serialize};
use testangel_ipc::prelude::*;

mod ipc;

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
    open_type: OpenType,
    open_path: Option<PathBuf>,
    open_action: Option<Action>,
}

enum FileDialogAction {
    OpenAction,
    SaveAsAction,
}

#[derive(Default, PartialEq, Eq)]
enum OpenType {
    #[default]
    Nothing,
    Action,
}

#[derive(Default, Serialize, Deserialize)]
struct Action {
    instructions: Vec<Instruction>,
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
        match self.open_type {
            OpenType::Action => {
                if self.open_path.is_none() {
                    panic!("path not set");
                }
                if let Ok(data) = rmp_serde::to_vec(&self.open_action.as_ref().unwrap()) {
                    if let Ok(_) = fs::write(self.open_path.as_ref().unwrap(), data) {
                        return Ok(());
                    }
                }
            }
            _ => (),
        };
        return Err(());
    }

    fn close_file(&mut self) {
        self.open_type = OpenType::Nothing;
        self.open_action = None;
        self.open_path = None;
    }

    fn add_instruction_context(&mut self, ui: &mut Ui, index: usize) {
        for (_path, engine) in &self.engines {
            ui.menu_button(&engine.name, |ui| {
                for inst in &engine.instructions {
                    if ui.button(inst.friendly_name()).clicked() {
                        ui.close_menu();
                        self.open_action.as_mut().unwrap().instructions.insert(index, inst.clone());
                    }
                }
            });
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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

        // Handle file dialog.
        let error_modal = Modal::new(ctx, "file_error_modal");
        error_modal.show(|ui| {
            error_modal.title(ui, "Error");
            error_modal.frame(ui, |ui| {
                error_modal.body(ui, &self.error);
            });
            error_modal.buttons(ui, |ui| {
                let _ = error_modal.button(ui, "Close");
            });
        });

        if let Some((action, dialog)) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(path) = dialog.path() {
                    match action {
                        FileDialogAction::OpenAction => {
                            if let Ok(file) = File::open(path) {
                                if let Ok(action) = rmp_serde::from_read(BufReader::new(file)) {
                                    self.open_type = OpenType::Action;
                                    self.open_path = Some(path.to_path_buf());
                                    self.open_action = action;
                                } else {
                                    self.error = "Failed to open action.".to_owned();
                                    error_modal.open();
                                }
                            } else {
                                self.error = "Failed to open action.".to_owned();
                                error_modal.open();
                            }
                        }
                        FileDialogAction::SaveAsAction => {
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

        // Render UI
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
                    ui.button("New").clicked();
                    let _ = ui.button("Open...");
                    let _ = ui.button("Save");
                    let _ = ui.button("Save as...");
                });
                ui.menu_button("Actions", |ui| {
                    if ui.button("New").clicked() {
                        ui.close_menu();
                        self.close_file();
                        self.open_type = OpenType::Action;
                        self.open_action = Some(Action::default());
                    }
                    if ui.button("Open...").clicked() {
                        ui.close_menu();
                        self.close_file();
                        let mut dialog = FileDialog::open_file(None);
                        dialog.open();
                        self.file_dialog = Some((FileDialogAction::OpenAction, dialog));
                    }
                    ui.add_enabled_ui(self.open_type == OpenType::Action, |ui| {
                        if ui.button("Save").clicked() {
                            ui.close_menu();
                            self.close_file();
                            if self.open_path.is_none() {
                                let mut dialog = FileDialog::save_file(None);
                                dialog.open();
                                self.file_dialog = Some((FileDialogAction::SaveAsAction, dialog));
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
                            self.file_dialog = Some((FileDialogAction::SaveAsAction, dialog));
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

        if self.open_type == OpenType::Action {
            egui::CentralPanel::default().show(ctx, |ui| {
                ui.menu_button("+ Add instruction", |ui| self.add_instruction_context(ui, 0));
                let mut index = 0;
                for instruction in self.open_action.as_ref().unwrap().instructions.clone() {
                    // add ui for instruction
                    ui.collapsing(instruction.friendly_name(), |ui| {
                        // TODO
                        ui.label("todo: build items here");
                    });
                    index += 1;
                    ui.menu_button("+ Add instruction", |ui| self.add_instruction_context(ui, index));
                }
            });
        }
    }
}
