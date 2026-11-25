//! How each Cabinet 4 enemy moves and fires, as plain state machines that
//! can be tested at fixed steps (docs/cabinets/4-climber.md, "Enemies").
//!
//! **Spaces.** Enemies on the tower (Limpets, Pickets, Pitchers, Vents and
//! canisters) live in tower space, `(x, height)` (see `tower.rs`); a
//! walker's `pos` is its feet, a Limpet's and a Vent's their centre.
//! Flyers, Scrap, Plumbs, Motes and the Winch live on screen, `pos` their
//! centre. [`Ctx::hero`] is the Hero's feet in tower space; [`Ctx::hero_on_screen`]
//! gives it on screen.
//!
//! A brain moves its position and returns an [`Out`]: the shots it wants
//! (from an offset to its position, in its own space), or that it is done.
//! The ECS side applies the fairness rules before firing.

use std::f32::consts::{FRAC_PI_2, PI, TAU};

use bevy::math::Vec2;

use crate::cabinets::climber::terrain::TILE;
use crate::cabinets::climber::tower::{Tower, screen_y, tower_y};
use crate::cabinets::climber::tuning::{EnemyRules, EnemyTypes};
use crate::cabinets::shooter::pattern::{fan, home, turn_toward};
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::display::{HALF_H, HALF_W};

/// What an enemy can see this frame.
#[derive(Clone, Copy)]
pub struct Ctx<'a> {
    pub dt: f32,
    pub camera: f32,
    /// The Hero's feet, in tower space.
    pub hero: Option<Vec2>,
    pub tower: &'a Tower,
}

impl Ctx<'_> {
    /// The Hero's feet on screen.
    pub fn hero_on_screen(&self) -> Option<Vec2> {
        self.hero
            .map(|h| Vec2::new(h.x, screen_y(h.y, self.camera)))
    }
}

/// One shot a brain asks for.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shot {
    /// From the enemy's position.
    pub offset: Vec2,
    pub velocity: Vec2,
}

impl Shot {
    fn new(offset: Vec2, velocity: Vec2) -> Self {
        Self { offset, velocity }
    }
}

/// What a brain asks for this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    pub shots: Vec<Shot>,
    /// A Pitcher throws a canister: its launch offset and velocity (tower
    /// space).
    pub canister: Option<(Vec2, Vec2)>,
    /// Done, without points (gone off screen, reeled away, dissolved).
    pub despawn: bool,
    /// Broke up on a solid ledge (Scrap, a canister), without points.
    pub shattered: bool,
    /// A warning started this frame (for its sound).
    pub warned: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Brain {
    Flit(Flit),
    Limpet(Limpet),
    Scrap(Scrap),
    Picket(Picket),
    Vent(Vent),
    Plumb(Plumb),
    Pitcher(Pitcher),
    Canister(Canister),
    Mote(Mote),
    Winch(Winch),
}

impl Brain {
    pub fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        match self {
            Brain::Flit(b) => b.step(pos, ctx, t),
            Brain::Limpet(b) => b.step(pos, ctx, t),
            Brain::Scrap(b) => b.step(pos, ctx, t),
            Brain::Picket(b) => b.step(pos, ctx, t, r),
            Brain::Vent(b) => b.step(pos, ctx, t),
            Brain::Plumb(b) => b.step(pos, ctx, t, r),
            Brain::Pitcher(b) => b.step(pos, ctx, t, r),
            Brain::Canister(b) => b.step(pos, ctx),
            Brain::Mote(b) => b.step(pos, ctx, t),
            Brain::Winch(b) => b.step(pos, ctx, t, r),
        }
    }

    /// True while the enemy flashes a warning.
    pub fn telegraph(&self) -> bool {
        match self {
            Brain::Picket(b) => matches!(b.phase, PicketPhase::Flash(_)),
            Brain::Vent(b) => b.phase == VentPhase::Glowing,
            Brain::Plumb(b) => matches!(b.phase, PlumbPhase::Hang(_)) && !b.fired,
            Brain::Pitcher(b) => b.winding,
            Brain::Winch(b) => matches!(b.phase, WinchPhase::Shake(_)),
            _ => false,
        }
    }
}

// --- Flit ------------------------------------------------------------------

