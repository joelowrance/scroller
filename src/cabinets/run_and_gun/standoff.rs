//! Cabinet 3's Standoffs (docs/cabinets/3-run-and-gun.md, "Standoffs"): the
//! shared clock (`on_foot::standoff`) run with this Cabinet's enemies.

use bevy::prelude::*;

use super::enemies::{Leaving, StandoffMember};
use super::power_ups::PowerUp;
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
pub use crate::cabinets::on_foot::standoff::{KIND, Outcome, Standoff, StandoffParams};
use crate::kit::{SpawnRequested, StageRestarted};
use crate::tuning::Tuning;

/// Runs the Standoff: sends its waves, and ends it.
#[allow(clippy::too_many_arguments)]
pub(super) fn run_standoff(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    standoff: Option<ResMut<Standoff>>,
    members: Query<Entity, With<StandoffMember>>,
    hero: Single<&Transform, With<crate::kit::Hero>>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut score: MessageWriter<ScoreGained>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let Some(mut standoff) = standoff else {
        return;
    };
    let (due, outcome) = standoff.tick(time.delta_secs(), members.iter().count());
    for wave in due {
        spawns.write(Standoff::request(wave));
    }
    if outcome.is_some() {
        sfx.write(PlaySfx(Sfx::StandoffRelease));
    }
    match outcome {
        Some(Outcome::Cleared { seconds_left }) => {
            let bonus = seconds_left as u64 * tuning.run_and_gun.score.standoff_time_bonus;
            if bonus > 0 {
                score.write(ScoreGained(bonus));
            }
            if let Some(drop) = standoff.drop.and_then(PowerUp::from_letter) {
                let at = hero.translation.truncate() + Vec2::new(24.0, 48.0);
                spawns.write(SpawnRequested::new(drop.kind(), at));
            }
            commands.remove_resource::<Standoff>();
        }
        Some(Outcome::TimedOut) => {
            for e in &members {
                commands
                    .entity(e)
                    .try_insert(Leaving)
                    .try_remove::<StandoffMember>();
            }
            commands.remove_resource::<Standoff>();
        }
        None => {}
    }
}

/// A rewind goes back to before the Standoff, which replays from the start.
pub(super) fn end_on_restart(mut commands: Commands, mut restarted: MessageReader<StageRestarted>) {
    if restarted.read().count() > 0 {
        commands.remove_resource::<Standoff>();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// As a Stage file writes it, read through the timeline's params.
    #[test]
    fn a_stage_files_standoff_entry_parses() {
        let timeline = crate::kit::StageTimeline::parse(
            r#"(length: 100.0, speed: [(from: 0.0, speed: 10.0)], spawns: [
                (at: 10.0, kind: "standoff", pos: (0.0, 0.0), params: {
                    "max_seconds": 25.0, "drop": "H",
                    "waves": [
                        {"after": 0.5, "kind": "strider", "pos": (176.0, -88.0)},
                        {"after": 8.0, "kind": "skitter", "pos": (-176.0, -88.0), "params": {"size": 3}},
                    ],
                }),
            ])"#,
        )
        .unwrap();
        let s = &timeline.spawns[0];
        let request = SpawnRequested {
            kind: s.kind.clone(),
            pos: Vec2::ZERO,
            params: s.params.clone(),
        };
        let params: StandoffParams = request.params().unwrap();
        let standoff = Standoff::new(params);
        assert_eq!(standoff.max_seconds, 25.0);
        assert_eq!(
            standoff.drop.and_then(PowerUp::from_letter),
            Some(PowerUp::Hustle)
        );
        assert_eq!(standoff.waves.len(), 2);
        assert_eq!(standoff.waves[1].pos, (-176.0, -88.0));
    }

    #[test]
    fn the_gallery_standoff_parses() {
        let timeline = crate::cabinets::run_and_gun::enemies::gallery_timeline();
        let s = timeline.spawns.iter().find(|s| s.kind == KIND).unwrap();
        let request = SpawnRequested {
            kind: s.kind.clone(),
            pos: Vec2::ZERO,
            params: s.params.clone(),
        };
        let params: StandoffParams = request.params().unwrap();
        assert_eq!(params.waves.len(), 2);
    }
}
