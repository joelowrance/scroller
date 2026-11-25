//! Explosions, hit flashes, screen shake and the Hero's invulnerability blink.
//!
//! Each effect is asked for with a message, so any system (kit or Cabinet)
//! can trigger one: [`Explode`], [`Flash`] and [`Shake`].
//!
//! Destroying a big enemy (one worth at least `big_explosion_score`) adds a
//! shake and a brief [`ImpactFreeze`], the hit-stop that sells the blast.

use bevy::prelude::*;

use super::combat::{EnemyDestroyed, Invulnerable};
use super::motion::{Lifetime, Velocity};
use crate::display::GameCamera;
use crate::flow::Screen;
use crate::tuning::Tuning;

/// Burst of particles at a point, tinted with the given colour.
#[derive(Message, Clone, Copy, Debug)]
pub struct Explode {
    pub at: Vec2,
    pub color: Color,
}

/// Briefly tints an entity's sprite white.
#[derive(Message, Clone, Copy, Debug)]
pub struct Flash(pub Entity);

/// Adds screen shake; 1.0 is the strongest.
#[derive(Message, Clone, Copy, Debug)]
pub struct Shake(pub f32);

#[derive(Component)]
pub(super) struct HitFlash {
    left: f32,
    original: Color,
}

#[derive(Component)]
pub(super) struct Particle {
    total: f32,
}

/// Seconds left of a big explosion's hit-stop. While it runs the Stage is
/// frozen like the Hero's own hit-stop (see [`stage_live`](super::stage_live)).
#[derive(Resource, Default, Debug)]
pub struct ImpactFreeze(pub f32);

/// Starts the shake and hit-stop for each big enemy destroyed this frame.
/// Runs after the explosion is spawned, so the blast shows during the freeze.
pub(super) fn start_impact_freeze(
    tuning: Res<Tuning>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    mut freeze: ResMut<ImpactFreeze>,
    mut shake: MessageWriter<Shake>,
) {
    let k = &tuning.kit;
    let big = destroyed
        .read()
        .filter(|d| d.score >= k.big_explosion_score)
        .count();
    if big > 0 {
        shake.write(Shake(k.shake_on_big_explosion));
        freeze.0 = freeze.0.max(k.big_explosion_hit_stop);
    }
}

/// Counts down the hit-stop. Runs while frozen.
pub(super) fn tick_impact_freeze(time: Res<Time>, mut freeze: ResMut<ImpactFreeze>) {
    if freeze.0 > 0.0 {
        freeze.0 = (freeze.0 - time.delta_secs()).max(0.0);
    }
}

/// Current screen-shake strength, 0 to 1, fading over time.
#[derive(Resource, Default)]
pub(super) struct Trauma(pub(super) f32);

/// Small deterministic random numbers for effects (xorshift).
#[derive(Resource)]
pub(super) struct EffectRng(u32);

impl Default for EffectRng {
    fn default() -> Self {
        Self(0x9E37_79B9)
    }
}

impl EffectRng {
    /// A number in 0..1.
    fn next(&mut self) -> f32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        (x >> 8) as f32 / (1u32 << 24) as f32
    }
}

pub(super) fn spawn_explosions(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut rng: ResMut<EffectRng>,
    mut requests: MessageReader<Explode>,
) {
    let k = &tuning.kit;
    for Explode { at, color } in requests.read() {
        for i in 0..k.explosion_particles {
            let angle = rng.next() * std::f32::consts::TAU;
            let speed = k.explosion_speed * (0.4 + 0.6 * rng.next());
            let life = k.explosion_seconds * (0.6 + 0.4 * rng.next());
            let tint = if i % 3 == 0 { Color::WHITE } else { *color };
            commands.spawn((
                Sprite::from_color(tint, Vec2::splat(2.0)),
                Transform::from_translation(at.round().extend(30.0)),
                Velocity(Vec2::from_angle(angle) * speed),
                Lifetime(life),
                Particle { total: life },
                DespawnOnExit(Screen::Playing),
            ));
        }
    }
}

pub(super) fn fade_particles(mut particles: Query<(&Particle, &Lifetime, &mut Sprite)>) {
    for (particle, life, mut sprite) in &mut particles {
        let alpha = (life.0 / particle.total).clamp(0.0, 1.0);
        sprite.color.set_alpha(alpha);
    }
}

pub(super) fn start_flashes(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut requests: MessageReader<Flash>,
    mut sprites: Query<(&Sprite, Option<&mut HitFlash>)>,
) {
    for Flash(entity) in requests.read() {
        let Ok((sprite, flash)) = sprites.get_mut(*entity) else {
            continue;
        };
        let left = tuning.kit.hit_flash_seconds;
        match flash {
            Some(mut flash) => flash.left = left,
            None => {
                commands.entity(*entity).try_insert(HitFlash {
                    left,
                    original: sprite.color,
                });
            }
        }
    }
}

pub(super) fn run_flashes(
    mut commands: Commands,
    time: Res<Time>,
    mut flashes: Query<(Entity, &mut HitFlash, &mut Sprite)>,
) {
    for (entity, mut flash, mut sprite) in &mut flashes {
        flash.left -= time.delta_secs();
        if flash.left > 0.0 {
            sprite.color = Color::WHITE;
        } else {
            sprite.color = flash.original;
            // The entity may be despawned this frame (shot down, gone off screen).
            commands.entity(entity).try_remove::<HitFlash>();
        }
    }
}

pub(super) fn shake_camera(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut rng: ResMut<EffectRng>,
    mut trauma: ResMut<Trauma>,
    mut requests: MessageReader<Shake>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    for Shake(amount) in requests.read() {
        trauma.0 = (trauma.0 + amount).min(1.0);
    }
    trauma.0 = (trauma.0 - tuning.kit.shake_decay * time.delta_secs()).max(0.0);
    // Squared so small shakes stay subtle; whole pixels keep the art crisp.
    let strength = trauma.0 * trauma.0 * tuning.kit.shake_max_offset;
    let offset = Vec2::new(rng.next() * 2.0 - 1.0, rng.next() * 2.0 - 1.0) * strength;
    camera.translation.x = offset.x.round();
    camera.translation.y = offset.y.round();
}

pub(super) fn reset_camera(
    mut trauma: ResMut<Trauma>,
    mut freeze: ResMut<ImpactFreeze>,
    mut camera: Single<&mut Transform, With<GameCamera>>,
) {
    trauma.0 = 0.0;
    freeze.0 = 0.0;
    camera.translation.x = 0.0;
    camera.translation.y = 0.0;
}

pub(super) fn blink(time: Res<Time>, mut blinkers: Query<(&mut Invulnerable, &mut Visibility)>) {
    for (mut invulnerable, mut visibility) in &mut blinkers {
        invulnerable.0 = (invulnerable.0 - time.delta_secs()).max(0.0);
        let on = invulnerable.0 == 0.0 || ((invulnerable.0 * 10.0) as u32).is_multiple_of(2);
        let wanted = if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
        visibility.set_if_neq(wanted);
    }
}
