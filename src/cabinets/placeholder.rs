//! A stand-in Cabinet used for every slot until the real Cabinets are built.
//!
//! Built on the gameplay kit: the Stage comes from `assets/stages/`, the Hero
//! (a square) moves and fires with A, drones fly in and gunners shoot back.
//! It scrolls in the Cabinet's direction and clears when the scroll ends.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::{CABINETS, Scroll, climber};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::input::{Button, Controls};
use crate::kit::{
    DespawnOffscreen, Health, Hero, Hitbox, Invulnerable, KitSet, Layer, ScoreValue,
    SpawnRequested, StageRun, StageTimeline, Velocity, projectile, stage_live,
};
use crate::run::Session;
use crate::sprites::{SpriteAnimation, Sprites};
use crate::tuning::Tuning;

const HERO_SIZE: f32 = 8.0;
const DRONE_SIZE: f32 = 10.0;
const STRIPE_COUNT: usize = 24;
const PROGRESS_BAR_W: f32 = 120.0;

const SIDEWAYS_STAGE: &str = include_str!("../../assets/stages/placeholder-sideways.ron");
const UPWARD_STAGE: &str = include_str!("../../assets/stages/placeholder-upward.ron");

/// Seconds until the Hero may fire again.
#[derive(Component)]
struct Gun(f32);

/// A gunner drone's seconds until its next shot.
#[derive(Component)]
struct Gunner(f32);

#[derive(Component)]
struct Stripe {
    speed: f32,
}

#[derive(Component)]
struct ProgressFill;

/// Unit vector pointing "ahead" (the way the Stage scrolls towards).
fn ahead(scroll: Scroll) -> Vec2 {
    match scroll {
        Scroll::Sideways => Vec2::X,
        Scroll::Upward => Vec2::Y,
    }
}

fn stage_for(session: &Session) -> StageTimeline {
    if session.cabinet == climber::CABINET {
        return climber_stage(session.stage);
    }
    stage_timeline(CABINETS[session.cabinet].scroll)
}

/// Cabinet 4's real Stages (#25), played on the placeholder until its
/// gameplay (#24) replaces it: the scenery, terrain and length are real, and
/// the enemy kinds are skipped. The terrain is drawn as scenery.
fn climber_stage(stage: usize) -> StageTimeline {
    let text = climber::STAGES[stage.min(1)];
    let mut timeline = StageTimeline::parse(text).expect("Cabinet 4 Stage files must be valid");
    let terrain =
        climber::terrain::StageTerrain::parse(text).expect("Cabinet 4 terrain must be valid");
    timeline.spawns.extend(terrain.scenery());
    // The timeline plays its spawns in order.
    timeline.spawns.sort_by(|a, b| a.at.total_cmp(&b.at));
    timeline
}

fn stage_timeline(scroll: Scroll) -> StageTimeline {
    let text = match scroll {
        Scroll::Sideways => SIDEWAYS_STAGE,
        Scroll::Upward => UPWARD_STAGE,
    };
    StageTimeline::parse(text).expect("placeholder Stage file must be valid")
}

