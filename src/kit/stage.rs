//! Running a Stage: scrolling its timeline, losing a life and restarting at a
//! Restart Point, clearing it at the end, and the debug keys.

use bevy::prelude::*;

use super::collision::Layer;
use super::combat::{Hero, HeroStruck, Invulnerable};
use super::effects::{Explode, Shake};
use super::timeline::{SpawnRequested, StageRun};
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::tuning::Tuning;

/// The Hero lost a life and the Stage went back to a Restart Point. Enemies
/// and shots are already cleared and the Hero moved; a Cabinet resets
/// anything of its own here.
#[derive(Message, Clone, Copy, Debug)]
pub struct StageRestarted {
    pub hero: Vec2,
}

pub(super) fn advance_timeline(
    time: Res<Time>,
    mut stage: ResMut<StageRun>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut cleared: MessageWriter<StageCleared>,
) {
    let StageRun { timeline, cursor } = &mut *stage;
    let step = cursor.advance(timeline, time.delta_secs());
    for entry in &timeline.spawns[step.spawns] {
        spawns.write(SpawnRequested {
            kind: entry.kind.clone(),
            pos: Vec2::new(entry.pos.0, entry.pos.1),
            params: entry.params.clone(),
        });
    }
    if step.cleared {
        cleared.write_default();
    }
}

/// The Hero was hit: a life is lost and the Stage rewinds to the latest
/// Restart Point. If that was the last life, the flow moves to Game Over.
pub(super) fn restart_after_hit(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut struck: MessageReader<HeroStruck>,
    mut stage: ResMut<StageRun>,
    mut hero: Single<(&mut Transform, &mut Invulnerable, Option<&Sprite>), With<Hero>>,
    clearable: Query<(Entity, &Layer)>,
    mut lost: MessageWriter<LifeLost>,
    mut restarted: MessageWriter<StageRestarted>,
    mut explode: MessageWriter<Explode>,
    mut shake: MessageWriter<Shake>,
) {
    // One life per frame at most, however many things hit at once.
    if struck.read().count() == 0 || stage.cursor.is_cleared() {
        return;
    }
    let (transform, invulnerable, sprite) = &mut *hero;
    if invulnerable.0 > 0.0 {
        return;
    }
    lost.write_default();
    let at = transform.translation.truncate();
    explode.write(Explode {
        at,
        color: sprite.map_or(Color::WHITE, |s| s.color),
    });
    shake.write(Shake(tuning.kit.shake_on_hero_down));

    for (entity, layer) in &clearable {
        if matches!(
            layer,
            Layer::Enemy | Layer::EnemyShot | Layer::HeroShot | Layer::Pickup
        ) {
            commands.entity(entity).try_despawn();
        }
    }
    let StageRun { timeline, cursor } = &mut *stage;
    let point = cursor.rewind(timeline);
    let hero_at = point.hero_pos();
    transform.translation.x = hero_at.x;
    transform.translation.y = hero_at.y;
    invulnerable.0 = tuning.kit.invulnerable_seconds;
    restarted.write(StageRestarted { hero: hero_at });
}

/// Developer keys, on when `debug_keys` is set: K loses a life, P adds 1000
/// points, C clears the Stage at once.
pub(super) fn debug_keys(
    keys: Res<ButtonInput<KeyCode>>,
    tuning: Res<Tuning>,
    mut stage: ResMut<StageRun>,
    mut struck: MessageWriter<HeroStruck>,
    mut score: MessageWriter<ScoreGained>,
    mut cleared: MessageWriter<StageCleared>,
) {
    if !tuning.debug_keys {
        return;
    }
    if keys.just_pressed(KeyCode::KeyK) {
        struck.write_default();
    }
    if keys.just_pressed(KeyCode::KeyP) {
        score.write(ScoreGained(1000));
    }
    if keys.just_pressed(KeyCode::KeyC) && stage.cursor.force_clear() {
        cleared.write_default();
    }
}

/// The debug time-scale multiplier, applied while a Stage is playing.
pub(super) fn apply_time_scale(tuning: Res<Tuning>, mut time: ResMut<Time<Virtual>>) {
    let scale = if tuning.debug_keys {
        tuning.time_scale.clamp(0.05, 20.0)
    } else {
        1.0
    };
    if time.relative_speed() != scale {
        time.set_relative_speed(scale);
    }
}

pub(super) fn reset_time_scale(mut time: ResMut<Time<Virtual>>) {
    time.set_relative_speed(1.0);
}

pub(super) fn end_stage(mut commands: Commands) {
    commands.remove_resource::<StageRun>();
}
