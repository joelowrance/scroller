//! The Stage clock: game seconds played in the current Stage and how far it
//! has scrolled, for every Cabinet.
//!
//! With `debug_keys` on, the web build also writes it to localStorage as
//! `scroller.debug.stage`, so the unattended browser test
//! (`tests/web/stages.mjs`) waits on the game rather than the wall clock,
//! which a slow browser can't keep up with.

use bevy::prelude::*;

use super::timeline::StageRun;
use crate::run::Session;

/// Game seconds played in this Stage (stops while paused or in hit-stop).
#[derive(Resource, Default, Clone, Copy, serde::Serialize)]
pub struct StageClock {
    pub cabinet: usize,
    pub stage: usize,
    pub seconds: f32,
    /// Fraction of the Stage scrolled, 0 to 1.
    pub progress: f32,
    /// Seconds the Stage takes at normal speed, from its file.
    pub designed: f32,
}

/// Starts the clock again whenever a new Stage begins.
pub(super) fn start_clock(
    stage: Res<StageRun>,
    session: Option<Res<Session>>,
    mut clock: ResMut<StageClock>,
) {
    if stage.is_added() {
        let (cabinet, index) = session.map_or((0, 0), |s| (s.cabinet, s.stage));
        *clock = StageClock {
            cabinet,
            stage: index,
            designed: stage.timeline.duration(),
            ..default()
        };
    }
}

pub(super) fn tick_clock(time: Res<Time>, stage: Res<StageRun>, mut clock: ResMut<StageClock>) {
    clock.seconds += time.delta_secs();
    clock.progress = stage.progress();
}

#[cfg(target_arch = "wasm32")]
pub(super) fn report_clock(
    tuning: Res<crate::tuning::Tuning>,
    real: Res<Time<Real>>,
    clock: Res<StageClock>,
    mut every: Local<f32>,
) {
    *every -= real.delta_secs();
    if tuning.debug_keys && *every <= 0.0 {
        *every = 0.2;
        crate::storage::store("debug.stage", &*clock);
    }
}

/// The final reading as the Stage ends.
#[cfg(target_arch = "wasm32")]
pub(super) fn report_final_clock(tuning: Res<crate::tuning::Tuning>, clock: Res<StageClock>) {
    if tuning.debug_keys {
        crate::storage::store("debug.stage", &*clock);
    }
}
