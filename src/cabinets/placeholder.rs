//! A stand-in Cabinet used for every slot until the real Cabinets are built.
//!
//! It scrolls in the Cabinet's direction, lets the Hero (a square) move
//! around, and clears the Stage after a timer, so the whole Run flow can be
//! played end to end. With `debug_keys` on: K loses a life, P adds 1000
//! points, C clears the Stage at once.

use bevy::prelude::*;

use super::{CABINETS, LifeLost, ScoreGained, Scroll, StageCleared};
use crate::display::{HALF_H, HALF_W};
use crate::flow::{Screen, gameplay_running};
use crate::input::Controls;
use crate::run::Session;
use crate::tuning::Tuning;

const HERO_SIZE: f32 = 8.0;
const STRIPE_COUNT: usize = 24;
const PROGRESS_BAR_W: f32 = 120.0;

#[derive(Component)]
struct Hero {
    /// Seconds of post-restart blinking left.
    invulnerable: f32,
}

#[derive(Component)]
struct Stripe {
    speed: f32,
}

#[derive(Component)]
struct ProgressFill;

#[derive(Resource)]
struct StageClock {
    elapsed: f32,
    done: bool,
}

fn spawn_stage(mut commands: Commands, session: Res<Session>) {
    let cabinet = &CABINETS[session.cabinet];
    commands.insert_resource(StageClock {
        elapsed: 0.0,
        done: false,
    });

    // Parallax stripes: three depths, scrolling in the Cabinet's direction.
    for i in 0..STRIPE_COUNT {
        let depth = (i % 3) as f32 + 1.0;
        let along = (i as f32 * 37.0) % (HALF_W * 2.0) - HALF_W;
        let across = (i as f32 * 53.0) % (HALF_H * 2.0) - HALF_H;
        let color = cabinet.accent.with_alpha(0.15 * depth);
        let (size, pos) = match cabinet.scroll {
            Scroll::Sideways => (Vec2::new(4.0 * depth, 1.0), Vec2::new(along, across)),
            Scroll::Upward => (Vec2::new(1.0, 4.0 * depth), Vec2::new(across, along)),
        };
        commands.spawn((
            Sprite::from_color(color, size),
            Transform::from_xyz(pos.x.round(), pos.y.round(), 1.0),
            Stripe { speed: depth },
            DespawnOnExit(Screen::Playing),
        ));
    }

    commands.spawn((
        Sprite::from_color(cabinet.accent, Vec2::splat(HERO_SIZE)),
        Transform::from_xyz(0.0, 0.0, 5.0),
        Hero { invulnerable: 0.0 },
        DespawnOnExit(Screen::Playing),
    ));

    // Progress bar along the bottom of the screen.
    let bar_y = -HALF_H + 6.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(PROGRESS_BAR_W, 2.0)),
        Transform::from_xyz(0.0, bar_y, 20.0),
        DespawnOnExit(Screen::Playing),
    ));
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 2.0)),
        Transform::from_xyz(-PROGRESS_BAR_W / 2.0, bar_y, 21.0),
        ProgressFill,
        DespawnOnExit(Screen::Playing),
    ));
}

fn scroll_stripes(
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    mut stripes: Query<(&Stripe, &mut Transform)>,
) {
    let scroll = CABINETS[session.cabinet].scroll;
    let base = tuning.placeholder.scroll_speed * time.delta_secs();
    for (stripe, mut transform) in &mut stripes {
        let pos = &mut transform.translation;
        match scroll {
            Scroll::Sideways => {
                pos.x -= base * stripe.speed;
                if pos.x < -HALF_W - 8.0 {
                    pos.x += HALF_W * 2.0 + 16.0;
                }
            }
            Scroll::Upward => {
                pos.y -= base * stripe.speed;
                if pos.y < -HALF_H - 8.0 {
                    pos.y += HALF_H * 2.0 + 16.0;
                }
            }
        }
    }
}

fn move_hero(
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    mut hero: Single<(&mut Hero, &mut Transform, &mut Visibility)>,
) {
    let (hero, transform, visibility) = &mut *hero;
    let step = controls.direction().normalize_or_zero()
        * tuning.placeholder.hero_speed
        * time.delta_secs();
    let limit = Vec2::new(HALF_W - HERO_SIZE, HALF_H - HERO_SIZE - 10.0);
    let pos = (transform.translation.truncate() + step).clamp(-limit, limit);
    transform.translation.x = pos.x;
    transform.translation.y = pos.y;

    hero.invulnerable = (hero.invulnerable - time.delta_secs()).max(0.0);
    let blink_on =
        hero.invulnerable == 0.0 || ((hero.invulnerable * 10.0) as u32).is_multiple_of(2);
    **visibility = if blink_on {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
}

fn advance_clock(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut clock: ResMut<StageClock>,
    mut fill: Single<(&mut Sprite, &mut Transform), With<ProgressFill>>,
    mut cleared: MessageWriter<StageCleared>,
) {
    if clock.done {
        return;
    }
    clock.elapsed += time.delta_secs();
    let progress = (clock.elapsed / tuning.placeholder.stage_seconds.max(0.1)).min(1.0);
    let width = (PROGRESS_BAR_W * progress).round().max(1.0);
    let (sprite, transform) = &mut *fill;
    sprite.custom_size = Some(Vec2::new(width, 2.0));
    transform.translation.x = -PROGRESS_BAR_W / 2.0 + width / 2.0;
    if progress >= 1.0 {
        clock.done = true;
        cleared.write_default();
    }
}

fn debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    tuning: Res<Tuning>,
    mut clock: ResMut<StageClock>,
    mut hero: Single<(&mut Hero, &mut Transform)>,
    mut lost: MessageWriter<LifeLost>,
    mut score: MessageWriter<ScoreGained>,
    mut cleared: MessageWriter<StageCleared>,
) {
    if !tuning.debug_keys {
        return;
    }
    if keys.just_pressed(KeyCode::KeyK) && hero.0.invulnerable == 0.0 {
        lost.write_default();
        // Back to the Restart Point: the middle of the screen, briefly blinking.
        hero.0.invulnerable = 2.0;
        hero.1.translation.x = 0.0;
        hero.1.translation.y = 0.0;
    }
    if keys.just_pressed(KeyCode::KeyP) {
        score.write(ScoreGained(1000));
    }
    if keys.just_pressed(KeyCode::KeyC) && !clock.done {
        clock.done = true;
        cleared.write_default();
    }
}

pub struct PlaceholderCabinetPlugin;

impl Plugin for PlaceholderCabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Playing), spawn_stage)
            .add_systems(
                Update,
                (scroll_stripes, move_hero, advance_clock, debug_keys)
                    .run_if(in_state(Screen::Playing))
                    .run_if(gameplay_running),
            );
    }
}
