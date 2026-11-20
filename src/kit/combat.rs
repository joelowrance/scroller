//! Health, damage and score: what each [`Contact`] means.
//!
//! - A Hero shot hitting an enemy deals its [`Damage`]; an enemy with no
//!   [`Health`] left explodes and awards its [`ScoreValue`] as
//!   [`ScoreGained`].
//! - An enemy, enemy shot or terrain touching the Hero sends [`HeroStruck`]
//!   (see `restart.rs` for what follows).
//! - A [`Pickup`] touching the Hero is collected.
//! - Shots stop at terrain.

use std::collections::HashSet;

use bevy::prelude::*;

use super::collision::{Contact, Layer};
use super::effects::{Explode, Flash};
use crate::cabinets::ScoreGained;
use crate::tuning::Tuning;

/// The entity the player controls. Exactly one exists during a Stage.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Hero;

/// Seconds the Hero still can't be hit (it blinks meanwhile).
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Invulnerable(pub f32);

#[derive(Component, Clone, Copy, Debug)]
pub struct Health(pub u32);

impl Health {
    /// Takes damage and returns true if this destroyed it.
    pub fn take(&mut self, damage: u32) -> bool {
        let alive = self.0 > 0;
        self.0 = self.0.saturating_sub(damage);
        alive && self.0 == 0
    }
}

/// An enemy that shots can't hurt right now (a closed shell, say). Shots
/// that hit it are still used up.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Immune;

/// Points awarded when this enemy is destroyed.
#[derive(Component, Clone, Copy, Debug)]
pub struct ScoreValue(pub u64);

/// Damage a shot deals. Shots without it deal 1.
#[derive(Component, Clone, Copy, Debug)]
pub struct Damage(pub u32);

/// A collectable; the Cabinet decides what each kind does.
#[derive(Component, Clone, Debug)]
pub struct Pickup {
    pub kind: String,
}

/// Something hit the Hero. The debug K key sends it too.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct HeroStruck;

#[derive(Message, Clone, Debug)]
pub struct PickupCollected {
    pub kind: String,
    pub at: Vec2,
}

/// An enemy was shot down (its points are sent separately as [`ScoreGained`]).
#[derive(Message, Clone, Copy, Debug)]
pub struct EnemyDestroyed {
    /// The enemy (already being despawned).
    pub entity: Entity,
    pub at: Vec2,
    pub score: u64,
}

/// The debug aid that ignores hits on the Hero.
pub fn hero_invincible(tuning: &Tuning) -> bool {
    tuning.debug_keys && tuning.invincible_hero
}

pub(super) fn resolve_contacts(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut contacts: MessageReader<Contact>,
    damage: Query<&Damage>,
    mut enemies: Query<(
        &mut Health,
        Option<&ScoreValue>,
        &Transform,
        Option<&Sprite>,
    )>,
    immune: Query<(), With<Immune>>,
    pickups: Query<(&Pickup, &Transform)>,
    mut struck: MessageWriter<HeroStruck>,
    mut score: MessageWriter<ScoreGained>,
    mut destroyed: MessageWriter<EnemyDestroyed>,
    mut collected: MessageWriter<PickupCollected>,
    mut explode: MessageWriter<Explode>,
    mut flash: MessageWriter<Flash>,
) {
    // Entities used up earlier this frame (despawns apply later).
    let mut gone = HashSet::new();
    let mut hero_hit = false;
    for c in contacts.read() {
        if gone.contains(&c.a) || gone.contains(&c.b) {
            continue;
        }
        match (c.a_layer, c.b_layer) {
            (Layer::HeroShot, Layer::Enemy) => {
                gone.insert(c.a);
                commands.entity(c.a).try_despawn();
                let Ok((mut health, value, transform, sprite)) = enemies.get_mut(c.b) else {
                    continue;
                };
                if immune.contains(c.b) {
                    continue;
                }
                let dealt = damage.get(c.a).map_or(1, |d| d.0);
                let at = transform.translation.truncate();
                if health.take(dealt) {
                    gone.insert(c.b);
                    commands.entity(c.b).try_despawn();
                    let points = value.map_or(0, |v| v.0);
                    if points > 0 {
                        score.write(ScoreGained(points));
                    }
                    destroyed.write(EnemyDestroyed {
                        entity: c.b,
                        at,
                        score: points,
                    });
                    let color = sprite.map_or(Color::WHITE, |s| s.color);
                    explode.write(Explode { at, color });
                } else {
                    flash.write(Flash(c.b));
                }
            }
            (Layer::EnemyShot | Layer::Enemy | Layer::Terrain, Layer::Hero) => {
                if c.a_layer == Layer::EnemyShot {
                    gone.insert(c.a);
                    commands.entity(c.a).try_despawn();
                }
                if !hero_invincible(&tuning) {
                    hero_hit = true;
                }
            }
            (Layer::Pickup, Layer::Hero) => {
                gone.insert(c.a);
                commands.entity(c.a).try_despawn();
                if let Ok((pickup, transform)) = pickups.get(c.a) {
                    collected.write(PickupCollected {
                        kind: pickup.kind.clone(),
                        at: transform.translation.truncate(),
                    });
                }
            }
            (Layer::Terrain, _) => {
                gone.insert(c.b);
                commands.entity(c.b).try_despawn();
            }
            _ => {}
        }
    }
    if hero_hit {
        struck.write_default();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_reports_the_killing_blow_once() {
        let mut h = Health(3);
        assert!(!h.take(1));
        assert!(!h.take(1));
        assert!(h.take(5));
        assert_eq!(h.0, 0);
        assert!(!h.take(1));
    }
}
