//! Small key/value persistence: JSON files on desktop, `localStorage` on the web.
//!
//! Used for the Save, the High Score Table, which Cabinets have been cleared
//! and the sound settings.
//! Failures are logged and otherwise ignored: losing a high score is better than
//! crashing the game.

use bevy::log::warn;
use serde::{Serialize, de::DeserializeOwned};

pub const SAVE_KEY: &str = "save";
pub const HIGH_SCORES_KEY: &str = "high_scores";
pub const PROGRESS_KEY: &str = "progress";
pub const AUDIO_KEY: &str = "audio";

pub fn load<T: DeserializeOwned>(key: &str) -> Option<T> {
    let text = backend::read(key)?;
    match serde_json::from_str(&text) {
        Ok(value) => Some(value),
        Err(err) => {
            warn!("Ignoring unreadable stored '{key}': {err}");
            None
        }
    }
}

pub fn store<T: Serialize>(key: &str, value: &T) {
    match serde_json::to_string_pretty(value) {
        Ok(text) => backend::write(key, &text),
        Err(err) => warn!("Could not encode '{key}': {err}"),
    }
}

pub fn remove(key: &str) {
    backend::delete(key);
}

pub fn exists(key: &str) -> bool {
    backend::read(key).is_some()
}

#[cfg(not(target_arch = "wasm32"))]
mod backend {
    use bevy::log::warn;
    use std::path::PathBuf;

    fn dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("scroller")
    }

    fn path(key: &str) -> PathBuf {
        dir().join(format!("{key}.json"))
    }

    pub fn read(key: &str) -> Option<String> {
        std::fs::read_to_string(path(key)).ok()
    }

    pub fn write(key: &str, text: &str) {
        // Write to a temp file and rename, so a crash never leaves half a file.
        let result = std::fs::create_dir_all(dir()).and_then(|_| {
            let tmp = path(&format!("{key}.tmp"));
            std::fs::write(&tmp, text)?;
            std::fs::rename(tmp, path(key))
        });
        if let Err(err) = result {
            warn!("Could not store '{key}': {err}");
        }
    }

    pub fn delete(key: &str) {
        let _ = std::fs::remove_file(path(key));
    }
}

#[cfg(target_arch = "wasm32")]
mod backend {
    use bevy::log::warn;

    fn storage() -> Option<web_sys::Storage> {
        web_sys::window()?.local_storage().ok().flatten()
    }

    fn item(key: &str) -> String {
        format!("scroller.{key}")
    }

    pub fn read(key: &str) -> Option<String> {
        storage()?.get_item(&item(key)).ok().flatten()
    }

    pub fn write(key: &str, text: &str) {
        let ok = storage().is_some_and(|s| s.set_item(&item(key), text).is_ok());
        if !ok {
            warn!("Could not store '{key}' in localStorage");
        }
    }

    pub fn delete(key: &str) {
        if let Some(s) = storage() {
            let _ = s.remove_item(&item(key));
        }
    }
}
