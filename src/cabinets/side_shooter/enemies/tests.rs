//! Enemies, Formations and the Power-up drop rule in a headless App.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::*;
use crate::cabinets::{LifeLost, StageCleared};
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::kit::{Invulnerable, KitPlugin, projectile};

const DT: f32 = 1.0 / 60.0;

fn tuning() -> Tuning {
    Tuning {
        debug_keys: false,
        ..Tuning::default()
    }
}

#[derive(Resource, Default)]
struct Log {
    score: Vec<u64>,
    spawns: Vec<(String, Vec2)>,
}

fn collect(
    mut log: ResMut<Log>,
    mut score: MessageReader<ScoreGained>,
    mut spawns: MessageReader<SpawnRequested>,
) {
    log.score.extend(score.read().map(|s| s.0));
    log.spawns
        .extend(spawns.read().map(|s| (s.kind.clone(), s.pos)));
}

/// A Stage 1 App with the kit and the enemies, no timeline spawns, and a Hero
/// far to the left.
fn app() -> App {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            DT,
        )))
        .insert_resource(tuning())
        .insert_resource(Session::new_run(&tuning()))
        .init_resource::<Paused>()
        .init_resource::<Log>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_state(Screen::Playing)
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((KitPlugin, EnemiesPlugin))
        .add_systems(Last, collect)
        .insert_resource(StageRun::new(
            StageTimeline::parse("(length: 100000.0, speed: [(from: 0.0, speed: 30.0)])").unwrap(),
        ));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut().spawn((
        Hero,
        Invulnerable(0.0),
        Hitbox::new(4.0, 4.0),
        Layer::Hero,
        Transform::from_xyz(-150.0, -100.0, 0.0),
    ));
    app.update();
    app
}

fn request(app: &mut App, kind: &str, pos: Vec2, params: &str) {
    let params = ron::from_str(params).unwrap();
    app.world_mut().write_message(SpawnRequested {
        kind: kind.into(),
        pos,
        params,
    });
}

fn enemies(app: &mut App) -> Vec<(Entity, Kind, Vec2)> {
    let mut q = app.world_mut().query::<(Entity, &Enemy, &Transform)>();
    q.iter(app.world())
        .map(|(e, enemy, t)| (e, enemy.kind, t.translation.truncate()))
        .collect()
}

/// Puts a still Hero shot on every enemy for one frame.
fn shoot_all(app: &mut App) {
    for (_, _, at) in enemies(app) {
        app.world_mut().spawn(projectile(
            Layer::HeroShot,
            at,
            Vec2::ZERO,
            Vec2::splat(2.0),
            Color::WHITE,
            1.0,
        ));
    }
    app.update();
}

fn power_ups(app: &App) -> Vec<String> {
    app.world()
        .resource::<Log>()
        .spawns
        .iter()
        .filter(|(k, _)| k.starts_with("power_up:"))
        .map(|(k, _)| k.clone())
        .collect()
}

#[test]
fn a_skiff_v_spawns_five_in_a_v() {
    let mut app = app();
    request(&mut app, "skiff", Vec2::new(150.0, 0.0), "{}");
    app.update();
    let mut found = enemies(&mut app);
    assert_eq!(found.len(), 5);
    found.sort_by(|a, b| a.2.x.total_cmp(&b.2.x));
    // The tip is in front, the wings trail behind above and below.
    assert_eq!(found[0].2.y, 0.0);
    assert!(found[1].2.y * found[2].2.y < 0.0);
    assert!(found[3].2.x > found[1].2.x);
}

#[test]
fn clearing_a_formation_drops_its_power_up_once() {
    let mut app = app();
    request(
        &mut app,
        "skiff",
        Vec2::new(100.0, 0.0),
        r#"{"size": 3, "drop": "G"}"#,
    );
    app.update();
    assert_eq!(enemies(&mut app).len(), 3);
    shoot_all(&mut app);
    app.update();
    assert!(enemies(&mut app).is_empty());
    assert_eq!(power_ups(&app), ["power_up:G"]);
    let score = &app.world().resource::<Log>().score;
    assert_eq!(score.iter().sum::<u64>(), 300, "points per Skiff, no bonus");
    app.update();
    assert_eq!(power_ups(&app).len(), 1);
}