/// The crossing flyer: across the screen in a sine wave, or up from the
/// bottom edge and out through the nearer side. It doesn't fire.
#[derive(Clone, Debug, PartialEq)]
pub struct Flit {
    /// +1 moving right, -1 left (a side Flit).
    pub dir: f32,
    /// Rising from below: its heading (radians), once it has risen.
    pub rising: Option<f32>,
    base_y: Option<f32>,
    age: f32,
}

impl Flit {
    pub fn crossing(dir: f32) -> Self {
        Self {
            dir,
            rising: None,
            base_y: None,
            age: 0.0,
        }
    }

    pub fn from_below() -> Self {
        Self {
            dir: 0.0,
            rising: Some(FRAC_PI_2),
            base_y: None,
            age: 0.0,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let f = &t.flit;
        self.age += ctx.dt;
        let start = *self.base_y.get_or_insert(pos.y);
        if let Some(heading) = &mut self.rising {
            // Straight up for its first 64 px, then out through the nearer
            // side at 90°/s.
            if pos.y - start > 64.0 {
                let out = if pos.x >= 0.0 { 0.0 } else { PI };
                *heading = turn_toward(*heading, out, FRAC_PI_2 * ctx.dt);
            }
            *pos += Vec2::from_angle(*heading) * f.speed * ctx.dt;
        } else {
            pos.x += self.dir * f.speed * ctx.dt;
            pos.y = start + f.wave_amplitude * (TAU * self.age / f.wave_period.max(0.01)).sin();
        }
        let gone = pos.x.abs() > HALF_W + 16.0 || pos.y > HALF_H + 16.0;
        Out {
            despawn: gone && self.age > 0.5,
            ..Out::default()
        }
    }
}

// --- Limpet ----------------------------------------------------------------

/// The ledge crawler: all the way round its ledge (along the top, round the
/// end, along the underside and back). Its `pos` is its centre, 3 px out
/// from the ledge's surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Limpet {
    pub ledge: usize,
    /// How far round the ledge it is, from the top's left end, clockwise.
    pub along: f32,
}

/// Half a Limpet's height: how far its centre sits off the surface.
const LIMPET_LIFT: f32 = 3.0;

impl Limpet {
    pub fn new(ledge: usize, along: f32) -> Self {
        Self { ledge, along }
    }

    /// The length of the path round a ledge from `x0` to `x1`: the ledge
    /// grown by the Limpet's lift on every side.
    pub fn round(x0: f32, x1: f32) -> f32 {
        2.0 * (x1 - x0 + 2.0 * LIMPET_LIFT) + 2.0 * (TILE + 2.0 * LIMPET_LIFT)
    }

    /// The centre at `along` round a ledge from `x0` to `x1` with its top at
    /// `top`, starting at the top's left corner.
    pub fn place(x0: f32, x1: f32, top: f32, along: f32) -> Vec2 {
        let (l, r) = (x0 - LIMPET_LIFT, x1 + LIMPET_LIFT);
        let (t, b) = (top + LIMPET_LIFT, top - TILE - LIMPET_LIFT);
        let (w, h) = (r - l, t - b);
        let s = along.rem_euclid(2.0 * (w + h));
        if s < w {
            Vec2::new(l + s, t)
        } else if s < w + h {
            Vec2::new(r, t - (s - w))
        } else if s < 2.0 * w + h {
            Vec2::new(r - (s - w - h), b)
        } else {
            Vec2::new(l, b + (s - 2.0 * w - h))
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        self.along += t.limpet.speed * ctx.dt;
        if let Some(l) = ctx.tower.ledges.get(self.ledge) {
            *pos = Self::place(l.from_x, l.to_x, l.h, self.along);
        }
        Out::default()
    }
}

// --- Scrap -----------------------------------------------------------------

/// Falling junk: a chevron at the top edge first, then straight down,
/// through thin ledges, shattering on a solid one.
#[derive(Clone, Debug, PartialEq)]
pub struct Scrap {
    /// Seconds of warning left.
    pub warning: f32,
}

impl Scrap {
    pub fn new(t: &EnemyTypes) -> Self {
        Self {
            warning: t.scrap.warning_seconds,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        if self.warning > 0.0 {
            self.warning -= ctx.dt;
            // Held above the top edge until the warning is over.
            pos.y = pos.y.max(PLAY_TOP + 8.0);
            return Out::default();
        }
        pos.y -= t.scrap.speed * ctx.dt;
        let bottom = tower_y(pos.y - 5.0, ctx.camera);
        let solid = ctx.tower.in_solid(pos.x, bottom);
        Out {
            shattered: solid,
            despawn: pos.y < -HALF_H - 16.0,
            ..Out::default()
        }
    }
}

/// Picks where each piece of a scrapfall lands: random within the span, but
/// never within `min_spacing` of the last piece.
#[derive(Clone, Debug, PartialEq)]
pub struct Scrapfall {
    pub from_x: f32,
    pub to_x: f32,
    /// Seconds left.
    pub left: f32,
    /// Seconds to the next piece.
    pub next: f32,
    pub last_x: Option<f32>,
    seed: u32,
}

impl Scrapfall {
    pub fn new(from_x: f32, to_x: f32, seconds: f32, seed: u32) -> Self {
        Self {
            from_x: from_x.min(to_x),
            to_x: from_x.max(to_x),
            left: seconds,
            next: 0.0,
            last_x: None,
            seed: seed.max(1),
        }
    }

