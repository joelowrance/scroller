//! The Hero on foot: Cabinet 3's run-and-gun Hero.
//!
//! Runs and jumps on the kit's platform physics (`kit::platform`), aims in
//! eight directions, crouches, and fires while B is held. The Hero lives in
//! world x (see `terrain.rs`); its screen position follows the camera, and
//! running right of the push line pushes the camera forward (the kit's
//! [`DrivenScroll`]).

use bevy::prelude::*;

use super::palette;
use super::power_ups::PowerUps;
use super::standoff::Standoff;
use super::terrain::{Terrain, screen_x, world_x};
use super::tuning::{RunAndGunTuning, vec2};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::on_foot;
use crate::display::HALF_W;
use crate::flow::Screen;
use crate::input::{Button, Controls};
use crate::kit::platform::{self, Body, Control, TileQuery};
use crate::kit::{
    Damage, DrivenScroll, Explode, Hero, HeroStruck, HitStop, Hitbox, Hurt, Invulnerable, Layer,
    Pierce, Shield, ShieldAbsorbed, StageRestarted, StageRun, combat::hero_invincible, projectile,
};
use crate::sprites::Sprites;
use crate::tuning::Tuning;

/// The Hero's state on foot.
#[derive(Component, Debug)]
pub struct Runner {
    pub body: Body,
    /// +1 facing right, -1 left.
    pub facing: f32,
    pub crouching: bool,
    /// The shot direction this frame.
    pub aim: Vec2,
    /// The Jet's mid-air jump has been used since the last landing.
    pub jet_used: bool,
    /// Dropping in from above the top edge: no control until it lands.
    pub entering: bool,
    /// Fell into a pit; the life is being lost.
    pub fell: bool,
    /// Running on the ground this frame (for the run animation).
    pub running: bool,
    gun_cooldown: f32,
    flare_cooldown: f32,
}

/// Pit falls the invincible debug Hero was rescued from (the unattended run
/// fails if there are any).
#[derive(Resource, Default, Debug)]
pub struct Rescues(pub u32, pub Vec<f32>);

/// A thrown Flare: it falls under gravity and bursts on terrain or an enemy.
#[derive(Component)]
pub struct Flare {
    pub vel: Vec2,
}

/// The Ward's frame around the Hero.
#[derive(Component)]
pub(super) struct WardGlow;

pub use crate::cabinets::on_foot::{ShotSpec, aim_for};

/// Where shots leave from, relative to the feet: chest height, a little in
/// front, or knee height when crouching.
pub fn muzzle(aim: Vec2, facing: f32, crouching: bool, t: &RunAndGunTuning) -> Vec2 {
    on_foot::muzzle(aim, facing, crouching, t.shot.muzzle, t.shot.crouch_muzzle)
}

/// The shots of one volley along `aim` (a unit vector).
pub fn volley(power: &PowerUps, aim: Vec2, from: Vec2, t: &RunAndGunTuning) -> Vec<ShotSpec> {
    on_foot::volley(power.gun, aim, from, &t.shot.gun())
}

/// Seconds between volleys at this gun level.
pub fn fire_interval(power: &PowerUps, t: &RunAndGunTuning) -> f32 {
    on_foot::fire_interval(power.gun, &t.shot.gun())
}

/// The autopilot's input: run right, hold fire, and jump at every pit,
/// step and wall, and off a shaking crumbling ledge, with the base jump.
pub fn autopilot(body: &Body, terrain: &Terrain) -> (Vec2, bool, bool) {
    let ahead = body.x + body.half_width + 6.0;
    let pit = body.grounded && terrain.ground_at(ahead).is_none() && !body.on_ledge(terrain);
    let step = body.grounded
        && terrain
            .ground_top(ahead, ahead + 6.0)
            .is_some_and(|t| t > body.feet + 0.5);
    let off_ledge = body.grounded
        && body.on_ledge(terrain)
        && terrain.ledge_tops(ahead, ahead).is_empty()
        && terrain
            .ground_at(ahead)
            .is_none_or(|g| g < body.feet - 48.0);
    // A crumbling ledge that has started to shake: jump off it.
    let shaking = body.grounded
        && terrain
            .ledge_under(
                body.x - body.half_width,
                body.x + body.half_width,
                body.feet,
            )
            .is_some_and(|i| matches!(terrain.crumbles[i], super::terrain::Crumble::Shaking(_)));
    (Vec2::X, pit || step || off_ledge || shaking, true)
}

