//! The kit's systems working together in a headless App.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::*;
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::tuning::Tuning;

const STAGE: &str = r#"(
    length: 100.0,
    speed: [(from: 0.0, speed: 100.0)],
    restart_points: [(at: 0.0, hero: (-50.0, 0.0)), (at: 20.0, hero: (-60.0, 10.0))],
    spawns: [(at: 5.0, kind: "drone", pos: (100.0, 0.0))],
)"#;

/// Everything the kit reported, collected each frame.
#[derive(Resource, Default)]
struct Log {
    score: Vec<u64>,
    lives_lost: usize,
    cleared: usize,
    spawns: Vec<String>,
    restarts: Vec<Vec2>,
}

fn collect(
    mut log: ResMut<Log>,
    mut score: MessageReader<ScoreGained>,
    mut lost: MessageReader<LifeLost>,
    mut cleared: MessageReader<StageCleared>,
    mut spawns: MessageReader<SpawnRequested>,
    mut restarts: MessageReader<StageRestarted>,
) {
    log.score.extend(score.read().map(|s| s.0));
    log.lives_lost += lost.read().count();
    log.cleared += cleared.read().count();
    log.spawns.extend(spawns.read().map(|s| s.kind.clone()));
    log.restarts.extend(restarts.read().map(|r| r.hero));
}

/// An App playing a Stage, with a frame step of `dt` seconds.
fn app(dt: f32, tuning: Tuning) -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            dt,
        )))
        .insert_resource(tuning)
        .init_resource::<Paused>()
        .init_resource::<Log>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_state(Screen::Playing)
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins(KitPlugin)
        .add_systems(Last, collect)
        .insert_resource(StageRun::new(StageTimeline::parse(STAGE).unwrap()));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app
}

fn quiet_tuning() -> Tuning {
    Tuning {
        debug_keys: false,
        ..Tuning::default()
    }
}

fn spawn_hero(app: &mut App, at: Vec2) -> Entity {
    app.world_mut()
        .spawn((
            Hero,
            Invulnerable(0.0),
            Hitbox::new(6.0, 6.0),
            Layer::Hero,
            Transform::from_translation(at.extend(0.0)),
            Visibility::default(),
        ))
        .id()
}

fn spawn_enemy(app: &mut App, at: Vec2, hp: u32, score: u64) -> Entity {
    app.world_mut()
        .spawn((
            Hitbox::new(10.0, 10.0),
            Layer::Enemy,
            Health(hp),
            ScoreValue(score),
            Transform::from_translation(at.extend(0.0)),
        ))
        .id()
}

fn spawn_shot(app: &mut App, layer: Layer, at: Vec2) -> Entity {
    app.world_mut()
        .spawn(projectile(
            layer,
            at,
            Vec2::ZERO,
            Vec2::splat(2.0),
            Color::WHITE,
            5.0,
        ))
        .id()
}

fn alive(app: &App, e: Entity) -> bool {
    app.world().get_entity(e).is_ok()
}

#[test]
fn destroying_an_enemy_scores_its_value_once() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::new(-100.0, 0.0));
    let enemy = spawn_enemy(&mut app, Vec2::new(50.0, 0.0), 2, 150);

    // Two shots in the same frame: the first damages, the second destroys.
    let s1 = spawn_shot(&mut app, Layer::HeroShot, Vec2::new(48.0, 0.0));
    let s2 = spawn_shot(&mut app, Layer::HeroShot, Vec2::new(52.0, 0.0));
    app.update();
    assert!(!alive(&app, s1) && !alive(&app, s2), "shots are spent");
    assert!(!alive(&app, enemy));
    assert_eq!(app.world().resource::<Log>().score, [150]);

    // A third shot arriving later finds nothing and scores nothing.
    let s3 = spawn_shot(&mut app, Layer::HeroShot, Vec2::new(50.0, 0.0));
    app.update();
    app.update();
    assert!(alive(&app, s3));
    assert_eq!(app.world().resource::<Log>().score, [150]);
}

#[test]
fn a_damaged_enemy_survives_and_scores_nothing() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::new(-100.0, 0.0));
    let enemy = spawn_enemy(&mut app, Vec2::new(50.0, 0.0), 3, 100);
    spawn_shot(&mut app, Layer::HeroShot, Vec2::new(50.0, 0.0));
    app.update();
    assert!(alive(&app, enemy));
    assert_eq!(app.world().get::<Health>(enemy).unwrap().0, 2);
    assert!(app.world().resource::<Log>().score.is_empty());
}

#[test]
fn hero_hit_loses_a_life_and_rewinds_to_the_latest_restart_point() {
    let mut app = app(0.1, quiet_tuning());
    let hero = spawn_hero(&mut app, Vec2::new(-50.0, 0.0));
    // Scroll to position 30: past the Restart Point at 20.
    app.update(); // first frame has no delta
    app.update();
    app.update();
    app.update();
    assert_eq!(app.world().resource::<StageRun>().cursor.position, 30.0);
    assert_eq!(app.world().resource::<Log>().spawns, ["drone"]);

    let enemy = spawn_enemy(&mut app, Vec2::new(100.0, 50.0), 1, 10);
    let shot = spawn_shot(&mut app, Layer::EnemyShot, Vec2::new(-50.0, 0.0));
    app.update();

    let log = app.world().resource::<Log>();
    assert_eq!(log.lives_lost, 1);
    assert_eq!(log.restarts, [Vec2::new(-60.0, 10.0)]);
    assert!(
        !alive(&app, enemy) && !alive(&app, shot),
        "Stage is cleared of enemies"
    );
    let at = app.world().get::<Transform>(hero).unwrap().translation;
    assert_eq!(at.truncate(), Vec2::new(-60.0, 10.0));
    assert!(app.world().get::<Invulnerable>(hero).unwrap().0 > 0.0);
    // Rewound to 20, then one more frame of scroll.
    assert!(app.world().resource::<StageRun>().cursor.position <= 30.0);

    // While invulnerable, more hits are ignored.
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::new(-60.0, 10.0));
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
}

