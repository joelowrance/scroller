//! Cabinet 1 running in a headless App: Power-ups reach the Session.

use std::time::Duration;

use bevy::state::app::StatesPlugin;
use bevy::time::TimeUpdateStrategy;

use super::hero::Wren;
use super::power_ups::{PowerUp, PowerUps};
use super::*;
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::display::GameCamera;
use crate::flow::Paused;
use crate::input::Controls;
use crate::kit::{HeroStruck, Invulnerable, KitPlugin};

fn app() -> App {
    app_at(Duration::from_millis(100))
}

/// The App stepping `step` per frame, with the Wren flown in.
fn app_at(step: Duration) -> App {
    let mut tuning = Tuning {
        debug_keys: false,
        ..Tuning::default()
    };
    // No hit-stop, so a lost life lands in the frame it happens.
    tuning.side_shooter.hero.hit_stop_seconds = 0.0;
    let mut session = Session::new_run(&tuning);
    session.cabinet = CABINET;
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, StatesPlugin))
        .insert_resource(TimeUpdateStrategy::ManualDuration(step))
        .insert_resource(tuning)
        .insert_resource(session)
        .init_resource::<Paused>()
        .init_resource::<Controls>()
        .init_resource::<ButtonInput<KeyCode>>()
        .init_state::<Screen>()
        .add_message::<ScoreGained>()
        .add_message::<LifeLost>()
        .add_message::<StageCleared>()
        .add_plugins((KitPlugin, SideShooterPlugin));
    app.world_mut().spawn((GameCamera, Transform::default()));
    app.world_mut()
        .resource_mut::<NextState<Screen>>()
        .set(Screen::Playing);
    // Fly in: 1 s, plus a little.
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

#[test]
fn the_wren_flies_in_to_its_start() {
    let mut app = app();
    assert_eq!(wren_at(&mut app), Vec2::new(-110.0, 0.0));
}

#[test]
fn collected_power_ups_are_written_to_the_session() {
    let mut app = app();
    let at = wren_at(&mut app);
    app.world_mut().write_message(SpawnRequested::new(
        PowerUp::Gun.kind(),
        at + Vec2::new(10.0, 0.0),
    ));
    app.update();
    app.update();
    assert_eq!(app.world().resource::<PowerUps>().gun, 2);
    assert_eq!(saved(&app).gun, 2);
}

