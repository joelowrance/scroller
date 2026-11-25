//! The Standoff clock the on-foot Cabinets share (docs/cabinets/3-run-and-gun.md
//! and 4-climber.md, "Standoffs"): the scroll stops and waves come on their
//! own clock. Clearing every wave early pays a time bonus and the Standoff's
//! drop; at the time-out the survivors leave. Each Cabinet runs it with its
//! own systems, since what the waves spawn is its own.
//!
//! ```ron
//! (at: 3600.0, kind: "standoff", pos: (0.0, 0.0), params: {
//!     "max_seconds": 14.0, "drop": "G",
//!     "waves": [{"after": 0.5, "kind": "skitter", "pos": (176.0, -88.0), "params": {"size": 4}}],
//! }),
//! ```

use bevy::prelude::*;
use serde::Deserialize;

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
    /// The letter of the Power-up it drops when cleared.
    pub drop: Option<char>,
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
                .and_then(|d| d.chars().next())
                .map(|c| c.to_ascii_uppercase()),
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

    /// The spawn request for one wave, marked as a member of this Standoff.
    pub fn request(wave: Wave) -> crate::kit::SpawnRequested {
        let mut params = wave.params;
        params.insert(
            ron::Value::String(MEMBER_PARAM.into()),
            ron::Value::Bool(true),
        );
        crate::kit::SpawnRequested {
            kind: wave.kind,
            pos: Vec2::new(wave.pos.0, wave.pos.1),
            params,
        }
    }
}

/// Seconds a Stage's Standoffs hold the scroll when each runs its full
/// length.
pub fn standoff_seconds(timeline: &crate::kit::StageTimeline) -> f32 {
    timeline
        .spawns
        .iter()
        .filter(|s| s.kind == KIND)
        .filter_map(|s| {
            ron::Value::Map(s.params.clone())
                .into_rust::<StandoffParams>()
                .ok()
        })
        .map(|p| p.max_seconds.max(0.0))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn standoff() -> Standoff {
        let params: StandoffParams = ron::from_str(
            r#"(max_seconds: 14.0, drop: "g", waves: [
                (after: 3.0, kind: "skitter", pos: (-176.0, -88.0)),
                (after: 0.5, kind: "skitter", pos: (176.0, -88.0), params: {"size": 4}),
            ])"#,
        )
        .unwrap();
        Standoff::new(params)
    }

    #[test]
    fn waves_come_on_their_own_clock() {
        let mut s = standoff();
        assert_eq!(s.drop, Some('G'));
        assert!(s.tick(0.4, 0).0.is_empty());
        let (due, end) = s.tick(0.2, 0);
        assert_eq!(due.len(), 1);
        assert_eq!(due[0].pos.0, 176.0, "sorted by time");
        assert_eq!(end, None, "a wave is still to come");
        let (due, _) = s.tick(2.5, 4);
        assert_eq!(due[0].pos.0, -176.0);
        let request = Standoff::request(due[0].clone());
        assert_eq!(
            request.params.get(&ron::Value::String(MEMBER_PARAM.into())),
            Some(&ron::Value::Bool(true))
        );
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
