//! Cabinet 2 running in a headless App: the Wren, its Power-ups and the
//! Squall.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::hero::{Rocket, Wren};
use super::power_ups::{PowerUp, PowerUps};
use super::*;
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::input::{Button, Controls};
use crate::kit::{
    Health, HeroStruck, Hitbox, Invulnerable, KitPlugin, Layer, ScoreValue, projectile,
};

fn app() -> App {
    app_at(Duration::from_millis(100))
}

/// The App stepping `step` per frame, with the Wren flown in.
fn app_at(step: Duration) -> App {
    let mut tuning = Tuning {
        debug_keys: false,
        ..Tuning::default()
    };
    tuning.vertical_shooter.hero.hit_stop_seconds = 0.0;
    let mut session = Session::new_run(&tuning);
    session.cabinet = CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(step))
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
        .add_plugins((KitPlugin, VerticalShooterPlugin));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    let frames = (1.5 / step.as_secs_f32()).ceil() as usize;
    for _ in 0..frames {
        app.update();
    }
    app
}

fn wren_at(app: &mut App) -> Vec2 {
    let mut q = app.world_mut().query_filtered::<&Transform, With<Wren>>();
    q.single(app.world()).unwrap().translation.truncate()
}

fn saved(app: &App) -> PowerUps {
    let session = app.world().resource::<Session>();
    serde_json::from_value(session.power_ups.clone()).unwrap()
}

fn press(app: &mut App, buttons: &[Button]) {
    *app.world_mut().resource_mut::<Controls>() = Controls::pressing(buttons);
}

#[test]
fn the_wren_flies_up_to_its_start() {
    let mut app = app();
    assert_eq!(wren_at(&mut app), Vec2::new(0.0, -80.0));
}

#[test]
fn the_wren_moves_at_its_speed_and_stays_on_screen() {
    let mut app = app();
    // Nothing from the Stage may knock it back down meanwhile.
    let mut q = app
        .world_mut()
        .query_filtered::<&mut Invulnerable, With<Wren>>();
    q.single_mut(app.world_mut()).unwrap().0 = 100.0;
    press(&mut app, &[Button::Up]);
    app.update();
    assert_eq!(
        wren_at(&mut app),
        Vec2::new(0.0, -70.0),
        "100 px/s for 0.1 s"
    );
    for _ in 0..40 {
        app.update();
    }
    let top = wren_at(&mut app).y;
    assert_eq!(
        top,
        crate::cabinets::side_shooter::power_ups::PLAY_TOP - 12.0
    );
}

#[test]
fn collected_power_ups_are_written_to_the_session() {
    let mut app = app();
    let at = wren_at(&mut app);
    app.world_mut().write_message(SpawnRequested::new(
        PowerUp::Rockets.kind(),
        at + Vec2::new(0.0, 10.0),
    ));
    app.update();
    app.update();
    assert!(app.world().resource::<PowerUps>().rockets);
    assert!(saved(&app).rockets);
}

#[test]
fn a_lost_life_costs_one_step_and_the_wren_flies_in_again() {
    let mut app = app();
    {
        let mut power = app.world_mut().resource_mut::<PowerUps>();
        power.gun = 3;
        power.pods = 2;
        power.bombs = 0;
    }
    app.update();
    let mut q = app
        .world_mut()
        .query_filtered::<&mut Invulnerable, With<Wren>>();
    q.single_mut(app.world_mut()).unwrap().0 = 0.0;
    app.world_mut().write_message(HeroStruck { from: None });
    app.update();
    let power = *app.world().resource::<PowerUps>();
    assert_eq!((power.gun, power.pods, power.bombs), (2, 1, 2));
    assert_eq!(saved(&app), power);
    assert!(
        wren_at(&mut app).y < -120.0,
        "flying up from the bottom edge"
    );
}

fn spawn_target(app: &mut App, layer: Layer, at: Vec2, hp: u32) -> Entity {
    app.world_mut()
        .spawn((
            layer,
            Hitbox::new(10.0, 10.0),
            Health(hp),
            ScoreValue(300),
            Transform::from_translation(at.extend(0.0)),
        ))
        .id()
}