    fn random(&mut self) -> f32 {
        // xorshift: the same fall every time a Stage is played.
        self.seed ^= self.seed << 13;
        self.seed ^= self.seed >> 17;
        self.seed ^= self.seed << 5;
        (self.seed % 10_000) as f32 / 10_000.0
    }

    /// Advances the fall; returns the x of each piece due now, and whether
    /// the fall is over.
    pub fn step(&mut self, dt: f32, t: &EnemyTypes) -> (Vec<f32>, bool) {
        let mut pieces = Vec::new();
        self.next -= dt;
        while self.next <= 0.0 && self.left > 0.0 {
            self.next += t.scrap.fall_interval.max(0.05);
            let span = self.to_x - self.from_x;
            let mut x = self.from_x;
            for _ in 0..16 {
                x = self.from_x + self.random() * span;
                if self
                    .last_x
                    .is_none_or(|l| (x - l).abs() >= t.scrap.min_spacing)
                {
                    break;
                }
            }
            // A span too narrow to keep the spacing: the far end from the last.
            if let Some(l) = self.last_x
                && (x - l).abs() < t.scrap.min_spacing
            {
                x = if l - self.from_x > self.to_x - l {
                    self.from_x
                } else {
                    self.to_x
                };
            }
            self.last_x = Some(x);
            pieces.push(x);
        }
        self.left -= dt;
        (pieces, self.left <= 0.0)
    }
}

// --- Picket ----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PicketPhase {
    Walk,
    /// Stopped, flashing: seconds left.
    Flash(f32),
    /// Firing: shots left, seconds to the next.
    Burst(u32, f32),
    /// Waiting after a burst.
    Cooldown(f32),
}

/// The ledge guard: paces its ledge, and fires a burst along its own level
/// at chest height when the Hero is level with it on the side it faces.
#[derive(Clone, Debug, PartialEq)]
pub struct Picket {
    pub ledge: usize,
    /// +1 facing and walking right.
    pub dir: f32,
    pub phase: PicketPhase,
}

impl Picket {
    pub fn new(ledge: usize, dir: f32) -> Self {
        Self {
            ledge,
            dir,
            phase: PicketPhase::Walk,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let p = &t.picket;
        let mut out = Out::default();
        let level = ctx.hero.is_some_and(|h| {
            (h.y - pos.y).abs() <= p.level_window && (h.x - pos.x) * self.dir > 0.0
        });
        self.phase = match self.phase {
            PicketPhase::Walk if level => {
                out.warned = true;
                PicketPhase::Flash(p.telegraph_seconds)
            }
            PicketPhase::Walk => {
                if let Some(l) = ctx.tower.ledges.get(self.ledge) {
                    let half = 6.0;
                    pos.x += self.dir * p.speed * ctx.dt;
                    if pos.x + half >= l.to_x {
                        pos.x = l.to_x - half;
                        self.dir = -1.0;
                    } else if pos.x - half <= l.from_x {
                        pos.x = l.from_x + half;
                        self.dir = 1.0;
                    }
                }
                PicketPhase::Walk
            }
            PicketPhase::Flash(left) if left - ctx.dt <= 0.0 => PicketPhase::Burst(p.burst, 0.0),
            PicketPhase::Flash(left) => PicketPhase::Flash(left - ctx.dt),
            PicketPhase::Burst(0, _) => PicketPhase::Cooldown(p.cooldown_seconds),
            PicketPhase::Burst(n, wait) if wait - ctx.dt <= 0.0 => {
                out.shots.push(Shot::new(
                    Vec2::new(self.dir * 6.0, r.chest_height),
                    Vec2::new(self.dir * p.shot_speed, 0.0),
                ));
                PicketPhase::Burst(n - 1, p.burst_interval)
            }
            PicketPhase::Burst(n, wait) => PicketPhase::Burst(n, wait - ctx.dt),
            PicketPhase::Cooldown(left) if left - ctx.dt <= 0.0 => PicketPhase::Walk,
            PicketPhase::Cooldown(left) => PicketPhase::Cooldown(left - ctx.dt),
        };
        out
    }
}

// --- Vent ------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VentPhase {
    /// Not yet fully on screen: closed, its cycle not started.
    Waiting,
    Closed,
    Glowing,
    Open,
}

/// The timing lock: a grille in the face, closed and immune, then glowing,
/// then open (and hittable), puffing a fan of three shots downward.
#[derive(Clone, Debug, PartialEq)]
pub struct Vent {
    pub phase: VentPhase,
    clock: f32,
}

impl Default for Vent {
    fn default() -> Self {
        Self {
            phase: VentPhase::Waiting,
            clock: 0.0,
        }
    }
}

impl Vent {
    /// Only an open Vent takes damage.
    pub fn open(&self) -> bool {
        self.phase == VentPhase::Open
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let v = &t.vent;
        let mut out = Out::default();
        if self.phase == VentPhase::Waiting {
            let y = screen_y(pos.y, ctx.camera);
            if y + 6.0 <= PLAY_TOP && y - 6.0 >= -HALF_H {
                self.phase = VentPhase::Closed;
            }
            return out;
        }
        self.clock += ctx.dt;
        let cycle = v.closed_seconds + v.telegraph_seconds + v.open_seconds;
        let p = self.clock.rem_euclid(cycle.max(0.01));
        let phase = if p < v.closed_seconds {
            VentPhase::Closed
        } else if p < v.closed_seconds + v.telegraph_seconds {
            VentPhase::Glowing
        } else {
            VentPhase::Open
        };
        if phase == VentPhase::Glowing && self.phase == VentPhase::Closed {
            out.warned = true;
        }
        if phase == VentPhase::Open && self.phase != VentPhase::Open {
            for d in fan(Vec2::NEG_Y, v.fan_count, 2.0 * v.fan_degrees) {
                out.shots
                    .push(Shot::new(Vec2::new(0.0, -6.0), d * v.shot_speed));
            }
        }
        self.phase = phase;
        out
    }
}

// --- Plumb -----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum PlumbPhase {
    Drop,
    /// Hanging level with the Hero's chest: seconds into the hang.
    Hang(f32),
    Reel,
}

