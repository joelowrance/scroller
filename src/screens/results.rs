//! The end of a Run: Game Over or the Ending, then initials for the High
//! Score Table when the score earns a place.

use bevy::prelude::*;

use super::title::ShowScoresFirst;
use crate::display::Backdrop;
use crate::flow::Screen;
use crate::font::{Blink, PixelText, text_at};
use crate::highscore::{HighScoreTable, INITIALS_LEN};
use crate::input::{Button, Controls};
use crate::run::{Mode, Session};
use crate::storage::{self, HIGH_SCORES_KEY};

const GOLD: Color = Color::srgb(1.0, 0.85, 0.2);
const LETTERS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789 ";

/// Seconds before the result screen accepts input, so a button mashed
/// during play doesn't skip it.
const INPUT_DELAY: f32 = 1.0;

#[derive(Resource)]
struct ResultDelay(Timer);

fn spawn_result(
    mut commands: Commands,
    session: Res<Session>,
    state: Res<State<Screen>>,
    mut backdrop: ResMut<Backdrop>,
) {
    let screen = *state.get();
    commands.insert_resource(ResultDelay(Timer::from_seconds(
        INPUT_DELAY,
        TimerMode::Once,
    )));
    let (headline, subline, color) = match (screen, session.mode) {
        (Screen::Ending, _) => ("YOU FINISHED!", "EVERY CABINET CLEARED", GOLD),
        (_, Mode::Practice) => ("PRACTICE OVER", "", Color::WHITE),
        _ => ("GAME OVER", "", Color::srgb(1.0, 0.25, 0.25)),
    };
    backdrop.0 = if screen == Screen::Ending {
        Color::srgb(0.06, 0.03, 0.0)
    } else {
        Color::BLACK
    };
    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(screen),
        ))
        .with_children(|p| {
            p.spawn(text_at(PixelText::new(headline, color), 0.0, 50.0, 3));
            p.spawn(text_at(PixelText::new(subline, Color::WHITE), 0.0, 20.0, 1));
            if session.mode == Mode::Run {
                p.spawn(text_at(
                    PixelText::new(format!("SCORE {}", session.score), Color::WHITE),
                    0.0,
                    -10.0,
                    2,
                ));
            }
            p.spawn((
                text_at(PixelText::new("PRESS A", Color::WHITE), 0.0, -90.0, 1),
                Blink::every(0.5),
            ));
        });
}

fn update_result(
    mut commands: Commands,
    time: Res<Time<Real>>,
    controls: Res<Controls>,
    session: Res<Session>,
    scores: Res<HighScoreTable>,
    mut delay: ResMut<ResultDelay>,
    mut next: ResMut<NextState<Screen>>,
) {
    if !delay.0.tick(time.delta()).is_finished() {
        return;
    }
    if !(controls.just_pressed(Button::A) || controls.just_pressed(Button::Start)) {
        return;
    }
    let target = match session.mode {
        Mode::Practice => Screen::PracticeSelect,
        // Never reached (a demo ends itself), but harmless if it were.
        Mode::Demo => Screen::Title,
        Mode::Run if scores.qualifies(session.score) => Screen::NameEntry,
        Mode::Run => {
            commands.insert_resource(ShowScoresFirst);
            Screen::Title
        }
    };
    next.set(target);
}

#[derive(Resource)]
struct NameEntry {
    letters: [usize; INITIALS_LEN],
    slot: usize,
}

#[derive(Component)]
struct Slot(usize);

fn spawn_name_entry(mut commands: Commands, session: Res<Session>, mut backdrop: ResMut<Backdrop>) {
    backdrop.0 = Color::srgb(0.02, 0.0, 0.06);
    commands.insert_resource(NameEntry {
        letters: [0; INITIALS_LEN],
        slot: 0,
    });
    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::NameEntry),
        ))
        .with_children(|p| {
            p.spawn(text_at(
                PixelText::new("NEW HIGH SCORE!", GOLD),
                0.0,
                80.0,
                2,
            ));
            p.spawn(text_at(
                PixelText::new(format!("{}", session.score), Color::WHITE),
                0.0,
                55.0,
                2,
            ));
            p.spawn(text_at(
                PixelText::new("ENTER YOUR INITIALS", Color::WHITE),
                0.0,
                25.0,
                1,
            ));
            for i in 0..INITIALS_LEN {
                p.spawn((
                    Slot(i),
                    text_at(
                        PixelText::new("", Color::WHITE),
                        (i as f32 - 1.0) * 30.0,
                        -10.0,
                        4,
                    ),
                ));
            }
            p.spawn(text_at(
                PixelText::new(
                    "UP/DOWN: LETTER  A: NEXT  B: BACK",
                    Color::srgb(0.6, 0.6, 0.6),
                ),
                0.0,
                -90.0,
                1,
            ));
        });
}

fn update_name_entry(
    mut commands: Commands,
    controls: Res<Controls>,
    session: Res<Session>,
    mut entry: ResMut<NameEntry>,
    mut scores: ResMut<HighScoreTable>,
    mut slots: Query<(&Slot, &mut PixelText)>,
    mut next: ResMut<NextState<Screen>>,
) {
    let slot = entry.slot;
    let n = LETTERS.len();
    if controls.just_pressed(Button::Up) {
        entry.letters[slot] = (entry.letters[slot] + 1) % n;
    }
    if controls.just_pressed(Button::Down) {
        entry.letters[slot] = (entry.letters[slot] + n - 1) % n;
    }
    if controls.just_pressed(Button::B) && slot > 0 {
        entry.slot -= 1;
    }
    if controls.just_pressed(Button::A) || controls.just_pressed(Button::Start) {
        if slot + 1 < INITIALS_LEN {
            entry.slot += 1;
        } else {
            let initials: String = entry.letters.iter().map(|&i| LETTERS[i] as char).collect();
            scores.insert(&initials, session.score);
            storage::store(HIGH_SCORES_KEY, &*scores);
            commands.insert_resource(ShowScoresFirst);
            next.set(Screen::Title);
            return;
        }
    }
    for (Slot(i), mut text) in &mut slots {
        let letter = (LETTERS[entry.letters[*i]] as char).to_string();
        let shown = if letter == " " {
            "_".to_string()
        } else {
            letter
        };
        let color = if *i == entry.slot { GOLD } else { Color::WHITE };
        if text.text != shown || text.color != color {
            text.text = shown;
            text.color = color;
        }
    }
}

fn showing_result(state: Res<State<Screen>>) -> bool {
    matches!(state.get(), Screen::GameOver | Screen::Ending)
}

pub struct ResultsPlugin;

impl Plugin for ResultsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::GameOver), spawn_result)
            .add_systems(OnEnter(Screen::Ending), spawn_result)
            .add_systems(OnEnter(Screen::NameEntry), spawn_name_entry)
            .add_systems(
                Update,
                (
                    update_result.run_if(showing_result),
                    update_name_entry.run_if(in_state(Screen::NameEntry)),
                ),
            );
    }
}
