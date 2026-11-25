//! The Hero climbing: Cabinet 4's on-foot Hero
//! (docs/cabinets/4-climber.md, "The Hero, climbing").
//!
//! The same body, jump and gun as Cabinet 3 under `climber.` keys, on the
//! kit's platform physics with ladders, in tower space (see `tower.rs`).
//! The camera rises on its own; the Hero can't go above the top of the play
//! area (the lid), and feet below the bottom edge (the fall line) cost a
//! life. A lost life drops the Hero in under a canopy at the Restart Point,
//! with the scroll held until it lands.

use bevy::prelude::*;

use super::palette;
use super::power_ups::PowerUps;
use super::tower::{Tower, screen_y, tower_y};
use super::tuning::{ClimberTuning, vec2};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::on_foot::standoff::Standoff;
use crate::cabinets::on_foot::{self, aim_for};
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::flow::Screen;
use crate::input::{Button, Controls};
use crate::kit::platform::{self, Body, Control, TileQuery};
use crate::kit::{
    Damage, DrivenScroll, Explode, Hero, HeroStruck, HitStop, Hitbox, Invulnerable, Layer, Pierce,
    Shield, ShieldAbsorbed, StageRestarted, StageRun, combat::hero_invincible, projectile,
};
use crate::sprites::Sprites;
use crate::tuning::Tuning;

/// The Hero's sprite (Cabinet 3's, with Cabinet 4's climbing frames).
pub const SPRITE: &str = "c3_hero";

/// Frames of [`SPRITE`].
mod frame {
    pub const STAND: usize = 0;
    pub const RUN: [usize; 4] = [1, 2, 3, 4];
    pub const JUMP: usize = 5;
    pub const FALL: usize = 6;
    pub const CROUCH: usize = 7;
    pub const AIM_UP: usize = 8;
    pub const AIM_DIAGONAL: usize = 9;
    pub const CLIMB: [usize; 2] = [10, 11];
    pub const HANG: usize = 12;
}

/// The Hero's state on the tower.
#[derive(Component, Debug)]
pub struct Climber {
    /// Feet in tower space.
    pub body: Body,
    /// +1 facing right, -1 left.
    pub facing: f32,
    pub crouching: bool,
    /// The shot direction this frame.
    pub aim: Vec2,
    /// The Jet's mid-air jump has been used since the last landing or
    /// ladder.
    pub jet_used: bool,
    /// The Canopy is open (falling slowly).
    pub canopy_open: bool,
    /// Dropping in under the canopy after a lost life: no control until it
    /// lands, and the scroll waits.
    pub entering: bool,
    /// Fell below the bottom edge; the life is being lost.
    pub fell: bool,
    /// The next foothold of the Stage's route (the autopilot's target).
    pub route_next: usize,
    gun_cooldown: f32,
    /// Seconds into the current run or climb animation.
    anim: f32,
}

/// Falls below the bottom edge the invincible debug Hero was rescued from
/// (the unattended run fails if there are any), with the heights.
#[derive(Resource, Default, Debug)]
pub struct Rescues(pub u32, pub Vec<f32>);

/// The Ward's frame around the Hero.
#[derive(Component)]
pub(super) struct WardGlow;

/// The canopy over the Hero.
#[derive(Component)]
pub(super) struct CanopyShade;

impl Climber {
    fn new(x: f32, feet: f32, t: &ClimberTuning) -> Self {
        let mut body = Body::new(x, feet, t.hero.body.0 / 2.0);
        body.height = t.hero.body.1;
        Self {
            body,
            facing: 1.0,
            crouching: false,
            aim: Vec2::Y,
            jet_used: false,
            canopy_open: false,
            entering: false,
            fell: false,
            route_next: 0,
            gun_cooldown: 0.0,
            anim: 0.0,
        }
    }

    /// Standing at screen position `centre` (a Restart Point's `hero`).
    pub fn standing(centre: Vec2, camera: f32, t: &ClimberTuning) -> Self {
        let feet = tower_y(centre.y - t.hero.body.1 / 2.0, camera);
        Self::new(centre.x, feet, t)
    }

    /// Dropping in under the canopy from the top edge at screen x `x`.
    pub fn dropping_in(x: f32, camera: f32, t: &ClimberTuning) -> Self {
        let feet = tower_y(PLAY_TOP, camera) - t.hero.body.1;
        Self {
            entering: true,
            ..Self::new(x, feet, t)
        }
    }

    /// Head height in tower space.
    pub fn head(&self) -> f32 {
        self.body.feet + self.body.height
    }

    pub fn on_ladder(&self) -> bool {
        self.body.ladder.is_some()
    }
}

