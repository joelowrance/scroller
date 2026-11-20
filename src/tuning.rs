//! Every gameplay number, read from `assets/tuning.ron`.
//!
//! The file is compiled into the binary, so the web build always has the
//! committed values. On desktop the file on disk is also read at startup and
//! re-read whenever it changes, so numbers can be tuned while the game runs.

use bevy::prelude::*;
use serde::Deserialize;

const EMBEDDED: &str = include_str!("../assets/tuning.ron");

#[derive(Resource, Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Tuning {
    pub starting_lives: u32,
    pub max_lives: u32,
    pub first_extra_life: u64,
    pub extra_life_every: u64,
    /// Seconds of idle time before Attract Mode flips between panels.
    pub attract_panel_seconds: f32,
    /// Length of a Cabinet Transition (placeholder until each is built).
    pub transition_seconds: f32,
    /// Enables developer keys (K: lose a life, P: +1000 points, C: clear Stage).
    pub debug_keys: bool,
    pub placeholder: PlaceholderTuning,
}

/// Values for the placeholder Cabinet used until the real ones are built.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PlaceholderTuning {
    pub stage_seconds: f32,
    pub hero_speed: f32,
    pub scroll_speed: f32,
}

impl Default for Tuning {
    fn default() -> Self {
        Self {
            starting_lives: 3,
            max_lives: 5,
            first_extra_life: 20_000,
            extra_life_every: 50_000,
            attract_panel_seconds: 8.0,
            transition_seconds: 3.0,
            debug_keys: true,
            placeholder: PlaceholderTuning::default(),
        }
    }
}

impl Default for PlaceholderTuning {
    fn default() -> Self {
        Self {
            stage_seconds: 20.0,
            hero_speed: 90.0,
            scroll_speed: 40.0,
        }
    }
}

impl Tuning {
    pub fn parse(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }
}

pub struct TuningPlugin;

impl Plugin for TuningPlugin {
    fn build(&self, app: &mut App) {
        let tuning = Tuning::parse(EMBEDDED).expect("embedded assets/tuning.ron must be valid");
        app.insert_resource(tuning);

        #[cfg(not(target_arch = "wasm32"))]
        app.init_resource::<live::TuningWatch>()
            .add_systems(Update, live::reload_tuning);
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod live {
    use std::path::PathBuf;
    use std::time::SystemTime;

    use super::Tuning;
    use bevy::prelude::*;

    /// Tracks the tuning file on disk so edits are picked up while running.
    #[derive(Resource)]
    pub struct TuningWatch {
        path: PathBuf,
        modified: Option<SystemTime>,
        timer: Timer,
    }

    impl Default for TuningWatch {
        fn default() -> Self {
            Self {
                path: asset_root().join("tuning.ron"),
                modified: None,
                timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            }
        }
    }

    /// Mirrors how Bevy finds `assets/`: `BEVY_ASSET_ROOT`, then the crate
    /// directory under `cargo run`, then next to the executable.
    fn asset_root() -> PathBuf {
        let base = std::env::var_os("BEVY_ASSET_ROOT")
            .or_else(|| std::env::var_os("CARGO_MANIFEST_DIR"))
            .map(PathBuf::from)
            .or_else(|| {
                std::env::current_exe()
                    .ok()
                    .and_then(|exe| exe.parent().map(PathBuf::from))
            })
            .unwrap_or_default();
        base.join("assets")
    }

    pub fn reload_tuning(
        time: Res<Time<Real>>,
        mut watch: ResMut<TuningWatch>,
        mut tuning: ResMut<Tuning>,
    ) {
        let first_check = watch.modified.is_none();
        if !first_check && !watch.timer.tick(time.delta()).just_finished() {
            return;
        }
        let Ok(modified) = std::fs::metadata(&watch.path).and_then(|m| m.modified()) else {
            return;
        };
        if watch.modified == Some(modified) {
            return;
        }
        watch.modified = Some(modified);
        match std::fs::read_to_string(&watch.path)
            .map_err(|e| e.to_string())
            .and_then(|text| Tuning::parse(&text).map_err(|e| e.to_string()))
        {
            Ok(new) => {
                if !first_check {
                    info!("Reloaded {}", watch.path.display());
                }
                *tuning = new;
            }
            Err(err) => warn!("Ignoring invalid {}: {err}", watch.path.display()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_tuning_file_parses() {
        Tuning::parse(EMBEDDED).unwrap();
    }

    #[test]
    fn missing_fields_fall_back_to_defaults() {
        let t = Tuning::parse("(starting_lives: 9)").unwrap();
        assert_eq!(t.starting_lives, 9);
        assert_eq!(t.max_lives, Tuning::default().max_lives);
    }
}
