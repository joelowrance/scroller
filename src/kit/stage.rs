//! Running a Stage: scrolling its timeline, losing a life and restarting at a
//! Restart Point, clearing it at the end, and the debug keys.

use bevy::prelude::*;

use super::collision::Layer;
use super::combat::{Hero, HeroStruck, HitStop, Invulnerable, Shield, ShieldAbsorbed};
use super::effects::{Explode, Shake};
use super::timeline::{SpawnRequested, StageRun};
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::flow::{Paused, Screen};
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

/// A lost life waiting out its hit-stop: seconds left. While it runs, the
/// Stage is frozen (see [`stage_live`]).
#[derive(Resource, Default)]
pub struct HitStopTimer(pub Option<f32>);

/// Run condition for Cabinet gameplay systems: a Stage is playing, not
/// paused, and not frozen in a hit-stop.
pub fn stage_live(
    screen: Res<State<Screen>>,
    paused: Res<Paused>,
    stage: Option<Res<StageRun>>,
    hit_stop: Res<HitStopTimer>,
) -> bool {
    *screen.get() == Screen::Playing && !paused.0 && stage.is_some() && hit_stop.0.is_none()
}

/// The Hero was hit. A [`Shield`] charge absorbs it; otherwise, after the
/// Hero's [`HitStop`], a life is lost and the Stage rewinds to the latest
/// Restart Point. If that was the last life, the flow moves to Game Over.
pub(super) fn restart_after_hit(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut struck: MessageReader<HeroStruck>,
    mut hit_stop: ResMut<HitStopTimer>,
    mut stage: ResMut<StageRun>,
    mut hero: Single<
        (
            &mut Transform,
            &mut Invulnerable,
            Option<&mut Shield>,
            Option<&HitStop>,
            Option<&Sprite>,
        ),
        With<Hero>,
    >,
    clearable: Query<(Entity, &Layer)>,
    mut lost: MessageWriter<LifeLost>,
    mut absorbed: MessageWriter<ShieldAbsorbed>,
    mut restarted: MessageWriter<StageRestarted>,
    mut explode: MessageWriter<Explode>,
    mut shake: MessageWriter<Shake>,
) {
    let (transform, invulnerable, shield, stop, sprite) = &mut *hero;
    // One hit per frame at most, however many things hit at once.
    let hit = struck.read().last().copied();
    match hit_stop.0 {
        Some(left) => {
            let left = left - time.delta_secs();
            if left > 0.0 {
                hit_stop.0 = Some(left);
                return;
            }
            hit_stop.0 = None;
        }
        None => {
            let Some(hit) = hit else { return };
            if stage.cursor.is_cleared() || invulnerable.0 > 0.0 {
                return;
            }
            if let Some(shield) = shield.as_mut().filter(|s| s.charges > 0) {
                shield.charges -= 1;
                invulnerable.0 = shield.invulnerable_seconds;
                absorbed.write(ShieldAbsorbed { from: hit.from });
                return;
            }
            let seconds = stop.map_or(0.0, |s| s.0);
            if seconds > 0.0 {
                hit_stop.0 = Some(seconds);
                return;
            }
        }
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
        struck.write(HeroStruck { from: None });
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

pub(super) fn end_stage(mut commands: Commands, mut hit_stop: ResMut<HitStopTimer>) {
    hit_stop.0 = None;
    commands.remove_resource::<StageRun>();
}
