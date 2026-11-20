//! Cabinet 1's music and enemy sounds (docs/cabinets/1-side-shooter.md,
//! "Sound and music").
//!
//! - Music: which song each Stage plays, here.
//! - Enemy hits, explosions and shell pings: here, from the kit's messages.
//! - The Wren's shots, Undertow, Ward and Power-ups, the Arc Gate hum and the
//!   Hook warning: sent where they happen (`hero.rs`, `power_ups.rs`,
//!   `enemies/mod.rs`). Lost and extra lives: `src/audio`, for every Cabinet.

use std::collections::HashMap;

use bevy::prelude::*;

use super::CABINET;
use super::enemies::{Enemy, Kind};
use crate::audio::{PlaySfx, Sfx, Song, StageMusic};
use crate::cabinets::cabinet_is;
use crate::flow::Screen;
use crate::kit::{EnemyDestroyed, Flash, Hurt, Immune, KitSet, StageRun};
use crate::run::Session;

/// Stage 2 switches to the climbing run-out music when this many seconds of
/// scroll are left (the design's "last 20 s").
const CLIMB_SECONDS: f32 = 20.0;

/// The song for a Cabinet 1 Stage, given the seconds of scroll left.
pub fn stage_song(stage: usize, seconds_left: f32) -> Song {
    match stage {
        0 => Song::LowTide,
        _ if seconds_left <= CLIMB_SECONDS => Song::FoundryClimb,
        _ => Song::Foundry,
    }
}

/// Seconds until the scroll ends, following the Stage's speed keys, so a
/// speed change in the last stretch can't flip the song back and forth.
fn seconds_left(run: &StageRun) -> f32 {
    let timeline = &run.timeline;
    let end = timeline.length;
    let mut at = run.cursor.position.min(end);
    let mut seconds = 0.0;
    while at < end {
        // The next speed key after `at` ends this stretch.
        let next = timeline
            .speed
            .iter()
            .map(|k| k.from)
            .filter(|&from| from > at)
            .fold(end, f32::min);
        seconds += (next - at) / timeline.speed_at(at).max(1.0);
        at = next;
    }
    seconds
}

/// Keeps [`StageMusic`] on the right song. It runs every frame, so a rewind
/// to a Restart Point before the run-out brings the Stage 2 music back.
fn choose_stage_music(
    session: Res<Session>,
    run: Option<Res<StageRun>>,
    mut music: ResMut<StageMusic>,
) {
    let left = run.map_or(f32::INFINITY, |r| seconds_left(&r));
    music.set_if_neq(StageMusic(Some(stage_song(session.stage, left))));
}

/// The explosion for a destroyed enemy: big ones get the large one.
pub fn explosion_for(kind: Kind) -> Sfx {
    match kind {
        Kind::Ferry | Kind::Lamplighter | Kind::ArcPylon => Sfx::ExplosionLarge,
        _ => Sfx::ExplosionSmall,
    }
}

/// Each live enemy's kind. [`EnemyDestroyed`] arrives as the enemy is being
/// despawned, so its kind is remembered from when it spawned.
#[derive(Resource, Default)]
struct EnemyKinds(HashMap<Entity, Kind>);

fn enemy_sounds(
    mut kinds: ResMut<EnemyKinds>,
    spawned: Query<(Entity, &Enemy), Added<Enemy>>,
    mut gone: RemovedComponents<Enemy>,
    mut hurts: MessageReader<Hurt>,
    mut flashes: MessageReader<Flash>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    shells: Query<(), (With<Enemy>, With<Immune>)>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for (entity, enemy) in &spawned {
        kinds.0.insert(entity, enemy.kind);
    }
    // A shot (or the Undertow) that can't hurt a closed shell glances off.
    if hurts.read().any(|h| shells.contains(h.target)) {
        sfx.write(PlaySfx(Sfx::ShellPing));
    }
    // The kit flashes an enemy that was hit and survived.
    if flashes.read().count() > 0 {
        sfx.write(PlaySfx(Sfx::EnemyHit));
    }
    for d in destroyed.read() {
        let kind = kinds.0.get(&d.entity).copied().unwrap_or(Kind::Skiff);
        sfx.write(PlaySfx(explosion_for(kind)));
    }
    // Forget enemies only after their EnemyDestroyed has been read.
    for entity in gone.read() {
        kinds.0.remove(&entity);
    }
}

pub struct SoundPlugin;

impl Plugin for SoundPlugin {
    fn build(&self, app: &mut App) {
        // Also set up by the audio plugin; repeated so the Cabinet runs
        // (and its tests build) without it.
        app.init_resource::<StageMusic>()
            .add_message::<PlaySfx>()
            .init_resource::<EnemyKinds>()
            .add_systems(
                Update,
                (
                    choose_stage_music.run_if(resource_exists::<Session>),
                    enemy_sounds.after(KitSet::Resolve).before(KitSet::Effects),
                )
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
    use crate::kit::StageTimeline;

    #[test]
    fn each_stage_has_its_song_and_stage_2_climbs_at_the_end() {
        assert_eq!(stage_song(0, 5.0), Song::LowTide);
        assert_eq!(stage_song(1, 60.0), Song::Foundry);
        assert_eq!(stage_song(1, 20.0), Song::FoundryClimb);
        assert_eq!(stage_song(1, 0.0), Song::FoundryClimb);
    }

    #[test]
    fn big_enemies_explode_large() {
        assert_eq!(explosion_for(Kind::Ferry), Sfx::ExplosionLarge);
        assert_eq!(explosion_for(Kind::Lamplighter), Sfx::ExplosionLarge);
        assert_eq!(explosion_for(Kind::ArcPylon), Sfx::ExplosionLarge);
        assert_eq!(explosion_for(Kind::Skiff), Sfx::ExplosionSmall);
        assert_eq!(explosion_for(Kind::Mote), Sfx::ExplosionSmall);
    }

    #[test]
    fn seconds_left_follows_the_speed_keys() {
        let timeline = StageTimeline::parse(
            "(length: 1000.0, speed: [(from: 0.0, speed: 30.0), (from: 600.0, speed: 40.0), \
             (from: 900.0, speed: 20.0)])",
        )
        .unwrap();
        let mut run = StageRun::new(timeline);
        run.cursor.position = 300.0;
        assert_eq!(seconds_left(&run), 10.0 + 7.5 + 5.0);
        run.cursor.position = 800.0;
        assert_eq!(seconds_left(&run), 2.5 + 5.0);
        run.cursor.position = 1000.0;
        assert_eq!(seconds_left(&run), 0.0);
    }

    #[test]
    fn a_slowdown_near_the_end_does_not_undo_the_climb() {
        // 40 px/s, then 10 px/s for the last 100 px: once 20 s are left the
        // estimate only falls, so the song never flips back.
        let timeline = StageTimeline::parse(
            "(length: 1000.0, speed: [(from: 0.0, speed: 40.0), (from: 900.0, speed: 10.0)])",
        )
        .unwrap();
        let mut run = StageRun::new(timeline);
        let mut last = f32::INFINITY;
        for step in 0..=100 {
            run.cursor.position = step as f32 * 10.0;
            let left = seconds_left(&run);
            assert!(left <= last);
            last = left;
        }
    }
}
