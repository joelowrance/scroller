//! How each Cabinet 3 enemy moves and fires, as plain state machines that
//! can be tested at fixed steps (docs/cabinets/3-run-and-gun.md, "Enemies").
//!
//! **Spaces.** A walker's `pos` is its feet in world x (see `terrain.rs`):
//! `(world x, feet y)`. A flyer's `pos` is its centre on screen, as in
//! Cabinets 1 and 2. [`Ctx::hero`] is given in the same space as the brain's
//! `pos`, as the Hero's feet.
//!
//! A brain moves its position and returns an [`Out`]: the shots it wants
//! (from an offset to its position, so in either space), a stomp, or that
//! it is done. The ECS side applies the fairness rules before firing.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::math::Vec2;

use crate::cabinets::run_and_gun::terrain::{Terrain, screen_x};
use crate::cabinets::run_and_gun::tuning::{EnemyRules, EnemyTypes};
use crate::cabinets::shooter::pattern::{dir, home, turn_toward};
use crate::display::{HALF_H, HALF_W};
use crate::kit::platform::{self, Body, Control, JumpParams, TileQuery};

/// Gravity for walkers' hops and falls, px/s² (the Hero's falling gravity).
pub const GRAVITY: f32 = 900.0;

/// What an enemy can see this frame.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub dt: f32,
    pub camera: f32,
    /// The Hero's feet, in this brain's space.
    pub hero: Option<Vec2>,
    pub terrain: &'a Terrain,
}

/// One shot a brain asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    /// From the enemy's position.
    pub offset: Vec2,
    pub velocity: Vec2,
    /// Gravity, px/s² (0 for straight shots).
    pub gravity: f32,
    /// Bursts into two ankle shots along the ground when it lands (the
    /// Dropper's canister).
    pub burst: bool,
    /// Aimed at the Hero (the aimed-shot fairness rule applies).
    pub aimed: bool,
}

impl Shot {
    fn straight(offset: Vec2, velocity: Vec2) -> Self {
        Self {
            offset,
            velocity,
            gravity: 0.0,
            burst: false,
            aimed: false,
        }
    }
}

/// What a brain asks for this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    pub shots: Vec<Shot>,
    /// A Strider's foot came down: shockwaves along the ground.
    pub stomp: bool,
    /// Done, without points (a Mote dissolving).
    pub despawn: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Brain {
    Skitter(Skitter),
    Tick(Tick),
    Kite(Kite),
    Sentry(Sentry),
    Burrower(Burrower),
    Dropper(Dropper),
    Hauler(Hauler),
    Turret(Turret),
    Mote(Mote),
    Strider(Strider),
}

impl Brain {
    pub fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        match self {
            Brain::Skitter(b) => b.step(pos, ctx, t),
            Brain::Tick(b) => b.step(pos, ctx, t),
            Brain::Kite(b) => b.step(pos, ctx, t),
            Brain::Sentry(b) => b.step(pos, ctx, t, r),
            Brain::Burrower(b) => b.step(pos, ctx, t, r),
            Brain::Dropper(b) => b.step(pos, ctx, t),
            Brain::Hauler(b) => b.step(pos, ctx, t),
            Brain::Turret(b) => b.step(pos, ctx, t),
            Brain::Mote(b) => b.step(pos, ctx, t),
            Brain::Strider(b) => b.step(pos, ctx, t, r),
        }
    }

    /// The physics body of a walker that can be carried along by a conveyor
    /// (the Skitter, Tick and Hauler); the rest stand fixed or fly.
    pub fn carried_body(&mut self) -> Option<&mut Body> {
        match self {
            Brain::Skitter(b) => b.body.as_mut(),
            Brain::Tick(b) => b.body.as_mut(),
            Brain::Hauler(b) => b.body.as_mut(),
            _ => None,
        }
    }

    /// True while the enemy flashes a warning.
    pub fn telegraph(&self, t: &EnemyTypes) -> bool {
        match self {
            Brain::Tick(b) => matches!(b.phase, TickPhase::Squat(_)),
            Brain::Kite(b) => matches!(b.phase, KitePhase::Warn(_)),
            Brain::Sentry(b) => b.phase(t) == SentryPhase::Rising,
            Brain::Burrower(b) => matches!(b.phase, BurrowPhase::Dust(_)),
            Brain::Dropper(b) => b.timer <= t.dropper.telegraph_seconds,
            Brain::Hauler(b) => b.timer <= t.hauler.telegraph_seconds,
            Brain::Turret(b) => b.timer <= t.turret.telegraph_seconds,
            Brain::Strider(b) => b.telegraph(t),
            _ => false,
        }
    }
}