#[test]
fn many_hits_in_one_frame_cost_one_life() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::ZERO);
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::new(1.0, 0.0));
    spawn_enemy(&mut app, Vec2::ZERO, 1, 0);
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
}

#[test]
fn invincible_hero_debug_aid_ignores_hits() {
    let tuning = Tuning {
        debug_keys: true,
        invincible_hero: true,
        ..Tuning::default()
    };
    let mut app = app(0.0, tuning);
    spawn_hero(&mut app, Vec2::ZERO);
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    spawn_enemy(&mut app, Vec2::ZERO, 1, 0);
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 0);
}

#[test]
fn stage_clears_once_at_the_end_of_the_scroll() {
    let mut app = app(0.25, quiet_tuning());
    spawn_hero(&mut app, Vec2::ZERO);
    for _ in 0..20 {
        app.update();
    }
    let log = app.world().resource::<Log>();
    assert_eq!(log.cleared, 1);
    assert_eq!(log.spawns, ["drone"]);
}

#[test]
fn pickups_are_collected_by_the_hero() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::ZERO);
    let pickup = app
        .world_mut()
        .spawn((
            Pickup {
                kind: "spread".into(),
            },
            Layer::Pickup,
            Hitbox::new(4.0, 4.0),
            Transform::default(),
        ))
        .id();
    app.update();
    assert!(!alive(&app, pickup));
    let messages = app.world().resource::<Messages<PickupCollected>>();
    let kinds: Vec<_> = messages
        .iter_current_update_messages()
        .map(|m| m.kind.clone())
        .collect();
    assert_eq!(kinds, ["spread"]);
}

#[test]
fn a_shield_charge_absorbs_one_hit() {
    let mut app = app(0.0, quiet_tuning());
    let hero = spawn_hero(&mut app, Vec2::ZERO);
    app.world_mut().entity_mut(hero).insert(Shield {
        charges: 1,
        invulnerable_seconds: 1.0,
    });
    let shot = spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 0);
    assert_eq!(app.world().get::<Shield>(hero).unwrap().charges, 0);
    assert_eq!(app.world().get::<Invulnerable>(hero).unwrap().0, 1.0);
    let absorbed = app.world().resource::<Messages<ShieldAbsorbed>>();
    let from: Vec<_> = absorbed
        .iter_current_update_messages()
        .map(|m| m.from)
        .collect();
    assert_eq!(from, [Some(shot)]);

    // Once spent and the invulnerability is over, the next hit costs a life.
    app.world_mut().get_mut::<Invulnerable>(hero).unwrap().0 = 0.0;
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
}

#[test]
fn hit_stop_freezes_the_stage_before_the_life_is_lost() {
    let mut app = app(0.05, quiet_tuning());
    let hero = spawn_hero(&mut app, Vec2::ZERO);
    app.world_mut().entity_mut(hero).insert(HitStop(0.15));
    app.update(); // first frame has no delta
    spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    app.update(); // hit: the freeze starts
    let frozen_at = app.world().resource::<StageRun>().cursor.position;
    app.update();
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 0);
    assert_eq!(
        app.world().resource::<StageRun>().cursor.position,
        frozen_at,
        "the timeline doesn't move during a hit-stop"
    );
    app.update();
    app.update();
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
    assert!(app.world().resource::<HitStopTimer>().0.is_none());
}

#[test]
fn immune_enemies_use_up_shots_but_take_no_damage() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::new(-100.0, 0.0));
    let enemy = spawn_enemy(&mut app, Vec2::new(50.0, 0.0), 1, 100);
    app.world_mut().entity_mut(enemy).insert(Immune);
    let shot = spawn_shot(&mut app, Layer::HeroShot, Vec2::new(50.0, 0.0));
    app.update();
    assert!(!alive(&app, shot));
    assert!(alive(&app, enemy));
    assert_eq!(app.world().get::<Health>(enemy).unwrap().0, 1);
}

#[test]
fn hurt_from_any_system_destroys_and_scores() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::new(-100.0, 0.0));
    let enemy = spawn_enemy(&mut app, Vec2::new(50.0, 0.0), 5, 300);
    app.world_mut().write_message(Hurt {
        target: enemy,
        damage: 12,
    });
    app.update();
    assert!(!alive(&app, enemy));
    assert_eq!(app.world().resource::<Log>().score, [300]);
}

#[test]
fn shot_blockers_stop_enemy_shots_only() {
    let mut app = app(0.0, quiet_tuning());
    spawn_hero(&mut app, Vec2::new(-100.0, 0.0));
    app.world_mut().spawn((
        Layer::ShotBlocker,
        Hitbox::new(8.0, 8.0),
        Transform::default(),
    ));
    let bullet = spawn_shot(&mut app, Layer::EnemyShot, Vec2::ZERO);
    let own = spawn_shot(&mut app, Layer::HeroShot, Vec2::ZERO);
    let enemy = spawn_enemy(&mut app, Vec2::ZERO, 9, 0);
    app.update();
    assert!(!alive(&app, bullet));
    assert!(
        !alive(&app, own),
        "the Hero's shot hit the enemy, not the blocker"
    );
    assert!(alive(&app, enemy));
}