/// The Hero: the `c3_hero` sprite (12x20, facing right) and the Ward frame.
/// Frames: 0 stand, 1-4 run, 5 jump, 6 fall, 7 crouch, 8 aim up, 9 aim
/// diagonal.
fn hero_visual(t: &RunAndGunTuning, sprites: &Sprites) -> impl Bundle {
    let body = vec2(t.hero.body);
    (
        sprites.frame("c3_hero", super::CABINET, frame::STAND),
        children![(ward_visual(body), Visibility::Hidden, WardGlow,)],
    )
}

/// The frames of `c3_hero`.
mod frame {
    pub const STAND: usize = 0;
    pub const RUN: [usize; 4] = [1, 2, 3, 4];
    pub const JUMP: usize = 5;
    pub const FALL: usize = 6;
    pub const CROUCH: usize = 7;
    pub const AIM_UP: usize = 8;
    pub const AIM_DIAGONAL: usize = 9;
}

/// Seconds each run frame shows.
const RUN_FRAME_SECONDS: f32 = 0.08;

/// Which `c3_hero` frame the Hero shows. `clock` is seconds of run time.
pub fn frame_for(runner: &Runner, clock: f32) -> usize {
    let body = &runner.body;
    if body.grounded && runner.crouching {
        return frame::CROUCH;
    }
    if runner.aim.y > 0.5 {
        return if runner.aim.x.abs() > 0.1 {
            frame::AIM_DIAGONAL
        } else {
            frame::AIM_UP
        };
    }
    if !body.grounded {
        return if body.vy > 0.0 {
            frame::JUMP
        } else {
            frame::FALL
        };
    }
    if runner.running {
        return frame::RUN[(clock / RUN_FRAME_SECONDS) as usize % frame::RUN.len()];
    }
    frame::STAND
}

/// Draws the Hero: the frame for what it is doing, turned the way it faces.
pub(super) fn animate(
    time: Res<Time>,
    mut hero: Single<(&Runner, &mut Sprite)>,
    mut clock: Local<f32>,
) {
    let (runner, sprite) = &mut *hero;
    *clock = if runner.running {
        *clock + time.delta_secs()
    } else {
        0.0
    };
    let facing_left = runner.facing < 0.0;
    if sprite.flip_x != facing_left {
        sprite.flip_x = facing_left;
    }
    let index = frame_for(runner, *clock);
    if let Some(atlas) = &mut sprite.texture_atlas
        && atlas.index != index
    {
        atlas.index = index;
    }
}

fn ward_visual(body: Vec2) -> impl Bundle {
    let (w, h) = (body.x + 6.0, body.y + 6.0);
    let bar = |size: Vec2, x: f32, y: f32| {
        (
            Sprite::from_color(palette::SPROUT.with_alpha(0.8), size),
            Transform::from_xyz(x, y, 0.0),
        )
    };
    (
        Transform::from_xyz(0.0, 0.0, 0.2),
        children![
            bar(Vec2::new(w, 1.0), 0.0, h / 2.0),
            bar(Vec2::new(w, 1.0), 0.0, -h / 2.0),
            bar(Vec2::new(1.0, h), w / 2.0, 0.0),
            bar(Vec2::new(1.0, h), -w / 2.0, 0.0),
        ],
    )
}

