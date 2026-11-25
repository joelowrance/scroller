//! Cabinet 4 running in a headless App: the Hero on the tower, the rising
//! camera, the fall line, the canopy respawn, Power-ups, Standoffs, the
//! Stage bonuses, and the autopilot's unattended run.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::hero::{Climber, Rescues};
use super::*;
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::input::Controls;
use crate::kit::{Invulnerable, KitPlugin};

const DT: f32 = 1.0 / 60.0;

#[derive(Resource, Default)]
struct Log {
    lives_lost: usize,
    cleared: usize,
    score: Vec<u64>,
}

fn collect(
    mut log: ResMut<Log>,
    mut lost: MessageReader<LifeLost>,
    mut cleared: MessageReader<StageCleared>,
    mut score: MessageReader<ScoreGained>,
) {
    log.lives_lost += lost.read().count();
    log.cleared += cleared.read().count();
    log.score.extend(score.read().map(|s| s.0));
}

fn app_with(tuning: Tuning) -> App {
    let mut session = Session::new_run(&tuning);
    session.cabinet = CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            DT,
        )))
        .insert_resource(tuning)
        .insert_resource(session)
        .init_resource::<Paused>()
        .init_resource::<Sprites>()
        .init_resource::<Controls>()
        .init_resource::<Log>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_state::<Screen>()
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((KitPlugin, ClimberPlugin))
        .add_systems(Last, collect);
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    app
}

fn quiet() -> Tuning {
    let mut t = Tuning {
        debug_keys: false,
        ..Tuning::default()
    };
    t.climber.hero.hit_stop_seconds = 0.0;
    t
}

fn app() -> App {
    let mut app = app_with(quiet());
    app.update();
    app
}

fn seconds(app: &mut App, s: f32) {
    for _ in 0..(s / DT).round() as usize {
        app.update();
    }
}

fn climber(app: &mut App) -> (f32, f32, bool, bool) {
    let mut q = app.world_mut().query::<&Climber>();
    let c = q.single(app.world()).unwrap();
    (c.body.x, c.body.feet, c.body.grounded, c.entering)
}

fn camera(app: &App) -> f32 {
    app.world().resource::<StageRun>().cursor.position
}

#[test]
fn the_hero_starts_standing_on_the_first_landing() {
    let mut app = app();
    seconds(&mut app, 0.2);
    let (x, feet, grounded, entering) = climber(&mut app);
    let start = app.world().resource::<StageRun>().start_point().hero_pos();
    assert!(grounded && !entering);
    assert_eq!(x, start.x);
    // Its centre is the Restart Point's `hero`: feet 10 px below.
    assert_eq!(tower::screen_y(feet, 0.0), start.y - 10.0);
    let tower = app.world().resource::<Tower>();
    assert!(
        tower.ledges.iter().any(|l| l.is_landing() && l.h == feet),
        "a landing"
    );
}

#[test]
fn the_camera_rises_on_its_own_at_the_stage_speed() {
    let mut app = app();
    seconds(&mut app, 2.0);
    let rise = camera(&app);
    assert!((rise - 28.0).abs() < 0.5, "14 px/s: {rise}");
}

#[test]
fn standing_still_the_bottom_edge_catches_up_and_a_ward_doesnt_save_it() {
    let mut app = app();
    app.world_mut().resource_mut::<PowerUps>().ward = true;
    {
        let mut q = app.world_mut().query::<&mut Invulnerable>();
        q.single_mut(app.world_mut()).unwrap().0 = 0.0;
    }
    // The first landing is 56 px above the bottom edge: 4 s at 14 px/s.
    seconds(&mut app, 3.8);
    assert_eq!(app.world().resource::<Log>().lives_lost, 0);
    seconds(&mut app, 0.5);
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
    assert!(!app.world().resource::<PowerUps>().ward);
}

