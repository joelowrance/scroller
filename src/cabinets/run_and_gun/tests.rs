//! Cabinet 3 running in a headless App: the Hero on foot, the camera, pits,
//! Power-ups, Standoffs, and the autopilot's unattended run.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::hero::{Rescues, Runner};
use super::power_ups::{PowerUp, PowerUps};
use super::*;
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::display::{GameCamera, HALF_W};
use crate::flow::Paused;
use crate::input::{Button, Controls};
use crate::kit::{Invulnerable, KitPlugin};
use crate::sprites::Sprites;

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
        .add_plugins((KitPlugin, RunAndGunPlugin))
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
    t.run_and_gun.hero.hit_stop_seconds = 0.0;
    t
}

/// An App with the Hero landed at its start.
fn app() -> App {
    let mut app = app_with(quiet());
    seconds(&mut app, 1.5);
    app
}

fn seconds(app: &mut App, s: f32) {
    for _ in 0..(s / DT).round() as usize {
        app.update();
    }
}

fn press(app: &mut App, buttons: &[Button]) {
    *app.world_mut().resource_mut::<Controls>() = Controls::pressing(buttons);
}

fn runner(app: &mut App) -> (f32, f32, bool) {
    let mut q = app.world_mut().query::<&Runner>();
    let r = q.single(app.world()).unwrap();
    (r.body.x, r.body.feet, r.body.grounded)
}

fn camera(app: &App) -> f32 {
    app.world().resource::<StageRun>().cursor.position
}

#[test]
fn the_hero_drops_in_and_lands_on_the_ground() {
    let mut app = app();
    let (x, feet, grounded) = runner(&mut app);
    assert!(grounded && feet == -96.0, "{feet}");
    // The Restart Point's screen x -96 is world x -256 at camera 0.
    assert_eq!(x, -96.0 - HALF_W);
}

#[test]
fn running_right_pushes_the_camera_no_faster_than_the_scroll() {
    let mut app = app();
    let start = camera(&app);
    press(&mut app, &[Button::Right]);
    seconds(&mut app, 3.0);
    let moved = camera(&app) - start;
    assert!(
        moved > 60.0 && moved <= 40.0 * 3.0 + 0.5,
        "{moved} px in 3 s at most 40 px/s"
    );
    let (x, _, _) = runner(&mut app);
    let sx = terrain::screen_x(x, camera(&app));
    assert!(sx <= 64.0 + 0.5, "never past max_x: {sx}");
}

#[test]
fn standing_still_never_moves_the_camera_and_the_left_edge_is_a_wall() {
    let mut app = app();
    seconds(&mut app, 2.0);
    assert_eq!(camera(&app), 0.0);
    press(&mut app, &[Button::Left]);
    seconds(&mut app, 3.0);
    let (x, _, _) = runner(&mut app);
    assert!(terrain::screen_x(x, camera(&app)) >= -HALF_W + 4.0 + 5.0 - 0.01);
}

#[test]
fn falling_into_a_pit_costs_a_life_and_the_ward_doesnt_save_it() {
    let mut app = app();
    app.world_mut().resource_mut::<PowerUps>().ward = true;
    // Put the camera and the Hero over Stage 3-1's first pit (world 440..464).
    app.world_mut().resource_mut::<StageRun>().cursor.position = 560.0;
    {
        let mut q = app.world_mut().query::<(&mut Runner, &mut Invulnerable)>();
        let (mut r, mut inv) = q.single_mut(app.world_mut()).unwrap();
        r.body.x = 452.0;
        r.body.grounded = false;
        inv.0 = 5.0;
    }
    seconds(&mut app, 1.0);
    assert_eq!(app.world().resource::<Log>().lives_lost, 1);
    assert!(!app.world().resource::<PowerUps>().ward);
}