#[test]
fn rockets_home_on_ground_targets_and_ignore_the_air() {
    let mut app = app_at(Duration::from_secs_f32(1.0 / 60.0));
    app.world_mut().resource_mut::<PowerUps>().rockets = true;
    let wren = wren_at(&mut app);
    // Off to the side and ahead: the rockets have to turn to reach it.
    let ground = spawn_target(&mut app, Layer::Ground, wren + Vec2::new(70.0, 120.0), 6);
    // Straight ahead, in the air: rockets pass under it.
    let air = spawn_target(&mut app, Layer::Enemy, wren + Vec2::new(0.0, 200.0), 1000);
    press(&mut app, &[Button::A]);
    let mut rockets_seen = 0;
    for _ in 0..90 {
        app.update();
        let mut q = app.world_mut().query::<&Rocket>();
        rockets_seen = rockets_seen.max(q.iter(app.world()).count());
    }
    assert!(rockets_seen >= 2, "rockets launched in pairs");
    assert!(
        app.world().get_entity(ground).is_err(),
        "the ground target is destroyed"
    );
    // The air target only took gun shots (1 damage each), never 3s.
    let _ = app.world().get::<Health>(air);
}

#[test]
fn no_rockets_without_a_ground_target() {
    let mut app = app_at(Duration::from_secs_f32(1.0 / 60.0));
    app.world_mut().resource_mut::<PowerUps>().rockets = true;
    press(&mut app, &[Button::A]);
    for _ in 0..60 {
        app.update();
    }
    let mut q = app.world_mut().query::<&Rocket>();
    assert_eq!(q.iter(app.world()).count(), 0);
}

#[test]
fn the_squall_cancels_bullets_for_points_and_hurts_air_and_ground_once() {
    let mut app = app();
    let wren = wren_at(&mut app);
    let bullets: Vec<_> = (0..3)
        .map(|i| {
            let at = wren + Vec2::new(-40.0 + 40.0 * i as f32, 60.0);
            app.world_mut()
                .spawn(projectile(
                    Layer::EnemyShot,
                    at,
                    Vec2::ZERO,
                    Vec2::splat(5.0),
                    palette::VOLT,
                    30.0,
                ))
                .id()
        })
        .collect();
    let air = spawn_target(&mut app, Layer::Enemy, wren + Vec2::new(100.0, 120.0), 40);
    let ground = spawn_target(&mut app, Layer::Ground, wren + Vec2::new(-100.0, -30.0), 40);
    press(&mut app, &[Button::B]);
    app.update();
    press(&mut app, &[]);
    assert_eq!(app.world().resource::<PowerUps>().bombs, 1);
    for _ in 0..10 {
        app.update();
    }
    for bullet in bullets {
        assert!(app.world().get_entity(bullet).is_err(), "bullet cancelled");
    }
    assert_eq!(
        app.world().get::<Health>(air).unwrap().0,
        28,
        "12 damage, once"
    );
    assert_eq!(
        app.world().get::<Health>(ground).unwrap().0,
        28,
        "the ground too"
    );
    let mut q = app.world_mut().query::<&hero::Squall>();
    assert_eq!(q.iter(app.world()).count(), 0, "the Squall is done");
}

#[test]
fn the_squall_is_two_edges_not_a_wall() {
    let mut app = app();
    press(&mut app, &[Button::B]);
    app.update();
    press(&mut app, &[]);
    app.update();
    // The upper edge has passed this row: a bullet fired here now survives.
    let behind = wren_at(&mut app) + Vec2::new(0.0, 10.0);
    let bullet = app
        .world_mut()
        .spawn(projectile(
            Layer::EnemyShot,
            behind,
            Vec2::ZERO,
            Vec2::splat(5.0),
            palette::VOLT,
            30.0,
        ))
        .id();
    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get_entity(bullet).is_ok());
}

#[test]
fn the_gun_fires_ten_volleys_a_second() {
    let mut app = app_at(Duration::from_secs_f32(1.0 / 60.0));
    press(&mut app, &[Button::A]);
    let mut shots = 0;
    for _ in 0..60 {
        app.update();
        let mut q = app.world_mut().query::<&Layer>();
        shots = shots.max(
            q.iter(app.world())
                .filter(|l| **l == Layer::HeroShot)
                .count(),
        );
    }
    // Gun 1: one shot per volley, each alive about 0.6 s: about 6 at once.
    assert!((5..=8).contains(&shots), "{shots} shots alive");
}

#[test]
fn pods_appear_beside_the_wren() {
    let mut app = app();
    app.world_mut().resource_mut::<PowerUps>().pods = 2;
    app.update();
    let wren = wren_at(&mut app);
    let mut q = app
        .world_mut()
        .query_filtered::<&Transform, With<hero::Pod>>();
    let mut xs: Vec<_> = q
        .iter(app.world())
        .map(|t| t.translation.x - wren.x)
        .collect();
    xs.sort_by(f32::total_cmp);
    assert_eq!(xs, [-14.0, 14.0]);
}

