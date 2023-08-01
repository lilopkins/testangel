#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{
    collections::HashMap,
    fs::{self, File},
    io::BufReader,
    path::{Path, PathBuf},
};

use egui::Ui;
use egui_file::FileDialog;
use egui_modal::Modal;
use ipc::Engine;
use serde::{Deserialize, Serialize};

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
    _engines: HashMap<PathBuf, Engine>,
    state: AppState,
    error: String,
    file_dialog: Option<(FileDialogAction, FileDialog)>,
}

enum FileDialogAction {
    OpenAction,
    SaveAsAction,
}

#[derive(Default)]
enum AppState {
    #[default]
    Nothing,
    ActionEdit {
        action: Action,
        path: Option<PathBuf>,
    },
    About,
}

impl AppState {
    fn is_action_edit(&self) -> bool {
        match self {
            Self::ActionEdit { action: _, path: _ } => true,
            _ => false,
        }
    }

    fn get_current_file(&self) -> &Option<PathBuf> {
        match self {
            Self::ActionEdit { action: _, path } => path,
            _ => &None,
        }
    }

    fn open_action(&mut self, path: &Path) -> Result<(), ()> {
        if let Ok(file) = File::open(path) {
            if let Ok(action) = rmp_serde::from_read(BufReader::new(file)) {
                *self = Self::ActionEdit {
                    action,
                    path: Some(path.to_path_buf()),
                };
                return Ok(());
            }
        }
        Err(())
    }

    fn set_file_path(&mut self, new_path: &Path) {
        match self {
            Self::ActionEdit { action: _, path } => *path = Some(new_path.to_path_buf()),
            _ => (),
        };
    }

    fn save(&self) -> Result<(), ()> {
        match self {
            Self::ActionEdit { action, path } => {
                if path.is_none() {
                    panic!("path not set");
                }
                if let Ok(data) = rmp_serde::to_vec(action) {
                    if let Ok(_) = fs::write(path.as_ref().unwrap(), data) {
                        return Ok(());
                    }
                }
            }
            _ => (),
        };
        return Err(());
    }
}

#[derive(Default, Serialize, Deserialize)]
struct Action {}

impl App {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self {
            _engines: ipc::get_engines(),
            ..Default::default()
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Handle file dialog.
        let error_modal = Modal::new(ctx, "file_error_modal");
        error_modal.show(|ui| {
            error_modal.title(ui, "Error");
            error_modal.frame(ui, |ui| {
                error_modal.body(ui, &self.error);
            });
            error_modal.buttons(ui, |ui| {
                let _ = error_modal.button(ui, "close");
            });
        });

        if let Some((action, dialog)) = &mut self.file_dialog {
            if dialog.show(ctx).selected() {
                if let Some(file) = dialog.path() {
                    match action {
                        FileDialogAction::OpenAction => {
                            if let Err(()) = self.state.open_action(file) {
                                self.error = "Failed to open action.".to_owned();
                                error_modal.open();
                            }
                        }
                        FileDialogAction::SaveAsAction => {
                            self.state.set_file_path(file);
                            if let Err(()) = self.state.save() {
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
                    self.state = AppState::Nothing;
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
                        self.state = AppState::ActionEdit {
                            action: Action::default(),
                            path: None,
                        }
                    }
                    if ui.button("Open...").clicked() {
                        ui.close_menu();
                        let mut dialog = FileDialog::open_file(None);
                        dialog.open();
                        self.file_dialog = Some((FileDialogAction::OpenAction, dialog));
                    }
                    ui.add_enabled_ui(self.state.is_action_edit(), |ui| {
                        if ui.button("Save").clicked() {
                            ui.close_menu();
                            if self.state.get_current_file().is_none() {
                                let mut dialog =
                                    FileDialog::save_file(None);
                                dialog.open();
                                self.file_dialog = Some((FileDialogAction::SaveAsAction, dialog));
                            } else {
                                if let Err(()) = self.state.save() {
                                    self.error = "Failed to save action.".to_owned();
                                    error_modal.open();
                                }
                            }
                        }
                        if ui.button("Save as...").clicked() {
                            ui.close_menu();
                            let mut dialog =
                                FileDialog::save_file(None);
                            dialog.open();
                            self.file_dialog = Some((FileDialogAction::SaveAsAction, dialog));
                        }
                    });
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() {
                        ui.close_menu();
                        self.state = AppState::About;
                    }
                    ui.hyperlink_to("GitHub", "https://github.com/lilopkins/testangel");
                });
            });
        });

        egui::CentralPanel::default().show(ctx,match &self.state {
            AppState::About => |ui: &mut Ui| {
                ui.heading("TestAngel");
                ui.label("by Lily Hopkins and contributors");
                ui.separator();
                ui.label("TestAngel automates testing across a number of tools by providing a standardised interface to communicate actions to perform.");
            },
            AppState::ActionEdit { action: _, path: _ } => |_ui: &mut Ui| {

            },
            AppState::Nothing => |_: &mut Ui| {},
        }
         );
    }
}