/// A walker's body: its feet on the ground, half its width.
fn walker_body(pos: Vec2, half_width: f32) -> Body {
    let mut b = Body::new(pos.x, pos.y, half_width);
    b.grounded = true;
    b
}

/// Hop physics for walkers: a jump of `height` px under [`GRAVITY`].
fn hop_params(height: f32) -> JumpParams {
    JumpParams {
        speed: (2.0 * GRAVITY * height).sqrt(),
        gravity_up: GRAVITY,
        gravity_down: GRAVITY,
        cut: 1.0,
        max_fall: 300.0,
        coyote_seconds: 0.0,
        // Long enough for the press to count on the frame it is made.
        buffer_seconds: 0.05,
    }
}

/// Whether a walker with `body` can take a step of `dx` without walking off
/// its ground or ledge, or into a wall.
fn can_walk(body: &Body, dx: f32, terrain: &Terrain) -> bool {
    let lead = body.x + dx + dx.signum() * body.half_width;
    terrain
        .floor_at(lead, body.feet)
        .is_some_and(|f| (f - body.feet).abs() <= 0.5)
        && terrain
            .ground_top(lead, lead)
            .is_none_or(|g| g <= body.feet + 0.5)
}

/// Which way the Hero is from `x` (+1 right).
fn toward(hero: Option<Vec2>, x: f32) -> f32 {
    hero.map_or(-1.0, |h| if h.x >= x { 1.0 } else { -1.0 })
}

// --- Skitter ---------------------------------------------------------------

/// The runner: along the ground toward the Hero, hopping small gaps.
#[derive(Clone, Debug, PartialEq)]
pub struct Skitter {
    pub dir: f32,
    body: Option<Body>,
}

impl Skitter {
    pub fn new(dir: f32) -> Self {
        Self { dir, body: None }
    }

    fn gap_ahead(body: &Body, dir: f32, max: f32, terrain: &Terrain) -> Option<f32> {
        // The width of open ground ahead, if the far side is at our height.
        let edge = body.x + dir * body.half_width;
        let mut d = 1.0;
        while d <= max + 1.0 {
            let x = edge + dir * d;
            if let Some(top) = terrain.ground_at(x) {
                return ((top - body.feet).abs() <= 0.5).then_some(d);
            }
            d += 1.0;
        }
        None
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let s = &t.skitter;
        let body = self.body.get_or_insert_with(|| walker_body(*pos, 5.0));
        let dx = self.dir * s.speed * ctx.dt;
        let mut input = Control {
            run: self.dir * s.speed,
            jump_held: true,
            ..Control::default()
        };
        if body.grounded && !can_walk(body, dx, ctx.terrain) {
            match Self::gap_ahead(body, self.dir, s.max_hop, ctx.terrain) {
                Some(_) => input.jump_pressed = true,
                None => {
                    self.dir = -self.dir;
                    input.run = 0.0;
                }
            }
        }
        platform::step(body, input, &hop_params(s.hop_height), ctx.terrain, ctx.dt);
        *pos = Vec2::new(body.x, body.feet);
        Out {
            despawn: body.feet < -HALF_H - 16.0,
            ..Out::default()
        }
    }
}

// --- Tick ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum TickPhase {
    Rest(f32),
    /// Squatting before a hop: seconds left.
    Squat(f32),
    Hop,
}

/// The hopper: rests, squats, then hops toward the Hero, never into a pit.
#[derive(Clone, Debug, PartialEq)]
pub struct Tick {
    pub phase: TickPhase,
    pub dir: f32,
    body: Option<Body>,
}

impl Tick {
    /// Member `index` of a group rests `stagger` s longer than the last.
    pub fn new(index: u32, t: &EnemyTypes) -> Self {
        Self {
            phase: TickPhase::Rest(t.tick.rest_seconds + index as f32 * t.tick.stagger),
            dir: -1.0,
            body: None,
        }
    }

