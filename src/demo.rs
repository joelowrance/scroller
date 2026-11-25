//! Attract Mode's gameplay demo: a few seconds of a Cabinet played from a
//! script, between the title and the High Score Table.
//!
//! The demo runs through the real Cabinet systems on [`Screen::Playing`]. Only
//! the controls are faked: while a [`Demo`] is on, [`Controls`] is fed from a
//! per-Cabinet script instead of the devices. The Session is [`Mode::Demo`],
//! which the flow never Saves, scores or marks as Progress, so storage is not
//! touched. Any real button press ends the demo and returns to the title.
//! Practice-locked Cabinets appear too: the demo is a teaser.

use bevy::prelude::*;

use crate::cabinets::CABINET_COUNT;
use crate::display::HALF_H;
use crate::flow::{Paused, Screen};
use crate::font::{Blink, PixelText, text_at};
use crate::input::{Button, Controls, read_controls};
use crate::run::{Mode, Session};
use crate::screens::ShowScoresFirst;
use crate::tuning::Tuning;

use Button::{A, B, Down, Left, Right, Up};

/// One beat of a script: hold these buttons for this many seconds.
type Step = (f32, &'static [Button]);

/// Cabinet 1: hold fire, weave up and down the screen.
const SIDE_SHOOTER: &[Step] = &[
    (0.8, &[A]),
    (0.9, &[A, Up]),
    (0.5, &[A, Right]),
    (1.3, &[A, Down]),
    (0.6, &[A, Left]),
    (0.9, &[A, Up]),
    (0.7, &[A]),
];

/// Cabinet 2: hold fire, sweep left and right.
const VERTICAL_SHOOTER: &[Step] = &[
    (0.6, &[A]),
    (0.9, &[A, Left]),
    (0.4, &[A, Up]),
    (1.6, &[A, Right]),
    (0.8, &[A, Left]),
    (0.5, &[A, Down]),
];

/// Cabinet 3: run right, firing (B), hopping (A) now and then.
const RUN_AND_GUN: &[Step] = &[
    (0.9, &[Right, B]),
    (0.4, &[Right, A, B]),
    (0.7, &[Right, B]),
    (0.4, &[Right, A, B]),
    (0.5, &[B]),
    (0.4, &[Right, A, B]),
];

/// Cabinet 4: climb by hopping (A) side to side, firing (B).
const CLIMBER: &[Step] = &[
    (0.5, &[Right, B]),
    (0.35, &[Right, A, B]),
    (0.5, &[Right, B]),
    (0.35, &[Left, A, B]),
    (0.5, &[Left, B]),
    (0.35, &[Right, A, B]),
];

const SCRIPTS: [&[Step]; CABINET_COUNT] = [SIDE_SHOOTER, VERTICAL_SHOOTER, RUN_AND_GUN, CLIMBER];

/// The Cabinet the next demo shows; each Attract Mode round takes the next.
#[derive(Resource, Default)]
pub struct DemoCursor(pub usize);

/// A demo is playing.
#[derive(Resource)]
pub struct Demo {
    /// Seconds since the demo began.
    elapsed: f32,
}

/// The script's buttons at `seconds` in; the script loops.
fn buttons_at(script: &'static [Step], seconds: f32) -> &'static [Button] {
    let total: f32 = script.iter().map(|(len, _)| len).sum();
    let mut t = seconds % total.max(f32::EPSILON);
    for (len, buttons) in script {
        if t < *len {
            return buttons;
        }
        t -= len;
    }
    &[]
}

/// Starts the demo of the next Cabinet, straight into the Stage.
pub fn start(
    commands: &mut Commands,
    cursor: &mut DemoCursor,
    tuning: &Tuning,
    next: &mut NextState<Screen>,
) {
    commands.insert_resource(Session::new_demo(cursor.0 % CABINET_COUNT, tuning));
    commands.insert_resource(Demo { elapsed: 0.0 });
    cursor.0 = (cursor.0 + 1) % CABINET_COUNT;
    next.set(Screen::Playing);
}

/// Ends the demo and returns to the title: onto the High Score Table when the
/// demo ran its course, onto the menu when the player interrupted it.
pub fn finish(commands: &mut Commands, next: &mut NextState<Screen>, show_scores: bool) {
    commands.remove_resource::<Demo>();
    if show_scores {
        commands.insert_resource(ShowScoresFirst);
    }
    next.set(Screen::Title);
}

fn drive(
    mut commands: Commands,
    time: Res<Time<Real>>,
    tuning: Res<Tuning>,
    paused: Res<Paused>,
    session: Res<Session>,
    mut demo: ResMut<Demo>,
    mut controls: ResMut<Controls>,
    mut next: ResMut<NextState<Screen>>,
) {
    // Any press, or a pause (the window lost focus), hands back to the player.
    if controls.device_pressed() || paused.0 {
        finish(&mut commands, &mut next, false);
        return;
    }
    demo.elapsed += time.delta_secs();
    if demo.elapsed >= tuning.attract_demo_seconds {
        finish(&mut commands, &mut next, true);
        return;
    }
    controls.script(buttons_at(SCRIPTS[session.cabinet], demo.elapsed));
}

fn label(mut commands: Commands) {
    commands.spawn((
        text_at(
            PixelText::new("DEMO  PRESS ANY BUTTON", Color::WHITE),
            0.0,
            -HALF_H + 8.0,
            1,
        ),
        Blink::every(0.6),
        DespawnOnExit(Screen::Playing),
    ));
}

fn demo_on(demo: Option<Res<Demo>>) -> bool {
    demo.is_some()
}

pub struct DemoPlugin;

impl Plugin for DemoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DemoCursor>()
            .add_systems(
                OnEnter(Screen::Playing),
                label.run_if(demo_on).run_if(session_is_demo),
            )
            .add_systems(
                PreUpdate,
                drive
                    .after(read_controls)
                    .run_if(demo_on)
                    .run_if(in_state(Screen::Playing)),
            );
    }
}

fn session_is_demo(session: Option<Res<Session>>) -> bool {
    session.is_some_and(|s| s.mode == Mode::Demo)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn script_plays_in_order_and_loops() {
        assert_eq!(buttons_at(SIDE_SHOOTER, 0.0), &[A]);
        assert_eq!(buttons_at(SIDE_SHOOTER, 1.0), &[A, Up]);
        let total: f32 = SIDE_SHOOTER.iter().map(|(len, _)| len).sum();
        assert_eq!(buttons_at(SIDE_SHOOTER, total + 0.1), &[A]);
    }

    #[test]
    fn every_cabinet_has_a_script_that_moves_and_acts() {
        for script in SCRIPTS {
            assert!(script.iter().all(|(len, _)| *len > 0.0));
            assert!(script.iter().any(|(_, b)| b.contains(&A) || b.contains(&B)));
        }
    }

    #[test]
    fn scripting_replaces_the_devices_and_real_presses_still_show() {
        let mut controls = Controls::default();
        controls.script(&[A, Right]);
        assert!(controls.just_pressed(A) && controls.held(Right));
        assert!(!controls.device_pressed());
        controls.script(&[A]);
        assert!(!controls.just_pressed(A) && !controls.held(Right));
    }
}