impl Runner {
    /// A Runner standing at world x `x` with its feet at `feet` (tests).
    #[cfg(test)]
    pub fn standing(x: f32, feet: f32) -> Self {
        let mut body = Body::new(x, feet, 5.0);
        body.grounded = true;
        Self {
            body,
            facing: 1.0,
            crouching: false,
            aim: Vec2::X,
            jet_used: false,
            entering: false,
            fell: false,
            running: false,
            gun_cooldown: 0.0,
            flare_cooldown: 0.0,
        }
    }
}

/// A Runner dropping in at screen x `x` from above the top edge.
fn entering_runner(x: f32, camera: f32, t: &RunAndGunTuning) -> Runner {
    let mut body = Body::new(world_x(x, camera), t.hero.entry_from_y, t.hero.body.0 / 2.0);
    body.vy = 0.0;
    Runner {
        body,
        facing: 1.0,
        crouching: false,
        aim: Vec2::X,
        jet_used: false,
        entering: true,
        fell: false,
        running: false,
        gun_cooldown: 0.0,
        flare_cooldown: 0.0,
    }
}

pub(super) fn spawn_hero(
    commands: &mut Commands,
    t: &RunAndGunTuning,
    power: &PowerUps,
    start_x: f32,
    camera: f32,
    sprites: &Sprites,
) {
    let hitbox = vec2(t.hero.hitbox);
    commands.spawn((
        hero_visual(t, sprites),
        Transform::from_xyz(start_x, t.hero.entry_from_y, 5.0),
        (Hero, Layer::Hero),
        Hitbox::new(hitbox.x, hitbox.y),
        // Safe through the drop-in; the respawn time starts on landing.
        Invulnerable(1000.0),
        Shield {
            charges: power.ward as u32,
            invulnerable_seconds: t.power_up.ward_invulnerable_seconds,
        },
        HitStop(t.hero.hit_stop_seconds),
        entering_runner(start_x, camera, t),
        DespawnOnExit(Screen::Playing),
    ));
}

