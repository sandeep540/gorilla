use crate::model::Settings;
use std::fs;
use std::path::PathBuf;

fn settings_path() -> Option<PathBuf> {
    let base = dirs::config_dir()?;
    Some(base.join("gorilla-rust").join("settings.toml"))
}

pub fn load_settings() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    match fs::read_to_string(path) {
        Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
        Err(_) => Settings::default(),
    }
}

pub fn save_settings(settings: &Settings) {
    let Some(path) = settings_path() else {
        return;
    };
    let Some(parent) = path.parent() else {
        return;
    };
    if fs::create_dir_all(parent).is_err() {
        return;
    }
    let Ok(contents) = toml::to_string_pretty(settings) else {
        return;
    };
    let _ = fs::write(path, contents);
}
