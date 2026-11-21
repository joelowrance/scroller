//! Cabinet 2's enemies, the air and ground layers, Formations and the
//! fairness rules in a headless App.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::*;
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::{LifeLost, StageCleared};
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::kit::{Invulnerable, KitPlugin, projectile};

const DT: f32 = 1.0 / 60.0;
const HERO: Vec2 = Vec2::new(0.0, -80.0);

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
    lives_lost: usize,
    sfx: Vec<Sfx>,
}

fn collect(
    mut log: ResMut<Log>,
    mut score: MessageReader<ScoreGained>,
    mut spawns: MessageReader<SpawnRequested>,
    mut lost: MessageReader<LifeLost>,
    mut sfx: MessageReader<PlaySfx>,
) {
    log.sfx.extend(sfx.read().map(|s| s.0));
    log.score.extend(score.read().map(|s| s.0));
    log.spawns
        .extend(spawns.read().map(|s| (s.kind.clone(), s.pos)));
    log.lives_lost += lost.read().count();
}

/// A Cabinet 2 Stage 1 App with the kit and the enemies, no timeline
/// spawns, and the Hero at its start.
fn app() -> App {
    let mut session = Session::new_run(&tuning());
    session.cabinet = super::super::CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            DT,
        )))
        .insert_resource(tuning())
        .insert_resource(session)
        .init_resource::<Paused>()
        .init_resource::<Log>()
        .init_resource::<Sprites>()
        // The Cabinet's Stage set-up adds this; the enemies wait for it.
        .init_resource::<super::super::power_ups::PowerUps>()
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_state(Screen::Playing)
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((KitPlugin, EnemiesPlugin, super::super::sound::SoundPlugin))
        .add_systems(Last, collect)
        .insert_resource(StageRun::new(
            StageTimeline::parse("(length: 100000.0, speed: [(from: 0.0, speed: 24.0)])").unwrap(),
        ));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut().spawn((
        Hero,
        Invulnerable(1000.0),
        Hitbox::new(4.0, 4.0),
        Layer::Hero,
        Transform::from_translation(HERO.extend(0.0)),
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

fn run(app: &mut App, seconds: f32) {
    for _ in 0..(seconds / DT).round() as usize {
        app.update();
    }
}

fn enemies(app: &mut App) -> Vec<(Entity, Kind, Vec2, Layer)> {
    let mut q = app
        .world_mut()
        .query::<(Entity, &Enemy, &Transform, &Layer)>();
    q.iter(app.world())
        .map(|(e, enemy, t, l)| (e, enemy.kind, t.translation.truncate(), *l))
        .collect()
}

fn of_kind(app: &mut App, kind: Kind) -> Vec<(Entity, Vec2, Layer)> {
    enemies(app)
        .into_iter()
        .filter(|(_, k, _, _)| *k == kind)
        .map(|(e, _, at, l)| (e, at, l))
        .collect()
}

/// Puts a still Hero shot on each enemy for one frame.
fn shoot(app: &mut App, targets: &[Entity]) {
    for &e in targets {
        let at = app
            .world()
            .get::<Transform>(e)
            .unwrap()
            .translation
            .truncate();
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

fn bullets(app: &mut App) -> usize {
    let mut q = app.world_mut().query_filtered::<(), With<EnemyBullet>>();
    q.iter(app.world()).count()
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
fn every_kind_in_the_doc_spawns() {
    for name in [
        "wisp", "swift", "bellows", "loom", "crawler", "silo", "riser", "flak", "mote",
    ] {
        let mut app = app();
        request(&mut app, name, Vec2::new(0.0, 124.0), "{}");
        app.update();
        assert!(!enemies(&mut app).is_empty(), "{name} spawned nothing");
    }
}

#[test]
fn a_wisp_column_is_a_formation_that_drops_its_power_up() {
    let mut app = app();
    request(&mut app, "wisp", Vec2::new(60.0, 124.0), r#"{"drop": "G"}"#);
    app.update();
    let wisps: Vec<_> = of_kind(&mut app, Kind::Wisp);
    assert_eq!(wisps.len(), 6);
    assert!(
        wisps
            .iter()
            .all(|(_, at, l)| at.x == 60.0 && *l == Layer::Enemy)
    );
    let ids: Vec<_> = wisps.iter().map(|w| w.0).collect();
    shoot(&mut app, &ids);
    app.update();
    assert_eq!(power_ups(&app), ["power_up:G"]);
}

#[test]
fn ground_targets_are_on_the_ground_layer_and_scroll_down() {
    let mut app = app();
    request(
        &mut app,
        "crawler",
        Vec2::new(-150.0, 128.0),
        r#"{"heading": 0.0, "drop": "R"}"#,
    );
    request(&mut app, "silo", Vec2::new(60.0, 128.0), "{}");
    app.update();
    let crawlers = of_kind(&mut app, Kind::Crawler);
    assert_eq!(crawlers.len(), 3);
    assert!(crawlers.iter().all(|(_, _, l)| *l == Layer::Ground));
    let silo = of_kind(&mut app, Kind::Silo)[0];
    assert_eq!(silo.2, Layer::Ground);
    run(&mut app, 1.0);
    let moved = of_kind(&mut app, Kind::Silo)[0].1;
    assert!(
        (silo.1.y - moved.y - 24.0).abs() < 1.0,
        "down at the scroll speed"
    );
    // Shots clear the convoy: a ground Formation drops its Power-up too.
    let ids: Vec<_> = of_kind(&mut app, Kind::Crawler)
        .iter()
        .map(|c| c.0)
        .collect();
    for _ in 0..10 {
        shoot(&mut app, &ids);
    }
    app.update();
    assert!(power_ups(&app).contains(&"power_up:R".to_string()));
}

#[test]
fn a_silo_is_immune_until_it_opens() {
    let mut app = app();
    request(&mut app, "silo", Vec2::new(60.0, 100.0), "{}");
    app.update();
    let silo = of_kind(&mut app, Kind::Silo)[0].0;
    for _ in 0..20 {
        shoot(&mut app, &[silo]);
    }
    assert_eq!(app.world().get::<Health>(silo).unwrap().0, 16, "closed");
    run(&mut app, 2.5);
    assert!(app.world().get::<Immune>(silo).is_none(), "open");
    shoot(&mut app, &[silo]);
    assert_eq!(app.world().get::<Health>(silo).unwrap().0, 15);
}

#[test]
fn a_loom_core_is_shielded_until_both_shuttles_are_shot_off() {
    let mut app = app();
    request(&mut app, "loom", Vec2::new(0.0, 142.0), r#"{"drop": "B"}"#);
    app.update();
    let core = of_kind(&mut app, Kind::LoomCore)[0].0;
    let shuttles: Vec<_> = of_kind(&mut app, Kind::LoomShuttle);
    assert_eq!(shuttles.len(), 2);
    let xs: Vec<_> = shuttles.iter().map(|s| s.1.x).collect();
    assert!(
        xs.contains(&-28.0) && xs.contains(&28.0),
        "docked either side: {xs:?}"
    );
    shoot(&mut app, &[core]);
    assert_eq!(app.world().get::<Health>(core).unwrap().0, 240, "shielded");

    // Shuttles move with the core.
    run(&mut app, 2.0);
    let core_y = app.world().get::<Transform>(core).unwrap().translation.y;
    for (s, _, _) in of_kind(&mut app, Kind::LoomShuttle) {
        assert_eq!(
            app.world().get::<Transform>(s).unwrap().translation.y,
            core_y
        );
    }

    let ids: Vec<_> = shuttles.iter().map(|s| s.0).collect();
    for _ in 0..60 {
        shoot(&mut app, &ids);
    }
    assert!(of_kind(&mut app, Kind::LoomShuttle).is_empty());
    assert!(
        app.world().get::<Immune>(core).is_none(),
        "the shield is down"
    );
    for _ in 0..240 {
        shoot(&mut app, &[core]);
    }
    app.update();
    assert!(
        power_ups(&app).contains(&"power_up:B".to_string()),
        "drops when the core dies"
    );
}

#[test]
fn a_tether_needs_both_balloons_and_its_line_dies_with_one() {
    let mut app = app();
    request(
        &mut app,
        "tether",
        Vec2::new(0.0, 60.0),
        r#"{"left_x": -148.0}"#,
    );
    app.update();
    assert!(of_kind(&mut app, Kind::TetherBalloon).is_empty(), "skipped");

    request(
        &mut app,
        "tether",
        Vec2::new(0.0, 60.0),
        r#"{"left_x": -148.0, "right_x": 148.0}"#,
    );
    app.update();
    let balloons = of_kind(&mut app, Kind::TetherBalloon);
    assert_eq!(balloons.len(), 2);
    let mut lines = app.world_mut().query::<&TetherLine>();
    assert_eq!(lines.iter(app.world()).count(), 1);
    for _ in 0..40 {
        shoot(&mut app, &[balloons[0].0]);
    }
    app.update();
    let mut lines = app.world_mut().query::<&TetherLine>();
    assert_eq!(lines.iter(app.world()).count(), 0);
}

#[test]
fn a_riser_pod_is_ground_while_it_climbs_then_air() {
    let mut app = app();
    request(&mut app, "riser", Vec2::new(0.0, 80.0), "{}");
    // First launch 1.0 s after it is fully on screen.
    run(&mut app, 1.1);
    let pods = of_kind(&mut app, Kind::RiserPod);
    assert_eq!(pods.len(), 1);
    assert_eq!(pods[0].2, Layer::Ground, "climbing: can't touch the Wren");
    assert_eq!(app.world().get::<ScoreValue>(pods[0].0).unwrap().0, 150);
    run(&mut app, 1.0);
    let pod = of_kind(&mut app, Kind::RiserPod)[0];
    assert_eq!(pod.2, Layer::Enemy, "airborne");
    assert_eq!(app.world().get::<ScoreValue>(pod.0).unwrap().0, 50);
}

#[test]
fn a_riser_pod_near_the_hero_fizzles_instead_of_bursting() {
    let mut app = app();
    // The launcher sits just above the Hero, so the pod drifts onto it.
    request(&mut app, "riser", Vec2::new(0.0, -50.0), "{}");
    run(&mut app, 4.0);
    assert!(
        of_kind(&mut app, Kind::RiserPod).is_empty(),
        "the pod is gone"
    );
    assert_eq!(bullets(&mut app), 0, "no point-blank ring");
}

#[test]
fn a_flak_burst_rings_only_when_the_hero_has_moved_well_away() {
    let mut app = app();
    fn nudge(app: &mut App, dx: f32) {
        let mut q = app
            .world_mut()
            .query_filtered::<&mut Transform, With<Hero>>();
        q.single_mut(app.world_mut()).unwrap().translation.x += dx;
    }
    request(&mut app, "flak", Vec2::new(100.0, 100.0), "{}");
    // First mark at 1.2 s; burst 0.6 s later. A small sidestep only.
    run(&mut app, 1.25);
    let mut crosshairs = app.world_mut().query::<&Crosshair>();
    assert_eq!(crosshairs.iter(app.world()).count(), 1, "marked");
    nudge(&mut app, 15.0);
    run(&mut app, 0.6);
    assert_eq!(
        bullets(&mut app),
        0,
        "a sidestep escapes the burst and gets no ring"
    );

    // Next mark at 3.6 s: this time move far away.
    run(&mut app, 1.85);
    nudge(&mut app, -80.0);
    run(&mut app, 0.6);
    assert_eq!(bullets(&mut app), 6, "a ring, at a safe distance");
}

#[test]
fn the_flak_burst_hits_a_hero_who_stays_put() {
    let mut app = app();
    {
        let mut q = app
            .world_mut()
            .query_filtered::<&mut Invulnerable, With<Hero>>();
        q.single_mut(app.world_mut()).unwrap().0 = 0.0;
    }
    request(&mut app, "flak", Vec2::new(100.0, 100.0), "{}");
    run(&mut app, 2.0);
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
}

#[test]
fn ground_targets_fall_silent_once_below_the_hero() {
    let mut app = app();
    // A Crawler already below the Hero's row never fires.
    request(
        &mut app,
        "crawler",
        Vec2::new(-60.0, -100.0),
        r#"{"size": 1}"#,
    );
    run(&mut app, 4.0);
    assert_eq!(bullets(&mut app), 0);
}

#[test]
fn a_swift_pair_climbs_from_below_with_chevrons() {
    let mut app = app();
    request(&mut app, "swift", Vec2::ZERO, "{}");
    app.update();
    let swifts = of_kind(&mut app, Kind::Swift);
    assert_eq!(swifts.len(), 2);
    let xs: Vec<_> = swifts.iter().map(|s| s.1.x).collect();
    assert!(xs.contains(&-50.0) && xs.contains(&50.0), "{xs:?}");
    let mut chevrons = app.world_mut().query::<&Chevron>();
    assert_eq!(chevrons.iter(app.world()).count(), 2);
}

#[test]
fn the_fairness_gate_skips_whole_patterns() {
    let rules = super::super::tuning::EnemyRules::default();
    let from = Vec2::new(0.0, 60.0);
    assert!(may_fire(&rules, from, 1.0, Some(HERO), 3, 0));
    assert!(!may_fire(&rules, from, 0.2, Some(HERO), 3, 0), "grace");
    assert!(
        !may_fire(&rules, from, 1.0, Some(Vec2::new(0.0, 80.0)), 3, 0),
        "below the Hero"
    );
    assert!(
        !may_fire(&rules, from, 1.0, Some(Vec2::new(0.0, 30.0)), 3, 0),
        "too close"
    );
    assert!(
        !may_fire(&rules, from, 1.0, Some(HERO), 3, 38),
        "cap: all or nothing"
    );
}

#[test]
fn the_gallery_stage_parses_and_names_only_known_kinds() {
    let timeline = StageTimeline::parse(GALLERY_STAGE).unwrap();
    for spawn in &timeline.spawns {
        assert!(
            kind_for(&spawn.kind).is_some() || PowerUp::from_kind(&spawn.kind).is_some(),
            "unknown kind {:?}",
            spawn.kind
        );
    }
}

#[test]
fn enemy_events_play_their_sounds() {
    let mut app = app();
    request(&mut app, "swift", Vec2::ZERO, "{}");
    request(&mut app, "loom", Vec2::new(0.0, 142.0), "{}");
    request(&mut app, "riser", Vec2::new(-100.0, 80.0), "{}");
    request(&mut app, "silo", Vec2::new(100.0, 100.0), "{}");
    request(
        &mut app,
        "tether",
        Vec2::new(0.0, 100.0),
        r#"{"left_x": -148.0, "right_x": 148.0}"#,
    );
    app.update();
    let shuttles: Vec<_> = of_kind(&mut app, Kind::LoomShuttle)
        .iter()
        .map(|s| s.0)
        .collect();
    for _ in 0..60 {
        shoot(&mut app, &shuttles);
    }
    run(&mut app, 3.0);
    let heard = &app.world().resource::<Log>().sfx;
    for want in [
        Sfx::SwiftWarning,
        Sfx::ShuttleFall,
        Sfx::LoomShieldBreak,
        Sfx::LoomTick,
        Sfx::RiserLaunch,
        Sfx::SiloClank,
        Sfx::TetherHum,
        Sfx::EnemyHit,
    ] {
        assert!(heard.contains(&want), "no {want:?} in {heard:?}");
    }
}

#[test]
fn flak_plays_its_mark_and_burst() {
    let mut app = app();
    request(&mut app, "flak", Vec2::new(100.0, 100.0), "{}");
    run(&mut app, 2.0);
    let heard = &app.world().resource::<Log>().sfx;
    assert!(heard.contains(&Sfx::FlakMark) && heard.contains(&Sfx::FlakBurst));
}
