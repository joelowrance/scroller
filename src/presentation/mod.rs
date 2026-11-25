//! How the game looks around the play itself: the title logo, the optional
//! CRT effect and the fades between screens. (Screen shake and hit-stop on
//! big explosions belong to the gameplay kit; see `kit::effects`.)

use bevy::prelude::*;
use serde::Deserialize;

pub mod crt;
mod fade;
pub mod logo;

/// The `presentation` section of `assets/tuning.ron`.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct PresentationTuning {
    /// Seconds each screen takes to fade in from black (0: a hard cut).
    pub fade_seconds: f32,
    /// Darkness of the CRT scanlines, 0 to 1.
    pub crt_scanline_alpha: f32,
    /// Darkness of the CRT vignette in the corners, 0 to 1.
    pub crt_vignette_alpha: f32,
}

impl Default for PresentationTuning {
    fn default() -> Self {
        Self {
            fade_seconds: 0.35,
            crt_scanline_alpha: 0.35,
            crt_vignette_alpha: 0.45,
        }
    }
}

pub struct PresentationPlugin;

impl Plugin for PresentationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((crt::plugin, fade::plugin));
    }
}