/// Input, physics, the camera push, pits and the screen position.
#[allow(clippy::too_many_arguments)]
pub(super) fn run(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    demo: Option<Res<crate::demo::Demo>>,
    power: Res<PowerUps>,
    stage: Res<StageRun>,
    standoff: Option<Res<Standoff>>,
    mut terrain: ResMut<Terrain>,
    mut rescues: ResMut<Rescues>,
    mut hero: Single<(
        &mut Runner,
        &mut Transform,
        &mut Hitbox,
        &mut Invulnerable,
        &mut Shield,
    )>,
    mut struck: MessageWriter<HeroStruck>,
    mut sfx: MessageWriter<PlaySfx>,
    mut footstep: Local<f32>,
) {
    let t = &tuning.run_and_gun;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    let (runner, transform, hitbox, invulnerable, shield) = &mut *hero;
    if runner.fell {
        return;
    }
    let auto = demo.is_some() || (tuning.debug_keys && t.debug_autopilot);
    let walking_off = stage.cursor.is_cleared();
    let (dir, jump_pressed, jump_held) = if walking_off {
        // The Stage is clear: no control, the Hero walks off to the right.
        (Vec2::X, false, false)
    } else if auto {
        let (d, jump, _) = autopilot(&runner.body, &terrain);
        (d, jump, true)
    } else {
        (
            controls.direction(),
            controls.just_pressed(Button::A),
            controls.held(Button::A),
        )
    };
    let mut input = Control::default();
    if !runner.entering {
        if dir.x.abs() > 0.5 {
            runner.facing = dir.x.signum();
        }
        let down = dir.y < -0.5;
        runner.crouching = down && runner.body.grounded;
        let speed = if runner.body.grounded {
            power.run_speed(t)
        } else {
            power.air_speed(t)
        };
        input = Control {
            run: if runner.crouching {
                0.0
            } else {
                dir.x.signum() * (dir.x.abs() > 0.5) as u8 as f32 * speed
            },
            // Down + A drops through a ledge; on solid ground it does nothing.
            jump_pressed: jump_pressed && !down,
            jump_held,
            drop_through: jump_pressed && down,
            air_jump: (power.jet && !runner.jet_used)
                .then_some(t.jump.speed * t.power_up.jet_speed_fraction),
            ..Control::default()
        };
        runner.aim = aim_for(dir, runner.facing, runner.body.grounded);
    }
    runner.running = runner.body.grounded && input.run != 0.0;
    let before_x = runner.body.x;
    let moved = platform::step(&mut runner.body, input, &t.jump.params(), &*terrain, dt);
    if moved.air_jumped {
        runner.jet_used = true;
        sfx.write(PlaySfx(Sfx::JetPuff));
    }
    if moved.jumped && !runner.entering {
        sfx.write(PlaySfx(Sfx::Jump));
    }
    if moved.landed {
        sfx.write(PlaySfx(Sfx::Land));
    }
    // A soft step about every other run frame while running on the ground.
    if runner.body.grounded && input.run != 0.0 {
        *footstep -= dt;
        if *footstep <= 0.0 {
            *footstep = 0.25;
            sfx.write(PlaySfx(Sfx::Footstep));
        }
    } else {
        *footstep = 0.0;
    }
    if runner.body.grounded {
        runner.jet_used = false;
        if runner.entering {
            runner.entering = false;
            invulnerable.0 = t.hero.respawn_invulnerable_seconds;
        }
    }

    // The left edge is a wall. On the ground the Hero can't get more than
    // `max_x` ahead of the camera; a jump keeps its full length (the camera
    // catches up after), so every gap stays clearable at the cap.
    let half = runner.body.half_width;
    let min_x = world_x(-HALF_W + t.hero.edge_margin + half, camera);
    let max_x = world_x(t.camera.max_x, camera);
    let right_wall = world_x(HALF_W - t.hero.edge_margin - half, camera);
    // Walking off at the Stage's end, nothing holds the Hero in.
    let mut x = if walking_off {
        runner.body.x
    } else {
        runner.body.x.clamp(min_x, right_wall)
    };
    if walking_off {
        // Nothing can hurt the Hero on its way out.
        invulnerable.0 = invulnerable.0.max(1.0);
    } else if runner.body.grounded && x > max_x {
        x = x.min(before_x.max(max_x));
    }
    runner.body.x = x;

    // Crumbling ledges start to shake when the Hero lands on one.
    let standing_on = runner
        .body
        .grounded
        .then(|| terrain.ledge_under(runner.body.x - half, runner.body.x + half, runner.body.feet))
        .flatten();
    let was_whole = standing_on.map(|i| terrain.crumbles[i]);
    terrain.tick_crumbles(standing_on, &t.terrain, dt);
    if let (Some(i), Some(super::terrain::Crumble::Whole)) = (standing_on, was_whole)
        && matches!(terrain.crumbles[i], super::terrain::Crumble::Shaking(_))
    {
        sfx.write(PlaySfx(Sfx::LedgeCreak));
    }

    // Fell into a pit.
    if runner.body.feet < -crate::display::HALF_H {
        if hero_invincible(&tuning) {
            // Put back on the far side, and count the rescue.
            warn!("pit rescue at world x {:.0}", runner.body.x);
            rescues.1.push(runner.body.x);
            if let Some((edge, top)) = terrain.next_ground(runner.body.x, 1.0) {
                runner.body.x = edge + half + 2.0;
                runner.body.feet = top;
                runner.body.vy = 0.0;
                runner.body.grounded = true;
            }
            rescues.0 += 1;
        } else {
            // The Ward doesn't save a fall, and nor does invulnerability.
            runner.fell = true;
            shield.charges = 0;
            invulnerable.0 = 0.0;
            sfx.write(PlaySfx(Sfx::FallWhistle));
            struck.write(HeroStruck { from: None });
        }
    }

    // Push the camera: the Hero right of the push line moves it forward,
    // never during a Standoff.
    let sx = screen_x(runner.body.x, camera);
    let push = if standoff.is_some_and(|s| s.active()) {
        0.0
    } else {
        (sx - t.camera.push_x).max(0.0)
    };
    commands.insert_resource(DrivenScroll(push));

    let (body, hit) = if runner.crouching {
        (vec2(t.hero.crouch_body), vec2(t.hero.crouch_hitbox))
    } else {
        (vec2(t.hero.body), vec2(t.hero.hitbox))
    };
    // The sprite is the standing size even when crouching (the crouch frame
    // sits at the bottom of it), so the hitbox moves down instead.
    hitbox.size = hit;
    hitbox.offset.y = (body.y - t.hero.body.1) / 2.0;
    transform.translation.x = sx.round();
    transform.translation.y = (runner.body.feet + t.hero.body.1 / 2.0).round();
}

