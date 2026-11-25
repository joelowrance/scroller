//! Pausing during a Stage. Quitting from here forfeits the Run, because the
//! Save was used up when the Stage began.

use bevy::prelude::*;
use bevy::window::WindowFocused;

use super::menu::Menu;
use crate::flow::{Paused, Screen};
use crate::font::{PixelText, text_at};
use crate::input::{Button, Controls};
use crate::run::{Mode, Session};

const RESUME: &str = "RESUME";
const QUIT_RUN: &str = "QUIT RUN";
const QUIT_PRACTICE: &str = "QUIT PRACTICE";

#[derive(Component)]
struct PauseOverlay;

fn toggle(
    mut commands: Commands,
    controls: Res<Controls>,
    session: Res<Session>,
    mut paused: ResMut<Paused>,
    mut time: ResMut<Time<Virtual>>,
    overlay: Query<Entity, With<PauseOverlay>>,
    menu: Query<&Menu, With<PauseOverlay>>,
    mut next: ResMut<NextState<Screen>>,
    mut focus: MessageReader<WindowFocused>,
) {
    // Losing window focus mid-Stage pauses, so a Run is never lost to a stray click.
    let lost_focus = focus.read().any(|f| !f.focused);
    if !paused.0 {
        if controls.just_pressed(Button::Start) || lost_focus {
            paused.0 = true;
            time.pause();
            spawn_overlay(&mut commands, session.mode);
        }
        return;
    }

    let choice = if controls.just_pressed(Button::Start) || controls.just_pressed(Button::B) {
        Some(RESUME)
    } else if controls.just_pressed(Button::A) {
        menu.single().ok().map(|m| m.selected_item())
    } else {
        None
    };
    match choice {
        Some(RESUME) => {
            paused.0 = false;
            time.unpause();
            for e in &overlay {
                commands.entity(e).despawn();
            }
        }
        Some(QUIT_RUN) => next.set(Screen::GameOver),
        Some(QUIT_PRACTICE) => next.set(Screen::PracticeSelect),
        _ => {}
    }
}

fn spawn_overlay(commands: &mut Commands, mode: Mode) {
    let (quit, warning) = match mode {
        Mode::Run => (QUIT_RUN, "QUITTING NOW ENDS THIS RUN"),
        Mode::Practice | Mode::Demo => (QUIT_PRACTICE, ""),
    };
    commands.spawn((
        PauseOverlay,
        Sprite::from_color(Color::srgba(0.0, 0.0, 0.0, 0.75), Vec2::new(320.0, 240.0)),
        Transform::from_xyz(0.0, 0.0, 80.0),
        DespawnOnExit(Screen::Playing),
    ));
    for (text, y, scale) in [("PAUSED", 40.0, 3), (warning, -60.0, 1)] {
        commands.spawn((
            PauseOverlay,
            text_at(PixelText::new(text, Color::WHITE), 0.0, y, scale),
            DespawnOnExit(Screen::Playing),
        ));
    }
    let menu = Menu::new(vec![RESUME, quit], Color::srgb(1.0, 0.85, 0.2)).spawn(commands, 0.0);
    commands.entity(menu).insert((
        PauseOverlay,
        Transform::from_xyz(0.0, 0.0, 90.0),
        DespawnOnExit(Screen::Playing),
    ));
}

pub struct PausePlugin;

impl Plugin for PausePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle.run_if(in_state(Screen::Playing)));
    }
}
