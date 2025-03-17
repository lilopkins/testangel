#![cfg_attr(
    all(not(debug_assertions), not(feature = "windows-keep-console-window")),
    windows_subsystem = "windows"
)]

use std::{env, path::PathBuf, sync::Mutex};

use relm4::tokio::runtime;
use testangel::version;
use tracing_subscriber_multi::{AnsiStripper, AppendCount, Compression, ContentLimit, DualWriter, FmtSubscriber, RotatingFile};

#[cfg(feature = "ui")]
mod ui;

fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_max_level(
            if cfg!(debug_assertions) || env::var("TA_DEBUG").is_ok_and(|v| !v.is_empty()) {
                tracing::Level::TRACE
            } else {
                tracing::Level::INFO
            },
        )
        .with_ansi(true)
        .with_writer(Mutex::new(DualWriter::new(
            std::io::stderr(),
            AnsiStripper::new(RotatingFile::new(
                env::current_exe()
                    .ok()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .unwrap_or(PathBuf::from("."))
                    .join("testangel.log"),
                AppendCount::new(3),
                ContentLimit::Lines(1000),
                Compression::OnRotate(0),
            )),
        )))
        .finish();
    tracing::subscriber::set_global_default(subscriber).expect("failed to initialise logger");

    tracing::info!("Using locale: {}", ui::lang::initialise_i18n());

    if let Ok(rt) = runtime::Builder::new_current_thread().enable_all().build() {
        let _is_latest = rt.block_on(version::check_is_latest());
    }

    ui::initialise_ui();
}