    /// Horizontal hop speed for a hop of `hop_length` px.
    fn hop_speed(t: &EnemyTypes) -> f32 {
        let v = hop_params(t.tick.hop_height).speed;
        t.tick.hop_length / (2.0 * v / GRAVITY)
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let k = &t.tick;
        let body = self.body.get_or_insert_with(|| walker_body(*pos, 4.0));
        let mut input = Control {
            jump_held: true,
            ..Control::default()
        };
        match &mut self.phase {
            TickPhase::Rest(left) => {
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.phase = TickPhase::Squat(k.telegraph_seconds);
                }
            }
            TickPhase::Squat(left) => {
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.dir = toward(ctx.hero, body.x);
                    // Never into a pit: hop the other way if the landing
                    // spot has nothing to stand on.
                    let land = body.x + self.dir * k.hop_length;
                    if ctx.terrain.ground_at(land).is_none()
                        && ctx.terrain.ledge_tops(land, land).is_empty()
                    {
                        self.dir = -self.dir;
                    }
                    let land = body.x + self.dir * k.hop_length;
                    if ctx.terrain.ground_at(land).is_none()
                        && ctx.terrain.ledge_tops(land, land).is_empty()
                    {
                        self.phase = TickPhase::Rest(k.rest_seconds);
                    } else {
                        input.jump_pressed = true;
                        self.phase = TickPhase::Hop;
                    }
                }
            }
            TickPhase::Hop => {}
        }
        if self.phase == TickPhase::Hop {
            input.run = self.dir * Self::hop_speed(t);
        }
        let moved = platform::step(body, input, &hop_params(k.hop_height), ctx.terrain, ctx.dt);
        if moved.landed && self.phase == TickPhase::Hop {
            self.phase = TickPhase::Rest(k.rest_seconds);
        }
        *pos = Vec2::new(body.x, body.feet);
        Out {
            despawn: body.feet < -HALF_H - 16.0,
            ..Out::default()
        }
    }
}

// --- Kite ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum KitePhase {
    /// Gliding in to the cruise row, then along it toward the Hero.
    Cruise,
    Warn(f32),
    /// Diving to `floor`, then along it until past the Hero.
    Dive {
        floor: f32,
        past: f32,
    },
    Climb,
}

/// The swooper (a flyer, in screen space).
#[derive(Clone, Debug, PartialEq)]
pub struct Kite {
    pub phase: KitePhase,
    pub dir: f32,
}

impl Kite {
    pub fn new(dir: f32) -> Self {
        Self {
            phase: KitePhase::Cruise,
            dir,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let k = &t.kite;
        match &mut self.phase {
            KitePhase::Cruise => {
                if pos.y > k.cruise_y {
                    pos.y = (pos.y - k.speed * ctx.dt).max(k.cruise_y);
                } else {
                    pos.x += self.dir * k.speed * ctx.dt;
                }
                if let Some(h) = ctx.hero
                    && (pos.x - h.x).abs() <= k.dive_window
                    && pos.y <= k.cruise_y
                {
                    self.phase = KitePhase::Warn(k.telegraph_seconds);
                }
            }
            KitePhase::Warn(left) => {
                *left -= ctx.dt;
                pos.x += self.dir * k.speed * ctx.dt;
                if *left <= 0.0 {
                    // Down until its underside is `dive_floor` above the
                    // ground under the Hero (4 px is half its height).
                    let ground = ctx
                        .hero
                        .and_then(|h| ctx.terrain.ground_at(h.x + ctx.camera - HALF_W))
                        .unwrap_or(-96.0);
                    let past = ctx.hero.map_or(pos.x, |h| h.x) + self.dir * 40.0;
                    self.phase = KitePhase::Dive {
                        floor: ground + k.dive_floor + 4.0,
                        past,
                    };
                }
            }
            KitePhase::Dive { floor, past } => {
                // Down to its floor first, then along it past the Hero.
                pos.x += self.dir * k.dive_speed * 0.4 * ctx.dt;
                pos.y = (pos.y - k.dive_speed * ctx.dt).max(*floor);
                if pos.y <= *floor && (pos.x - *past) * self.dir >= 0.0 {
                    self.phase = KitePhase::Climb;
                }
            }
            KitePhase::Climb => {
                pos.x += self.dir * k.dive_speed * 0.7 * ctx.dt;
                pos.y += k.dive_speed * ctx.dt;
            }
        }
        Out::default()
    }
}

// --- Sentry ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SentryPhase {
    Down,
    Rising,
    Up,
}

/// The timing lock: crouched behind its shield (immune), then up and firing
/// a chest-height burst.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Sentry {
    /// Seconds since it was fully on screen.
    pub age: Option<f32>,
    fired: u32,
}

impl Sentry {
    fn cycle(t: &EnemyTypes) -> f32 {
        let s = &t.sentry;
        (s.down_seconds + s.telegraph_seconds + s.up_seconds).max(0.01)
    }

