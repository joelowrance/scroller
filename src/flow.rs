//! Which screen is showing, and how a Run moves between them.
//!
//! ```text
//! Title ──New Run / Continue──▶ StageCard ──▶ Playing ──Stage cleared──▶ StageCard
//!   ▲                              ▲              │  └─Cabinet cleared─▶ CabinetTransition ─▶ StageCard
//!   │                              │              ├─Final Cabinet cleared─▶ Ending ─┐
//!   │                              │              └─Last life lost / quit─▶ GameOver ─┤
//!   └──────────── NameEntry (if the score makes the High Score Table) ◀──────────────┘
//! ```
//!
//! Save rules: a Save is written when a Stage is cleared and removed the
//! moment the next Stage starts. So a Save only exists *between* Stages, and
//! quitting during a Stage always forfeits the Run.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::cabinets::{CABINET_COUNT, LifeLost, ScoreGained, StageCleared};
use crate::highscore::HighScoreTable;
use crate::run::{Advance, LifeOutcome, Mode, Session};
use crate::storage::{self, HIGH_SCORES_KEY, PROGRESS_KEY, SAVE_KEY};
use crate::tuning::Tuning;

#[derive(States, Clone, Copy, PartialEq, Eq, Hash, Debug, Default)]
pub enum Screen {
    /// Title menu and Attract Mode.
    #[default]
    Title,
    PracticeSelect,
    StageCard,
    Playing,
    CabinetTransition,
    GameOver,
    Ending,
    NameEntry,
}

/// Why the Stage Card is being shown, which decides what it says.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum CardReason {
    NewRun,
    Resumed,
    StageCleared,
    AfterTransition,
    Practice,
}

/// Whether gameplay is paused (only meaningful while Playing).
#[derive(Resource, Default)]
pub struct Paused(pub bool);

/// Which Cabinets have ever been cleared in a Run; unlocks Practice.
#[derive(Resource, Clone, Default, Serialize, Deserialize)]
pub struct Progress {
    pub cleared: [bool; CABINET_COUNT],
}

impl Progress {
    pub fn mark_cleared(&mut self, cabinet: usize) {
        if !self.cleared[cabinet] {
            self.cleared[cabinet] = true;
            storage::store(PROGRESS_KEY, self);
        }
    }
}

/// Run condition: the Stage is live (not paused).
pub fn gameplay_running(paused: Res<Paused>) -> bool {
    !paused.0
}

pub fn save_exists() -> bool {
    storage::exists(SAVE_KEY)
}

/// Starts a new Run, abandoning any Save.
pub fn start_new_run(commands: &mut Commands, tuning: &Tuning, next: &mut NextState<Screen>) {
    storage::remove(SAVE_KEY);
    commands.insert_resource(Session::new_run(tuning));
    commands.insert_resource(CardReason::NewRun);
    next.set(Screen::StageCard);
}

/// Resumes the saved Run. Returns false if there is no readable Save.
pub fn continue_run(commands: &mut Commands, next: &mut NextState<Screen>) -> bool {
    match storage::load::<Session>(SAVE_KEY) {
        Some(session) if session.mode == Mode::Run => {
            commands.insert_resource(session);
            commands.insert_resource(CardReason::Resumed);
            next.set(Screen::StageCard);
            true
        }
        _ => {
            storage::remove(SAVE_KEY);
            false
        }
    }
}

pub fn start_practice(
    commands: &mut Commands,
    cabinet: usize,
    tuning: &Tuning,
    next: &mut NextState<Screen>,
) {
    commands.insert_resource(Session::new_practice(cabinet, tuning));
    commands.insert_resource(CardReason::Practice);
    next.set(Screen::StageCard);
}

/// The Stage has started: from here on, quitting forfeits the Run.
fn begin_stage(session: Res<Session>, mut paused: ResMut<Paused>) {
    paused.0 = false;
    if session.mode == Mode::Run {
        storage::remove(SAVE_KEY);
    }
}

fn end_stage(mut paused: ResMut<Paused>, mut time: ResMut<Time<Virtual>>) {
    paused.0 = false;
    time.unpause();
}

fn on_score(
    mut gained: MessageReader<ScoreGained>,
    mut session: ResMut<Session>,
    tuning: Res<Tuning>,
) {
    for ScoreGained(points) in gained.read() {
        session.add_score(*points, &tuning);
    }
}

fn on_life_lost(
    mut lost: MessageReader<LifeLost>,
    mut session: ResMut<Session>,
    mut next: ResMut<NextState<Screen>>,
) {
    for _ in lost.read() {
        if session.lose_life() == LifeOutcome::GameOver {
            next.set(Screen::GameOver);
            break;
        }
    }
}

fn on_stage_cleared(
    mut commands: Commands,
    mut cleared: MessageReader<StageCleared>,
    mut session: ResMut<Session>,
    mut progress: ResMut<Progress>,
    mut next: ResMut<NextState<Screen>>,
) {
    // A Stage cleared in the same frame as the last life was lost is still Game Over.
    if cleared.read().last().is_none() || session.lives == 0 {
        return;
    }
    let cabinet = session.cabinet;
    let advance = session.clear_stage();
    if !matches!(advance, Advance::NextStage) {
        progress.mark_cleared(cabinet);
    }
    if session.mode == Mode::Run
        && matches!(advance, Advance::NextStage | Advance::NextCabinet { .. })
    {
        storage::store(SAVE_KEY, &*session);
    }
    match advance {
        Advance::NextStage => {
            commands.insert_resource(CardReason::StageCleared);
            next.set(Screen::StageCard);
        }
        Advance::NextCabinet { .. } => next.set(Screen::CabinetTransition),
        Advance::Ending => next.set(Screen::Ending),
        Advance::PracticeComplete => next.set(Screen::PracticeSelect),
    }
}

/// The flow's reaction to a Stage's messages: score, lives and clearing.
/// Systems that add points when a Stage clears run before it.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct FlowSet;

pub struct FlowPlugin;

impl Plugin for FlowPlugin {
    fn build(&self, app: &mut App) {
        // Loaded while building the app so they exist before the first screen opens.
        app.insert_resource(storage::load::<HighScoreTable>(HIGH_SCORES_KEY).unwrap_or_default())
            .insert_resource(storage::load::<Progress>(PROGRESS_KEY).unwrap_or_default())
            .init_state::<Screen>()
            .init_resource::<Paused>()
            .add_systems(OnEnter(Screen::Playing), begin_stage)
            .add_systems(OnExit(Screen::Playing), end_stage)
            .add_systems(
                Update,
                (on_score, on_life_lost, on_stage_cleared)
                    .chain()
                    .in_set(FlowSet)
                    .run_if(in_state(Screen::Playing)),
            );
    }
}