#[test]
fn after_a_lost_life_the_hero_drops_in_under_the_canopy_and_the_scroll_waits() {
    let mut app = app();
    app.world_mut().resource_mut::<PowerUps>().gun = 3;
    {
        let mut q = app.world_mut().query::<&mut Invulnerable>();
        q.single_mut(app.world_mut()).unwrap().0 = 0.0;
    }
    seconds(&mut app, 4.5);
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
    let (x, _, _, entering) = climber(&mut app);
    let start = app.world().resource::<StageRun>().start_point().hero_pos();
    assert!(entering, "dropping in");
    assert_eq!(x, start.x, "at the Restart Point's x");
    assert_eq!(
        app.world().resource::<PowerUps>().gun,
        2,
        "a gun level lost"
    );
    let held = camera(&app);
    seconds(&mut app, 1.0);
    assert_eq!(camera(&app), held, "the scroll waits for the landing");
    seconds(&mut app, 3.0);
    let (_, feet, grounded, entering) = climber(&mut app);
    assert!(grounded && !entering, "landed");
    assert_eq!(
        tower::screen_y(feet, held),
        start.y - 10.0,
        "on its landing"
    );
    let invulnerable = {
        let mut q = app.world_mut().query::<&Invulnerable>();
        q.single(app.world()).unwrap().0
    };
    assert!(
        invulnerable > 0.0 && invulnerable <= 2.0,
        "2 s after landing"
    );
    seconds(&mut app, 1.0);
    assert!(camera(&app) > held, "then it rises again");
}

#[test]
fn collected_power_ups_reach_the_session() {
    let mut app = app();
    seconds(&mut app, 0.2);
    let at = {
        let mut q = app
            .world_mut()
            .query_filtered::<&Transform, With<Climber>>();
        q.single(app.world()).unwrap().translation.truncate()
    };
    app.world_mut().write_message(SpawnRequested::new(
        PowerUp::Canopy.kind(),
        at + Vec2::new(0.0, 20.0),
    ));
    seconds(&mut app, 0.5);
    assert!(app.world().resource::<PowerUps>().canopy);
    let session = app.world().resource::<Session>();
    assert_eq!(session.power_ups["canopy"], serde_json::json!(true));
}

#[test]
fn a_standoff_stops_the_scroll_until_it_is_cleared() {
    let mut app = app();
    seconds(&mut app, 0.5);
    let mut params = ron::Map::new();
    params.insert(
        ron::Value::String("max_seconds".into()),
        ron::Value::Number(3.0.into()),
    );
    let mut wave = ron::Map::new();
    wave.insert(
        ron::Value::String("after".into()),
        ron::Value::Number(2.0.into()),
    );
    wave.insert(
        ron::Value::String("kind".into()),
        ron::Value::String("mote".into()),
    );
    wave.insert(
        ron::Value::String("pos".into()),
        ron::Value::Seq(vec![
            ron::Value::Number((-150.0).into()),
            ron::Value::Number(100.0.into()),
        ]),
    );
    params.insert(
        ron::Value::String("waves".into()),
        ron::Value::Seq(vec![ron::Value::Map(wave)]),
    );
    app.world_mut().write_message(SpawnRequested {
        kind: standoff::KIND.into(),
        pos: Vec2::ZERO,
        params,
    });
    app.update();
    let held = camera(&app);
    seconds(&mut app, 1.0);
    assert!(app.world().get_resource::<Standoff>().is_some());
    assert_eq!(camera(&app), held, "held");
    // The Motes aren't shot: it times out at 3 s and the scroll resumes.
    seconds(&mut app, 2.5);
    assert!(app.world().get_resource::<Standoff>().is_none());
    seconds(&mut app, 0.5);
    assert!(camera(&app) > held, "the scroll resumes");
}

#[test]
fn a_stage_clear_pays_the_clear_and_no_miss_bonuses() {
    let t = tuning::ClimberTuning::default();
    assert_eq!(stage_bonus_points(&t, 0), 8000);
    assert_eq!(stage_bonus_points(&t, 1), 3000);
    let mut app = app();
    app.world_mut().write_message(StageCleared);
    seconds(&mut app, 0.1);
    let log = app.world().resource::<Log>();
    assert_eq!(log.cleared, 1);
    assert!(log.score.contains(&8000), "{:?}", log.score);
}