fn spawn_stage(mut commands: Commands, session: Res<Session>, sprites: Res<Sprites>) {
    let cabinet = &CABINETS[session.cabinet];
    let stage = StageRun::new(stage_for(&session));
    let start = stage.start_point().hero_pos();

    // Parallax stripes: three depths, scrolling in the Cabinet's direction.
    // (Cabinet 4 brings its own scenery and terrain.)
    let stripes = if session.cabinet == climber::CABINET {
        0
    } else {
        STRIPE_COUNT
    };
    for i in 0..stripes {
        let depth = (i % 3) as f32 + 1.0;
        let along = (i as f32 * 37.0) % (HALF_W * 2.0) - HALF_W;
        let across = (i as f32 * 53.0) % (HALF_H * 2.0) - HALF_H;
        let (turn, pos) = match cabinet.scroll {
            Scroll::Sideways => (0.0, Vec2::new(along, across)),
            Scroll::Upward => (FRAC_PI_2, Vec2::new(across, along)),
        };
        commands.spawn((
            sprites.frame("stripe", session.cabinet, i % 3),
            Transform::from_xyz(pos.x.round(), pos.y.round(), 1.0)
                .with_rotation(Quat::from_rotation_z(turn)),
            Stripe { speed: depth },
            DespawnOnExit(Screen::Playing),
        ));
    }

    commands.spawn((
        sprites.sprite("hero", session.cabinet),
        SpriteAnimation::looping([0, 1], 0.08),
        Transform::from_translation(start.extend(5.0)).with_rotation(Quat::from_rotation_z(
            match cabinet.scroll {
                Scroll::Sideways => 0.0,
                Scroll::Upward => FRAC_PI_2,
            },
        )),
        Hero,
        Invulnerable(0.0),
        // A forgiving hitbox, smaller than the sprite.
        Hitbox::new(HERO_SIZE - 2.0, HERO_SIZE - 2.0),
        Layer::Hero,
        Gun(0.0),
        DespawnOnExit(Screen::Playing),
    ));

    // Progress bar along the bottom, with a tick at each Restart Point.
    let bar_y = -HALF_H + 6.0;
    commands.spawn((
        Sprite::from_color(Color::srgb(0.2, 0.2, 0.2), Vec2::new(PROGRESS_BAR_W, 2.0)),
        Transform::from_xyz(0.0, bar_y, 20.0),
        DespawnOnExit(Screen::Playing),
    ));
    for point in &stage.timeline.restart_points[1..] {
        let x = -PROGRESS_BAR_W / 2.0 + PROGRESS_BAR_W * point.at / stage.timeline.length;
        commands.spawn((
            Sprite::from_color(Color::srgb(0.6, 0.6, 0.6), Vec2::new(1.0, 4.0)),
            Transform::from_xyz(x.round(), bar_y, 20.5),
            DespawnOnExit(Screen::Playing),
        ));
    }
    commands.spawn((
        Sprite::from_color(Color::WHITE, Vec2::new(1.0, 2.0)),
        Transform::from_xyz(-PROGRESS_BAR_W / 2.0, bar_y, 21.0),
        ProgressFill,
        DespawnOnExit(Screen::Playing),
    ));

    commands.insert_resource(stage);
}

fn scroll_stripes(
    time: Res<Time>,
    stage: Res<StageRun>,
    session: Res<Session>,
    mut stripes: Query<(&Stripe, &mut Transform)>,
) {
    let scroll = CABINETS[session.cabinet].scroll;
    let base = stage.speed() * time.delta_secs();
    for (stripe, mut transform) in &mut stripes {
        let pos = &mut transform.translation;
        match scroll {
            Scroll::Sideways => {
                pos.x -= base * stripe.speed;
                if pos.x < -HALF_W - 8.0 {
                    pos.x += HALF_W * 2.0 + 16.0;
                }
            }
            Scroll::Upward => {
                pos.y -= base * stripe.speed;
                if pos.y < -HALF_H - 8.0 {
                    pos.y += HALF_H * 2.0 + 16.0;
                }
            }
        }
    }
}

fn move_and_fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    session: Res<Session>,
    mut hero: Single<(&mut Transform, &mut Gun), With<Hero>>,
) {
    let t = &tuning.placeholder;
    let cabinet = &CABINETS[session.cabinet];
    let (transform, gun) = &mut *hero;
    let step = controls.direction().normalize_or_zero() * t.hero_speed * time.delta_secs();
    let limit = Vec2::new(HALF_W - HERO_SIZE, HALF_H - HERO_SIZE - 10.0);
    let pos = (transform.translation.truncate() + step).clamp(-limit, limit);
    transform.translation.x = pos.x;
    transform.translation.y = pos.y;

    gun.0 = (gun.0 - time.delta_secs()).max(0.0);
    if controls.held(Button::A) && gun.0 == 0.0 {
        gun.0 = t.fire_seconds;
        let dir = ahead(cabinet.scroll);
        let size = if dir.x != 0.0 {
            Vec2::new(6.0, 2.0)
        } else {
            Vec2::new(2.0, 6.0)
        };
        commands.spawn(projectile(
            Layer::HeroShot,
            pos + dir * HERO_SIZE,
            dir * t.shot_speed,
            size,
            Color::WHITE,
            3.0,
        ));
    }
}