/// The drop on a line: down from the top edge to the Hero's chest height,
/// a flash, one shot each way, then back up.
#[derive(Clone, Debug, PartialEq)]
pub struct Plumb {
    pub phase: PlumbPhase,
    pub fired: bool,
}

impl Default for Plumb {
    fn default() -> Self {
        Self {
            phase: PlumbPhase::Drop,
            fired: false,
        }
    }
}

impl Plumb {
    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let p = &t.plumb;
        let mut out = Out::default();
        match self.phase {
            PlumbPhase::Drop => {
                // Level with the Hero's chest, wherever the Hero is now.
                let chest = ctx
                    .hero_on_screen()
                    .map_or(0.0, |h| h.y + r.chest_height)
                    .clamp(-HALF_H + 16.0, PLAY_TOP - 8.0);
                pos.y -= p.drop_speed * ctx.dt;
                if pos.y <= chest {
                    // Level with the chest, or where it was if the chest is above.
                    pos.y = chest.min(pos.y + p.drop_speed * ctx.dt);
                    self.phase = PlumbPhase::Hang(0.0);
                    out.warned = true;
                }
            }
            PlumbPhase::Hang(time) => {
                let time = time + ctx.dt;
                if !self.fired && time >= p.telegraph_seconds {
                    self.fired = true;
                    let hero = ctx.hero_on_screen();
                    for side in [-1.0, 1.0] {
                        // Skip a side where the Hero is within 40 px.
                        let close = hero.is_some_and(|h| {
                            (h.x - pos.x) * side >= 0.0
                                && Vec2::new(h.x, h.y + r.chest_height).distance(*pos)
                                    < r.min_fire_distance
                        });
                        if !close {
                            out.shots.push(Shot::new(
                                Vec2::new(side * 5.0, 0.0),
                                Vec2::new(side * p.shot_speed, 0.0),
                            ));
                        }
                    }
                }
                self.phase = if time >= p.hang_seconds {
                    PlumbPhase::Reel
                } else {
                    PlumbPhase::Hang(time)
                };
            }
            PlumbPhase::Reel => {
                pos.y += p.reel_speed * ctx.dt;
                out.despawn = pos.y > HALF_H + 8.0;
            }
        }
        out
    }
}

