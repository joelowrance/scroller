//! Cabinet 2's enemy sounds (docs/cabinets/2-vertical-shooter.md, "Sound and
//! music").
//!
//! - Which song each Stage plays: `src/audio/stage_music.rs`.
//! - Enemy hits, explosions, shield sparks and falling shuttles: here, from
//!   the kit's messages.
//! - The Wren's shots, Rockets, Squall, Ward and Power-ups, and each enemy's
//!   warnings and machinery (Swift chevrons, flak, Riser launches, the Loom,
//!   Silos, Tethers): sent where they happen (`hero.rs`, `power_ups.rs`,
//!   `enemies/mod.rs`).
//! - The final hit that ends Stage 2 belongs to the 2 → 3 Cabinet
//!   Transition (#21).

use std::collections::HashMap;

use bevy::prelude::*;

use super::CABINET;
use super::enemies::{Enemy, Kind};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::cabinet_is;
use crate::flow::Screen;
use crate::kit::{EnemyDestroyed, Flash, Hurt, Immune, KitSet};

/// The sound of a destroyed enemy: big ones get the large explosion, and a
/// shot-off Loom shuttle falls away.
pub fn destroyed_sound(kind: Kind) -> Sfx {
    match kind {
        Kind::LoomShuttle => Sfx::ShuttleFall,
        Kind::LoomCore | Kind::Bellows | Kind::Flak | Kind::TetherBalloon => Sfx::ExplosionLarge,
        _ => Sfx::ExplosionSmall,
    }
}

/// Each live enemy's kind, remembered from when it spawned, because
/// [`EnemyDestroyed`] arrives as the enemy is being despawned.
#[derive(Resource, Default)]
struct EnemyKinds(HashMap<Entity, Kind>);

fn enemy_sounds(
    mut kinds: ResMut<EnemyKinds>,
    spawned: Query<(Entity, &Enemy), Added<Enemy>>,
    mut gone: RemovedComponents<Enemy>,
    mut hurts: MessageReader<Hurt>,
    mut flashes: MessageReader<Flash>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    shielded: Query<(), (With<Enemy>, With<Immune>)>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for (entity, enemy) in &spawned {
        kinds.0.insert(entity, enemy.kind);
    }
    // A shot that can't hurt a closed Silo or a shielded Loom sparks off.
    if hurts.read().any(|h| shielded.contains(h.target)) {
        sfx.write(PlaySfx(Sfx::ShellPing));
    }
    if flashes.read().count() > 0 {
        sfx.write(PlaySfx(Sfx::EnemyHit));
    }
    for d in destroyed.read() {
        let kind = kinds.0.get(&d.entity).copied().unwrap_or(Kind::Wisp);
        sfx.write(PlaySfx(destroyed_sound(kind)));
    }
    for entity in gone.read() {
        kinds.0.remove(&entity);
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        // Also set up by the audio plugin; repeated so the Cabinet runs
        // (and its tests build) without it.
        app.add_message::<PlaySfx>()
            .init_resource::<EnemyKinds>()
            .add_systems(
                Update,
                enemy_sounds
                    .after(KitSet::Resolve)
                    .before(KitSet::Effects)
                    .run_if(in_state(Screen::Playing))
                    .run_if(cabinet_is(CABINET)),
            )
            .add_systems(OnExit(Screen::Playing), |mut kinds: ResMut<EnemyKinds>| {
                kinds.0.clear();
            });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shuttles_fall_and_big_enemies_explode_large() {
        assert_eq!(destroyed_sound(Kind::LoomShuttle), Sfx::ShuttleFall);
        assert_eq!(destroyed_sound(Kind::LoomCore), Sfx::ExplosionLarge);
        assert_eq!(destroyed_sound(Kind::Wisp), Sfx::ExplosionSmall);
    }
}
