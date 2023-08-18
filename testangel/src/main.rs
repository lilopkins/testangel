#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use testangel::*;

mod action_loader;
mod ui;

fn main() {
    pretty_env_logger::init();
    ui::initialise_ui();
}
