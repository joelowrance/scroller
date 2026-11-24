//! Which song each Stage plays, for every Cabinet, in one table.
//!
//! Each Cabinet's design doc ("Sound and music") asks for one track per Stage,
//! and for the second Stage a change in its last 20 s of scroll (Cabinet 1
//! climbs, Cabinet 2 builds like an alarm). The choice is made every frame
//! from the scroll left, so a rewind to a Restart Point before the run-out
//! brings the Stage's main track back.

use bevy::prelude::*;

use super::{Song, StageMusic};
use crate::flow::Screen;
use crate::kit::StageRun;
use crate::run::Session;

/// The second Stage switches to its run-out music when this many seconds of
/// scroll are left (both design docs say "the last 20 s").
pub const RUN_OUT_SECONDS: f32 = 20.0;

/// The song for a Stage, given the seconds of scroll left. `None` for a
/// Cabinet that has no music yet.
pub fn stage_song(cabinet: usize, stage: usize, seconds_left: f32) -> Option<Song> {
    let run_out = stage == 1 && seconds_left <= RUN_OUT_SECONDS;
    Some(match (cabinet, stage) {
        (0, 0) => Song::LowTide,
        (0, _) if run_out => Song::FoundryClimb,
        (0, _) => Song::Foundry,
        (1, 0) => Song::Cloudline,
        (1, _) if run_out => Song::OuterYardAlarm,
        (1, _) => Song::OuterYard,
        (2, 0) => Song::Wreckfall,
        (2, _) if run_out => Song::RootsClimb,
        (2, _) => Song::Roots,
        _ => return None,
    })
}

/// Seconds until the scroll ends, following the Stage's speed keys, so a
/// speed change in the last stretch can't flip the song back and forth.
pub fn seconds_left(run: &StageRun) -> f32 {
    let timeline = &run.timeline;
    let end = timeline.length;
    let mut at = run.cursor.position.min(end);
    let mut seconds = 0.0;
    while at < end {
        // The next speed key after `at` ends this stretch.
        let next = timeline
            .speed
            .iter()
            .map(|k| k.from)
            .filter(|&from| from > at)
            .fold(end, f32::min);
        seconds += (next - at) / timeline.speed_at(at).max(1.0);
        at = next;
    }
    seconds
}

fn choose_stage_music(
    session: Res<Session>,
    run: Option<Res<StageRun>>,
    mut music: ResMut<StageMusic>,
) {
    let left = run.map_or(f32::INFINITY, |r| seconds_left(&r));
    music.set_if_neq(StageMusic(stage_song(session.cabinet, session.stage, left)));
}

pub struct StageMusicPlugin;

impl Plugin for StageMusicPlugin {
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
    fn each_stage_has_its_song_and_the_second_changes_at_the_end() {
        assert_eq!(stage_song(0, 0, 5.0), Some(Song::LowTide));
        assert_eq!(stage_song(0, 1, 60.0), Some(Song::Foundry));
        assert_eq!(stage_song(0, 1, 20.0), Some(Song::FoundryClimb));
        assert_eq!(stage_song(1, 0, 5.0), Some(Song::Cloudline));
        assert_eq!(stage_song(1, 1, 60.0), Some(Song::OuterYard));
        assert_eq!(stage_song(1, 1, 0.0), Some(Song::OuterYardAlarm));
        assert_eq!(stage_song(2, 0, 5.0), Some(Song::Wreckfall));
        assert_eq!(stage_song(2, 1, 60.0), Some(Song::Roots));
        assert_eq!(stage_song(2, 1, 20.0), Some(Song::RootsClimb));
        assert_eq!(stage_song(3, 0, 60.0), None);
    }

    #[test]
    fn seconds_left_follows_the_speed_keys() {
        let timeline = StageTimeline::parse(
            "(length: 1000.0, speed: [(from: 0.0, speed: 30.0), (from: 600.0, speed: 40.0), \
             (from: 900.0, speed: 20.0)])",
        )
        .unwrap();
        let mut run = StageRun::new(timeline);
        run.cursor.position = 300.0;
        assert_eq!(seconds_left(&run), 10.0 + 7.5 + 5.0);
        run.cursor.position = 800.0;
        assert_eq!(seconds_left(&run), 2.5 + 5.0);
        run.cursor.position = 1000.0;
        assert_eq!(seconds_left(&run), 0.0);
    }

    #[test]
    fn a_slowdown_near_the_end_does_not_undo_the_run_out() {
        // 40 px/s, then 10 px/s for the last 100 px: the estimate only falls
        // as the Stage scrolls, so the song never flips back.
        let timeline = StageTimeline::parse(
            "(length: 1000.0, speed: [(from: 0.0, speed: 40.0), (from: 900.0, speed: 10.0)])",
        )
        .unwrap();
        let mut run = StageRun::new(timeline);
        let mut last = f32::INFINITY;
        for step in 0..=100 {
            run.cursor.position = step as f32 * 10.0;
            let left = seconds_left(&run);
            assert!(left <= last);
            last = left;
        }
    }
}
