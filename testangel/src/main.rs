#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

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

    #[cfg(feature = "ui")]
    {
        use relm4::tokio::runtime;
        use testangel::version;

        log::info!("Using locale: {}", ui::lang::initialise_i18n());

        if let Ok(rt) = runtime::Builder::new_current_thread().enable_all().build() {
            let _is_latest = rt.block_on(version::check_is_latest());
        }

        ui::initialise_ui();
    }
}
