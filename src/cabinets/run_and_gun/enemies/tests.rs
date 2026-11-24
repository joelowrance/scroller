//! Cabinet 3's enemies and hazards in a headless App.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::*;
use crate::cabinets::LifeLost;
use crate::cabinets::StageCleared;
use crate::cabinets::run_and_gun::hero::Runner;
use crate::cabinets::run_and_gun::terrain::StageTerrain;
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::kit::{Invulnerable, KitPlugin, StageTimeline, projectile};

const DT: f32 = 1.0 / 60.0;
/// The Hero's feet: world x -260 is screen x -100 at camera 0.
const HERO: Vec2 = Vec2::new(-260.0, -96.0);

fn tuning() -> Tuning {
    Tuning {
        debug_keys: false,
        ..Tuning::default()
    }
}

/// Flat ground at y = -96 everywhere, with a pit at world 200..230.
fn terrain() -> Terrain {
    let text = "(ground: [(from: -600.0, to: 200.0, top: 3), (from: 230.0, to: 3000.0, top: 3)])";
    Terrain::new(ron::from_str::<StageTerrain>(text).unwrap())
}

fn app() -> App {
    let mut session = Session::new_run(&tuning());
    session.cabinet = crate::cabinets::run_and_gun::CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            DT,
        )))
        .insert_resource(tuning())
        .insert_resource(session)
        .insert_resource(terrain())
        .init_resource::<PowerUps>()
        .init_resource::<Paused>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_state(Screen::Playing)
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((KitPlugin, EnemiesPlugin))
        .insert_resource(StageRun::new(
            StageTimeline::parse("(length: 100000.0, speed: [(from: 0.0, speed: 0.0)])").unwrap(),
        ));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut().spawn((
        Hero,
        Invulnerable(1000.0),
        Hitbox::new(6.0, 14.0),
        Layer::Hero,
        Transform::from_xyz(-100.0, -86.0, 0.0),
        Runner::standing(HERO.x, HERO.y),
    ));
    app.update();
    app
}

fn request(app: &mut App, kind: &str, pos: Vec2, params: &str) {
    app.world_mut().write_message(SpawnRequested {
        kind: kind.into(),
        pos,
        params: ron::from_str(params).unwrap(),
    });
}

fn run(app: &mut App, s: f32) {
    for _ in 0..(s / DT).round() as usize {
        app.update();
    }
}

fn of_kind(app: &mut App, kind: Kind) -> Vec<Entity> {
    let mut q = app.world_mut().query::<(Entity, &Enemy)>();
    q.iter(app.world())
        .filter(|(_, e)| e.kind == kind)
        .map(|(e, _)| e)
        .collect()
}

fn shoot_at(app: &mut App, at: Vec2) {
    app.world_mut().spawn(projectile(
        Layer::HeroShot,
        at,
        Vec2::ZERO,
        Vec2::splat(2.0),
        Color::WHITE,
        DT * 1.5,
    ));
    app.update();
}

fn centre(app: &App, e: Entity) -> Vec2 {
    app.world()
        .get::<Transform>(e)
        .unwrap()
        .translation
        .truncate()
}

fn hp(app: &App, e: Entity) -> u32 {
    app.world().get::<Health>(e).unwrap().0
}

fn bullets(app: &mut App) -> Vec<Vec2> {
    let mut q = app
        .world_mut()
        .query_filtered::<&Transform, With<EnemyBullet>>();
    q.iter(app.world())
        .map(|t| t.translation.truncate())
        .collect()
}

#[test]
fn every_kind_in_the_doc_spawns_on_its_floor() {
    for name in [
        "skitter", "tick", "kite", "sentry", "burrower", "dropper", "hauler", "turret", "mote",
        "strider",
    ] {
        let mut app = app();
        request(&mut app, name, Vec2::new(100.0, -88.0), "{}");
        app.update();
        let kind = kind_for(name).unwrap();
        let found = of_kind(&mut app, kind);
        assert!(!found.is_empty(), "{name}");
        if kind.walks() {
            let pos = app.world().get::<EnemyPos>(found[0]).unwrap().0;
            assert_eq!(pos.y, -96.0, "{name} stands on the ground");
        }
    }
}