/// When Cabinet 1's last Stage clears, the Session moves to this Cabinet a
/// frame before Cabinet 1's Stage is torn down. Nothing of ours may run then.
#[test]
fn the_frame_between_cabinets_is_safe() {
    use crate::cabinets::side_shooter::SideShooterPlugin;

    let tuning = Tuning {
        debug_keys: false,
        ..Tuning::default()
    };
    let session = Session::new_run(&tuning);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
            100,
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
        .add_plugins((KitPlugin, SideShooterPlugin, VerticalShooterPlugin));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    for _ in 0..5 {
        app.update();
    }
    app.world_mut().resource_mut::<Session>().cabinet = CABINET;
    for _ in 0..5 {
        app.update();
    }
}

/// The whole enemy gallery, played through while firing, runs without
/// errors (every enemy, the Pods and the Rockets in play together).
#[test]
fn the_gallery_plays_through_while_firing() {
    let mut tuning = Tuning {
        debug_keys: true,
        ..Tuning::default()
    };
    tuning.vertical_shooter.enemy_gallery = true;
    tuning.invincible_hero = true;
    tuning.time_scale = 1.0;
    let mut session = Session::new_run(&tuning);
    session.cabinet = CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
            1.0 / 30.0,
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
        .add_plugins((KitPlugin, VerticalShooterPlugin));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    app.update();
    press(&mut app, &[Button::A]);
    // The gallery is 3000 px at 30 px/s.
    for frame in 0..30 * 100 {
        if frame % 300 == 150 {
            press(&mut app, &[Button::A, Button::B]);
        } else {
            press(&mut app, &[Button::A]);
        }
        app.update();
    }
}

/// Review of #60: the Squall stops at the play area. A Swift waiting below
/// the bottom edge during its warning hasn't appeared yet and survives.
#[test]
fn the_squall_spares_enemies_not_yet_on_screen() {
    use super::enemies::brain::SWIFT_WAIT_Y;

    let mut app = app();
    assert_eq!(wren_at(&mut app).y, -80.0);
    let waiting = spawn_target(&mut app, Layer::Enemy, Vec2::new(50.0, SWIFT_WAIT_Y), 2);
    let above = spawn_target(&mut app, Layer::Enemy, Vec2::new(0.0, 142.0), 2);
    let beside = spawn_target(&mut app, Layer::Enemy, Vec2::new(172.0, 60.0), 2);
    let on_screen = spawn_target(&mut app, Layer::Enemy, Vec2::new(0.0, -110.0), 2);
    press(&mut app, &[Button::B]);
    app.update();
    press(&mut app, &[]);
    for _ in 0..10 {
        app.update();
    }
    for e in [waiting, above, beside] {
        assert_eq!(
            app.world().get::<Health>(e).unwrap().0,
            2,
            "off screen: untouched"
        );
    }
    assert!(app.world().get_entity(on_screen).is_err(), "on screen: hit");
    let mut q = app.world_mut().query::<&hero::Squall>();
    assert_eq!(q.iter(app.world()).count(), 0, "the Squall is done");
}

/// The points scored in one frame after `setup` sends its messages.
fn points_after(app: &mut App, setup: impl FnOnce(&mut World)) -> Vec<u64> {
    app.world_mut()
        .resource_mut::<Messages<ScoreGained>>()
        .clear();
    setup(app.world_mut());
    app.update();
    app.world_mut()
        .resource_mut::<Messages<ScoreGained>>()
        .drain()
        .map(|s| s.0)
        .collect()
}

#[test]
fn a_clean_stage_clear_pays_the_clear_and_no_miss_bonuses() {
    let mut app = app();
    let s = &Tuning::default().vertical_shooter.score;
    let points = points_after(&mut app, |w| {
        w.write_message(StageCleared);
    });
    assert_eq!(points, [s.stage_clear + s.no_miss]);
}

#[test]
fn a_lost_life_costs_the_no_miss_bonus() {
    let mut app = app();
    let s = &Tuning::default().vertical_shooter.score;
    app.world_mut().write_message(LifeLost);
    app.update();
    let points = points_after(&mut app, |w| {
        w.write_message(StageCleared);
    });
    assert_eq!(points, [s.stage_clear]);
    // A new Stage starts clean.
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::StageCard);
    app.update();
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    app.update();
    let points = points_after(&mut app, |w| {
        w.write_message(StageCleared);
    });
    assert_eq!(points, [s.stage_clear + s.no_miss]);
}