/// What the player (or the autopilot) asks for this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wants {
    pub dir: Vec2,
    pub jump_pressed: bool,
    pub jump_held: bool,
    pub fire: bool,
}

/// The rules of one frame of climbing, without the ECS: input to motion,
/// ladders, the Jet and the Canopy, gusts, the lid and the face's walls.
/// Returns what the platform physics reported.
pub fn advance(
    c: &mut Climber,
    wants: Wants,
    power: &PowerUps,
    tower: &Tower,
    camera: f32,
    t: &ClimberTuning,
    dt: f32,
) -> platform::Moved {
    if c.entering {
        // Straight down under the canopy, no control, through thin ledges
        // onto the Restart Point's landing.
        let next = c.body.feet - t.hero.respawn_fall_speed * dt;
        let (x0, x1) = (c.body.x - c.body.half_width, c.body.x + c.body.half_width);
        let floor = tower
            .solid_tops(x0, x1)
            .into_iter()
            .filter(|top| *top <= c.body.feet + 0.5 && *top >= next)
            .reduce(f32::max);
        c.canopy_open = true;
        if let Some(top) = floor {
            c.body.feet = top;
            c.body.vy = 0.0;
            c.body.grounded = true;
            c.entering = false;
            c.canopy_open = false;
            return platform::Moved {
                landed: true,
                ..Default::default()
            };
        }
        c.body.feet = next;
        return platform::Moved::default();
    }

    let dir = wants.dir;
    if dir.x.abs() > 0.5 {
        c.facing = dir.x.signum();
    }
    let down = dir.y < -0.5;
    let climb = if dir.y > 0.5 {
        1.0
    } else if down {
        -1.0
    } else {
        0.0
    };
    let speed = if c.body.grounded {
        t.hero.run_speed
    } else {
        t.hero.air_speed
    };
    let run = if dir.x.abs() > 0.5 {
        dir.x.signum() * speed
    } else {
        0.0
    };
    // The wind, except on a ladder (the Hero holds on) or on a ledge with
    // Mag boots.
    let held_by_mag = power.mag && c.body.grounded;
    let push = if c.on_ladder() || held_by_mag {
        0.0
    } else {
        tower.push_at(c.body.feet, &t.hazards.gust)
    };
    let jet_ready = power.jet && !c.jet_used;
    let mut input = Control {
        run,
        jump_pressed: wants.jump_pressed && !down,
        jump_held: wants.jump_held,
        drop_through: wants.jump_pressed && down,
        air_jump: jet_ready.then_some(t.jump.speed * t.power_up.jet_speed_fraction),
        climb,
    };
    // On a ladder, A jumps off sideways or lets go; Down + A is a let-go too.
    if c.on_ladder() {
        input.jump_pressed = wants.jump_pressed;
    }
    let jump = t.jump.params();
    let moved = match platform::climb(&mut c.body, input, &t.ladder.params(), &jump, tower, dt) {
        Some(m) => {
            c.crouching = false;
            m
        }
        None => {
            c.crouching = down && c.body.grounded;
            if c.crouching {
                input.run = 0.0;
            }
            input.run += push;
            platform::step(&mut c.body, input, &jump, tower, dt)
        }
    };
    if moved.air_jumped {
        c.jet_used = true;
    }
    if c.body.grounded || c.on_ladder() {
        c.jet_used = false;
    }

    // The Canopy: holding A while falling, once the Jet (if any) is spent.
    let falling = !c.body.grounded && !c.on_ladder() && c.body.vy < 0.0;
    c.canopy_open = power.canopy
        && falling
        && wants.jump_held
        && !(power.jet && !c.jet_used)
        && !moved.air_jumped;
    if c.canopy_open {
        c.body.vy = c.body.vy.max(-t.power_up.canopy_fall_speed);
    }

    // The lid: the head stops at the top of the play area.
    let lid = tower_y(PLAY_TOP, camera);
    if c.head() > lid {
        c.body.feet = lid - c.body.height;
        c.body.vy = c.body.vy.min(0.0);
        if let Some(ladder) = c.body.ladder
            && c.body.feet < ladder.bottom
        {
            c.body.ladder = None;
        }
    }

    c.aim = if c.on_ladder() {
        Vec2::Y
    } else {
        aim_for(dir, c.facing, c.body.grounded)
    };
    moved
}