// --- Pitcher ---------------------------------------------------------------

/// The thrower above: while the Hero is below it and in range, it winds up
/// and lobs a canister that lands where the Hero stood.
#[derive(Clone, Debug, PartialEq)]
pub struct Pitcher {
    /// Seconds to the next throw (its wind-up is the last part of this).
    pub timer: f32,
    pub winding: bool,
}

impl Pitcher {
    pub fn new(t: &EnemyTypes) -> Self {
        Self {
            timer: t.pitcher.fire_interval,
            winding: false,
        }
    }

    /// The launch velocity, at most `speed`, of a lob from `from` that
    /// lands as near `to` as it can under `gravity`.
    pub fn lob(from: Vec2, to: Vec2, speed: f32, gravity: f32) -> Vec2 {
        let d = to - from;
        let mut best = Vec2::ZERO;
        let mut best_miss = f32::MAX;
        // Each flight time gives one exact launch; take the quickest that
        // stays within the speed, else the one that lands nearest.
        for i in 1..=60 {
            let time = i as f32 * 0.05;
            let v = Vec2::new(d.x / time, (d.y + 0.5 * gravity * time * time) / time);
            if v.length() <= speed {
                return v;
            }
            let capped = v.clamp_length_max(speed);
            // Where the capped lob is when it comes back down to `to`'s height.
            let (a, b, c) = (-0.5 * gravity, capped.y, -d.y);
            let disc = b * b - 4.0 * a * c;
            if disc < 0.0 {
                continue;
            }
            let land = (-b - disc.sqrt()) / (2.0 * a);
            let miss = (capped.x * land - d.x).abs();
            if miss < best_miss {
                best_miss = miss;
                best = capped;
            }
        }
        best
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let p = &t.pitcher;
        let mut out = Out::default();
        let target = ctx
            .hero
            .filter(|h| h.y < pos.y - 0.5 && (h.x - pos.x).abs() <= p.range);
        let Some(target) = target else {
            // Out of range: it waits, ready to wind up.
            self.timer = self.timer.max(p.telegraph_seconds);
            self.winding = false;
            return out;
        };
        self.timer -= ctx.dt;
        let winding = self.timer <= p.telegraph_seconds;
        out.warned = winding && !self.winding;
        self.winding = winding;
        if self.timer <= 0.0 {
            self.timer += p.fire_interval.max(0.1);
            self.winding = false;
            let from = Vec2::new(0.0, r.chest_height);
            let v = Self::lob(*pos + from, target, p.launch_speed, p.canister_gravity);
            out.canister = Some((from, v));
        }
        out
    }
}

/// A thrown canister, in tower space: falls under gravity, through thin
/// ledges, and bursts on the first solid one.
#[derive(Clone, Debug, PartialEq)]
pub struct Canister {
    pub vel: Vec2,
    pub gravity: f32,
}

