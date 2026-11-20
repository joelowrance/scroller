//! Movement and clean-up for anything that flies: projectiles, enemies,
//! particles.

use bevy::prelude::*;

use super::collision::{Hitbox, Layer};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;

/// Pixels per second.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Velocity(pub Vec2);

/// Seconds left before the entity despawns.
#[derive(Component, Clone, Copy, Debug)]
pub struct Lifetime(pub f32);

/// Despawns the entity once it has been on screen and then left it by more
/// than `margin` pixels. Things that spawn off screen and fly in are safe
/// until they first appear.
#[derive(Component, Clone, Copy, Debug)]
pub struct DespawnOffscreen {
    pub margin: f32,
    pub entered: bool,
}

impl DespawnOffscreen {
    pub fn new(margin: f32) -> Self {
        Self {
            margin,
            entered: false,
        }
    }

    /// Updates for a new position and returns whether to despawn.
    pub fn should_despawn(&mut self, at: Vec2) -> bool {
        let inside = |m: f32| at.x.abs() <= HALF_W + m && at.y.abs() <= HALF_H + m;
        if inside(0.0) {
            self.entered = true;
        }
        self.entered && !inside(self.margin)
    }
}

impl Default for DespawnOffscreen {
    fn default() -> Self {
        Self::new(16.0)
    }
}

/// A shot as a plain coloured box: its hitbox matches its size, it flies at
/// `velocity`, and it despawns when it leaves the screen or its `lifetime`
/// (seconds) runs out.
pub fn projectile(
    layer: Layer,
    at: Vec2,
    velocity: Vec2,
    size: Vec2,
    color: Color,
    lifetime: f32,
) -> impl Bundle {
    (
        Sprite::from_color(color, size),
        Transform::from_translation(at.extend(8.0)),
        Hitbox::new(size.x, size.y),
        layer,
        Velocity(velocity),
        Lifetime(lifetime),
        DespawnOffscreen::new(4.0),
        DespawnOnExit(Screen::Playing),
    )
}

pub(super) fn apply_velocity(time: Res<Time>, mut movers: Query<(&Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (velocity, mut transform) in &mut movers {
        transform.translation += (velocity.0 * dt).extend(0.0);
    }
}

pub(super) fn expire(
    mut commands: Commands,
    time: Res<Time>,
    mut lifetimes: Query<(Entity, &mut Lifetime)>,
    mut offscreen: Query<(Entity, &mut DespawnOffscreen, &Transform)>,
) {
    for (entity, mut life) in &mut lifetimes {
        life.0 -= time.delta_secs();
        if life.0 <= 0.0 {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, mut rule, transform) in &mut offscreen {
        if rule.should_despawn(transform.translation.truncate()) {
            commands.entity(entity).try_despawn();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn offscreen_despawn_waits_until_the_entity_has_appeared() {
        let mut rule = DespawnOffscreen::new(8.0);
        assert!(!rule.should_despawn(Vec2::new(HALF_W + 30.0, 0.0)));
        assert!(!rule.should_despawn(Vec2::new(HALF_W - 1.0, 0.0)));
        assert!(!rule.should_despawn(Vec2::new(HALF_W + 5.0, 0.0)));
        assert!(rule.should_despawn(Vec2::new(HALF_W + 9.0, 0.0)));
    }
}