/// The autopilot's input: follow the Stage file's route with the base
/// moves (walks, jumps of the base jump, drops and ladders), firing up.
/// Before a jump up it waits for the lid (`camera`) to leave room to land.
pub fn autopilot(c: &mut Climber, tower: &Tower, camera: f32) -> Wants {
    let route = &tower.route;
    let b = &c.body;
    // Past every foothold it has reached: standing on it, or passing it
    // on a ladder (a route may stop part-way up one).
    while let Some(f) = route.get(c.route_next) {
        let standing = b.grounded && (b.feet - f.h).abs() <= 0.5 && (b.x - f.x).abs() <= 4.0;
        let climbing = c.on_ladder() && (b.feet - f.h).abs() <= 1.0 && (b.x - f.x).abs() <= 1.0;
        if standing || climbing {
            c.route_next += 1;
        } else {
            break;
        }
    }
    let mut wants = Wants {
        jump_held: true,
        fire: true,
        ..Wants::default()
    };
    let Some(mut target) = route.get(c.route_next).copied() else {
        return wants;
    };
    // Knocked off the route (out of a jump's reach and no ladder there):
    // back to the nearest foothold from where it stands.
    let ladder_to = |h: f32| {
        tower.ladders.iter().any(|d| {
            (d.x - target.x).abs() <= 1.0
                && d.bottom <= b.feet.min(h) + 0.5
                && d.top >= b.feet.max(h) - 0.5
        })
    };
    if b.grounded && target.h - b.feet > 48.0 && !ladder_to(target.h) {
        let from = c.route_next.saturating_sub(8);
        c.route_next = from + route_index_at_slice(&route[from..], b.x, b.feet);
        match route.get(c.route_next) {
            Some(f) => target = *f,
            None => return wants,
        }
    }
    let dx = target.x - b.x;
    let toward = Vec2::new(dx.signum(), 0.0);
    // A ladder from here to there.
    let ladder = tower.ladders.iter().find(|d| {
        let lo = b.feet.min(target.h);
        let hi = b.feet.max(target.h);
        (d.x - target.x).abs() <= 1.0
            && d.bottom <= lo + 0.5
            && d.top >= hi - 0.5
            && (target.h - b.feet).abs() > 0.5
    });
    if let Some(d) = ladder {
        if c.on_ladder() {
            wants.dir = Vec2::new(0.0, (target.h - b.feet).signum());
        } else if (d.x - b.x).abs() > 2.0 {
            wants.dir = Vec2::new((d.x - b.x).signum(), 0.0);
        } else {
            wants.dir = Vec2::new(0.0, (target.h - b.feet).signum());
        }
        return wants;
    }
    if c.on_ladder() {
        // At the ladder's end the climb puts it on the ledge.
        wants.dir = Vec2::new(0.0, (target.h - b.feet).signum());
        return wants;
    }
    if !b.grounded {
        // In the air: steer for the foothold.
        if dx.abs() > 1.0 {
            wants.dir = toward;
        }
        return wants;
    }
    let ahead = b.x + dx.signum() * (b.half_width + 2.0);
    let edge_ahead = tower
        .floor_below(ahead, b.feet)
        .is_none_or(|f| f < b.feet - 0.5);
    // Room under the lid to land at height `h`.
    let room = |h: f32| tower_y(PLAY_TOP, camera) - b.height - h >= 2.0;
    // A crumbling foothold can't be waited on: go only when the step after
    // it can be taken at once.
    let crumble = |f: &super::terrain::Foothold| {
        tower
            .ledges
            .iter()
            .position(|l| l.crumbles() && l.h == f.h && l.covers(f.x))
    };
    let then_blocked = route[c.route_next..]
        .iter()
        .find(|n| (n.h - target.h).abs() > 0.5)
        .is_some_and(|n| n.h > target.h && !room(n.h));
    let unsafe_crumble = crumble(&target).is_some_and(|i| {
        then_blocked || !matches!(tower.crumbles[i], super::tower::Crumble::Whole)
    });
    if target.h > b.feet + 0.5 {
        // A jump up: from close enough in, or from the edge, once the lid
        // has risen far enough to land under it.
        if !room(target.h) || unsafe_crumble {
            return hold_against_wind(wants, c, tower, &route[..c.route_next]);
        }
        wants.dir = if dx.abs() > 1.0 { toward } else { Vec2::ZERO };
        let rise = target.h - b.feet;
        // Close in so the arc's top is over the target.
        let reach = if rise > 24.0 { 20.0 } else { 26.0 };
        if dx.abs() <= reach || edge_ahead {
            wants.jump_pressed = true;
        }
    } else if target.h < b.feet - 0.5 {
        // A drop: through a thin ledge right above it, or off the edge.
        if dx.abs() <= 6.0
            && tower
                .ledge_under(b.x - b.half_width, b.x + b.half_width, b.feet)
                .is_some_and(|i| !Tower::is_solid(&tower.ledges[i]))
        {
            wants.dir = Vec2::NEG_Y;
            wants.jump_pressed = true;
        } else {
            wants.dir = toward;
        }
    } else if dx.abs() > 1.0 {
        wants.dir = toward;
    } else {
        return hold_against_wind(wants, c, tower, &route[..c.route_next]);
    }
    wants
}

