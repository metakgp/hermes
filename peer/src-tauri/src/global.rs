use std::{fs, path::PathBuf, sync::LazyLock};

pub(super) static APP_DATA_DIR: LazyLock<PathBuf> = LazyLock::new(|| {
    // use env var HEREMS_DATA_DIR if set, otherwise use default data directory
    let data_dir = std::env::var("HERMES_DATA_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| dirs::data_local_dir().unwrap_or_else(|| PathBuf::from(".")));

    let app_data_dir = data_dir.join("hermes");

    if !app_data_dir.exists() {
        fs::create_dir(&app_data_dir).unwrap();
    }

    app_data_dir
});
