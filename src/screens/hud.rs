//! Score, Stage and lives along the top of the screen during play.
//!
//! Every Cabinet gets the same HUD: a dark strip across the top (the 12
//! pixels above each Cabinet's play area) edged underneath in the Cabinet's
//! accent colour, with grey labels and white numbers, so it reads the same
//! over any scenery.

use bevy::prelude::*;

use crate::cabinets::CABINETS;
use crate::display::{HALF_H, HALF_W, WIDTH};
use crate::flow::Screen;
use crate::font::{Align, CELL_W, PixelText, text_at};
use crate::run::{Mode, Session};

const HUD_Y: f32 = HALF_H - 6.0;
const HUD_Z: f32 = 50.0;
/// Height of the strip: the band above every Cabinet's play area.
const STRIP_H: f32 = 12.0;
/// The strip overhangs the screen's top and sides, so screen shake never
/// shows a gap.
const OVERHANG: f32 = 8.0;
const STRIP_COLOR: Color = Color::srgba(0.0, 0.0, 0.0, 0.85);
const LABEL_COLOR: Color = Color::srgb(0.55, 0.55, 0.6);
const SCORE_LABEL: &str = "SCORE";
const LIVES_LABEL: &str = "LIVES";

#[derive(Component)]
enum HudField {
    Score,
    Lives,
}

/// Lives shown last frame, to spot an extra life being earned.
#[derive(Resource, Default)]
struct LastLives(Option<u32>);

/// Brief "1UP!" flash when an extra life is earned.
#[derive(Component)]
struct OneUp(Timer);

fn spawn(mut commands: Commands, session: Res<Session>) {
    let accent = CABINETS[session.cabinet].accent;
    let stage = match session.mode {
        Mode::Run => format!("{}-{}", session.cabinet + 1, session.stage + 1),
        Mode::Demo => "DEMO".to_string(),
        Mode::Practice => format!("PRACTICE {}-{}", session.cabinet + 1, session.stage + 1),
    };
    let at = |x: f32| Transform::from_xyz(x, HUD_Y, HUD_Z);
    let width = WIDTH as f32 + 2.0 * OVERHANG;
    commands.spawn((
        Sprite::from_color(STRIP_COLOR, Vec2::new(width, STRIP_H + OVERHANG)),
        Transform::from_xyz(0.0, HALF_H - (STRIP_H - OVERHANG) / 2.0, HUD_Z - 1.0),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        Sprite::from_color(accent, Vec2::new(width, 1.0)),
        Transform::from_xyz(0.0, HALF_H - STRIP_H - 0.5, HUD_Z - 1.0),
        DespawnOnExit(Screen::Playing),
    ));
    let label_w = |label: &str| (label.len() + 1) as f32 * CELL_W as f32;
    commands.spawn((
        PixelText::new(SCORE_LABEL, LABEL_COLOR).aligned(Align::Left),
        at(-HALF_W + 4.0),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        HudField::Score,
        PixelText::new("", Color::WHITE).aligned(Align::Left),
        at(-HALF_W + 4.0 + label_w(SCORE_LABEL)),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        PixelText::new(stage, accent),
        at(0.0),
        DespawnOnExit(Screen::Playing),
    ));
    // Lives is one digit (at most five), right-aligned under its label.
    commands.spawn((
        PixelText::new(LIVES_LABEL, LABEL_COLOR).aligned(Align::Right),
        at(HALF_W - 4.0 - CELL_W as f32 * 2.0),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        HudField::Lives,
        PixelText::new("", Color::WHITE).aligned(Align::Right),
        at(HALF_W - 4.0),
        DespawnOnExit(Screen::Playing),
    ));
}

fn update(
    mut commands: Commands,
    session: Res<Session>,
    mut fields: Query<(&HudField, &mut PixelText)>,
    mut last_lives: ResMut<LastLives>,
) {
    if !session.is_changed() {
        return;
    }
    for (field, mut text) in &mut fields {
        let wanted = match field {
            HudField::Score => format!("{:06}", session.score),
            HudField::Lives => session.lives.to_string(),
        };
        if text.text != wanted {
            text.text = wanted;
        }
    }
    if last_lives.0.is_some_and(|before| session.lives > before) {
        commands.spawn((
            OneUp(Timer::from_seconds(1.5, TimerMode::Once)),
            text_at(
                PixelText::new("1UP!", Color::srgb(0.4, 1.0, 0.4)),
                0.0,
                HUD_Y - 14.0,
                1,
            ),
            DespawnOnExit(Screen::Playing),
        ));
    }
    last_lives.0 = Some(session.lives);
}

fn reset_lives_memory(mut commands: Commands) {
    commands.insert_resource(LastLives::default());
}

fn expire_one_up(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut OneUp)>,
) {
    for (entity, mut flash) in &mut flashes {
        if flash.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

pub struct HudPlugin;

impl Plugin for HudPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastLives>()
            .add_systems(OnEnter(Screen::Playing), (spawn, reset_lives_memory))
            .add_systems(
                Update,
                (update, expire_one_up).run_if(in_state(Screen::Playing)),
            );
    }
}
