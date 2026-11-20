//! A Stage described as data: how fast it scrolls, what spawns where, its
//! Restart Points and where it ends.
//!
//! Stage files are RON under `assets/stages/`. Every distance is a scroll
//! position: how far the Stage has scrolled since it began, in pixels.
//!
//! ```ron
//! (
//!     length: 800.0,
//!     speed: [(from: 0.0, speed: 40.0), (from: 400.0, speed: 60.0)],
//!     restart_points: [(at: 0.0, hero: (-110.0, 0.0)), (at: 400.0, hero: (-110.0, 0.0))],
//!     spawns: [(at: 40.0, kind: "drone", pos: (176.0, 30.0))],
//! )
//! ```
//!
//! Scroll speed is keyed by scroll position rather than by seconds, so a
//! rewind to a Restart Point always replays the Stage at the same speeds.
//! Spawn and Hero positions are canvas coordinates (origin at the centre).

use bevy::prelude::*;
use serde::Deserialize;

/// A Stage's layout, loaded from a RON file.
#[derive(Clone, Debug, Deserialize)]
pub struct StageTimeline {
    /// Scroll position at which the Stage is cleared.
    pub length: f32,
    /// Scroll speed (pixels per second) from each position onward.
    pub speed: Vec<SpeedKey>,
    #[serde(default)]
    pub restart_points: Vec<RestartPoint>,
    #[serde(default)]
    pub spawns: Vec<SpawnEntry>,
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct SpeedKey {
    pub from: f32,
    pub speed: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct RestartPoint {
    pub at: f32,
    /// Where the Hero reappears.
    #[serde(default)]
    pub hero: (f32, f32),
}

impl RestartPoint {
    pub fn hero_pos(&self) -> Vec2 {
        Vec2::new(self.hero.0, self.hero.1)
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct SpawnEntry {
    pub at: f32,
    /// What to spawn; the Cabinet decides what each kind means.
    pub kind: String,
    pub pos: (f32, f32),
}

impl StageTimeline {
    /// Parses a Stage file, sorting its entries by position and making sure
    /// there is a Restart Point at the very start.
    pub fn parse(text: &str) -> Result<Self, String> {
        let mut timeline: Self = ron::from_str(text).map_err(|e| e.to_string())?;
        if timeline.length <= 0.0 {
            return Err("length must be positive".into());
        }
        if timeline.speed.is_empty() {
            return Err("speed needs at least one key".into());
        }
        let by_pos = |a: f32, b: f32| a.total_cmp(&b);
        timeline.speed.sort_by(|a, b| by_pos(a.from, b.from));
        timeline.restart_points.sort_by(|a, b| by_pos(a.at, b.at));
        // Stable sort keeps same-position spawns in file order.
        timeline.spawns.sort_by(|a, b| by_pos(a.at, b.at));
        if timeline.restart_points.first().is_none_or(|r| r.at > 0.0) {
            timeline.restart_points.insert(
                0,
                RestartPoint {
                    at: 0.0,
                    hero: (0.0, 0.0),
                },
            );
        }
        Ok(timeline)
    }

    /// Scroll speed at a scroll position.
    pub fn speed_at(&self, position: f32) -> f32 {
        self.speed
            .iter()
            .rev()
            .find(|k| k.from <= position)
            .unwrap_or(&self.speed[0])
            .speed
    }

    /// The latest Restart Point at or before a scroll position.
    pub fn restart_point_for(&self, position: f32) -> RestartPoint {
        *self
            .restart_points
            .iter()
            .rev()
            .find(|r| r.at <= position)
            .unwrap_or(&self.restart_points[0])
    }
}

/// How far a Stage has got through its timeline.
#[derive(Clone, Debug, Default)]
pub struct TimelineCursor {
    pub position: f32,
    /// Index of the first spawn not yet fired.
    next_spawn: usize,
    cleared: bool,
}

/// What happened during one [`TimelineCursor::advance`].
#[derive(Debug, Default, PartialEq)]
pub struct Step {
    /// Indices into [`StageTimeline::spawns`] that fired.
    pub spawns: std::ops::Range<usize>,
    /// True on the one step that reached the end.
    pub cleared: bool,
}

impl TimelineCursor {
    pub fn is_cleared(&self) -> bool {
        self.cleared
    }

    /// Scrolls forward by `dt` seconds, firing any spawns passed.
    pub fn advance(&mut self, timeline: &StageTimeline, dt: f32) -> Step {
        if self.cleared {
            return Step::default();
        }
        let speed = timeline.speed_at(self.position).max(0.0);
        self.position = (self.position + speed * dt).min(timeline.length);
        let first = self.next_spawn;
        while timeline
            .spawns
            .get(self.next_spawn)
            .is_some_and(|s| s.at <= self.position)
        {
            self.next_spawn += 1;
        }
        self.cleared = self.position >= timeline.length;
        Step {
            spawns: first..self.next_spawn,
            cleared: self.cleared,
        }
    }

    /// Ends the Stage now. Returns false if it had already ended.
    pub fn force_clear(&mut self) -> bool {
        !std::mem::replace(&mut self.cleared, true)
    }

    /// Goes back to the latest Restart Point reached, so every spawn from
    /// there on fires again. Does nothing once the Stage is cleared.
    pub fn rewind(&mut self, timeline: &StageTimeline) -> RestartPoint {
        let point = timeline.restart_point_for(self.position);
        if !self.cleared {
            self.position = point.at;
            self.next_spawn = timeline.spawns.partition_point(|s| s.at < point.at);
        }
        point
    }
}

/// The Stage being played. A Cabinet inserts it when the Stage begins; the
/// kit scrolls it, fires its spawns, rewinds it and clears it.
#[derive(Resource, Clone, Debug)]
pub struct StageRun {
    pub timeline: StageTimeline,
    pub cursor: TimelineCursor,
}

impl StageRun {
    pub fn new(timeline: StageTimeline) -> Self {
        Self {
            timeline,
            cursor: TimelineCursor::default(),
        }
    }

    /// Fraction of the Stage scrolled, 0 to 1.
    pub fn progress(&self) -> f32 {
        (self.cursor.position / self.timeline.length).clamp(0.0, 1.0)
    }

    pub fn speed(&self) -> f32 {
        self.timeline.speed_at(self.cursor.position)
    }

    /// Where the Hero starts the Stage.
    pub fn start_point(&self) -> RestartPoint {
        self.timeline.restart_point_for(0.0)
    }
}

/// The timeline asks the Cabinet to spawn something.
#[derive(Message, Clone, Debug)]
pub struct SpawnRequested {
    pub kind: String,
    pub pos: Vec2,
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAGE: &str = r#"(
        length: 100.0,
        speed: [(from: 50.0, speed: 20.0), (from: 0.0, speed: 10.0)],
        restart_points: [(at: 40.0, hero: (1.0, 2.0))],
        spawns: [
            (at: 60.0, kind: "c", pos: (0.0, 0.0)),
            (at: 0.0, kind: "a", pos: (5.0, 6.0)),
            (at: 40.0, kind: "b", pos: (0.0, 0.0)),
        ],
    )"#;

    fn stage() -> StageTimeline {
        StageTimeline::parse(STAGE).unwrap()
    }

    fn kinds(t: &StageTimeline, step: &Step) -> Vec<String> {
        t.spawns[step.spawns.clone()]
            .iter()
            .map(|s| s.kind.clone())
            .collect()
    }

    #[test]
    fn parse_sorts_and_adds_a_start_restart_point() {
        let t = stage();
        assert_eq!(
            kinds(
                &t,
                &Step {
                    spawns: 0..3,
                    cleared: false
                }
            ),
            ["a", "b", "c"]
        );
        assert_eq!(t.restart_points[0].at, 0.0);
        assert_eq!(t.restart_points[1].hero_pos(), Vec2::new(1.0, 2.0));
        assert_eq!(t.speed_at(0.0), 10.0);
        assert_eq!(t.speed_at(49.0), 10.0);
        assert_eq!(t.speed_at(50.0), 20.0);
    }

    #[test]
    fn parse_rejects_bad_stages() {
        assert!(StageTimeline::parse("(length: 0.0, speed: [(from: 0.0, speed: 1.0)])").is_err());
        assert!(StageTimeline::parse("(length: 10.0, speed: [])").is_err());
        assert!(StageTimeline::parse("not ron").is_err());
    }

    #[test]
    fn each_spawn_fires_exactly_once() {
        let t = stage();
        let mut c = TimelineCursor::default();
        let mut fired = Vec::new();
        for _ in 0..1000 {
            let step = c.advance(&t, 0.05);
            fired.extend(kinds(&t, &step));
        }
        assert_eq!(fired, ["a", "b", "c"]);
    }

    #[test]
    fn spawn_at_zero_fires_on_the_first_step() {
        let t = stage();
        let mut c = TimelineCursor::default();
        assert_eq!(kinds(&t, &c.advance(&t, 0.0)), ["a"]);
    }

    #[test]
    fn scroll_uses_the_speed_for_the_current_position() {
        let t = stage();
        let mut c = TimelineCursor::default();
        c.advance(&t, 5.0); // 10 px/s
        assert_eq!(c.position, 50.0);
        c.advance(&t, 1.0); // 20 px/s
        assert_eq!(c.position, 70.0);
    }

    #[test]
    fn clear_fires_once_at_the_end() {
        let t = stage();
        let mut c = TimelineCursor::default();
        let clears = (0..1000).filter(|_| c.advance(&t, 0.1).cleared).count();
        assert_eq!(clears, 1);
        assert_eq!(c.position, t.length);
        assert!(c.is_cleared());
        assert!(!c.force_clear());
    }

    #[test]
    fn force_clear_stops_the_timeline() {
        let t = stage();
        let mut c = TimelineCursor::default();
        assert!(c.force_clear());
        assert_eq!(c.advance(&t, 100.0), Step::default());
    }

    #[test]
    fn rewind_before_any_restart_point_goes_to_the_start() {
        let t = stage();
        let mut c = TimelineCursor::default();
        c.advance(&t, 3.0); // position 30: "a" fired
        let point = c.rewind(&t);
        assert_eq!(point.at, 0.0);
        assert_eq!(c.position, 0.0);
        assert_eq!(kinds(&t, &c.advance(&t, 0.0)), ["a"]);
    }

    #[test]
    fn rewind_goes_to_the_latest_restart_point_and_replays_its_spawns() {
        let t = stage();
        let mut c = TimelineCursor::default();
        c.advance(&t, 5.0); // 50: a, b
        c.advance(&t, 1.0); // 70: c
        let point = c.rewind(&t);
        assert_eq!(point.at, 40.0);
        assert_eq!(point.hero_pos(), Vec2::new(1.0, 2.0));
        assert_eq!(c.position, 40.0);
        // "b" sits exactly on the Restart Point, so it fires again; "a" does not.
        let mut fired = kinds(&t, &c.advance(&t, 0.0));
        for _ in 0..100 {
            fired.extend(kinds(&t, &c.advance(&t, 0.1)));
        }
        assert_eq!(fired, ["b", "c"]);
    }

    #[test]
    fn rewind_after_clear_changes_nothing() {
        let t = stage();
        let mut c = TimelineCursor::default();
        c.advance(&t, 100.0);
        c.rewind(&t);
        assert_eq!(c.position, t.length);
        assert!(c.is_cleared());
    }
}
