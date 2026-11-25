//! Score, Stage and lives along the top of the screen during play.

use bevy::prelude::*;

use crate::cabinets::CABINETS;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::font::{Align, PixelText, text_at};
use crate::run::{Mode, Session};

const HUD_Y: f32 = HALF_H - 6.0;
const HUD_Z: f32 = 50.0;

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
    commands.spawn((
        HudField::Score,
        PixelText::new("", Color::WHITE).aligned(Align::Left),
        at(-HALF_W + 4.0),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        PixelText::new(stage, accent),
        at(0.0),
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
            HudField::Lives => format!("LIVES {}", session.lives),
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