/// Fires volleys and Flares while B is held.
#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    demo: Option<Res<crate::demo::Demo>>,
    power: Res<PowerUps>,
    mut hero: Single<&mut Runner>,
    stage: Res<StageRun>,
    shots: Query<&Layer>,
    flares: Query<(), With<Flare>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.run_and_gun;
    let dt = time.delta_secs();
    let runner = &mut **hero;
    runner.gun_cooldown -= dt;
    runner.flare_cooldown -= dt;
    let auto = demo.is_some() || (tuning.debug_keys && t.debug_autopilot);
    if runner.entering
        || runner.fell
        || stage.cursor.is_cleared()
        || !(auto || controls.held(Button::B))
    {
        runner.gun_cooldown = runner.gun_cooldown.max(0.0);
        runner.flare_cooldown = runner.flare_cooldown.max(0.0);
        return;
    }
    let feet = Vec2::new(
        screen_x(runner.body.x, stage.cursor.position),
        runner.body.feet,
    );
    let from = muzzle(runner.aim, runner.facing, runner.crouching, t);

    if power.flare && runner.flare_cooldown <= 0.0 && flares.iter().count() < t.flare.max_on_screen
    {
        runner.flare_cooldown = (runner.flare_cooldown + t.flare.interval).max(0.0);
        sfx.write(PlaySfx(Sfx::FlareLob));
        let (fwd, up) = t.flare.launch;
        let vel = if runner.aim.y > 0.9 {
            Vec2::new(0.0, up)
        } else {
            Vec2::new(fwd * runner.facing, up)
        };
        let size = vec2(t.flare.size);
        commands.spawn((
            Sprite::from_color(palette::EMBER, size),
            Transform::from_translation((feet + from).extend(8.0)),
            Flare { vel },
            crate::kit::DespawnOffscreen::new(8.0),
            DespawnOnExit(Screen::Playing),
        ));
    }

    if runner.gun_cooldown > 0.0 {
        return;
    }
    runner.gun_cooldown = (runner.gun_cooldown + fire_interval(&power, t)).max(0.0);
    let volley = volley(&power, runner.aim, from, t);
    let alive = shots.iter().filter(|l| **l == Layer::HeroShot).count();
    if alive + volley.len() > t.shot.max_on_screen {
        return;
    }
    sfx.write(PlaySfx(if power.gun >= 4 {
        Sfx::Lance
    } else {
        Sfx::Shot
    }));
    for shot in volley {
        let at = feet + shot.offset;
        let mut e = commands.spawn(projectile(
            Layer::HeroShot,
            at,
            shot.direction * t.shot.speed,
            shot.size,
            palette::EMBER,
            2.0,
        ));
        e.insert((
            Transform::from_translation(at.extend(8.0))
                .with_rotation(Quat::from_rotation_z(shot.direction.to_angle())),
            Damage(shot.damage),
        ));
        if shot.pierce {
            e.insert(Pierce {
                through: 1,
                hit: Vec::new(),
            });
        }
    }
}

