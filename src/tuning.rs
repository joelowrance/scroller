//! Every gameplay number, read from `assets/tuning.ron`.
//!
//! The file is compiled into the binary, so the web build always has the
//! committed values. On desktop the file on disk is also read at startup and
//! re-read whenever it changes, so numbers can be tuned while the game runs.

use bevy::prelude::*;
use serde::Deserialize;

use crate::cabinets::side_shooter::tuning::SideShooterTuning;

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
    /// Enables developer keys (K: lose a life, P: +1000 points, C: clear Stage,
    /// F2: sprite sheet).
    pub debug_keys: bool,
    /// Debug aid (needs `debug_keys`): nothing can hit the Hero.
    pub invincible_hero: bool,
    /// Debug aid (needs `debug_keys`): game speed multiplier during a Stage.
    pub time_scale: f32,
    pub kit: KitTuning,
    pub placeholder: PlaceholderTuning,
    pub side_shooter: SideShooterTuning,
}

/// Values for the shared gameplay kit used by every Cabinet.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct KitTuning {
    /// Seconds the Hero blinks and can't be hit after a Restart Point.
    pub invulnerable_seconds: f32,
    pub hit_flash_seconds: f32,
    pub explosion_particles: u32,
    /// Fastest particle speed, pixels per second.
    pub explosion_speed: f32,
    pub explosion_seconds: f32,
    /// Shake added when the Hero is hit (1.0 is the most).
    pub shake_on_hero_down: f32,
    /// Largest camera offset at full shake, pixels.
    pub shake_max_offset: f32,
    /// Shake lost per second.
    pub shake_decay: f32,
}

/// Values for the placeholder Cabinet used until the real ones are built.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PlaceholderTuning {
    pub hero_speed: f32,
    pub shot_speed: f32,
    /// Seconds between Hero shots while A is held.
    pub fire_seconds: f32,
    pub drone_speed: f32,
    pub drone_health: u32,
    pub drone_score: u64,
    /// Seconds between shots from a gunner drone.
    pub gunner_fire_seconds: f32,
    pub enemy_shot_speed: f32,
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
            invincible_hero: false,
            time_scale: 1.0,
            kit: KitTuning::default(),
            placeholder: PlaceholderTuning::default(),
            side_shooter: SideShooterTuning::default(),
        }
    }
}

impl Default for PlaceholderTuning {
    fn default() -> Self {
        Self {
            hero_speed: 90.0,
            shot_speed: 240.0,
            fire_seconds: 0.18,
            drone_speed: 50.0,
            drone_health: 2,
            drone_score: 100,
            gunner_fire_seconds: 1.6,
            enemy_shot_speed: 90.0,
        }
    }
}

impl Default for KitTuning {
    fn default() -> Self {
        Self {
            invulnerable_seconds: 2.0,
            hit_flash_seconds: 0.06,
            explosion_particles: 10,
            explosion_speed: 70.0,
            explosion_seconds: 0.5,
            shake_on_hero_down: 0.7,
            shake_max_offset: 4.0,
            shake_decay: 1.5,
        }
    }
}

impl Tuning {
    pub fn parse(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }

    /// Applies debug-aid overrides from a URL query such as
    /// `?invincible&time_scale=8`, so the web smoke test can play a whole
    /// Stage unattended. Like the aids themselves, they need `debug_keys`.
    #[cfg_attr(not(any(target_arch = "wasm32", test)), allow(dead_code))]
    pub fn apply_query(&mut self, query: &str) {
        for pair in query.trim_start_matches('?').split('&') {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            match key {
                "invincible" => self.invincible_hero = value != "0" && value != "false",
                "gallery" => self.side_shooter.enemy_gallery = value != "0" && value != "false",
                "time_scale" => {
                    if let Ok(scale) = value.parse() {
                        self.time_scale = scale;
                    }
                }
                _ => {}
            }
        }
    }
}

pub struct TuningPlugin;

impl Plugin for TuningPlugin {
    fn build(&self, app: &mut App) {
        #[allow(unused_mut)]
        let mut tuning = Tuning::parse(EMBEDDED).expect("embedded assets/tuning.ron must be valid");
        #[cfg(target_arch = "wasm32")]
        if let Some(query) = web_sys::window().and_then(|w| w.location().search().ok()) {
            tuning.apply_query(&query);
        }
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

    #[test]
    fn url_query_sets_debug_aids() {
        let mut t = Tuning::default();
        t.apply_query("?invincible&time_scale=8&other=1");
        assert!(t.invincible_hero);
        assert_eq!(t.time_scale, 8.0);
        t.apply_query("invincible=0&time_scale=oops");
        assert!(!t.invincible_hero);
        assert_eq!(t.time_scale, 8.0);
    }
}
