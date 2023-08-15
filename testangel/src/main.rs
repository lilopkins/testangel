#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::IconData;
use testangel::*;

mod action_loader;
mod ui;

fn main() {
    pretty_env_logger::init();

    let mut native_options = eframe::NativeOptions::default();
    native_options.icon_data =
        Some(IconData::try_from_png_bytes(include_bytes!("../../icon.png")).unwrap());
    if let Err(err) = eframe::run_native(
        "TestAngel",
        native_options,
        Box::new(|cc| Box::new(ui::App::new(cc))),
    ) {
        log::error!("Error initialising window: {err}");
    }
}