#[test]
fn collected_power_ups_reach_the_session() {
    let mut app = app();
    let at = {
        let mut q = app.world_mut().query_filtered::<&Transform, With<Runner>>();
        q.single(app.world()).unwrap().translation.truncate()
    };
    app.world_mut().write_message(SpawnRequested::new(
        PowerUp::Jet.kind(),
        at + Vec2::new(0.0, 20.0),
    ));
    seconds(&mut app, 0.5);
    assert!(app.world().resource::<PowerUps>().jet);
    let session = app.world().resource::<Session>();
    assert_eq!(session.power_ups["jet"], serde_json::json!(true));
}

#[test]
fn a_standoff_stops_the_camera_until_it_is_cleared() {
    let mut app = app();
    let mut params = ron::Map::new();
    params.insert(
        ron::Value::String("max_seconds".into()),
        ron::Value::Number(3.0.into()),
    );
    app.world_mut().write_message(SpawnRequested {
        kind: standoff::KIND.into(),
        pos: Vec2::ZERO,
        params,
    });
    app.update();
    press(&mut app, &[Button::Right]);
    let before = camera(&app);
    seconds(&mut app, 0.05);
    // No waves: it clears at once, paying whole seconds left.
    seconds(&mut app, 0.5);
    assert!(app.world().get_resource::<Standoff>().is_none());
    assert!(
        app.world().resource::<Log>().score.contains(&100),
        "2 s left × 50"
    );
    seconds(&mut app, 1.0);
    assert!(camera(&app) > before, "the scroll resumes");
}

/// The autopilot plays both Stages, enemies and Standoffs included, with
/// the base jump and no rescues: every gap is passable, and each clears at
/// about its pace time.
#[test]
fn the_autopilot_clears_both_stages_with_no_rescues() {
    for stage in 0..2 {
        let mut t = quiet();
        t.debug_keys = true;
        t.invincible_hero = true;
        t.run_and_gun.debug_autopilot = true;
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
            "3-{}: cleared at {}",
            stage + 1,
            camera(&app)
        );
        assert_eq!(
            app.world().resource::<Rescues>().0,
            0,
            "3-{}: rescues",
            stage + 1
        );
        let clock = app.world().resource::<crate::kit::StageClock>();
        assert!(
            (clock.seconds - clock.designed).abs() <= 5.0,
            "3-{}: {} s against {} s at pace",
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
    t.run_and_gun.debug_autopilot = true;
    t.run_and_gun.enemy_gallery = true;
    let mut app = app_with(t);
    for _ in 0..(200.0 / DT) as usize {
        app.update();
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
}

/// When Cabinet 2's last Stage clears, the Session moves here a frame before
/// that Stage is torn down. Nothing of ours may run then.
#[test]
fn the_frame_between_cabinets_is_safe() {
    let mut tuning = quiet();
    tuning.debug_keys = false;
    let mut app = App::new();
    let mut session = Session::new_run(&tuning);
    session.cabinet = 1;
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
            crate::cabinets::vertical_shooter::VerticalShooterPlugin,
            RunAndGunPlugin,
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
    let standoffs = crate::cabinets::on_foot::standoff::standoff_seconds(&stage.timeline);
    assert!(standoffs > 0.0, "the stand-in Stage has a Standoff");
    let clock = app.world().resource::<crate::kit::StageClock>();
    assert_eq!(clock.designed, scroll + standoffs);
}

#[test]
fn the_hero_walks_off_to_the_right_when_the_stage_clears() {
    let mut app = app();
    let (start, ..) = runner(&mut app);
    // The player holds left; the clear takes the control away.
    press(&mut app, &[Button::Left]);
    app.world_mut()
        .resource_mut::<StageRun>()
        .cursor
        .force_clear();
    seconds(&mut app, 1.0);
    let (x, _, grounded) = runner(&mut app);
    let speed = Tuning::default().run_and_gun.hero.run_speed;
    assert!(grounded);
    assert!(
        x - start > speed * 0.8,
        "walked {} px in 1 s at {speed} px/s",
        x - start
    );
}
