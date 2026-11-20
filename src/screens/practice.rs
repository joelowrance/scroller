//! Choosing a cleared Cabinet to Practice.

use bevy::prelude::*;

use crate::cabinets::{CABINET_COUNT, CABINETS};
use crate::display::Backdrop;
use crate::flow::{Progress, Screen, start_practice};
use crate::font::{Align, CELL_H, PixelText, text_at};
use crate::input::{Button, Controls};
use crate::tuning::Tuning;

const ROW_SPACING: f32 = CELL_H as f32 + 10.0;

#[derive(Resource)]
struct Cursor(usize);

#[derive(Component)]
struct Row(usize);

fn spawn(mut commands: Commands, progress: Res<Progress>, mut backdrop: ResMut<Backdrop>) {
    backdrop.0 = Color::srgb(0.02, 0.0, 0.06);
    let first = progress.cleared.iter().position(|c| *c).unwrap_or(0);
    commands.insert_resource(Cursor(first));
    let root = (
        Transform::default(),
        Visibility::default(),
        DespawnOnExit(Screen::PracticeSelect),
    );
    commands.spawn(root).with_children(|p| {
        p.spawn(text_at(
            PixelText::new("PRACTICE", Color::srgb(1.0, 0.85, 0.2)),
            0.0,
            90.0,
            2,
        ));
        for i in 0..CABINET_COUNT {
            p.spawn((
                Row(i),
                text_at(
                    PixelText::new("", Color::WHITE).aligned(Align::Left),
                    -130.0,
                    40.0 - i as f32 * ROW_SPACING,
                    1,
                ),
            ));
        }
        p.spawn(text_at(
            PixelText::new("A: PLAY    B: BACK", Color::srgb(0.6, 0.6, 0.6)),
            0.0,
            -90.0,
            1,
        ));
    });
}

fn update(
    mut commands: Commands,
    controls: Res<Controls>,
    tuning: Res<Tuning>,
    progress: Res<Progress>,
    mut cursor: ResMut<Cursor>,
    mut rows: Query<(&Row, &mut PixelText)>,
    mut next: ResMut<NextState<Screen>>,
) {
    if controls.just_pressed(Button::Down) {
        cursor.0 = (cursor.0 + 1) % CABINET_COUNT;
    }
    if controls.just_pressed(Button::Up) {
        cursor.0 = (cursor.0 + CABINET_COUNT - 1) % CABINET_COUNT;
    }
    for (row, mut text) in &mut rows {
        let unlocked = progress.cleared[row.0];
        let name = if unlocked {
            CABINETS[row.0].genre
        } else {
            "LOCKED"
        };
        let marker = if row.0 == cursor.0 { ">" } else { " " };
        let wanted = format!("{marker} {}  {name}", row.0 + 1);
        let color = match (row.0 == cursor.0, unlocked) {
            (_, false) => Color::srgb(0.35, 0.35, 0.35),
            (true, true) => CABINETS[row.0].accent,
            (false, true) => Color::WHITE,
        };
        if text.text != wanted || text.color != color {
            text.text = wanted;
            text.color = color;
        }
    }
    if controls.just_pressed(Button::B) {
        next.set(Screen::Title);
    } else if controls.just_pressed(Button::A) && progress.cleared[cursor.0] {
        start_practice(&mut commands, cursor.0, &tuning, &mut next);
    }
}

pub struct PracticePlugin;

impl Plugin for PracticePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::PracticeSelect), spawn)
            .add_systems(Update, update.run_if(in_state(Screen::PracticeSelect)));
    }
}