impl Canister {
    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx) -> Out {
        self.vel.y -= self.gravity * ctx.dt;
        *pos += self.vel * ctx.dt;
        let bottom = pos.y - 4.0;
        Out {
            shattered: self.vel.y < 0.0 && ctx.tower.in_solid(pos.x, bottom),
            despawn: screen_y(pos.y, ctx.camera) < -HALF_H - 16.0,
            ..Out::default()
        }
    }
}

// --- Mote ------------------------------------------------------------------

/// The swarm (shared with Cabinets 1–3): homes slowly, then dissolves.
#[derive(Clone, Debug, PartialEq)]
pub struct Mote {
    pub heading: f32,
    age: f32,
}

impl Default for Mote {
    fn default() -> Self {
        Self {
            heading: -FRAC_PI_2,
            age: 0.0,
        }
    }
}

impl Mote {
    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let m = &t.mote;
        self.age += ctx.dt;
        let target = ctx.hero_on_screen().map(|h| h + Vec2::new(0.0, 10.0));
        home(&mut self.heading, pos, target, m.speed, m.turn_rate, ctx.dt);
        Out {
            despawn: self.age >= m.lifetime,
            ..Out::default()
        }
    }
}

// --- Winch -----------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum WinchPhase {
    /// The carriage slides to above the Hero: seconds left.
    Slide(f32),
    /// The cable shakes and the core glows: seconds left.
    Shake(f32),
    Drop,
    /// On the floor: seconds left.
    Rest(f32),
    Reel,
}

/// The set piece: a carriage on a rail with a heavy hook on a cable. Its
/// `pos` is the hook's centre on screen; the carriage is at
/// (`carriage_x`, `rail_y`).
#[derive(Clone, Debug, PartialEq)]
pub struct Winch {
    pub phase: WinchPhase,
    pub carriage_x: f32,
    /// The floor (a landing's top) on screen.
    pub floor: f32,
}

/// Where the hook hangs under the carriage (its centre, below the rail).
pub const HOOK_HANG: f32 = 24.0;

impl Winch {
    pub fn new(x: f32, floor: f32) -> Self {
        Self {
            phase: WinchPhase::Slide(1.4),
            carriage_x: x,
            floor,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes, r: &EnemyRules) -> Out {
        let w = &t.winch;
        let mut out = Out::default();
        let hang = w.rail_y - HOOK_HANG;
        self.phase = match self.phase {
            WinchPhase::Slide(left) => {
                let target = ctx.hero.map_or(self.carriage_x, |h| h.x);
                let step = w.speed * ctx.dt;
                let dx = (target - self.carriage_x).clamp(-step, step);
                self.carriage_x += dx;
                let there = (target - self.carriage_x).abs() < 0.5;
                if there || left - ctx.dt <= 0.0 {
                    out.warned = true;
                    WinchPhase::Shake(w.telegraph_seconds)
                } else {
                    WinchPhase::Slide(left - ctx.dt)
                }
            }
            WinchPhase::Shake(left) if left - ctx.dt <= 0.0 => WinchPhase::Drop,
            WinchPhase::Shake(left) => WinchPhase::Shake(left - ctx.dt),
            WinchPhase::Drop => {
                pos.y -= w.drop_speed * ctx.dt;
                let bottom = self.floor + 8.0;
                if pos.y <= bottom {
                    pos.y = bottom;
                    // Two shots along the floor, one each way, at the ankle.
                    for side in [-1.0, 1.0] {
                        out.shots.push(Shot::new(
                            Vec2::new(side * 9.0, r.ankle_height - 8.0),
                            Vec2::new(side * w.burst_speed, 0.0),
                        ));
                    }
                    WinchPhase::Rest(w.rest_seconds)
                } else {
                    WinchPhase::Drop
                }
            }
            WinchPhase::Rest(left) if left - ctx.dt <= 0.0 => WinchPhase::Reel,
            WinchPhase::Rest(left) => WinchPhase::Rest(left - ctx.dt),
            WinchPhase::Reel => {
                pos.y += w.reel_speed * ctx.dt;
                if pos.y >= hang {
                    pos.y = hang;
                    WinchPhase::Slide(1.4)
                } else {
                    WinchPhase::Reel
                }
            }
        };
        pos.x = self.carriage_x;
        out
    }
}

#[cfg(test)]
mod tests;