#[test]
fn a_lost_life_costs_one_step_and_the_wren_flies_in_again() {
    let mut app = app();
    {
        let mut power = app.world_mut().resource_mut::<PowerUps>();
        power.gun = 3;
        power.drones = 2;
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
    assert_eq!((power.gun, power.drones, power.bombs), (2, 1, 2));
    assert_eq!(saved(&app), power);
    assert!(wren_at(&mut app).x < -150.0, "flying in from the left edge");
}

#[test]
fn the_undertow_cancels_bullets_for_points_and_hurts_enemies_once() {
    use crate::input::Button;
    use crate::kit::{Health, Hitbox, Layer, ScoreValue, projectile};

    let mut app = app();
    let wren = wren_at(&mut app);
    let bullets: Vec<_> = (0..3)
        .map(|i| {
            let at = wren + Vec2::new(60.0, -20.0 + 20.0 * i as f32);
            app.world_mut()
                .spawn(projectile(
                    Layer::EnemyShot,
                    at,
                    Vec2::ZERO,
                    Vec2::splat(5.0),
                    palette::SIGNAL,
                    30.0,
                ))
                .id()
        })
        .collect();
    let tough = app
        .world_mut()
        .spawn((
            Layer::Enemy,
            Hitbox::new(10.0, 10.0),
            Health(40),
            ScoreValue(600),
            Transform::from_translation((wren + Vec2::new(100.0, 0.0)).extend(0.0)),
        ))
        .id();
    *app.world_mut().resource_mut::<Controls>() = Controls::pressing(&[Button::B]);
    app.update();
    *app.world_mut().resource_mut::<Controls>() = Controls::default();
    assert_eq!(app.world().resource::<PowerUps>().bombs, 1);
    for _ in 0..10 {
        app.update();
    }
    for bullet in bullets {
        assert!(app.world().get_entity(bullet).is_err(), "bullet cancelled");
    }
    assert_eq!(
        app.world().get::<Health>(tough).unwrap().0,
        28,
        "12 damage, once"
    );
    let mut q = app.world_mut().query::<&hero::Undertow>();
    assert_eq!(q.iter(app.world()).count(), 0, "the ring is gone");
}

#[test]
fn drones_blink_with_the_wren() {
    let mut app = app();
    app.world_mut().resource_mut::<PowerUps>().drones = 2;
    app.update();
    let mut q = app
        .world_mut()
        .query_filtered::<&mut Invulnerable, With<Wren>>();
    q.single_mut(app.world_mut()).unwrap().0 = 1.0;
    for _ in 0..6 {
        app.update();
        let mut wren = app.world_mut().query_filtered::<&Visibility, With<Wren>>();
        let wren = *wren.single(app.world()).unwrap();
        let mut drones = app
            .world_mut()
            .query_filtered::<&Visibility, With<hero::Drone>>();
        let drones: Vec<_> = drones.iter(app.world()).copied().collect();
        assert_eq!(drones.len(), 2);
        assert!(drones.iter().all(|d| *d == wren), "{drones:?} vs {wren:?}");
    }
}

#[test]
fn the_gun_fires_ten_volleys_a_second_at_any_frame_rate() {
    use crate::input::Button;

    for fps in [30, 60, 75, 120, 144] {
        let mut app = app_at(Duration::from_secs_f64(1.0 / fps as f64));
        let volleys = |app: &mut App| {
            let mut q = app.world_mut().query::<&hero::Gun>();
            q.single(app.world()).unwrap().volleys
        };
        let before = volleys(&mut app);
        *app.world_mut().resource_mut::<Controls>() = Controls::pressing(&[Button::A]);
        for _ in 0..fps * 10 {
            app.update();
        }
        let fired = volleys(&mut app) - before;
        assert!(
            (99..=101).contains(&fired),
            "{fired} volleys in 10 s at {fps} fps"
        );
    }
}

#[test]
fn the_undertow_is_a_ring_not_a_disc() {
    use crate::input::Button;
    use crate::kit::{Layer, projectile};

    let mut app = app();
    *app.world_mut().resource_mut::<Controls>() = Controls::pressing(&[Button::B]);
    app.update();
    *app.world_mut().resource_mut::<Controls>() = Controls::default();
    app.update();
    // The ring has grown past this point already: a bullet fired here now
    // is behind its edge and survives.
    let behind = wren_at(&mut app) + Vec2::new(10.0, 0.0);
    let bullet = app
        .world_mut()
        .spawn(projectile(
            Layer::EnemyShot,
            behind,
            Vec2::ZERO,
            Vec2::splat(5.0),
            palette::SIGNAL,
            30.0,
        ))
        .id();
    for _ in 0..10 {
        app.update();
    }
    assert!(app.world().get_entity(bullet).is_ok());
}

/// Issue #47: the Undertow kills through the kit's damage path, so a bombed
/// Formation still counts as cleared and drops its Power-up.
#[test]
fn a_bombed_formation_drops_its_power_up() {
    use crate::input::Button;

    let mut app = app();
    let wren = wren_at(&mut app);
    let mut request = SpawnRequested::new("skiff", wren + Vec2::new(90.0, 0.0));
    request.params = ron::from_str(r#"{"size": 3, "drop": "D"}"#).unwrap();
    app.world_mut().write_message(request);
    app.update();
    *app.world_mut().resource_mut::<Controls>() = Controls::pressing(&[Button::B]);
    app.update();
    *app.world_mut().resource_mut::<Controls>() = Controls::default();
    for _ in 0..4 {
        app.update();
    }
    let mut enemies = app.world_mut().query::<&super::enemies::Enemy>();
    assert_eq!(enemies.iter(app.world()).count(), 0, "all three bombed");
    let mut capsules = app.world_mut().query::<&crate::kit::Pickup>();
    // The stand-in Stage drops its own capsules too; count only the drop.
    let drops = capsules
        .iter(app.world())
        .filter(|p| p.kind == "power_up:D")
        .count();
    assert_eq!(drops, 1);
}