/// Waiting on a ledge: walk back to the last foothold if a gust has pushed
/// the Hero off it (walking against a gust still moves forward).
fn hold_against_wind(
    mut wants: Wants,
    c: &Climber,
    tower: &Tower,
    passed: &[super::terrain::Foothold],
) -> Wants {
    let b = &c.body;
    let anchor = passed
        .last()
        .filter(|f| (f.h - b.feet).abs() <= 0.5)
        .map_or(b.x, |f| f.x);
    if b.grounded && (anchor - b.x).abs() > 1.0 && tower.gust_at(b.feet).is_some() {
        wants.dir = Vec2::new((anchor - b.x).signum(), 0.0);
    }
    wants
}

/// The Hero's look: `c3_hero` in this Cabinet's palette, the canopy and the
/// Ward frame.
fn hero_visual(t: &ClimberTuning, sprites: &Sprites) -> impl Bundle {
    let body = vec2(t.hero.body);
    (
        sprites.frame(SPRITE, super::CABINET, frame::STAND),
        children![
            (
                sprites.sprite("parachute", super::CABINET),
                Transform::from_xyz(0.0, body.y / 2.0 + 8.0, -0.1),
                Visibility::Hidden,
                CanopyShade,
            ),
            (ward_visual(body), Visibility::Hidden, WardGlow),
        ],
    )
}

