#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

#[cfg(feature = "next-ui")]
mod next_ui;
#[cfg(feature = "ui")]
mod ui;

fn main() {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                chrono::Local::now(),
                record.level(),
                record.target(),
                message
            ))
        })
        .level(log::LevelFilter::Info)
        .level_for("testangel", log::LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(fern::log_file("testangel.log").expect("Couldn't open log file."))
        .apply()
        .expect("Couldn't start logger!");

    #[cfg(feature = "next-ui")]
    {
        next_ui::initialise_ui();
    }
    #[cfg(feature = "ui")]
    {
        ui::initialise_ui();
    }
}