/// The autopilot plays both Stages, enemies and Standoffs included, along
/// each Stage's route with the base moves and no rescues: rule 2 (the climb
/// stays ahead of the rise) checked a second way, beside the timed route
/// check in `stage_checks.rs`.
#[test]
fn the_autopilot_clears_both_stages_with_no_rescues() {
    for stage in 0..2 {
        let mut t = quiet();
        t.debug_keys = true;
        t.invincible_hero = true;
        t.climber.debug_autopilot = true;
        let mut app = app_with(t);
        app.world_mut().resource_mut::<Session>().stage = stage;
        let mut steps = 0;
        while steps < (240.0 / DT) as usize && app.world().resource::<Log>().cleared == 0 {
            app.update();
            steps += 1;
        }
        let log = app.world().resource::<Log>();
        assert_eq!(
            log.cleared,
            1,
            "4-{}: cleared at {}",
            stage + 1,
            camera(&app)
        );
        assert_eq!(
            app.world().resource::<Rescues>().0,
            0,
            "4-{}: rescues at {:?}",
            stage + 1,
            app.world().resource::<Rescues>().1
        );
        // At its pace time, or a little sooner where a Standoff is cleared
        // early (4-1 measured 161.7 s against 165).
        let clock = app.world().resource::<crate::kit::StageClock>();
        assert!(
            clock.seconds <= clock.designed + 3.0 && clock.seconds >= clock.designed - 10.0,
            "4-{}: {} s against {} s at pace",
            stage + 1,
            clock.seconds,
            clock.designed
        );
    }
}

#[test]
fn the_autopilot_plays_the_gallery_through() {
    let mut t = quiet();
    t.debug_keys = true;
    t.invincible_hero = true;
    t.climber.debug_autopilot = true;
    t.climber.enemy_gallery = true;
    let mut app = app_with(t);
    let mut kinds = std::collections::HashSet::new();
    for _ in 0..(200.0 / DT) as usize {
        app.update();
        let mut q = app.world_mut().query::<&enemies::Enemy>();
        kinds.extend(q.iter(app.world()).map(|e| e.kind));
        if app.world().resource::<Log>().cleared > 0 {
            break;
        }
    }
    assert_eq!(
        app.world().resource::<Log>().cleared,
        1,
        "cleared at {}",
        camera(&app)
    );
    assert_eq!(
        app.world().resource::<Rescues>().0,
        0,
        "{:?}",
        app.world().resource::<Rescues>()
    );
    for kind in [
        enemies::Kind::Flit,
        enemies::Kind::Limpet,
        enemies::Kind::Scrap,
        enemies::Kind::Picket,
        enemies::Kind::Vent,
        enemies::Kind::Plumb,
        enemies::Kind::Pitcher,
        enemies::Kind::Mote,
        enemies::Kind::Winch,
    ] {
        assert!(kinds.contains(&kind), "the gallery shows {kind:?}");
    }
}

/// When Cabinet 3's last Stage clears, the Session moves here a frame before
/// that Stage is torn down. Nothing of ours may run then.
#[test]
fn the_frame_between_cabinets_is_safe() {
    let mut tuning = quiet();
    tuning.debug_keys = false;
    let mut app = App::new();
    let mut session = Session::new_run(&tuning);
    session.cabinet = crate::cabinets::run_and_gun::CABINET;
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            DT,
        )))
        .insert_resource(tuning)
        .insert_resource(session)
        .init_resource::<Paused>()
        .init_resource::<Sprites>()
        .init_resource::<Controls>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_state::<Screen>()
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((
            KitPlugin,
            crate::cabinets::run_and_gun::RunAndGunPlugin,
            ClimberPlugin,
        ));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    seconds(&mut app, 0.2);
    app.world_mut().resource_mut::<Session>().cabinet = CABINET;
    seconds(&mut app, 0.2);
}

#[test]
fn the_designed_length_counts_each_standoff_in_full() {
    let mut app = app();
    seconds(&mut app, 0.2);
    let stage = app.world().resource::<StageRun>();
    let scroll = stage.timeline.duration();
    let standoffs = standoff::standoff_seconds(&stage.timeline);
    assert!(standoffs > 0.0, "Stage 1 has Standoffs");
    let clock = app.world().resource::<crate::kit::StageClock>();
    assert_eq!(clock.designed, scroll + standoffs);
}