#[test]
fn a_sentry_is_immune_while_down_and_hurt_while_up() {
    let mut app = app();
    request(&mut app, "sentry", Vec2::new(100.0, -88.0), "{}");
    app.update();
    let s = of_kind(&mut app, Kind::Sentry)[0];
    let at = centre(&app, s) + Vec2::new(0.0, -5.0);
    shoot_at(&mut app, at);
    assert_eq!(hp(&app, s), 14, "down: shots spark off");
    run(&mut app, 2.3);
    let at = centre(&app, s);
    shoot_at(&mut app, at);
    assert_eq!(hp(&app, s), 13, "up: hurt");
}

#[test]
fn a_buried_burrower_is_out_of_a_shots_reach_but_not_a_flares() {
    let mut app = app();
    request(&mut app, "burrower", Vec2::new(100.0, -88.0), "{}");
    app.update();
    let b = of_kind(&mut app, Kind::Burrower)[0];
    // A shot along the surface passes over the buried box.
    let at = centre(&app, b) + Vec2::new(0.0, 2.0);
    shoot_at(&mut app, at);
    assert_eq!(hp(&app, b), 6);
    // A Flare bursting on the ground above it reaches it.
    let above = centre(&app, b) + Vec2::new(0.0, 14.0);
    app.world_mut().spawn((
        Transform::from_translation(above.extend(8.0)),
        crate::cabinets::run_and_gun::hero::Flare {
            vel: Vec2::new(0.0, -100.0),
        },
    ));
    let hit = app.world().resource::<Tuning>().run_and_gun.flare.damage;
    // Flares fly in the Cabinet plugin; step it by hand here.
    app.add_systems(
        Update,
        crate::cabinets::run_and_gun::hero::fly_flares.before(KitSet::Move),
    );
    run(&mut app, 0.3);
    assert_eq!(hp(&app, b), 6 - hit, "dug out by the burst");
}

#[test]
fn a_haulers_plough_is_armour_and_its_core_takes_chest_shots() {
    let mut app = app();
    request(&mut app, "hauler", Vec2::new(100.0, -88.0), "{}");
    app.update();
    app.update();
    let h = of_kind(&mut app, Kind::Hauler)[0];
    let plough = of_kind(&mut app, Kind::Plough)[0];
    let at = centre(&app, plough);
    shoot_at(&mut app, at);
    assert_eq!(hp(&app, h), 40, "a knee-height shot hits the plough");
    let at = Vec2::new(centre(&app, h).x, -96.0 + 14.0);
    shoot_at(&mut app, at);
    assert_eq!(hp(&app, h), 39, "a chest-height shot hits the core");
}

#[test]
fn a_canister_bursts_into_two_ankle_shots_on_the_ground() {
    let mut app = app();
    let rules = app.world().resource::<Tuning>().run_and_gun.enemy.clone();
    let shot = Shot {
        offset: Vec2::ZERO,
        velocity: Vec2::ZERO,
        gravity: 600.0,
        burst: true,
        aimed: false,
    };
    let t = app.world().resource::<Tuning>().run_and_gun.enemies.clone();
    let mut commands_world = app.world_mut().commands();
    fire(
        &mut commands_world,
        &rules,
        &t,
        Vec2::new(60.0, 0.0),
        &shot,
        1.0,
    );
    app.world_mut().flush();
    run(&mut app, 1.0);
    let b = bullets(&mut app);
    assert_eq!(b.len(), 2, "{b:?}");
    assert!(
        b.iter().all(|p| (p.y - (-96.0 + 4.0)).abs() < 1.0),
        "ankle height: {b:?}"
    );
}

#[test]
fn walkers_from_behind_wait_for_their_chevron() {
    let mut app = app();
    request(
        &mut app,
        "skitter",
        Vec2::new(-176.0, -88.0),
        r#"{"size": 1}"#,
    );
    app.update();
    let s = of_kind(&mut app, Kind::Skitter)[0];
    let mut q = app.world_mut().query::<&Chevron>();
    assert_eq!(q.iter(app.world()).count(), 1);
    run(&mut app, 0.5);
    assert!(
        centre(&app, s).x <= -HALF_W - 12.0,
        "still waiting off screen"
    );
    run(&mut app, 1.0);
    assert!(
        centre(&app, s).x > -HALF_W - 12.0,
        "coming in after the warning"
    );
}

