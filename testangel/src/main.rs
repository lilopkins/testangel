#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

#[cfg(feature = "next-ui")]
mod next_ui;
#[cfg(feature = "ui")]
mod ui;

#[cfg(feature = "next-ui")]
rust_i18n::i18n!("locales", fallback = "en");

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
        log::info!("Deciding on locale...");
        let avail_locales = rust_i18n::available_locales!();
        let mut locale_is_default = true;
        for locale in sys_locale::get_locales() {
            log::info!("System offers locale: {locale}");
            if avail_locales.contains(&locale.as_ref()) {
                log::info!("This locale is available! Using: {locale}");
                rust_i18n::set_locale(&locale);
                locale_is_default = false;
                break;
            } else if let Some(lang_only) = &locale.split('-').next() {
                if avail_locales.contains(lang_only) {
                    log::info!("This language is available! Using: {lang_only}");
                    rust_i18n::set_locale(&locale);
                    locale_is_default = false;
                    break;
                }
            }
        }
        if locale_is_default {
            log::info!("No suitable locale found, using default.");
        }

        next_ui::initialise_ui();
    }
    #[cfg(feature = "ui")]
    {
        ui::initialise_ui();
    }
}