fn spawn_enemies(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    mut requests: MessageReader<SpawnRequested>,
) {
    let t = &tuning.placeholder;
    let cabinet = &CABINETS[session.cabinet];
    for request in requests.read() {
        let (color, gunner) = match request.kind.as_str() {
            "drone" => (Color::srgb(1.0, 0.35, 0.25), None),
            "gunner" => (
                Color::srgb(1.0, 0.8, 0.2),
                Some(Gunner(t.gunner_fire_seconds / 2.0)),
            ),
            // Kinds the placeholder doesn't know belong to real Cabinet code.
            _ => continue,
        };
        let mut enemy = commands.spawn((
            Sprite::from_color(color, Vec2::splat(DRONE_SIZE)),
            Transform::from_translation(request.pos.extend(4.0)),
            Hitbox::new(DRONE_SIZE, DRONE_SIZE),
            Layer::Enemy,
            Health(t.drone_health),
            ScoreValue(t.drone_score),
            Velocity(-ahead(cabinet.scroll) * t.drone_speed),
            DespawnOffscreen::default(),
            DespawnOnExit(Screen::Playing),
        ));
        if let Some(gunner) = gunner {
            enemy.insert(gunner);
        }
    }
}

fn gunners_fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    hero: Single<&Transform, With<Hero>>,
    mut gunners: Query<(&mut Gunner, &Transform)>,
) {
    let t = &tuning.placeholder;
    let target = hero.translation.truncate();
    for (mut gunner, transform) in &mut gunners {
        gunner.0 -= time.delta_secs();
        let from = transform.translation.truncate();
        let on_screen = from.x.abs() < HALF_W && from.y.abs() < HALF_H;
        if gunner.0 > 0.0 || !on_screen {
            continue;
        }
        gunner.0 = t.gunner_fire_seconds;
        let aim = (target - from).normalize_or(Vec2::NEG_X);
        commands.spawn(projectile(
            Layer::EnemyShot,
            from,
            aim * t.enemy_shot_speed,
            Vec2::splat(3.0),
            Color::srgb(1.0, 0.5, 1.0),
            6.0,
        ));
    }
}

fn update_progress(
    stage: Res<StageRun>,
    mut fill: Single<(&mut Sprite, &mut Transform), With<ProgressFill>>,
) {
    let width = (PROGRESS_BAR_W * stage.progress()).round().max(1.0);
    let (sprite, transform) = &mut *fill;
    sprite.custom_size = Some(Vec2::new(width, 2.0));
    transform.translation.x = -PROGRESS_BAR_W / 2.0 + width / 2.0;
}

/// Cabinets without their own code yet use the placeholder.
fn uses_placeholder(session: Option<Res<Session>>) -> bool {
    session.is_some_and(|s| {
        s.cabinet != super::side_shooter::CABINET
            && s.cabinet != super::vertical_shooter::CABINET
            && s.cabinet != super::run_and_gun::CABINET
    })
}

pub struct PlaceholderCabinetPlugin;

impl Plugin for PlaceholderCabinetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::Playing),
            spawn_stage.run_if(uses_placeholder),
        )
        .add_systems(
            Update,
            (
                spawn_enemies.after(KitSet::Timeline),
                (scroll_stripes, move_and_fire, gunners_fire, update_progress)
                    .after(spawn_enemies)
                    .before(KitSet::Move),
            )
                .run_if(uses_placeholder)
                .run_if(stage_live),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn both_stage_files_parse_and_only_use_known_kinds() {
        for scroll in [Scroll::Sideways, Scroll::Upward] {
            let t = stage_timeline(scroll);
            assert!(
                t.spawns
                    .iter()
                    .all(|s| s.kind == "drone" || s.kind == "gunner")
            );
        }
    }
}
