//! Cabinet 1's music: which song each Stage plays (docs/cabinets/1-side-shooter.md,
//! "Sound and music"). The sound effects are sent where things happen.

use bevy::prelude::*;

use crate::audio::{Song, StageMusic};
use crate::flow::Screen;
use crate::kit::StageRun;
use crate::run::Session;

/// Cabinet 1's index in the Run.
const CABINET: usize = 0;

/// Stage 2 switches to the climbing run-out music when this many seconds of
/// scroll are left (the design's "last 20 s").
const CLIMB_SECONDS: f32 = 20.0;

/// The song for a Cabinet 1 Stage, given the seconds of scroll left.
pub fn stage_song(stage: usize, seconds_left: f32) -> Song {
    match stage {
        0 => Song::LowTide,
        _ if seconds_left <= CLIMB_SECONDS => Song::FoundryClimb,
        _ => Song::Foundry,
    }
}

/// Seconds until the scroll ends at the current speed.
fn seconds_left(run: &StageRun) -> f32 {
    let left = (run.timeline.length - run.cursor.position).max(0.0);
    left / run.speed().max(1.0)
}

/// Keeps [`StageMusic`] on the right song. It runs every frame, so a rewind
/// to a Restart Point before the run-out brings the Stage 2 music back.
fn choose_stage_music(
    session: Res<Session>,
    run: Option<Res<StageRun>>,
    mut music: ResMut<StageMusic>,
) {
    if session.cabinet != CABINET {
        return;
    }
    let left = run.map_or(f32::INFINITY, |r| seconds_left(&r));
    music.set_if_neq(StageMusic(Some(stage_song(session.stage, left))));
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            choose_stage_music
                .run_if(in_state(Screen::Playing))
                .run_if(resource_exists::<Session>),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::StageTimeline;

    #[test]
    fn each_stage_has_its_song_and_stage_2_climbs_at_the_end() {
        assert_eq!(stage_song(0, 5.0), Song::LowTide);
        assert_eq!(stage_song(1, 60.0), Song::Foundry);
        assert_eq!(stage_song(1, 20.0), Song::FoundryClimb);
        assert_eq!(stage_song(1, 0.0), Song::FoundryClimb);
    }

    #[test]
    fn seconds_left_uses_the_current_speed() {
        let timeline = StageTimeline::parse(
            "(length: 1000.0, speed: [(from: 0.0, speed: 30.0), (from: 600.0, speed: 40.0)])",
        )
        .unwrap();
        let mut run = StageRun::new(timeline);
        run.cursor.position = 200.0;
        assert_eq!(seconds_left(&run), 800.0 / 30.0);
        run.cursor.position = 800.0;
        assert_eq!(seconds_left(&run), 5.0);
    }
}
