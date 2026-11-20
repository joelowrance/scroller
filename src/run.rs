//! The rules of a Run (or a Practice session): score, lives, extra lives and
//! how clearing a Stage advances through the Cabinets.
//!
//! Pure logic with no Bevy systems, so the arcade rules can be unit-tested.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

use crate::cabinets::CABINET_COUNT;
use crate::tuning::Tuning;

/// Each Cabinet has exactly two Stages.
pub const STAGES_PER_CABINET: usize = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mode {
    /// A real Run: scores count, progress is Saved.
    Run,
    /// Practice on a single cleared Cabinet: no Save, no High Score Table.
    Practice,
}

/// The state of the current Run or Practice session.
///
/// A Save is exactly this struct, captured at the start of a Stage.
#[derive(Resource, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Session {
    pub mode: Mode,
    /// Index into the Cabinet order (0-based).
    pub cabinet: usize,
    /// Index of the Stage within the Cabinet (0 or 1).
    pub stage: usize,
    pub score: u64,
    pub lives: u32,
    /// Score at which the next extra life is awarded.
    pub next_extra_life: u64,
    /// Power-ups owned by the current Cabinet, in whatever shape it chooses.
    /// Cleared at every Cabinet Transition.
    #[serde(default)]
    pub power_ups: serde_json::Value,
}

/// What happens after the current Stage is cleared.
#[derive(Debug, PartialEq, Eq)]
pub enum Advance {
    /// Go on to the next Stage of the same Cabinet.
    NextStage,
    /// The Cabinet is cleared; play the Cabinet Transition `from` -> `from + 1`.
    NextCabinet { from: usize },
    /// The final Cabinet is cleared: show the Ending.
    Ending,
    /// A Practice session finished its Cabinet.
    PracticeComplete,
}

/// What happens after the Hero loses a life.
#[derive(Debug, PartialEq, Eq)]
pub enum LifeOutcome {
    /// Lives remain: the Hero reappears at a Restart Point.
    Restart,
    GameOver,
}

impl Session {
    pub fn new_run(tuning: &Tuning) -> Self {
        Self::fresh(Mode::Run, 0, tuning)
    }

    pub fn new_practice(cabinet: usize, tuning: &Tuning) -> Self {
        Self::fresh(Mode::Practice, cabinet, tuning)
    }

    fn fresh(mode: Mode, cabinet: usize, tuning: &Tuning) -> Self {
        Self {
            mode,
            cabinet,
            stage: 0,
            score: 0,
            lives: tuning.starting_lives,
            next_extra_life: tuning.first_extra_life,
            power_ups: serde_json::Value::Null,
        }
    }

    /// Adds points and returns how many extra lives were earned by them.
    /// Extra lives beyond `max_lives` are not stored, but thresholds still advance.
    pub fn add_score(&mut self, points: u64, tuning: &Tuning) -> u32 {
        self.score = self.score.saturating_add(points);
        let mut earned = 0;
        while self.score >= self.next_extra_life {
            earned += 1;
            self.next_extra_life = self
                .next_extra_life
                .saturating_add(tuning.extra_life_every.max(1));
        }
        let before = self.lives;
        self.lives = (self.lives + earned).min(tuning.max_lives);
        self.lives - before
    }

    pub fn lose_life(&mut self) -> LifeOutcome {
        self.lives = self.lives.saturating_sub(1);
        if self.lives == 0 {
            LifeOutcome::GameOver
        } else {
            LifeOutcome::Restart
        }
    }

    /// Advances past the current Stage and reports what comes next.
    pub fn clear_stage(&mut self) -> Advance {
        if self.stage + 1 < STAGES_PER_CABINET {
            self.stage += 1;
            return Advance::NextStage;
        }
        match self.mode {
            Mode::Practice => Advance::PracticeComplete,
            Mode::Run if self.cabinet + 1 >= CABINET_COUNT => Advance::Ending,
            Mode::Run => {
                let from = self.cabinet;
                self.cabinet += 1;
                self.stage = 0;
                self.power_ups = serde_json::Value::Null;
                Advance::NextCabinet { from }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tuning() -> Tuning {
        Tuning {
            starting_lives: 3,
            max_lives: 5,
            first_extra_life: 20_000,
            extra_life_every: 50_000,
            ..Tuning::default()
        }
    }

    #[test]
    fn extra_life_at_first_threshold_then_every_interval() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        assert_eq!(s.add_score(19_999, &t), 0);
        assert_eq!(s.add_score(1, &t), 1);
        assert_eq!(s.lives, 4);
        assert_eq!(s.next_extra_life, 70_000);
        assert_eq!(s.add_score(50_000, &t), 1);
        assert_eq!(s.next_extra_life, 120_000);
    }

    #[test]
    fn lives_cap_at_max_but_thresholds_still_advance() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        assert_eq!(s.add_score(1_000_000, &t), 2);
        assert_eq!(s.lives, 5);
        assert!(s.next_extra_life > 1_000_000);
    }

    #[test]
    fn losing_last_life_is_game_over() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        assert_eq!(s.lose_life(), LifeOutcome::Restart);
        assert_eq!(s.lose_life(), LifeOutcome::Restart);
        assert_eq!(s.lose_life(), LifeOutcome::GameOver);
    }

    #[test]
    fn run_advances_through_every_cabinet_to_the_ending() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        for cabinet in 0..CABINET_COUNT {
            assert_eq!((s.cabinet, s.stage), (cabinet, 0));
            assert_eq!(s.clear_stage(), Advance::NextStage);
            let expected = if cabinet + 1 == CABINET_COUNT {
                Advance::Ending
            } else {
                Advance::NextCabinet { from: cabinet }
            };
            assert_eq!(s.clear_stage(), expected);
        }
    }

    #[test]
    fn power_ups_survive_stages_but_not_cabinets() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        s.power_ups = serde_json::json!({"spread": true});
        s.clear_stage();
        assert_eq!(s.power_ups, serde_json::json!({"spread": true}));
        s.clear_stage();
        assert_eq!(s.power_ups, serde_json::Value::Null);
    }

    #[test]
    fn practice_ends_after_its_cabinet() {
        let t = tuning();
        let mut s = Session::new_practice(2, &t);
        assert_eq!(s.clear_stage(), Advance::NextStage);
        assert_eq!(s.clear_stage(), Advance::PracticeComplete);
        assert_eq!(s.cabinet, 2);
    }

    #[test]
    fn session_round_trips_through_json() {
        let t = tuning();
        let mut s = Session::new_run(&t);
        s.add_score(1234, &t);
        s.power_ups = serde_json::json!(["shield"]);
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(serde_json::from_str::<Session>(&json).unwrap(), s);
    }
}
