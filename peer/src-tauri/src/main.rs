// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod global;

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

use crate::global::APP_DATA_DIR;

fn setup_logging() -> Option<WorkerGuard> {
    let subscriber = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_line_number(true);

    if cfg!(debug_assertions) {
        subscriber.init();
        None
    } else {
        let (writer, _guard) = {
            let file_appender = tracing_appender::rolling::daily(APP_DATA_DIR.join("logs"), "hermes.log");
            tracing_appender::non_blocking(file_appender)
        };

        subscriber
            .with_writer(writer)
            .with_ansi(false)
            .json()
            .init();
        Some(_guard)
    }
}


fn main() {
    let _guard = setup_logging();
    hermes_lib::run()
}