fn ward_visual(body: Vec2) -> impl Bundle {
    let (w, h) = (body.x + 6.0, body.y + 6.0);
    let bar = |size: Vec2, x: f32, y: f32| {
        (
            Sprite::from_color(palette::MINT.with_alpha(0.8), size),
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

pub(super) fn spawn_hero(
    commands: &mut Commands,
    t: &ClimberTuning,
    sprites: &Sprites,
    power: &PowerUps,
    climber: Climber,
) {
    let hitbox = vec2(t.hero.hitbox);
    let at = Vec2::new(climber.body.x, 0.0);
    commands.spawn((
        hero_visual(t, sprites),
        Transform::from_xyz(at.x, at.y, 5.0),
        (Hero, Layer::Hero),
        Hitbox::new(hitbox.x, hitbox.y),
        Invulnerable(t.hero.respawn_invulnerable_seconds),
        Shield {
            charges: power.ward as u32,
            invulnerable_seconds: t.power_up.ward_invulnerable_seconds,
        },
        HitStop(t.hero.hit_stop_seconds),
        climber,
        DespawnOnExit(Screen::Playing),
    ));
}

/// Input, physics, crumbling ledges, the fall line, the scroll hold and the
/// screen position.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub(super) fn run(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    demo: Option<Res<crate::demo::Demo>>,
    power: Res<PowerUps>,
    stage: Res<StageRun>,
    standoff: Option<Res<Standoff>>,
    mut tower: ResMut<Tower>,
    mut rescues: ResMut<Rescues>,
    mut hero: Single<(
        &mut Climber,
        &mut Transform,
        &mut Sprite,
        &mut Hitbox,
        &mut Invulnerable,
        &mut Shield,
    )>,
    mut canopy: Single<&mut Visibility, With<CanopyShade>>,
    sprites: Res<Sprites>,
    mut struck: MessageWriter<HeroStruck>,
    mut sfx: MessageWriter<PlaySfx>,
    mut sounds: Local<(f32, f32)>,
) {
    let t = &tuning.climber;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    tower.clock += dt;
    let (c, transform, sprite, hitbox, invulnerable, shield) = &mut *hero;
    if c.fell {
        return;
    }
    let auto = demo.is_some() || (tuning.debug_keys && t.debug_autopilot);
    let wants = if auto {
        autopilot(c, &tower, camera)
    } else {
        Wants {
            dir: controls.direction(),
            jump_pressed: controls.just_pressed(Button::A),
            jump_held: controls.held(Button::A),
            fire: controls.held(Button::B),
        }
    };
    let was_entering = c.entering;
    let was_open = c.canopy_open;
    let before_feet = c.body.feet;
    let moved = advance(c, wants, &power, &tower, camera, t, dt);
    if was_entering && !c.entering {
        invulnerable.0 = t.hero.respawn_invulnerable_seconds;
    }
    if moved.air_jumped {
        sfx.write(PlaySfx(Sfx::JetPuff));
    }
    if moved.jumped && !was_entering {
        sfx.write(PlaySfx(Sfx::Jump));
    }
    if moved.landed && !c.on_ladder() {
        sfx.write(PlaySfx(Sfx::Land));
    }
    if c.canopy_open && !was_open && !c.entering {
        sfx.write(PlaySfx(Sfx::CanopyOpen));
        sounds.1 = 0.5;
    }
    if c.canopy_open {
        sounds.1 -= dt;
        if sounds.1 <= 0.0 {
            sounds.1 = 0.5;
            sfx.write(PlaySfx(Sfx::CanopyFlutter));
        }
    }
    // A rung about every 8 px climbed; a step about every other run frame.
    let climbing = c.on_ladder() && c.body.feet != before_feet;
    let running = c.body.grounded && wants.dir.x.abs() > 0.5 && !c.crouching;
    if climbing || running {
        c.anim += dt;
        sounds.0 -= dt;
        if sounds.0 <= 0.0 {
            sounds.0 = if climbing { 1.0 / 6.0 } else { 0.25 };
            sfx.write(PlaySfx(if climbing {
                Sfx::LadderRung
            } else {
                Sfx::Footstep
            }));
        }
    } else {
        sounds.0 = 0.0;
    }

    // Crumbling ledges start to shake when the Hero lands on one.
    let half = c.body.half_width;
    let standing_on = c
        .body
        .grounded
        .then(|| tower.ledge_under(c.body.x - half, c.body.x + half, c.body.feet))
        .flatten();
    let was_whole = standing_on.map(|i| tower.crumbles[i]);
    tower.tick_crumbles(standing_on, power.crumble_factor(t), &t.terrain, dt);
    if let (Some(i), Some(super::tower::Crumble::Whole)) = (standing_on, was_whole)
        && matches!(tower.crumbles[i], super::tower::Crumble::Shaking(_))
    {
        sfx.write(PlaySfx(Sfx::LedgeCreak));
    }

    // Below the bottom edge.
    if screen_y(c.body.feet, camera) < t.hero.fall_line {
        if hero_invincible(&tuning) {
            rescue(c, &tower, camera, t, &mut rescues);
        } else {
            // The Ward doesn't save a fall, and nor does invulnerability.
            c.fell = true;
            shield.charges = 0;
            invulnerable.0 = 0.0;
            sfx.write(PlaySfx(Sfx::FallWhistle));
            struck.write(HeroStruck { from: None });
        }
    }

    // The scroll waits for a canopy drop and stops for a Standoff.
    let hold = c.entering || standoff.is_some_and(|s| s.active());
    commands.insert_resource(DrivenScroll(if hold { 0.0 } else { f32::INFINITY }));

    let (body, hit) = if c.crouching {
        (vec2(t.hero.crouch_body), vec2(t.hero.crouch_hitbox))
    } else {
        (vec2(t.hero.body), vec2(t.hero.hitbox))
    };
    hitbox.size = hit;
    // The sprite is drawn full height; a crouch is in its lower part.
    hitbox.offset.y = (body.y - t.hero.body.1) / 2.0;
    transform.translation.x = c.body.x.round();
    transform.translation.y = (screen_y(c.body.feet, camera) + t.hero.body.1 / 2.0).round();
    let f = look_frame(c, running);
    if sprite.texture_atlas.as_ref().is_some_and(|a| a.index != f) {
        *sprite.as_mut() = Sprite {
            flip_x: c.facing < 0.0,
            ..sprites.frame(SPRITE, super::CABINET, f)
        };
    }
    sprite.flip_x = c.facing < 0.0;
    canopy.set_if_neq(if c.canopy_open || c.entering {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// The sprite frame for the Hero's state.
fn look_frame(c: &Climber, running: bool) -> usize {
    if c.on_ladder() {
        return frame::CLIMB[(c.anim * 6.0) as usize % 2];
    }
    if c.entering || c.canopy_open {
        return frame::HANG;
    }
    if c.crouching {
        return frame::CROUCH;
    }
    if !c.body.grounded {
        return if c.body.vy > 0.0 {
            frame::JUMP
        } else {
            frame::FALL
        };
    }
    if c.aim.y > 0.9 {
        return frame::AIM_UP;
    }
    if c.aim.y > 0.5 {
        return frame::AIM_DIAGONAL;
    }
    if running {
        return frame::RUN[(c.anim * 10.0) as usize % 4];
    }
    frame::STAND
}

/// The invincible debug Hero fell below the edge: put it on the route's
/// next foothold that is on screen, and count it.
fn rescue(c: &mut Climber, tower: &Tower, camera: f32, t: &ClimberTuning, rescues: &mut Rescues) {
    warn!("fall rescue at height {:.0}", c.body.feet);
    rescues.0 += 1;
    rescues.1.push(c.body.feet);
    let low = tower_y(t.hero.fall_line, camera) + t.terrain.drop_margin;
    let foothold = tower.route.iter().enumerate().find(|(_, f)| {
        f.h >= low
            && screen_y(f.h, camera) + t.hero.body.1 < PLAY_TOP
            && tower.floor_below(f.x, f.h) == Some(f.h)
    });
    let (i, x, h) = match foothold {
        Some((i, f)) => (i, f.x, f.h),
        None => (c.route_next, c.body.x, low),
    };
    c.route_next = i;
    c.body.x = x;
    c.body.feet = h;
    c.body.vy = 0.0;
    c.body.ladder = None;
    c.body.grounded = false;
}

/// Fires volleys while B is held: straight up only on a ladder.
#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    demo: Option<Res<crate::demo::Demo>>,
    power: Res<PowerUps>,
    stage: Res<StageRun>,
    mut hero: Single<&mut Climber>,
    shots: Query<&Layer>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.climber;
    let dt = time.delta_secs();
    let c = &mut **hero;
    c.gun_cooldown -= dt;
    let auto = demo.is_some() || (tuning.debug_keys && t.debug_autopilot);
    if c.entering || c.fell || !(auto || controls.held(Button::B)) {
        c.gun_cooldown = c.gun_cooldown.max(0.0);
        return;
    }
    if c.gun_cooldown > 0.0 {
        return;
    }
    let gun = t.shot.gun();
    c.gun_cooldown = (c.gun_cooldown + on_foot::fire_interval(power.gun, &gun)).max(0.0);
    let aim = if auto { Vec2::Y } else { c.aim };
    let feet = Vec2::new(c.body.x, screen_y(c.body.feet, stage.cursor.position));
    let from = on_foot::muzzle(
        aim,
        c.facing,
        c.crouching,
        t.shot.muzzle,
        t.shot.crouch_muzzle,
    );
    let volley = on_foot::volley(power.gun, aim, from, &gun);
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

/// Keeps the Ward frame in step with the Power-ups.
pub(super) fn sync_equipment(
    tuning: Res<Tuning>,
    power: Res<PowerUps>,
    mut shield: Single<&mut Shield, With<Climber>>,
    mut glow: Single<&mut Visibility, With<WardGlow>>,
) {
    if power.is_changed() {
        shield.charges = power.ward as u32;
        shield.invulnerable_seconds = tuning.climber.power_up.ward_invulnerable_seconds;
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
    hero: Single<&Transform, With<Climber>>,
    mut explode: MessageWriter<Explode>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for _ in absorbed.read() {
        power.ward = false;
        sfx.write(PlaySfx(Sfx::WardBreak));
        explode.write(Explode {
            at: hero.translation.truncate(),
            color: palette::MINT,
        });
    }
}

/// After a lost life: one step of power is gone, crumbled ledges are back,
/// and the Hero drops in under the canopy at the Restart Point.
pub(super) fn after_restart(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut restarted: MessageReader<StageRestarted>,
    mut power: ResMut<PowerUps>,
    mut tower: ResMut<Tower>,
    mut hero: Single<(&mut Climber, &mut Transform, &mut Invulnerable)>,
) {
    let Some(restart) = restarted.read().last() else {
        return;
    };
    let t = &tuning.climber;
    let camera = stage.cursor.position;
    power.lose_life();
    tower.reset();
    let (c, transform, invulnerable) = &mut *hero;
    let feet = tower_y(restart.hero.y - t.hero.body.1 / 2.0, camera);
    let route_next = route_index_at(&tower, restart.hero.x, feet);
    **c = Climber::dropping_in(restart.hero.x, camera, t);
    c.route_next = route_next;
    transform.translation.y = PLAY_TOP - t.hero.body.1 / 2.0;
    // Safe through the drop; the respawn time starts on landing.
    invulnerable.0 = 1000.0;
    // The scroll waits from this frame on.
    commands.insert_resource(DrivenScroll(0.0));
}

/// The first route foothold at or above a Restart Point's.
pub fn route_index_at(tower: &Tower, x: f32, feet: f32) -> usize {
    route_index_at_slice(&tower.route, x, feet)
}

/// The first foothold in `route` that is where (`x`, `feet`) is, or above.
fn route_index_at_slice(route: &[super::terrain::Foothold], x: f32, feet: f32) -> usize {
    route
        .iter()
        .position(|f| f.h > feet + 0.5 || ((f.h - feet).abs() <= 0.5 && (f.x - x).abs() <= 4.0))
        .unwrap_or(route.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 120.0;

    const STAGE: &str = r#"(
        length: 1000.0,
        speed: [(from: 0.0, speed: 14.0)],
        ledges: [
            (h: 0.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
            (h: 40.0, from_x: -128.0, to_x: -64.0, kind: "thin"),
            (h: 80.0, from_x: -40.0, to_x: 8.0, kind: "thin"),
            (h: 40.0, from_x: 80.0, to_x: 112.0, kind: "thin"),
            (h: 176.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
        ],
        ladders: [(x: 92.0, from_h: 40.0, to_h: 176.0)],
        gusts: [(from_h: 200.0, to_h: 300.0, dir: "left", period: 6.0, offset: 0.0)],
        route: [(x: -100.0, h: 0.0), (x: -96.0, h: 40.0), (x: -16.0, h: 80.0),
                (x: 100.0, h: 0.0), (x: 92.0, h: 40.0), (x: 0.0, h: 176.0)],
    )"#;

    fn t() -> ClimberTuning {
        ClimberTuning::default()
    }

    fn tower() -> Tower {
        Tower::parse(STAGE, &t()).unwrap()
    }

    fn standing(x: f32, feet: f32) -> Climber {
        let mut c = Climber::new(x, feet, &t());
        c.body.grounded = true;
        c
    }

    fn hold(dir: Vec2) -> Wants {
        Wants {
            dir,
            ..Wants::default()
        }
    }

    /// Runs the Hero for `seconds` with the camera at `camera`.
    fn go(
        c: &mut Climber,
        tower: &Tower,
        power: &PowerUps,
        camera: f32,
        seconds: f32,
        wants: impl Fn(usize) -> Wants,
    ) {
        for i in 0..(seconds / DT).round() as usize {
            advance(c, wants(i), power, tower, camera, &t(), DT);
        }
    }

    #[test]
    fn it_runs_and_jumps_as_in_cabinet_3() {
        let w = tower();
        let p = PowerUps::fresh();
        let mut c = standing(0.0, 0.0);
        go(&mut c, &w, &p, 200.0, 1.0, |_| hold(Vec2::X));
        assert!((c.body.x - 72.0).abs() < 1.0, "72 px/s: {}", c.body.x);
        let mut peak: f32 = 0.0;
        for i in 0..120 {
            let wants = Wants {
                jump_pressed: i == 0,
                jump_held: true,
                ..Wants::default()
            };
            advance(&mut c, wants, &p, &w, 200.0, &t(), DT);
            peak = peak.max(c.body.feet);
        }
        assert!((peak - 48.0).abs() < 1.5, "48 px high: {peak}");
    }

    #[test]
    fn on_a_ladder_it_only_aims_straight_up() {
        let w = tower();
        let p = PowerUps::fresh();
        let mut c = standing(92.0, 40.0);
        go(&mut c, &w, &p, 300.0, 0.5, |_| hold(Vec2::new(1.0, 1.0)));
        assert!(c.on_ladder(), "climbing: {:?}", c.body);
        assert_eq!(c.aim, Vec2::Y);
        assert_eq!(c.body.x, 92.0, "held to the ladder");
        go(&mut c, &w, &p, 300.0, 3.0, |_| hold(Vec2::Y));
        assert!(
            c.body.grounded && c.body.feet == 176.0,
            "on the landing above"
        );
    }

    #[test]
    fn the_jet_is_a_second_jump_recharged_by_a_ladder() {
        let w = tower();
        let p = PowerUps {
            jet: true,
            ..PowerUps::fresh()
        };
        let mut c = standing(-16.0, 80.0);
        let mut presses = [0usize, 30].into_iter().peekable();
        for i in 0..40 {
            let press = presses.next_if_eq(&i).is_some();
            advance(
                &mut c,
                Wants {
                    jump_pressed: press,
                    jump_held: true,
                    ..Wants::default()
                },
                &p,
                &w,
                300.0,
                &t(),
                DT,
            );
        }
        assert!(c.jet_used);
        assert!(c.body.vy > 0.0, "rising again: {}", c.body.vy);
        // Catching a ladder recharges it.
        let mut on = standing(92.0, 40.0);
        on.jet_used = true;
        go(&mut on, &w, &p, 300.0, 0.1, |_| hold(Vec2::Y));
        assert!(on.on_ladder() && !on.jet_used);
    }

    #[test]
    fn the_canopy_caps_a_fall_only_while_a_is_held() {
        let w = tower();
        let p = PowerUps {
            canopy: true,
            ..PowerUps::fresh()
        };
        let mut c = standing(40.0, 150.0);
        c.body.grounded = false;
        go(&mut c, &w, &p, 300.0, 0.5, |_| Wants {
            jump_held: true,
            ..Wants::default()
        });
        assert!(c.canopy_open);
        assert_eq!(c.body.vy, -50.0);
        go(&mut c, &w, &p, 300.0, 0.1, |_| Wants::default());
        assert!(!c.canopy_open && c.body.vy < -50.0, "closed: {}", c.body.vy);
        // Without the Power-up, holding A does nothing.
        let mut plain = standing(40.0, 150.0);
        plain.body.grounded = false;
        go(&mut plain, &w, &PowerUps::fresh(), 300.0, 0.5, |_| Wants {
            jump_held: true,
            ..Wants::default()
        });
        assert!(!plain.canopy_open && plain.body.vy < -50.0);
    }

    #[test]
    fn with_the_jet_the_first_press_in_the_air_is_the_jet_then_the_canopy() {
        let w = tower();
        let p = PowerUps {
            jet: true,
            canopy: true,
            ..PowerUps::fresh()
        };
        let mut c = standing(40.0, 150.0);
        c.body.grounded = false;
        go(&mut c, &w, &p, 300.0, 0.2, |_| Wants {
            jump_held: true,
            ..Wants::default()
        });
        assert!(!c.canopy_open, "the Jet is still unspent");
        go(&mut c, &w, &p, 300.0, 0.01, |_| Wants {
            jump_pressed: true,
            jump_held: true,
            ..Wants::default()
        });
        assert!(c.jet_used);
        go(&mut c, &w, &p, 300.0, 0.6, |_| Wants {
            jump_held: true,
            ..Wants::default()
        });
        assert!(c.canopy_open, "falling after the Jet");
    }

    #[test]
    fn gusts_push_except_on_a_ladder_or_on_a_ledge_with_mag_boots() {
        let mut w = tower();
        w.clock = 1.0; // blowing (0.8 s of warning, then 2 s)
        let p = PowerUps::fresh();
        let mut c = standing(0.0, 176.0);
        c.body.feet = 176.0;
        // The zone is from 200, so stand the Hero in it on the landing by
        // moving the zone down.
        w.gusts[0].from_h = 170.0;
        let start = c.body.x;
        advance(&mut c, Wants::default(), &p, &w, 400.0, &t(), 0.5);
        assert!(
            (c.body.x - (start - 18.0)).abs() < 0.5,
            "36 px/s: {}",
            c.body.x
        );
        // Walking against it still moves forward at 36 px/s.
        let start = c.body.x;
        advance(&mut c, hold(Vec2::X), &p, &w, 400.0, &t(), 0.5);
        assert!((c.body.x - (start + 18.0)).abs() < 0.5);
        let mag = PowerUps {
            mag: true,
            ..PowerUps::fresh()
        };
        let start = c.body.x;
        advance(&mut c, Wants::default(), &mag, &w, 400.0, &t(), 0.5);
        assert_eq!(c.body.x, start, "Mag boots hold");
    }

    #[test]
    fn the_lid_stops_the_head_at_the_top_of_the_play_area() {
        let w = tower();
        let p = PowerUps::fresh();
        // Camera at 100: the lid is at height 100 - 12 = 88.
        let mut c = standing(-100.0, 40.0);
        let mut peak: f32 = 0.0;
        for i in 0..120 {
            advance(
                &mut c,
                Wants {
                    jump_pressed: i == 0,
                    jump_held: true,
                    ..Wants::default()
                },
                &p,
                &w,
                100.0,
                &t(),
                DT,
            );
            peak = peak.max(c.head());
        }
        assert!((peak - 88.0).abs() < 0.01, "head at the lid: {peak}");
    }

    #[test]
    fn the_canopy_drop_lands_on_the_landing_below() {
        let w = tower();
        let p = PowerUps::fresh();
        // Camera at 160: the top edge is at height 160 - 12.
        let mut c = Climber::dropping_in(-100.0, 160.0, &t());
        assert_eq!(c.body.feet, 160.0 - 12.0 - 20.0);
        go(&mut c, &w, &p, 160.0, 1.0, |_| hold(Vec2::X));
        assert!(c.entering && c.body.x == -100.0, "no control");
        assert!((c.body.feet - (128.0 - 60.0)).abs() < 0.6, "60 px/s");
        go(&mut c, &w, &p, 160.0, 2.0, |_| Wants::default());
        assert!(
            !c.entering && c.body.grounded && c.body.feet == 0.0,
            "through the thin ledge"
        );
    }

    #[test]
    fn the_autopilot_follows_the_route_with_base_moves() {
        let w = tower();
        let p = PowerUps::fresh();
        let mut c = standing(-100.0, 0.0);
        let mut camera = 160.0;
        let mut reached = 0;
        for _ in 0..(30.0 / DT) as usize {
            let wants = autopilot(&mut c, &w, camera);
            advance(&mut c, wants, &p, &w, camera, &t(), DT);
            reached = reached.max(c.route_next);
            camera = (camera + 14.0 * DT).min(300.0);
        }
        assert!(
            c.body.grounded && c.body.feet == 176.0,
            "on the top landing: {:?} (next foothold {reached})",
            c.body
        );
    }
}