#[test]
fn a_formation_without_a_drop_pays_the_bonus() {
    let mut app = app();
    request(&mut app, "skiff", Vec2::new(100.0, 0.0), r#"{"size": 3}"#);
    app.update();
    shoot_all(&mut app);
    app.update();
    let score = &app.world().resource::<Log>().score;
    assert_eq!(score.iter().sum::<u64>(), 300 + 300, "3 Skiffs and 3 × 100");
    assert!(power_ups(&app).is_empty());
}

#[test]
fn a_member_that_escapes_means_no_drop() {
    let mut app = app();
    request(
        &mut app,
        "mote",
        Vec2::new(100.0, 0.0),
        r#"{"size": 2, "drop": "S"}"#,
    );
    app.update();
    let motes = enemies(&mut app);
    // One Mote is shot, the other dissolves at the end of its life.
    app.world_mut().spawn(projectile(
        Layer::HeroShot,
        motes[0].2,
        Vec2::ZERO,
        Vec2::splat(2.0),
        Color::WHITE,
        0.1,
    ));
    let frames = (Tuning::default().side_shooter.enemies.mote.lifetime / DT) as usize + 10;
    for _ in 0..frames {
        app.update();
    }
    assert!(enemies(&mut app).is_empty());
    assert!(power_ups(&app).is_empty());
    let mut q = app.world_mut().query::<&Formation>();
    assert_eq!(q.iter(app.world()).count(), 0, "the Formation is gone");
}

#[test]
fn a_closed_clam_shrugs_off_shots() {
    let mut app = app();
    request(&mut app, "clam", Vec2::new(100.0, 0.0), r#"{"size": 1}"#);
    app.update();
    for _ in 0..5 {
        shoot_all(&mut app);
    }
    let mut q = app.world_mut().query::<&Health>();
    let hp: Vec<u32> = q.iter(app.world()).map(|h| h.0).collect();
    assert_eq!(hp, [Tuning::default().side_shooter.enemies.clam.hp]);
}

#[test]
fn a_ferry_launches_skiffs_that_belong_to_no_formation() {
    let mut app = app();
    request(
        &mut app,
        "ferry",
        Vec2::new(100.0, 40.0),
        r#"{"drop": "B"}"#,
    );
    for _ in 0..(1.2 / DT) as usize {
        app.update();
    }
    let skiffs: Vec<_> = enemies(&mut app)
        .into_iter()
        .filter(|e| e.1 == Kind::Skiff)
        .collect();
    assert_eq!(skiffs.len(), 2);
    for (e, ..) in skiffs {
        assert!(app.world().get::<Member>(e).is_none());
    }
}

#[test]
fn enemy_fire_waits_for_the_grace_period_and_respects_the_cap() {
    let mut app = app();
    // A Ferry already on screen: its first ring is due at 2.0 s.
    request(&mut app, "ferry", Vec2::new(60.0, 40.0), "{}");
    let mut bullets = app.world_mut().query_filtered::<(), With<EnemyBullet>>();
    for _ in 0..(1.9 / DT) as usize {
        app.update();
    }
    assert_eq!(bullets.iter(app.world()).count(), 0);
    for _ in 0..(0.2 / DT) as usize {
        app.update();
    }
    assert_eq!(bullets.iter(app.world()).count(), 8);

    // A full screen of bullets: the cap stops the next ring (due at 5.0 s).
    let cap = Tuning::default().side_shooter.enemy.max_bullets;
    for _ in 0..cap {
        app.world_mut().spawn(EnemyBullet);
    }
    let mut ids = app
        .world_mut()
        .query_filtered::<Entity, With<EnemyBullet>>();
    let before: Vec<Entity> = ids.iter(app.world()).collect();
    for _ in 0..(3.0 / DT) as usize {
        app.update();
    }
    let fired_since = ids
        .iter(app.world())
        .filter(|e| !before.contains(e))
        .count();
    assert_eq!(fired_since, 0);
}

#[test]
fn a_restart_clears_enemies_and_formations() {
    let mut app = app();
    request(&mut app, "gull", Vec2::new(100.0, 0.0), "{}");
    app.update();
    assert!(!enemies(&mut app).is_empty());
    app.world_mut()
        .write_message(StageRestarted { hero: Vec2::ZERO });
    app.update();
    app.update();
    assert!(enemies(&mut app).is_empty());
    let mut q = app.world_mut().query::<&Formation>();
    assert_eq!(q.iter(app.world()).count(), 0);
}

#[test]
fn an_arc_gate_has_two_pylons_and_an_arc_that_dies_with_one() {
    let mut app = app();
    request(
        &mut app,
        "arc_gate",
        Vec2::new(100.0, 0.0),
        r#"{"floor_y": -60.0, "ceiling_y": 60.0}"#,
    );
    app.update();
    let pylons = enemies(&mut app);
    assert_eq!(pylons.len(), 2);
    let mut arcs = app.world_mut().query::<&ArcBeam>();
    assert_eq!(arcs.iter(app.world()).count(), 1);
    // 40 HP: forty shots on the lower pylon.
    let low = pylons.iter().find(|p| p.2.y < 0.0).unwrap().2;
    for _ in 0..40 {
        app.world_mut().spawn(projectile(
            Layer::HeroShot,
            low,
            Vec2::ZERO,
            Vec2::splat(2.0),
            Color::WHITE,
            1.0,
        ));
    }
    app.update();
    app.update();
    assert_eq!(enemies(&mut app).len(), 1);
    assert_eq!(arcs.iter(app.world()).count(), 0);
}

#[test]
fn gallery_stage_parses_and_uses_only_enemy_kinds_with_valid_params() {
    let mut t = tuning();
    assert!(gallery_timeline(&t).is_none());
    t.debug_keys = true;
    t.side_shooter.enemy_gallery = true;
    let stage = gallery_timeline(&t).unwrap();
    let mut kinds: Vec<&str> = Vec::new();
    for spawn in &stage.spawns {
        assert!(kind_for(&spawn.kind).is_some(), "{}", spawn.kind);
        let request = SpawnRequested {
            kind: spawn.kind.clone(),
            pos: Vec2::ZERO,
            params: spawn.params.clone(),
        };
        request.params::<SpawnParams>().unwrap();
        if !kinds.contains(&spawn.kind.as_str()) {
            kinds.push(&spawn.kind);
        }
    }
    assert_eq!(kinds.len(), 10, "every enemy type appears");
}

#[test]
fn layouts_follow_the_tuning() {
    let t = Tuning::default().side_shooter.enemies;
    let p = SpawnParams::default();
    let lead = Vec2::new(176.0, 0.0);
    let motes = layout(Kind::Mote, lead, &p, None, &t);
    assert_eq!(motes.len(), 10);
    assert!(
        motes
            .iter()
            .all(|(at, _)| at.distance(lead) <= t.mote.cloud_radius)
    );
    let hooks = layout(Kind::Hook, lead, &p, Some(Vec2::new(-100.0, 0.0)), &t);
    let rows: Vec<f32> = hooks.iter().map(|(at, _)| at.y).collect();
    assert_eq!(rows, [50.0, -50.0]);
    assert!(hooks.iter().all(|(at, _)| at.x < -HALF_W), "from behind");
    let clams = layout(Kind::Clam, lead, &p, None, &t);
    assert_eq!(clams[1].0 - clams[0].0, Vec2::new(0.0, -48.0));
    let buoy = layout(Kind::Bellbuoy, lead, &p, None, &t);
    assert_eq!(buoy[0].0.y, t.bellbuoy.wave_y);
}
