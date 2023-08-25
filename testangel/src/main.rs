#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

mod ui;

fn main() {
    pretty_env_logger::init();
    ui::initialise_ui();
}