    pub fn phase(&self, t: &EnemyTypes) -> SentryPhase {
        let s = &t.sentry;
        let Some(age) = self.age else {
            return SentryPhase::Down;
        };
        let p = age.rem_euclid(Self::cycle(t));
        if p < s.down_seconds {
            SentryPhase::Down
        } else if p < s.down_seconds + s.telegraph_seconds {
            SentryPhase::Rising
        } else {
            SentryPhase::Up
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let s = &t.sentry;
        let mut out = Out::default();
        let sx = screen_x(pos.x, ctx.camera);
        if self.age.is_none() && sx.abs() <= HALF_W - 7.0 {
            self.age = Some(0.0);
        }
        let Some(age) = &mut self.age else {
            return out;
        };
        let before = (*age / Self::cycle(t)).floor();
        *age += ctx.dt;
        if (*age / Self::cycle(t)).floor() != before {
            self.fired = 0;
        }
        let into_up = age.rem_euclid(Self::cycle(t)) - s.down_seconds - s.telegraph_seconds;
        while into_up >= 0.0
            && self.fired < s.burst
            && into_up >= self.fired as f32 * s.burst_interval
        {
            let side = toward(ctx.hero, pos.x);
            out.shots.push(Shot::straight(
                Vec2::new(side * 7.0, r.chest_height),
                Vec2::new(side * s.shot_speed, 0.0),
            ));
            self.fired += 1;
        }
        out
    }
}

// --- Burrower --------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum BurrowPhase {
    Buried,
    Dust(f32),
    Up { t: f32, next_shot: f32 },
    Down(f32),
}

/// The ground that bites: buried until the Hero comes near, then up firing
/// ankle-height shots along the ground.
#[derive(Clone, Debug, PartialEq)]
pub struct Burrower {
    pub phase: BurrowPhase,
}

impl Default for Burrower {
    fn default() -> Self {
        Self {
            phase: BurrowPhase::Buried,
        }
    }
}

impl Burrower {
    pub fn risen(&self) -> bool {
        matches!(self.phase, BurrowPhase::Up { .. })
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let b = &t.burrower;
        let mut out = Out::default();
        match &mut self.phase {
            BurrowPhase::Buried => {
                if ctx
                    .hero
                    .is_some_and(|h| (h.x - pos.x).abs() <= b.wake_distance)
                {
                    self.phase = BurrowPhase::Dust(b.telegraph_seconds);
                }
            }
            BurrowPhase::Dust(left) => {
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.phase = BurrowPhase::Up {
                        t: 0.0,
                        next_shot: 0.0,
                    };
                }
            }
            BurrowPhase::Up { t: up, next_shot } => {
                *up += ctx.dt;
                if *up >= *next_shot {
                    *next_shot += b.fire_interval.max(0.1);
                    let side = toward(ctx.hero, pos.x);
                    out.shots.push(Shot::straight(
                        Vec2::new(side * 6.0, r.ankle_height),
                        Vec2::new(side * b.shot_speed, 0.0),
                    ));
                }
                if *up >= b.up_seconds {
                    self.phase = BurrowPhase::Down(b.down_seconds);
                }
            }
            BurrowPhase::Down(left) => {
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.phase = BurrowPhase::Buried;
                }
            }
        }
        out
    }
}

// --- Dropper ---------------------------------------------------------------

/// The gunner overhead (a flyer): crosses slowly and drops canisters that
/// burst along the ground, never right over the Hero.
#[derive(Clone, Debug, PartialEq)]
pub struct Dropper {
    pub timer: f32,
}

impl Dropper {
    pub fn new(t: &EnemyTypes) -> Self {
        Self {
            timer: t.dropper.fire_interval,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let d = &t.dropper;
        pos.x -= d.speed * ctx.dt;
        if pos.y > d.cruise_y {
            pos.y = (pos.y - d.speed * ctx.dt).max(d.cruise_y);
        }
        let mut out = Out::default();
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            if ctx
                .hero
                .is_some_and(|h| (h.x - pos.x).abs() < d.no_drop_window)
            {
                // Hold the canister until it is clear of the Hero.
                self.timer = 0.0;
            } else {
                self.timer += d.fire_interval.max(0.1);
                out.shots.push(Shot {
                    offset: Vec2::new(0.0, -6.0),
                    velocity: Vec2::ZERO,
                    gravity: d.canister_gravity,
                    burst: true,
                    aimed: false,
                });
            }
        }
        out
    }
}

// --- Hauler ----------------------------------------------------------------

