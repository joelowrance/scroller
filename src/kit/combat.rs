//! Health, damage and score: what each [`Contact`] means.
//!
//! - A Hero shot hitting an enemy deals its [`Damage`] as a [`Hurt`]. Any
//!   system can send [`Hurt`] (a bomb, say). An enemy with no [`Health`] left
//!   explodes and awards its [`ScoreValue`] as [`ScoreGained`]. An
//!   [`Immune`] enemy takes no damage, but still uses up the shot.
//! - An enemy, enemy shot or terrain touching the Hero sends [`HeroStruck`]
//!   (see `stage.rs` for what follows).
//! - A [`Pickup`] touching the Hero is collected.
//! - Shots stop at terrain, and enemy shots stop at a [`Layer::ShotBlocker`].

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

/// Absorbs hits on the Hero that would cost a life, one per charge, then
/// makes the Hero invulnerable for a moment.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct Shield {
    pub charges: u32,
    pub invulnerable_seconds: f32,
}

/// Seconds the game freezes after the Hero is hit, before the explosion and
/// the rewind. Put it on the Hero; without it there is no hit-stop.
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct HitStop(pub f32);

/// Something hit the Hero: `from` is what hit it (None for the debug K key).
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct HeroStruck {
    pub from: Option<Entity>,
}

/// A [`Shield`] charge absorbed a hit from `from`.
#[derive(Message, Clone, Copy, Debug, Default)]
pub struct ShieldAbsorbed {
    pub from: Option<Entity>,
}

/// Damage to an enemy, from a shot or anything else.
#[derive(Message, Clone, Copy, Debug)]
pub struct Hurt {
    pub target: Entity,
    pub damage: u32,
}

#[derive(Message, Clone, Debug)]
pub struct PickupCollected {
    pub kind: String,
    pub at: Vec2,
}

/// An enemy was shot down (its points are sent separately as [`ScoreGained`]).
#[derive(Message, Clone, Copy, Debug)]
pub struct EnemyDestroyed {
    /// The enemy, despawned at the end of this frame.
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
    pickups: Query<(&Pickup, &Transform)>,
    mut hurt: MessageWriter<Hurt>,
    mut struck: MessageWriter<HeroStruck>,
    mut collected: MessageWriter<PickupCollected>,
) {
    // Shots and pickups used up earlier this frame (despawns apply later).
    let mut spent = HashSet::new();
    let mut hero_hit = None;
    for c in contacts.read() {
        if spent.contains(&c.a) || spent.contains(&c.b) {
            continue;
        }
        match (c.a_layer, c.b_layer) {
            (Layer::HeroShot, Layer::Enemy | Layer::Ground) | (Layer::Rocket, Layer::Ground) => {
                spent.insert(c.a);
                commands.entity(c.a).try_despawn();
                hurt.write(Hurt {
                    target: c.b,
                    damage: damage.get(c.a).map_or(1, |d| d.0),
                });
            }
            (Layer::EnemyShot | Layer::Enemy | Layer::Terrain, Layer::Hero) => {
                if c.a_layer == Layer::EnemyShot {
                    spent.insert(c.a);
                    commands.entity(c.a).try_despawn();
                }
                if !hero_invincible(&tuning) && hero_hit.is_none() {
                    hero_hit = Some(c.a);
                }
            }
            (Layer::EnemyShot, Layer::ShotBlocker) => {
                spent.insert(c.a);
                commands.entity(c.a).try_despawn();
            }
            (Layer::Pickup, Layer::Hero) => {
                spent.insert(c.a);
                commands.entity(c.a).try_despawn();
                if let Ok((pickup, transform)) = pickups.get(c.a) {
                    collected.write(PickupCollected {
                        kind: pickup.kind.clone(),
                        at: transform.translation.truncate(),
                    });
                }
            }
            (Layer::Terrain, _) => {
                spent.insert(c.b);
                commands.entity(c.b).try_despawn();
            }
            _ => {}
        }
    }
    if let Some(from) = hero_hit {
        struck.write(HeroStruck { from: Some(from) });
    }
}

/// Applies every [`Hurt`]: flashes survivors, explodes and scores the rest.
pub(super) fn apply_hurt(
    mut commands: Commands,
    mut hurts: MessageReader<Hurt>,
    mut enemies: Query<(
        &mut Health,
        Option<&ScoreValue>,
        &Transform,
        Option<&Sprite>,
        Has<Immune>,
    )>,
    mut score: MessageWriter<ScoreGained>,
    mut destroyed: MessageWriter<EnemyDestroyed>,
    mut explode: MessageWriter<Explode>,
    mut flash: MessageWriter<Flash>,
) {
    for hurt in hurts.read() {
        let Ok((mut health, value, transform, sprite, immune)) = enemies.get_mut(hurt.target)
        else {
            continue;
        };
        if immune {
            continue;
        }
        if health.take(hurt.damage) {
            commands.entity(hurt.target).try_despawn();
            let at = transform.translation.truncate();
            let points = value.map_or(0, |v| v.0);
            if points > 0 {
                score.write(ScoreGained(points));
            }
            destroyed.write(EnemyDestroyed {
                entity: hurt.target,
                at,
                score: points,
            });
            let color = sprite.map_or(Color::WHITE, |s| s.color);
            explode.write(Explode { at, color });
        } else if health.0 > 0 {
            flash.write(Flash(hurt.target));
        }
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
