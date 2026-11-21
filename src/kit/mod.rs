//! The shared gameplay kit: the engine pieces every Cabinet uses, so a
//! Cabinet contains only its own rules and content.
//!
//! A Cabinet using the kit:
//! 1. On `OnEnter(Screen::Playing)`, inserts a [`StageRun`] built from its
//!    Stage file and spawns the [`Hero`] (with [`Invulnerable`], a [`Hitbox`]
//!    and [`Layer::Hero`]) at [`StageRun::start_point`].
//! 2. Reads [`SpawnRequested`] and spawns what each kind means, with
//!    [`Health`], [`ScoreValue`], a [`Hitbox`] and a [`Layer`].
//! 3. Fires shots with [`projectile`].
//!
//! The kit then moves everything with a [`Velocity`], finds [`Contact`]s,
//! applies damage and score, explodes enemies, turns hits on the Hero into
//! [`LifeLost`](crate::cabinets::LifeLost) plus a rewind to the latest
//! Restart Point ([`StageRestarted`]), and sends
//! [`StageCleared`](crate::cabinets::StageCleared) when the scroll reaches
//! the end.
//!
//! Each frame runs the [`KitSet`]s in order. Cabinet systems that react to
//! spawns go after [`KitSet::Timeline`]; movement and AI go before
//! [`KitSet::Collide`]. Gate them with the [`stage_live`] run condition, so
//! they stop for pauses and hit-stops.

// Parts of the kit's API are there for the real Cabinets and not used by the
// placeholder yet.
#![allow(dead_code, unused_imports)]

use bevy::prelude::*;

pub mod collision;
pub mod combat;
pub mod effects;
pub mod motion;
pub mod scenery;
pub mod stage;
pub mod timeline;

pub use collision::TerrainHitbox;
pub use collision::{Contact, Hitbox, Layer};
pub use combat::{
    Damage, EnemyDestroyed, Health, Hero, HeroStruck, HitStop, Hurt, Immune, Invulnerable, Pickup,
    PickupCollected, ScoreValue, Shield, ShieldAbsorbed,
};
pub use effects::{Explode, Flash, Shake};
pub use motion::{DespawnOffscreen, Lifetime, Velocity, projectile};
pub use stage::{HitStopTimer, StageRestarted, stage_live};
pub use timeline::{SpawnRequested, StageRun, StageTimeline};

use crate::flow::{Screen, gameplay_running};

/// The kit's steps within a frame, run in this order while a Stage plays.
#[derive(SystemSet, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum KitSet {
    /// Scroll the timeline and request spawns.
    Timeline,
    /// Velocity, lifetimes and off-screen clean-up.
    Move,
    /// Find contacts.
    Collide,
    /// Damage, score, lost lives and restarts.
    Resolve,
    /// Explosions, flashes, shake and blinking.
    Effects,
}

fn not_frozen(hit_stop: Res<stage::HitStopTimer>) -> bool {
    hit_stop.0.is_none()
}

pub struct KitPlugin;

impl Plugin for KitPlugin {
    fn build(&self, app: &mut App) {
        use KitSet::*;
        app.add_message::<Contact>()
            .add_message::<SpawnRequested>()
            .add_message::<HeroStruck>()
            .add_message::<PickupCollected>()
            .add_message::<EnemyDestroyed>()
            .add_message::<StageRestarted>()
            .add_message::<Explode>()
            .add_message::<Flash>()
            .add_message::<Shake>()
            .add_message::<Hurt>()
            .add_message::<ShieldAbsorbed>()
            .init_resource::<stage::HitStopTimer>()
            .init_resource::<effects::Trauma>()
            .init_resource::<effects::EffectRng>()
            .configure_sets(
                Update,
                (Timeline, Move, Collide, Resolve, Effects)
                    .chain()
                    .run_if(in_state(Screen::Playing))
                    .run_if(gameplay_running)
                    .run_if(resource_exists::<StageRun>),
            )
            // A hit-stop freezes everything except the step that ends it.
            .configure_sets(
                Update,
                (Timeline, Move, Collide, Effects).run_if(not_frozen),
            )
            .add_systems(
                Update,
                (
                    (stage::debug_keys, stage::advance_timeline)
                        .chain()
                        .in_set(Timeline),
                    (motion::apply_velocity, motion::expire)
                        .chain()
                        .in_set(Move),
                    collision::detect.in_set(Collide),
                    (
                        combat::resolve_contacts,
                        combat::apply_hurt,
                        stage::restart_after_hit,
                    )
                        .chain()
                        .in_set(Resolve),
                    (
                        effects::spawn_explosions,
                        effects::fade_particles,
                        effects::start_flashes,
                        effects::run_flashes,
                        effects::shake_camera,
                        effects::blink,
                    )
                        .chain()
                        .in_set(Effects),
                ),
            )
            .add_systems(
                Update,
                stage::apply_time_scale.run_if(in_state(Screen::Playing)),
            )
            .add_systems(
                Update,
                scenery::sync_scenery
                    .after(Timeline)
                    .run_if(in_state(Screen::Playing))
                    .run_if(resource_exists::<StageRun>)
                    .run_if(resource_exists::<crate::run::Session>),
            )
            .add_systems(
                OnExit(Screen::Playing),
                (
                    stage::end_stage,
                    stage::reset_time_scale,
                    effects::reset_camera,
                ),
            );
    }
}

#[cfg(test)]
mod tests;