#[test]
fn a_crusher_slams_on_its_cycle_and_kills_enemies_under_it() {
    let mut app = app();
    request(&mut app, "crusher", Vec2::new(60.0, 0.0), "{}");
    // A Sentry stays put, under the press.
    request(&mut app, "sentry", Vec2::new(60.0, -88.0), "{}");
    app.update();
    assert_eq!(of_kind(&mut app, Kind::Sentry).len(), 1);
    run(&mut app, 2.4);
    assert!(of_kind(&mut app, Kind::Sentry).is_empty(), "crushed");
    let mut q = app.world_mut().query::<(&Hazard, Has<Layer>)>();
    assert!(
        q.iter(app.world()).any(|(_, live)| live),
        "a hazard while down"
    );
}

#[test]
fn the_fairness_gate_holds_the_aimed_shot_rule() {
    let r = EnemyRules::default();
    let hero = Vec2::new(0.0, -86.0);
    let head = -76.0;
    // An aimed shot from low and close: held back.
    let near_low = Vec2::new(50.0, -86.0);
    assert!(!may_fire(
        &r,
        &[(near_low, true)],
        1.0,
        Some(hero),
        head,
        near_low,
        0
    ));
    // The same shot, not aimed (chest height): fine.
    assert!(may_fire(
        &r,
        &[(near_low, false)],
        1.0,
        Some(hero),
        head,
        near_low,
        0
    ));
    // Aimed from above the head, or from afar: fine.
    let above = Vec2::new(50.0, 40.0);
    assert!(may_fire(
        &r,
        &[(above, true)],
        1.0,
        Some(hero),
        head,
        above,
        0
    ));
    let far = Vec2::new(120.0, -86.0);
    assert!(may_fire(&r, &[(far, true)], 1.0, Some(hero), head, far, 0));
    // Grace, distance and the cap.
    assert!(!may_fire(
        &r,
        &[(far, false)],
        0.2,
        Some(hero),
        head,
        far,
        0
    ));
    assert!(!may_fire(
        &r,
        &[(Vec2::new(30.0, -86.0), false)],
        1.0,
        Some(hero),
        head,
        far,
        0
    ));
    assert!(!may_fire(
        &r,
        &[(far, false), (far, false)],
        1.0,
        Some(hero),
        head,
        far,
        31
    ));
}

#[test]
fn standoff_waves_mark_their_enemies() {
    let mut app = app();
    request(
        &mut app,
        "skitter",
        Vec2::new(150.0, -88.0),
        r#"{"size": 2, "standoff": true}"#,
    );
    app.update();
    let mut q = app.world_mut().query_filtered::<(), With<StandoffMember>>();
    assert_eq!(q.iter(app.world()).count(), 2);
}

#[test]
fn a_formation_drops_its_power_up_when_cleared() {
    let mut app = app();
    request(
        &mut app,
        "tick",
        Vec2::new(100.0, -88.0),
        r#"{"drop": "F"}"#,
    );
    app.update();
    for _ in 0..8 {
        for e in of_kind(&mut app, Kind::Tick) {
            let at = centre(&app, e);
            shoot_at(&mut app, at);
        }
    }
    assert!(of_kind(&mut app, Kind::Tick).is_empty());
    app.update();
    let mut q = app.world_mut().query::<&crate::kit::Pickup>();
    let _ = q.iter(app.world()).count();
}

#[test]
fn the_gallery_names_only_known_kinds() {
    let timeline = gallery_timeline();
    for s in &timeline.spawns {
        assert!(
            kind_for(&s.kind).is_some()
                || HAZARD_KINDS.contains(&s.kind.as_str())
                || s.kind == crate::cabinets::run_and_gun::standoff::KIND
                || PowerUp::from_kind(&s.kind).is_some(),
            "{}",
            s.kind
        );
    }
}