/// Flares fall, and burst on terrain or an enemy: damage to every enemy in
/// range (an area, so it reaches a Burrower buried under the surface).
#[allow(clippy::type_complexity)]
pub(super) fn fly_flares(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    terrain: Res<Terrain>,
    stage: Res<StageRun>,
    mut flares: Query<(Entity, &mut Flare, &mut Transform)>,
    enemies: Query<
        (Entity, &Layer, &Transform, &Hitbox),
        (With<crate::kit::Health>, Without<Flare>),
    >,
    mut hurt: MessageWriter<Hurt>,
    mut explode: MessageWriter<Explode>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.run_and_gun;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    for (entity, mut flare, mut transform) in &mut flares {
        flare.vel.y = (flare.vel.y - t.jump.gravity_down * dt).max(-t.jump.max_fall);
        transform.translation += (flare.vel * dt).extend(0.0);
        let at = transform.translation.truncate();
        let on_ground = terrain
            .ground_at(world_x(at.x, camera))
            .is_some_and(|top| at.y <= top);
        let touches = |pos: Vec2, half: Vec2, reach: f32| {
            let nearest = at.clamp(pos - half, pos + half);
            at.distance(nearest) <= reach
        };
        let enemy_hit = enemies.iter().any(|(_, layer, tr, hb)| {
            *layer == Layer::Enemy
                && touches(tr.translation.truncate() + hb.offset, hb.size / 2.0, 2.0)
        });
        if !on_ground && !enemy_hit {
            continue;
        }
        commands.entity(entity).despawn();
        sfx.write(PlaySfx(Sfx::FlareBurst));
        explode.write(Explode {
            at,
            color: palette::EMBER,
        });
        for (target, layer, tr, hb) in &enemies {
            if matches!(layer, Layer::Enemy)
                && touches(
                    tr.translation.truncate() + hb.offset,
                    hb.size / 2.0,
                    t.flare.radius,
                )
            {
                hurt.write(Hurt {
                    target,
                    damage: t.flare.damage,
                });
            }
        }
    }
}

