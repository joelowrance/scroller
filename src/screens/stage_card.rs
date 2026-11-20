//! The Stage Card shown before each Stage. After a Stage is cleared it also
//! tells the player the Run is saved and offers a safe place to quit.

use bevy::prelude::*;

use crate::cabinets::CABINETS;
use crate::display::Backdrop;
use crate::flow::{CardReason, Screen, save_exists};
use crate::font::{Blink, PixelText, text_at};
use crate::input::{Button, Controls};
use crate::run::{Mode, Session};

const DIM: Color = Color::srgb(0.6, 0.6, 0.6);

fn spawn(
    mut commands: Commands,
    session: Res<Session>,
    reason: Res<CardReason>,
    mut backdrop: ResMut<Backdrop>,
) {
    let cabinet = &CABINETS[session.cabinet];
    backdrop.0 = cabinet.backdrop;

    let headline = match *reason {
        CardReason::NewRun => "GET READY!",
        CardReason::Resumed => "WELCOME BACK!",
        CardReason::StageCleared => "STAGE CLEAR!",
        CardReason::AfterTransition => "GET READY!",
        CardReason::Practice => "PRACTICE",
    };
    let can_quit = session.mode == Mode::Run && save_exists();
    let saved_line = if *reason == CardReason::StageCleared && can_quit {
        "PROGRESS SAVED"
    } else {
        ""
    };
    let prompt = if can_quit {
        "A: PLAY    B: QUIT, RESUME LATER"
    } else {
        "A: PLAY"
    };

    commands
        .spawn((
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::StageCard),
        ))
        .with_children(|p| {
            p.spawn(text_at(
                PixelText::new(headline, Color::WHITE),
                0.0,
                80.0,
                2,
            ));
            p.spawn(text_at(PixelText::new(saved_line, DIM), 0.0, 60.0, 1));
            p.spawn(text_at(
                PixelText::new(format!("CABINET {}", session.cabinet + 1), cabinet.accent),
                0.0,
                25.0,
                2,
            ));
            p.spawn(text_at(
                PixelText::new(cabinet.genre, cabinet.accent),
                0.0,
                5.0,
                1,
            ));
            p.spawn(text_at(
                PixelText::new(format!("STAGE {}", session.stage + 1), Color::WHITE),
                0.0,
                -25.0,
                3,
            ));
            p.spawn((
                text_at(PixelText::new(prompt, Color::WHITE), 0.0, -90.0, 1),
                Blink::every(0.5),
            ));
        });
}

fn update(controls: Res<Controls>, session: Res<Session>, mut next: ResMut<NextState<Screen>>) {
    if controls.just_pressed(Button::A) || controls.just_pressed(Button::Start) {
        next.set(Screen::Playing);
    } else if controls.just_pressed(Button::B) && session.mode == Mode::Run && save_exists() {
        // The Save stays on disk; Continue on the title picks it up.
        next.set(Screen::Title);
    }
}

pub struct StageCardPlugin;

impl Plugin for StageCardPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::StageCard), spawn)
            .add_systems(Update, update.run_if(in_state(Screen::StageCard)));
    }
}
