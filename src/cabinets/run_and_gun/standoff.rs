//! Standoffs (docs/cabinets/3-run-and-gun.md, "Standoffs"): the scroll stops
//! and waves come from both sides on their own clock. Clearing every wave
//! early pays a time bonus and the Standoff's drop; at the time-out the
//! survivors leave the way they came.
//!
//! ```ron
//! (at: 3600.0, kind: "standoff", pos: (0.0, 0.0), params: {
//!     "max_seconds": 14.0, "drop": "G",
//!     "waves": [{"after": 0.5, "kind": "skitter", "pos": (176.0, -88.0), "params": {"size": 4}}],
//! }),
//! ```

use bevy::prelude::*;
use serde::Deserialize;

use super::enemies::{Leaving, StandoffMember};
use super::power_ups::PowerUp;
use crate::cabinets::ScoreGained;
use crate::kit::{SpawnRequested, StageRestarted};
use crate::tuning::Tuning;

/// The spawn kind of a Standoff.
pub const KIND: &str = "standoff";

/// The param that marks a wave's enemies as part of the running Standoff.
pub const MEMBER_PARAM: &str = "standoff";

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Wave {
    pub after: f32,
    pub kind: String,
    pub pos: (f32, f32),
    #[serde(default)]
    pub params: ron::Map,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StandoffParams {
    pub max_seconds: f32,
    #[serde(deserialize_with = "present")]
    pub drop: Option<String>,
    pub waves: Vec<Wave>,
}

/// Reads an optional param written plainly (`"drop": "G"`).
fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}

/// The running Standoff, while there is one.
#[derive(Resource, Clone, Debug)]
pub struct Standoff {
    pub elapsed: f32,
    pub max_seconds: f32,
    pub drop: Option<PowerUp>,
    pub waves: Vec<Wave>,
    /// Waves already sent.
    pub sent: usize,
}

/// How a Standoff ended.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Outcome {
    /// Every enemy destroyed: whole seconds left, for the bonus.
    Cleared {
        seconds_left: u32,
    },
    TimedOut,
}

impl Standoff {
    pub fn new(params: StandoffParams) -> Self {
        let mut waves = params.waves;
        waves.sort_by(|a, b| a.after.total_cmp(&b.after));
        Self {
            elapsed: 0.0,
            max_seconds: params.max_seconds.max(0.0),
            drop: params
                .drop
                .and_then(|d| d.chars().next().and_then(PowerUp::from_letter)),
            waves,
            sent: 0,
        }
    }

    pub fn active(&self) -> bool {
        true
    }

    /// Advances the clock; returns the waves due now, and how it ended if it
    /// did. `members_alive` counts the enemies its waves spawned.
    pub fn tick(&mut self, dt: f32, members_alive: usize) -> (Vec<Wave>, Option<Outcome>) {
        self.elapsed += dt;
        let mut due = Vec::new();
        while self
            .waves
            .get(self.sent)
            .is_some_and(|w| w.after <= self.elapsed)
        {
            due.push(self.waves[self.sent].clone());
            self.sent += 1;
        }
        let outcome = if self.sent == self.waves.len() && due.is_empty() && members_alive == 0 {
            let left = (self.max_seconds - self.elapsed).max(0.0);
            Some(Outcome::Cleared {
                seconds_left: left.floor() as u32,
            })
        } else if self.elapsed >= self.max_seconds {
            Some(Outcome::TimedOut)
        } else {
            None
        };
        (due, outcome)
    }
}

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
) {
    let Some(mut standoff) = standoff else {
        return;
    };
    let (due, outcome) = standoff.tick(time.delta_secs(), members.iter().count());
    for wave in due {
        let mut params = wave.params.clone();
        params.insert(
            ron::Value::String(MEMBER_PARAM.into()),
            ron::Value::Bool(true),
        );
        spawns.write(SpawnRequested {
            kind: wave.kind,
            pos: Vec2::new(wave.pos.0, wave.pos.1),
            params,
        });
    }
    match outcome {
        Some(Outcome::Cleared { seconds_left }) => {
            let bonus = seconds_left as u64 * tuning.run_and_gun.score.standoff_time_bonus;
            if bonus > 0 {
                score.write(ScoreGained(bonus));
            }
            if let Some(drop) = standoff.drop {
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

    fn standoff() -> Standoff {
        let params: StandoffParams = ron::from_str(
            r#"(max_seconds: 14.0, drop: "G", waves: [
                (after: 3.0, kind: "skitter", pos: (-176.0, -88.0)),
                (after: 0.5, kind: "skitter", pos: (176.0, -88.0), params: {"size": 4}),
            ])"#,
        )
        .unwrap();
        Standoff::new(params)
    }

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
        assert_eq!(standoff.drop, Some(PowerUp::Hustle));
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

    #[test]
    fn waves_come_on_their_own_clock() {
        let mut s = standoff();
        assert_eq!(s.drop, Some(PowerUp::Gun));
        assert!(s.tick(0.4, 0).0.is_empty());
        let (due, end) = s.tick(0.2, 0);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].pos.0, 176.0, "sorted by time");
        assert_eq!(end, None, "a wave is still to come");
        let (due, _) = s.tick(2.5, 4);
        assert_eq!(due[0].pos.0, -176.0);
    }

    #[test]
    fn clearing_early_pays_whole_seconds_left() {
        let mut s = standoff();
        s.tick(3.1, 3);
        let (_, end) = s.tick(2.0, 0);
        assert_eq!(end, Some(Outcome::Cleared { seconds_left: 8 }), "14 - 5.1");
    }

    #[test]
    fn it_times_out_with_enemies_left() {
        let mut s = standoff();
        s.tick(5.0, 3);
        let (_, end) = s.tick(9.5, 1);
        assert_eq!(end, Some(Outcome::TimedOut));
    }
}