/// The power check: a slow armoured crawler that lobs fans of shots.
#[derive(Clone, Debug, PartialEq)]
pub struct Hauler {
    pub dir: f32,
    pub timer: f32,
    body: Option<Body>,
}

impl Hauler {
    /// Which way it faces (+1 right).
    pub fn dir_x(&self) -> f32 {
        self.dir
    }

    pub fn new(dir: f32, t: &EnemyTypes) -> Self {
        Self {
            dir,
            timer: t.hauler.fire_interval,
            body: None,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let h = &t.hauler;
        let body = self.body.get_or_insert_with(|| walker_body(*pos, 12.0));
        self.dir = toward(ctx.hero, body.x);
        let dx = self.dir * h.speed * ctx.dt;
        let run = if can_walk(body, dx, ctx.terrain) {
            self.dir * h.speed
        } else {
            0.0
        };
        platform::step(
            body,
            Control {
                run,
                ..Control::default()
            },
            &hop_params(1.0),
            ctx.terrain,
            ctx.dt,
        );
        *pos = Vec2::new(body.x, body.feet);
        let mut out = Out::default();
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            self.timer += h.fire_interval.max(0.1);
            let n = h.fan_count.max(1);
            let step = if n > 1 {
                h.fan_degrees / (n - 1) as f32
            } else {
                0.0
            };
            for i in 0..n {
                // Up and forward: centred on 45° toward the Hero.
                let a = 45.0 - h.fan_degrees / 2.0 + step * i as f32;
                let d = dir(if self.dir > 0.0 { a } else { 180.0 - a });
                out.shots.push(Shot {
                    offset: Vec2::new(self.dir * 12.0, 18.0),
                    velocity: d * h.shot_speed,
                    gravity: h.shot_gravity,
                    burst: false,
                    aimed: false,
                });
            }
        }
        out
    }
}

// --- Turret ----------------------------------------------------------------

/// Which way a Turret's mount faces it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Mount {
    /// On the floor: it covers the upper half.
    #[default]
    Floor,
    /// Under a ceiling or gantry: the lower half.
    Ceiling,
    /// On a wall to its right: the left half.
    Wall,
}

impl Mount {
    /// The middle of the half-plane it covers, radians.
    fn facing(self) -> f32 {
        match self {
            Mount::Floor => FRAC_PI_2,
            Mount::Ceiling => -FRAC_PI_2,
            Mount::Wall => PI,
        }
    }
}

/// The wall gun: turns toward the Hero (within its half) and fires an aimed
/// shot along its barrel.
#[derive(Clone, Debug, PartialEq)]
pub struct Turret {
    pub mount: Mount,
    /// Barrel direction, radians.
    pub barrel: f32,
    pub timer: f32,
}

impl Turret {
    pub fn new(mount: Mount, t: &EnemyTypes) -> Self {
        Self {
            mount,
            barrel: mount.facing(),
            timer: t.turret.fire_interval,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let tt = &t.turret;
        let mut out = Out::default();
        let Some(hero) = ctx.hero else {
            return out;
        };
        // Aim at the Hero's chest (10 px above the feet), within the half
        // facing away from the mount.
        let want = (hero + Vec2::new(0.0, 10.0) - *pos).to_angle();
        let facing = self.mount.facing();
        let off = (want - facing + PI).rem_euclid(2.0 * PI) - PI;
        let want = facing + off.clamp(-FRAC_PI_2, FRAC_PI_2);
        self.barrel = turn_toward(self.barrel, want, tt.turn_rate.to_radians() * ctx.dt);
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            self.timer += tt.fire_interval.max(0.1);
            let d = bevy::math::Vec2::from_angle(self.barrel);
            out.shots.push(Shot {
                offset: d * 6.0,
                velocity: d * tt.shot_speed,
                gravity: 0.0,
                burst: false,
                aimed: true,
            });
        }
        out
    }
}

// --- Mote ------------------------------------------------------------------

/// The swarm (a flyer): homes slowly, then dissolves.
#[derive(Clone, Debug, PartialEq)]
pub struct Mote {
    pub heading: f32,
    age: f32,
}