/// Keeps the Ward frame in step with the Power-ups.
pub(super) fn sync_equipment(
    tuning: Res<Tuning>,
    power: Res<PowerUps>,
    mut hero: Single<(&Runner, &mut Shield)>,
    mut glow: Single<&mut Visibility, With<WardGlow>>,
) {
    let (_, shield) = &mut *hero;
    if power.is_changed() {
        shield.charges = power.ward as u32;
        shield.invulnerable_seconds = tuning.run_and_gun.power_up.ward_invulnerable_seconds;
    }
    glow.set_if_neq(if power.ward {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// The Ward took a hit: it's gone.
pub(super) fn ward_absorbed(
    mut absorbed: MessageReader<ShieldAbsorbed>,
    mut power: ResMut<PowerUps>,
    hero: Single<&Transform, With<Runner>>,
    mut explode: MessageWriter<Explode>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for _ in absorbed.read() {
        power.ward = false;
        sfx.write(PlaySfx(Sfx::WardBreak));
        explode.write(Explode {
            at: hero.translation.truncate(),
            color: palette::SPROUT,
        });
    }
}

/// After a lost life: one step of power is gone, crumbled ledges are back,
/// and the Hero drops in again at the Restart Point.
pub(super) fn after_restart(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut restarted: MessageReader<StageRestarted>,
    mut power: ResMut<PowerUps>,
    mut terrain: ResMut<Terrain>,
    mut hero: Single<(&mut Runner, &mut Transform, &mut Invulnerable)>,
    flares: Query<Entity, With<Flare>>,
) {
    let Some(restart) = restarted.read().last() else {
        return;
    };
    let t = &tuning.run_and_gun;
    power.lose_life();
    terrain.reset();
    let (runner, transform, invulnerable) = &mut *hero;
    **runner = entering_runner(restart.hero.x, stage.cursor.position, t);
    transform.translation.y = t.hero.entry_from_y;
    invulnerable.0 = 1000.0;
    for e in &flares {
        commands.entity(e).despawn();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t() -> RunAndGunTuning {
        RunAndGunTuning::default()
    }

    #[test]
    fn the_hero_shows_the_frame_for_what_it_is_doing() {
        let mut r = Runner::standing(0.0, 0.0);
        assert_eq!(frame_for(&r, 0.0), frame::STAND);
        r.running = true;
        assert_eq!(frame_for(&r, 0.0), 1);
        assert_eq!(frame_for(&r, RUN_FRAME_SECONDS * 1.5), 2);
        assert_eq!(frame_for(&r, RUN_FRAME_SECONDS * 4.0), 1, "the run loops");
        r.crouching = true;
        assert_eq!(frame_for(&r, 0.0), frame::CROUCH);
        r.crouching = false;
        r.aim = Vec2::Y;
        assert_eq!(frame_for(&r, 0.0), frame::AIM_UP);
        r.aim = Vec2::new(1.0, 1.0).normalize();
        assert_eq!(frame_for(&r, 0.0), frame::AIM_DIAGONAL);
        r.aim = Vec2::X;
        r.body.grounded = false;
        r.body.vy = 50.0;
        assert_eq!(frame_for(&r, 0.0), frame::JUMP);
        r.body.vy = -50.0;
        assert_eq!(frame_for(&r, 0.0), frame::FALL);
    }

    fn power(gun: u32) -> PowerUps {
        PowerUps {
            gun,
            ..PowerUps::fresh()
        }
    }

    #[test]
    fn gun_levels_follow_the_design_doc() {
        let t = t();
        let at = Vec2::ZERO;
        assert_eq!(volley(&power(1), Vec2::X, at, &t).len(), 1);
        assert_eq!(fire_interval(&power(1), &t), 0.125);
        assert_eq!(fire_interval(&power(2), &t), 0.083);
        let g3 = volley(&power(3), Vec2::X, at, &t);
        let angles: Vec<_> = g3
            .iter()
            .map(|s| s.direction.to_angle().to_degrees().round())
            .collect();
        assert_eq!(angles, [0.0, 12.0, -12.0]);
        let g4 = volley(&power(4), Vec2::X, at, &t);
        assert!(g4[0].pierce && g4[0].damage == 2 && g4[0].size == Vec2::new(10.0, 3.0));
        assert!(g4[1..].iter().all(|s| !s.pierce && s.damage == 1));
    }

    #[test]
    fn aim_follows_the_held_direction() {
        let deg = |v: Vec2| v.to_angle().to_degrees().round();
        assert_eq!(
            aim_for(Vec2::ZERO, -1.0, true),
            Vec2::NEG_X,
            "forward, facing left"
        );
        assert_eq!(aim_for(Vec2::Y, 1.0, true), Vec2::Y);
        assert_eq!(deg(aim_for(Vec2::new(1.0, 1.0), 1.0, true)), 45.0);
        assert_eq!(
            aim_for(Vec2::NEG_Y, 1.0, true),
            Vec2::X,
            "down on the ground crouches"
        );
        assert_eq!(
            aim_for(Vec2::NEG_Y, 1.0, false),
            Vec2::NEG_Y,
            "down in the air"
        );
        assert_eq!(deg(aim_for(Vec2::new(-1.0, -1.0), -1.0, false)), -135.0);
    }

    #[test]
    fn shots_leave_at_chest_height_or_knee_height_crouching() {
        let t = t();
        assert_eq!(muzzle(Vec2::X, 1.0, false, &t), Vec2::new(8.0, 14.0));
        assert_eq!(muzzle(Vec2::NEG_X, -1.0, false, &t), Vec2::new(-8.0, 14.0));
        assert_eq!(muzzle(Vec2::X, -1.0, true, &t), Vec2::new(-8.0, 6.0));
    }
}
