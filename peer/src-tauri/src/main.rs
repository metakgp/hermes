// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::EnvFilter;

fn setup_logging() -> Option<WorkerGuard> {
    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_writer(std::io::stdout)
            .with_line_number(true)
            .with_env_filter(EnvFilter::from_default_env())
            .init();
        None
    } else {
        let (writer, _guard) = {
            let file_appender = tracing_appender::rolling::daily("logs", "hermes.log");
            tracing_appender::non_blocking(file_appender)
        };

        tracing_subscriber::fmt()
            .with_writer(writer)
            .with_env_filter(EnvFilter::from_default_env())
            .with_ansi(false)
            .with_line_number(true)
            .json()
            .init();
        Some(_guard)
    }
}


fn main() {
    let _guard = setup_logging();
    hermes_lib::run()
}