impl Mote {
    pub fn new() -> Self {
        Self {
            heading: PI,
            age: 0.0,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let m = &t.mote;
        self.age += ctx.dt;
        let target = ctx.hero.map(|h| h + Vec2::new(0.0, 10.0));
        home(&mut self.heading, pos, target, m.speed, m.turn_rate, ctx.dt);
        Out {
            despawn: self.age >= m.lifetime,
            ..Out::default()
        }
    }
}

impl Default for Mote {
    fn default() -> Self {
        Self::new()
    }
}

// --- Strider ---------------------------------------------------------------

/// The set piece: walks in, paces, and on one clock stomps (a shockwave to
/// jump) then fires a fan (to crouch under).
#[derive(Clone, Debug, PartialEq)]
pub struct Strider {
    /// World x it paces around, once it has reached its hold position.
    pub hold: Option<f32>,
    pub dir: f32,
    /// Seconds into the current cycle (starts once it holds).
    pub cycle: f32,
    fanned: bool,
    stomped: bool,
}

impl Default for Strider {
    fn default() -> Self {
        Self {
            hold: None,
            dir: -1.0,
            cycle: 0.0,
            fanned: false,
            stomped: false,
        }
    }
}

impl Strider {
    /// A leg is lifted for the stomp.
    pub fn stomp_telegraph(&self, t: &EnemyTypes) -> bool {
        self.hold.is_some() && self.cycle < t.strider.stomp_telegraph_seconds
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        let s = &t.strider;
        if self.hold.is_none() {
            return false;
        }
        let fan_at = s.stomp_telegraph_seconds + s.fan_delay;
        self.cycle < s.stomp_telegraph_seconds
            || (self.cycle >= fan_at - s.telegraph_seconds && self.cycle < fan_at)
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let s = &t.strider;
        let mut out = Out::default();
        let sx = screen_x(pos.x, ctx.camera);
        let Some(hold) = self.hold else {
            pos.x -= s.speed * ctx.dt;
            if sx <= s.hold_x {
                self.hold = Some(pos.x);
            }
            return out;
        };
        // Pace back and forth around the hold position.
        pos.x += self.dir * s.speed * ctx.dt;
        if (pos.x - hold).abs() >= s.pace_range {
            pos.x = hold + self.dir * s.pace_range;
            self.dir = -self.dir;
        }
        let cycle = s.cycle_seconds.max(0.1);
        self.cycle += ctx.dt;
        if self.cycle >= cycle {
            self.cycle -= cycle;
            self.fanned = false;
            self.stomped = false;
        }
        if !self.stomped && self.cycle >= s.stomp_telegraph_seconds {
            self.stomped = true;
            out.stomp = true;
        }
        if !self.fanned && self.cycle >= s.stomp_telegraph_seconds + s.fan_delay {
            self.fanned = true;
            let side = toward(ctx.hero, pos.x);
            let n = s.fan_count.max(1);
            let step = if n > 1 {
                s.fan_degrees / (n - 1) as f32
            } else {
                0.0
            };
            for i in 0..n {
                // From level (chest height) up to `fan_degrees`.
                let a = step * i as f32;
                let d = dir(if side > 0.0 { a } else { 180.0 - a });
                out.shots.push(Shot::straight(
                    Vec2::new(side * 8.0, r.chest_height),
                    d * s.shot_speed,
                ));
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cabinets::run_and_gun::terrain::StageTerrain;

    const DT: f32 = 1.0 / 60.0;

    fn types() -> EnemyTypes {
        EnemyTypes::default()
    }

    fn rules() -> EnemyRules {
        EnemyRules::default()
    }

    /// Flat ground at y = -96 from -400 to 2000, with `pits` cut out.
    fn terrain(pits: &[(f32, f32)], ledges: &str) -> Terrain {
        let mut pieces = Vec::new();
        let mut from = -400.0;
        for &(a, b) in pits {
            pieces.push(format!("(from: {from}, to: {a}, top: 3)"));
            from = b;
        }
        pieces.push(format!("(from: {from}, to: 2000.0, top: 3)"));
        let text = format!("(ground: [{}], ledges: [{ledges}])", pieces.join(","));
        Terrain::new(ron::from_str::<StageTerrain>(&text).unwrap())
    }

    fn run(
        brain: &mut Brain,
        pos: &mut Vec2,
        hero: Option<Vec2>,
        ter: &Terrain,
        s: f32,
    ) -> Vec<Out> {
        let t = types();
        let r = rules();
        (0..(s / DT).round() as usize)
            .map(|_| {
                let ctx = Ctx {
                    dt: DT,
                    camera: 0.0,
                    hero,
                    terrain: ter,
                };
                brain.step(pos, &ctx, &t, &r)
            })
            .collect()
    }

    fn shots(outs: &[Out]) -> Vec<Shot> {
        outs.iter().flat_map(|o| o.shots.clone()).collect()
    }

    #[test]
    fn a_skitter_runs_at_the_hero_and_hops_a_small_gap() {
        let ter = terrain(&[(100.0, 120.0)], "");
        let mut b = Brain::Skitter(Skitter::new(1.0));
        let mut pos = Vec2::new(60.0, -96.0);
        run(&mut b, &mut pos, Some(Vec2::new(300.0, -96.0)), &ter, 2.0);
        assert!(
            pos.x > 150.0 && pos.y == -96.0,
            "across the 20 px gap: {pos}"
        );
    }

    #[test]
    fn a_skitter_turns_at_a_wide_gap() {
        let ter = terrain(&[(100.0, 160.0)], "");
        let mut b = Brain::Skitter(Skitter::new(1.0));
        let mut pos = Vec2::new(60.0, -96.0);
        run(&mut b, &mut pos, None, &ter, 2.0);
        assert!(pos.x < 100.0 && pos.y == -96.0, "turned back: {pos}");
    }

    #[test]
    fn a_tick_squats_then_hops_toward_the_hero_but_never_into_a_pit() {
        let t = types();
        let ter = terrain(&[], "");
        let mut b = Brain::Tick(Tick::new(0, &t));
        let mut pos = Vec2::new(200.0, -96.0);
        let mut squatted = false;
        // Rest 0.6 s, squat 0.25 s, then about 0.46 s in the air.
        for _ in 0..(1.4 / DT) as usize {
            run(&mut b, &mut pos, Some(Vec2::new(0.0, -96.0)), &ter, DT);
            squatted |= b.telegraph(&t);
        }
        assert!(squatted, "it squats before hopping");
        assert!(
            (pos.x - 168.0).abs() < 2.0 && pos.y == -96.0,
            "one 32 px hop: {pos}"
        );

        let ter = terrain(&[(140.0, 190.0)], "");
        let mut b = Brain::Tick(Tick::new(0, &t));
        let mut pos = Vec2::new(200.0, -96.0);
        run(&mut b, &mut pos, Some(Vec2::new(0.0, -96.0)), &ter, 3.0);
        assert_eq!(pos.y, -96.0, "never fell in");
    }

    #[test]
    fn a_kite_cruises_warns_and_dives_to_its_floor_then_climbs_away() {
        let t = types();
        let ter = terrain(&[], "");
        let mut b = Brain::Kite(Kite::new(-1.0));
        let mut pos = Vec2::new(60.0, 124.0);
        // The Hero is at screen x -40 (world -40 + 0 - ... at camera 0 the
        // terrain is shifted by HALF_W, so give a Hero on ground).
        let hero = Some(Vec2::new(-40.0, -96.0));
        let mut warned = false;
        let mut lowest = f32::MAX;
        for _ in 0..(5.0 / DT) as usize {
            run(&mut b, &mut pos, hero, &ter, DT);
            warned |= b.telegraph(&t);
            lowest = lowest.min(pos.y);
        }
        assert!(warned);
        assert!(
            (lowest - (-96.0 + 14.0 + 4.0)).abs() < 0.5,
            "dive floor: {lowest}"
        );
        assert!(
            pos.y > lowest + 50.0 && pos.x < -80.0,
            "climbing away: {pos}"
        );
    }

    #[test]
    fn a_sentry_is_down_then_rises_and_fires_a_chest_burst() {
        let _t = types();
        let ter = terrain(&[], "");
        let mut b = Brain::Sentry(Sentry::default());
        // World x -100 is on screen (x = 60) with the camera at 0.
        let mut pos = Vec2::new(-100.0, -96.0);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(-300.0, -96.0)), &ter, 3.6);
        let s = shots(&outs);
        assert_eq!(s.len(), 2, "one burst of two");
        assert!(
            s.iter()
                .all(|s| s.offset.y == 14.0 && s.velocity == Vec2::new(-60.0, 0.0))
        );
        let first = outs.iter().position(|o| !o.shots.is_empty()).unwrap() as f32 * DT;
        assert!(
            (first - 2.1).abs() < 0.05,
            "first opening 2.1 s in: {first}"
        );
    }

    #[test]
    fn a_burrower_wakes_near_the_hero_and_fires_ankle_shots() {
        let t = types();
        let ter = terrain(&[], "");
        let mut b = Brain::Burrower(Burrower::default());
        let mut pos = Vec2::new(100.0, -96.0);
        let far = run(&mut b, &mut pos, Some(Vec2::new(-100.0, -96.0)), &ter, 2.0);
        assert!(shots(&far).is_empty(), "asleep while the Hero is far");
        let near = run(&mut b, &mut pos, Some(Vec2::new(40.0, -96.0)), &ter, 2.5);
        let s = shots(&near);
        assert_eq!(s.len(), 2, "at 0.4 s and 2.0 s after waking");
        assert!(s.iter().all(|s| s.offset.y == 4.0 && s.velocity.x < 0.0));
        let Brain::Burrower(bb) = &b else {
            unreachable!()
        };
        assert!(bb.risen());
        let _ = t;
    }

    #[test]
    fn a_dropper_never_drops_over_the_hero() {
        let ter = terrain(&[], "");
        let mut b = Brain::Dropper(Dropper::new(&types()));
        let mut pos = Vec2::new(176.0, 72.0);
        // The Hero keeps exactly under it: nothing drops.
        let mut dropped = 0;
        for _ in 0..(4.0 / DT) as usize {
            let hero = Some(Vec2::new(pos.x, -96.0));
            dropped += shots(&run(&mut b, &mut pos, hero, &ter, DT)).len();
        }
        assert_eq!(dropped, 0);
        let away_x = pos.x - 100.0;
        let away = run(&mut b, &mut pos, Some(Vec2::new(away_x, -96.0)), &ter, 0.1);
        let s = shots(&away);
        assert_eq!(s.len(), 1);
        assert!(s[0].burst && s[0].gravity == 600.0, "a canister");
    }

    #[test]
    fn a_hauler_walks_at_the_hero_and_lobs_a_fan_under_gravity() {
        let ter = terrain(&[], "");
        let mut b = Brain::Hauler(Hauler::new(-1.0, &types()));
        let mut pos = Vec2::new(200.0, -96.0);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, -96.0)), &ter, 2.5);
        assert!((pos.x - (200.0 - 50.0)).abs() < 1.0, "20 px/s: {pos}");
        let s = shots(&outs);
        assert_eq!(s.len(), 3);
        assert!(
            s.iter()
                .all(|s| s.gravity == 120.0 && s.velocity.x < 0.0 && s.velocity.y > 0.0)
        );
    }

    #[test]
    fn a_turret_only_covers_the_half_facing_away_from_its_mount() {
        let ter = terrain(&[], "");
        let mut b = Brain::Turret(Turret::new(Mount::Ceiling, &types()));
        let mut pos = Vec2::new(0.0, 40.0);
        // The Hero is above a ceiling turret: it can't aim up past level.
        let outs = run(&mut b, &mut pos, Some(Vec2::new(-100.0, 80.0)), &ter, 2.1);
        let s = shots(&outs);
        assert_eq!(s.len(), 1);
        assert!(s[0].aimed && s[0].velocity.y <= 0.01, "{:?}", s[0]);
    }

    #[test]
    fn a_strider_walks_in_then_stomps_before_it_fans_every_cycle() {
        let t = types();
        let ter = terrain(&[], "");
        let mut b = Brain::Strider(Strider::default());
        let mut pos = Vec2::new(176.0, -96.0);
        // At camera 0 a world x of -88 is screen x 72, its hold.
        let mut stomps = Vec::new();
        let mut fans = Vec::new();
        for i in 0..(30.0 / DT) as usize {
            let out = &run(&mut b, &mut pos, Some(Vec2::new(-300.0, -96.0)), &ter, DT)[0];
            if out.stomp {
                stomps.push(i as f32 * DT);
            }
            if !out.shots.is_empty() {
                assert_eq!(out.shots.len(), 5);
                assert!(
                    out.shots[0].velocity.y.abs() < 1e-3,
                    "the lowest flies level"
                );
                fans.push(i as f32 * DT);
            }
        }
        assert!(stomps.len() >= 3 && fans.len() >= 3);
        for (s, f) in stomps.iter().zip(&fans) {
            assert!(
                (f - s - t.strider.fan_delay).abs() < 0.05,
                "fan 2.2 s after the stomp"
            );
        }
        let Brain::Strider(st) = &b else {
            unreachable!()
        };
        let hold = st.hold.unwrap();
        assert!((pos.x - hold).abs() <= t.strider.pace_range + 0.01);
    }

    #[test]
    fn a_mote_homes_then_dissolves() {
        let ter = terrain(&[], "");
        let mut b = Brain::Mote(Mote::new());
        let mut pos = Vec2::new(100.0, 60.0);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, -96.0)), &ter, 8.1);
        assert!(outs.last().unwrap().despawn);
    }
}
