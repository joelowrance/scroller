//! How each Cabinet 2 enemy moves and fires, as plain state machines with no
//! Bevy systems, so the paths can be tested at fixed time steps.
//!
//! Every frame a [`Brain`] is stepped with the enemy's position and a
//! [`Ctx`]; it moves the position and returns an [`Out`] saying what it
//! wants to fire, launch or mark. The ECS side (`mod.rs`) applies the
//! fairness rules before anything is fired. The screen scrolls upward, so
//! anything fixed to the ground moves *down* at the scroll speed.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::math::Vec2;

use crate::cabinets::shooter::pattern::{Pulse, bob, dir, home, ring, turn_toward};
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::cabinets::vertical_shooter::tuning::EnemyTypes;
use crate::display::{HALF_H, HALF_W};

/// y where a Swift waits during its warning, just below the bottom edge.
pub const SWIFT_WAIT_Y: f32 = -HALF_H - 12.0;

/// What an enemy can see this frame.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    pub dt: f32,
    /// The Hero's position, if there is one.
    pub hero: Option<Vec2>,
    /// Stage scroll speed, px/s. The ground moves down at it.
    pub scroll: f32,
}

/// What a brain asks for this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    /// Bullet velocities from the enemy's position, unscaled (the Stage 2
    /// multiplier comes later). The fairness rules may skip them all.
    pub shots: Vec<Vec2>,
    /// Things to launch from this enemy (Wisps from a Loom shuttle, a pod
    /// from a Riser launcher).
    pub launch: u32,
    /// A point to mark for a flak burst (Flak Battery).
    pub mark: Option<Vec2>,
    /// The enemy is done and goes without points (a Mote dissolving, a pod
    /// bursting).
    pub despawn: bool,
}

/// Whether a point is on screen.
pub fn on_screen(at: Vec2) -> bool {
    at.x.abs() <= HALF_W && at.y.abs() <= HALF_H
}

/// Aimed at the Hero (straight down without one).
fn aim(from: Vec2, hero: Option<Vec2>) -> Vec2 {
    hero.map_or(Vec2::NEG_Y, |h| (h - from).normalize_or(Vec2::NEG_Y))
}

#[derive(Clone, Debug, PartialEq)]
pub enum Brain {
    Wisp(Wisp),
    Swift(Swift),
    Bellows(Bellows),
    LoomCore(LoomCore),
    LoomShuttle(LoomShuttle),
    Crawler(Crawler),
    Silo(Silo),
    RiserLauncher(RiserLauncher),
    RiserPod(RiserPod),
    Flak(Flak),
    Mote(Mote),
    /// Moves with the ground and does nothing else (Tether balloons).
    Drift,
}

impl Brain {
    pub fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        match self {
            Brain::Wisp(b) => b.step(pos, ctx, t),
            Brain::Swift(b) => b.step(pos, ctx, t),
            Brain::Bellows(b) => b.step(pos, ctx, t),
            Brain::LoomCore(b) => b.step(pos, ctx, t),
            Brain::LoomShuttle(b) => b.step(*pos, ctx, t),
            Brain::Crawler(b) => b.step(pos, ctx, t),
            Brain::Silo(b) => b.step(pos, ctx, t),
            Brain::RiserLauncher(b) => b.step(pos, ctx, t),
            Brain::RiserPod(b) => b.step(pos, ctx, t),
            Brain::Flak(b) => b.step(pos, ctx, t),
            Brain::Mote(b) => b.step(pos, ctx, t),
            Brain::Drift => {
                pos.y -= ctx.scroll * ctx.dt;
                Out::default()
            }
        }
    }

    /// True while the enemy is flashing a warning before it fires.
    pub fn telegraph(&self, t: &EnemyTypes) -> bool {
        match self {
            Brain::Bellows(b) => b.telegraph(t),
            Brain::LoomShuttle(b) => b.telegraph(t),
            Brain::Crawler(b) => !b.silent && b.timer < t.crawler.telegraph_seconds,
            Brain::Silo(b) => b.telegraph(t),
            Brain::RiserPod(b) => b.telegraph(t),
            _ => false,
        }
    }
}

// --- Wisp ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum WispPhase {
    Fall,
    /// Curving toward the Hero's side; `target` is 0 (right) or π (left).
    Curve {
        target: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wisp {
    /// Launched by a Loom shuttle: falls straight down, no curve.
    pub launched: bool,
    pub phase: WispPhase,
    /// Radians.
    pub heading: f32,
}

impl Wisp {
    pub fn new() -> Self {
        Self {
            launched: false,
            phase: WispPhase::Fall,
            heading: -FRAC_PI_2,
        }
    }

    pub fn launched() -> Self {
        Self {
            launched: true,
            ..Self::new()
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let w = &t.wisp;
        if self.launched {
            pos.y -= w.launched_speed * ctx.dt;
            return Out::default();
        }
        match self.phase {
            WispPhase::Fall => {
                pos.y -= w.speed * ctx.dt;
                if pos.y <= w.turn_y {
                    let hero_x = ctx.hero.map_or(-pos.x, |h| h.x);
                    let target = if hero_x >= pos.x { 0.0 } else { PI };
                    self.phase = WispPhase::Curve { target };
                }
            }
            WispPhase::Curve { target } => {
                let max = w.turn_rate.to_radians() * ctx.dt;
                self.heading = turn_toward(self.heading, target, max);
                *pos += Vec2::from_angle(self.heading) * w.exit_speed * ctx.dt;
            }
        }
        Out::default()
    }
}

impl Default for Wisp {
    fn default() -> Self {
        Self::new()
    }
}

// --- Swift -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum SwiftPhase {
    /// Waiting below the bottom edge while the chevron blinks.
    Warn {
        left: f32,
    },
    Climb,
    /// Half-circle U-turn; `angle` runs 0 to π.
    Turn {
        angle: f32,
        centre: Vec2,
        side: f32,
    },
    Dive {
        column: f32,
    },
}

/// Cabinet 1's Hook turned 90°: up the screen from behind, a U-turn, then a
/// dive that slides onto the Hero's column.
#[derive(Clone, Debug, PartialEq)]
pub struct Swift {
    /// The column it climbs along.
    pub column: f32,
    pub phase: SwiftPhase,
}

impl Swift {
    /// A Swift climbing `offset` px right (positive) or left of the Hero,
    /// kept clear of the side edges.
    pub fn new(hero_x: f32, offset: f32, t: &EnemyTypes) -> Self {
        Self {
            column: clamp_column(hero_x + offset, t),
            phase: SwiftPhase::Warn {
                left: t.swift.warning_seconds,
            },
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let s = &t.swift;
        let r = s.turn_radius.max(1.0);
        match &mut self.phase {
            SwiftPhase::Warn { left } => {
                *pos = Vec2::new(self.column, SWIFT_WAIT_Y);
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.phase = SwiftPhase::Climb;
                }
            }
            SwiftPhase::Climb => {
                pos.x = self.column;
                pos.y += s.speed * ctx.dt;
                if pos.y >= s.turn_y {
                    let hero_x = ctx.hero.map_or(0.0, |h| h.x);
                    let side = if hero_x >= self.column { 1.0 } else { -1.0 };
                    self.phase = SwiftPhase::Turn {
                        angle: 0.0,
                        centre: Vec2::new(self.column + side * r, s.turn_y),
                        side,
                    };
                }
            }
            SwiftPhase::Turn {
                angle,
                centre,
                side,
            } => {
                *angle = (*angle + s.speed / r * ctx.dt).min(PI);
                *pos = *centre + Vec2::new(-*side * angle.cos(), angle.sin()) * r;
                if *angle >= PI {
                    let column = clamp_column(ctx.hero.map_or(pos.x, |h| h.x), t);
                    self.phase = SwiftPhase::Dive { column };
                }
            }
            SwiftPhase::Dive { column } => {
                pos.y -= s.dive_speed * ctx.dt;
                let max = s.dive_steer_speed * ctx.dt;
                pos.x += (*column - pos.x).clamp(-max, max);
            }
        }
        Out::default()
    }
}

fn clamp_column(x: f32, t: &EnemyTypes) -> f32 {
    let c = t.swift.edge_clearance;
    x.clamp(-HALF_W + c, HALF_W - c)
}

// --- Bellows ---------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum BellowsPhase {
    Enter,
    Hover { t: f32, fired: u32 },
    Exit,
}

/// The gunner: glides in from a side edge, hovers, fires two aimed bursts,
/// then climbs away.
#[derive(Clone, Debug, PartialEq)]
pub struct Bellows {
    /// -1 enters from the left edge, +1 from the right.
    pub side: f32,
    pub phase: BellowsPhase,
}

impl Bellows {
    pub fn new(side: f32) -> Self {
        Self {
            side: if side < 0.0 { -1.0 } else { 1.0 },
            phase: BellowsPhase::Enter,
        }
    }

    /// x where it enters, just off its edge.
    pub fn entry_x(side: f32) -> f32 {
        side * (HALF_W + 12.0)
    }

    fn stop_x(&self, t: &EnemyTypes) -> f32 {
        self.side * (HALF_W - t.bellows.stop_inset)
    }

    /// Seconds into the hover at which shot `n` (counting every burst) fires.
    fn shot_time(n: u32, t: &EnemyTypes) -> f32 {
        let b = &t.bellows;
        let per = b.burst.max(1);
        b.telegraph_seconds
            + (n / per) as f32 * b.fire_interval
            + (n % per) as f32 * b.burst_interval
    }

    fn total_shots(t: &EnemyTypes) -> u32 {
        t.bellows.burst.max(1) * t.bellows.bursts
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        let b = &t.bellows;
        match self.phase {
            BellowsPhase::Hover { t: now, fired } if fired < Self::total_shots(t) => {
                // Glows before each burst, through the burst itself.
                let burst_start = Self::shot_time(fired - fired % b.burst.max(1), t);
                now >= burst_start - b.telegraph_seconds
            }
            _ => false,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let b = &t.bellows;
        let mut out = Out::default();
        let stop_x = self.stop_x(t);
        match &mut self.phase {
            BellowsPhase::Enter => {
                let left = (pos.x - stop_x).abs();
                let floor = b.brake_floor.clamp(0.01, 1.0);
                let brake = (left / b.brake_distance.max(1.0)).sqrt().clamp(floor, 1.0);
                let step = b.speed * brake * ctx.dt;
                if step >= left {
                    pos.x = stop_x;
                    self.phase = BellowsPhase::Hover { t: 0.0, fired: 0 };
                } else {
                    pos.x -= self.side * step;
                }
            }
            BellowsPhase::Hover { t: now, fired } => {
                *now += ctx.dt;
                pos.x = stop_x + bob(b.sway_amplitude, b.sway_period, *now);
                while *fired < Self::total_shots(t) && *now >= Self::shot_time(*fired, t) {
                    out.shots.push(aim(*pos, ctx.hero) * b.shot_speed);
                    *fired += 1;
                }
                if *now >= b.hover_seconds {
                    self.phase = BellowsPhase::Exit;
                }
            }
            BellowsPhase::Exit => pos.y += b.exit_speed * ctx.dt,
        }
        out
    }
}

// --- Loom ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum LoomPhase {
    Enter,
    Hold { t: f32 },
    Retreat,
}

/// The Loom's core: it moves the whole Loom (the shuttles dock to it) and,
/// once both shuttles are gone, spirals.
#[derive(Clone, Debug, PartialEq)]
pub struct LoomCore {
    pub phase: LoomPhase,
    /// Set by the ECS once both shuttles are destroyed.
    pub exposed: bool,
    /// Seconds since the shield dropped.
    spiral_clock: f32,
    /// Spiral bullets fired in the current on-time.
    spiral_fired: u32,
    /// Direction of arm 0, degrees.
    pub spiral_angle: f32,
}

impl LoomCore {
    pub fn new() -> Self {
        Self {
            phase: LoomPhase::Enter,
            exposed: false,
            spiral_clock: 0.0,
            spiral_fired: 0,
            spiral_angle: -90.0,
        }
    }

    pub fn retreating(&self) -> bool {
        self.phase == LoomPhase::Retreat
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let l = &t.loom;
        let mut out = Out::default();
        match &mut self.phase {
            LoomPhase::Enter => {
                pos.y = (pos.y - l.speed * ctx.dt).max(l.hold_y);
                if pos.y <= l.hold_y {
                    self.phase = LoomPhase::Hold { t: 0.0 };
                }
            }
            LoomPhase::Hold { t: held } => {
                *held += ctx.dt;
                if *held >= l.hold_seconds {
                    self.phase = LoomPhase::Retreat;
                }
            }
            LoomPhase::Retreat => pos.y += l.retreat_speed * ctx.dt,
        }
        if !self.exposed || self.retreating() {
            return out;
        }
        self.spiral_angle += l.spiral_turn_rate * ctx.dt;
        self.spiral_clock += ctx.dt;
        let cycle = (l.spiral_seconds + l.spiral_pause).max(0.01);
        let into = self.spiral_clock.rem_euclid(cycle);
        if into < ctx.dt {
            // A new on-time begins.
            self.spiral_fired = 0;
        }
        if into < l.spiral_seconds {
            let interval = l.spiral_interval.max(0.01);
            while self.spiral_fired as f32 * interval <= into {
                out.shots.extend(
                    ring(l.spiral_arms, self.spiral_angle)
                        .into_iter()
                        .map(|d| d * l.spiral_speed),
                );
                self.spiral_fired += 1;
            }
        }
        out
    }
}

impl Default for LoomCore {
    fn default() -> Self {
        Self::new()
    }
}

/// A shuttle docked on a Loom's flank: it launches Wisps and fires aimed
/// pairs. The ECS keeps it docked to its core.
#[derive(Clone, Debug, PartialEq)]
pub struct LoomShuttle {
    /// -1 on the left flank, +1 on the right.
    pub side: f32,
    age: f32,
    next_launch: f32,
    /// When the next pair starts.
    next_pair: f32,
    /// Shots of the current pair already fired.
    pair_fired: u32,
}

impl LoomShuttle {
    pub fn new(side: f32, t: &EnemyTypes) -> Self {
        let l = &t.loom;
        // The two shuttles alternate: the right one fires half a cycle later.
        let stagger = if side > 0.0 {
            l.fire_interval / 2.0
        } else {
            0.0
        };
        Self {
            side,
            age: 0.0,
            next_launch: l.first_launch_delay,
            next_pair: l.first_launch_delay + l.telegraph_seconds + stagger,
            pair_fired: 0,
        }
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        self.age >= self.next_pair - t.loom.telegraph_seconds
    }

    /// Steps the timers. A shuttle doesn't move itself: the ECS keeps it
    /// docked at `pos` beside its core.
    fn step(&mut self, pos: Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let l = &t.loom;
        let mut out = Out::default();
        self.age += ctx.dt;
        if self.age >= self.next_launch {
            self.next_launch += l.launch_interval.max(0.1);
            out.launch = 1;
        }
        let shot_at = self.next_pair + self.pair_fired as f32 * l.pair_interval;
        if self.age >= shot_at {
            out.shots.push(aim(pos, ctx.hero) * l.shot_speed);
            self.pair_fired += 1;
            if self.pair_fired >= 2 {
                self.pair_fired = 0;
                self.next_pair += l.fire_interval.max(0.1);
            }
        }
        out
    }
}

// --- Crawler ---------------------------------------------------------------

/// A treaded turret on the ground: drives along its road and fires along a
/// turret that lags behind the Hero.
#[derive(Clone, Debug, PartialEq)]
pub struct Crawler {
    /// Its road direction, degrees (0 = right).
    pub heading: f32,
    /// Turret direction, radians.
    pub turret: f32,
    /// Seconds to the next shot.
    pub timer: f32,
    /// The Hero is above it: it never fires again.
    pub silent: bool,
}

impl Crawler {
    /// Member `index` of a convoy; each fires `stagger` s after the last.
    pub fn new(heading: f32, index: u32, t: &EnemyTypes) -> Self {
        Self {
            heading,
            turret: -FRAC_PI_2,
            timer: t.crawler.fire_interval + index as f32 * t.crawler.stagger,
            silent: false,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let c = &t.crawler;
        *pos += dir(self.heading) * c.speed * ctx.dt;
        pos.y -= ctx.scroll * ctx.dt;
        let mut out = Out::default();
        let Some(hero) = ctx.hero else {
            return out;
        };
        if hero.y > pos.y {
            self.silent = true;
        }
        if self.silent {
            return out;
        }
        let want = (hero - *pos).to_angle();
        self.turret = turn_toward(self.turret, want, c.turn_rate.to_radians() * ctx.dt);
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            self.timer += c.fire_interval.max(0.1);
            out.shots.push(Vec2::from_angle(self.turret) * c.shot_speed);
        }
        out
    }
}

// --- Silo ------------------------------------------------------------------

/// An armoured hatch: closed and immune, then open, firing a ring.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Silo {
    /// Seconds on screen: its cycle starts when it scrolls into view.
    pub age: f32,
}

impl Silo {
    /// Closed, then the warning flash, then open (doc: 2.4 s, 0.3 s, 1.2 s).
    fn cycle(t: &EnemyTypes) -> f32 {
        let s = &t.silo;
        (s.closed_seconds + s.telegraph_seconds + s.open_seconds).max(0.01)
    }

    pub fn open(&self, t: &EnemyTypes) -> bool {
        let s = &t.silo;
        self.age > 0.0
            && self.age.rem_euclid(Self::cycle(t)) >= s.closed_seconds + s.telegraph_seconds
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        let s = &t.silo;
        let p = self.age.rem_euclid(Self::cycle(t));
        self.age > 0.0 && p >= s.closed_seconds && p < s.closed_seconds + s.telegraph_seconds
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        pos.y -= ctx.scroll * ctx.dt;
        let was_open = self.open(t);
        if on_screen(*pos) {
            self.age += ctx.dt;
        }
        let mut out = Out::default();
        if self.open(t) && !was_open {
            out.shots = ring(t.silo.ring_count, -90.0)
                .into_iter()
                .map(|d| d * t.silo.shot_speed)
                .collect();
        }
        out
    }
}

// --- Riser -----------------------------------------------------------------

/// The launch pad: throws a pod into the air every few seconds while it is
/// above the Hero.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiserLauncher {
    /// Seconds to the next launch, once it is fully on screen.
    pub timer: Option<f32>,
}

impl RiserLauncher {
    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let r = &t.riser;
        pos.y -= ctx.scroll * ctx.dt;
        let mut out = Out::default();
        // "Fully on screen": its top edge (14 px tall) is below the HUD.
        if self.timer.is_none() && pos.y + 7.0 <= PLAY_TOP {
            self.timer = Some(r.first_launch_delay);
        }
        let Some(timer) = &mut self.timer else {
            return out;
        };
        *timer -= ctx.dt;
        if *timer <= 0.0 {
            *timer += r.launch_interval.max(0.1);
            if ctx.hero.is_none_or(|h| pos.y > h.y) {
                out.launch = 1;
            }
        }
        out
    }
}

/// A pod: on the ground while it climbs, then airborne, drifting toward the
/// Hero until its fuse runs out and it bursts.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RiserPod {
    pub age: f32,
}

impl RiserPod {
    pub fn airborne(&self, t: &EnemyTypes) -> bool {
        self.age >= t.riser.climb_seconds
    }

    /// 0 on the pad, 1 once airborne: how far its shadow has slid out.
    pub fn climb(&self, t: &EnemyTypes) -> f32 {
        (self.age / t.riser.climb_seconds.max(0.01)).min(1.0)
    }

    fn fuse_end(t: &EnemyTypes) -> f32 {
        let r = &t.riser;
        r.climb_seconds + r.fuse_seconds
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        self.age >= Self::fuse_end(t)
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let r = &t.riser;
        self.age += ctx.dt;
        pos.y -= ctx.scroll * ctx.dt;
        let mut out = Out::default();
        if !self.airborne(t) {
            return out;
        }
        if let Some(hero) = ctx.hero {
            let max = r.drift_speed * ctx.dt;
            pos.x += (hero.x - pos.x).clamp(-max, max);
        }
        if self.age >= Self::fuse_end(t) + r.telegraph_seconds {
            out.despawn = true;
            out.shots = ring(r.ring_count, -90.0)
                .into_iter()
                .map(|d| d * r.shot_speed)
                .collect();
        }
        out
    }
}

// --- Flak Battery ----------------------------------------------------------

/// A ground gun that marks where the Hero is; the burst follows later.
#[derive(Clone, Debug, PartialEq)]
pub struct Flak {
    /// Seconds to the next mark.
    pub timer: f32,
}

impl Flak {
    pub fn new(t: &EnemyTypes) -> Self {
        // The first mark comes half an interval after it appears.
        Self {
            timer: t.flak.fire_interval / 2.0,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        pos.y -= ctx.scroll * ctx.dt;
        let mut out = Out::default();
        let Some(hero) = ctx.hero else {
            return out;
        };
        if !on_screen(*pos) || hero.y > pos.y {
            return out;
        }
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            self.timer += t.flak.fire_interval.max(0.1);
            out.mark = Some(hero);
        }
        out
    }
}

// --- Mote ------------------------------------------------------------------

/// The swarm, shared in spirit with Cabinet 1: homes slowly, then dissolves.
#[derive(Clone, Debug, PartialEq)]
pub struct Mote {
    pub heading: f32,
    age: f32,
}

impl Mote {
    pub fn new() -> Self {
        Self {
            heading: -FRAC_PI_2,
            age: 0.0,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let m = &t.mote;
        self.age += ctx.dt;
        home(
            &mut self.heading,
            pos,
            ctx.hero,
            m.speed,
            m.turn_rate,
            ctx.dt,
        );
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

// --- Tether ----------------------------------------------------------------

/// The line between two Tether balloons: off, a flicker, then live.
pub fn tether_pulse(t: &EnemyTypes) -> Pulse {
    let x = &t.tether;
    Pulse {
        on_seconds: x.on_seconds,
        off_seconds: x.off_seconds,
        telegraph_seconds: x.telegraph_seconds,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 60.0;

    fn types() -> EnemyTypes {
        EnemyTypes::default()
    }

    fn ctx(hero: Option<Vec2>) -> Ctx {
        Ctx {
            dt: DT,
            hero,
            scroll: 24.0,
        }
    }

    /// Steps a brain for `seconds`, returning every Out.
    fn run(brain: &mut Brain, pos: &mut Vec2, hero: Option<Vec2>, seconds: f32) -> Vec<Out> {
        let t = types();
        let frames = (seconds / DT).round() as usize;
        (0..frames)
            .map(|_| brain.step(pos, &ctx(hero), &t))
            .collect()
    }

    fn shots(outs: &[Out]) -> usize {
        outs.iter().map(|o| o.shots.len()).sum()
    }

    fn close(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn a_wisp_falls_then_curves_toward_the_heros_side() {
        let mut b = Brain::Wisp(Wisp::new());
        let mut pos = Vec2::new(0.0, 124.0);
        let hero = Some(Vec2::new(-80.0, -80.0));
        // 104 px at 80 px/s to the turn row.
        run(&mut b, &mut pos, hero, 1.3);
        assert!(close(pos.y, 20.0, 1.5), "at the turn row: {pos}");
        let outs = run(&mut b, &mut pos, hero, 3.0);
        assert_eq!(shots(&outs), 0, "it never fires");
        assert!(pos.x < -HALF_W, "left through the Hero's side: {pos}");
        assert!(pos.y > -60.0, "it curved away rather than falling on");
    }

    #[test]
    fn a_launched_wisp_falls_straight() {
        let mut b = Brain::Wisp(Wisp::launched());
        let mut pos = Vec2::new(30.0, 60.0);
        run(&mut b, &mut pos, Some(Vec2::new(-100.0, -80.0)), 1.0);
        assert!(
            close(pos.x, 30.0, 0.01) && close(pos.y, -40.0, 0.5),
            "{pos}"
        );
    }

    #[test]
    fn a_swift_warns_climbs_turns_and_dives_onto_the_heros_column() {
        let t = types();
        let hero = Some(Vec2::new(0.0, -80.0));
        let mut b = Brain::Swift(Swift::new(0.0, 50.0, &t));
        let mut pos = Vec2::ZERO;
        run(&mut b, &mut pos, hero, 0.7);
        assert_eq!(
            pos,
            Vec2::new(50.0, SWIFT_WAIT_Y),
            "waits below during the warning"
        );
        run(&mut b, &mut pos, hero, 1.5);
        assert!(pos.y > 50.0, "climbed past the Hero: {pos}");
        let outs = run(&mut b, &mut pos, hero, 2.0);
        assert_eq!(shots(&outs), 0, "it rams, no bullets");
        assert!(close(pos.x, 0.0, 0.5), "slid onto the Hero's column: {pos}");
        assert!(pos.y < 0.0, "diving: {pos}");
    }

    #[test]
    fn swift_columns_stay_inside_the_edges() {
        let t = types();
        assert_eq!(Swift::new(150.0, 50.0, &t).column, HALF_W - 12.0);
        assert_eq!(Swift::new(-150.0, -50.0, &t).column, -HALF_W + 12.0);
    }

    #[test]
    fn a_bellows_glides_in_from_its_side_and_fires_two_bursts_of_three() {
        let t = types();
        let mut b = Brain::Bellows(Bellows::new(1.0));
        let mut pos = Vec2::new(Bellows::entry_x(1.0), 60.0);
        let hero = Some(Vec2::new(0.0, -80.0));
        let mut stopped_at = None;
        let mut first_shot_at = None;
        let mut fired = Vec::new();
        for frame in 0..(4.5 / DT) as usize {
            let out = b.step(&mut pos, &ctx(hero), &t);
            let now = frame as f32 * DT;
            if stopped_at.is_none()
                && matches!(
                    b,
                    Brain::Bellows(Bellows {
                        phase: BellowsPhase::Hover { .. },
                        ..
                    })
                )
            {
                stopped_at = Some(now);
                assert!(b.telegraph(&t), "glows as soon as it stops");
            }
            if !out.shots.is_empty() {
                first_shot_at.get_or_insert(now);
                fired.extend(out.shots);
            }
        }
        let stopped_at = stopped_at.expect("it stops");
        assert!(
            close(stopped_at, 1.0, 0.25),
            "stops about 1 s in: {stopped_at}"
        );
        assert!(close(first_shot_at.unwrap() - stopped_at, 0.4, 0.05));
        assert_eq!(fired.len(), 6);
        assert!(
            fired
                .iter()
                .all(|s| close(s.length(), 90.0, 0.01) && s.y < 0.0)
        );
        assert!(pos.y > 60.0, "climbing out after the hover");
    }

    #[test]
    fn a_loom_core_holds_and_only_spirals_once_exposed() {
        let t = types();
        let mut core = LoomCore::new();
        let mut pos = Vec2::new(0.0, 142.0);
        let c = ctx(Some(Vec2::new(0.0, -30.0)));
        let mut shots_shielded = 0;
        for _ in 0..(6.0 / DT) as usize {
            shots_shielded += core.step(&mut pos, &c, &t).shots.len();
        }
        assert_eq!(pos.y, t.loom.hold_y, "holding at y = +60");
        assert_eq!(shots_shielded, 0);
        core.exposed = true;
        let mut on = 0;
        for _ in 0..(2.0 / DT) as usize {
            on += core.step(&mut pos, &c, &t).shots.len();
        }
        // 2.0 s of spiral at one bullet per arm every 0.3 s: 7 volleys of 3.
        assert_eq!(on, 21);
        let mut paused = 0;
        for _ in 0..(1.9 / DT) as usize {
            paused += core.step(&mut pos, &c, &t).shots.len();
        }
        assert_eq!(paused, 0, "the pause is the window");
    }

    #[test]
    fn a_loom_retreats_after_its_hold() {
        let t = types();
        let mut core = LoomCore::new();
        let mut pos = Vec2::new(0.0, 142.0);
        let c = ctx(None);
        for _ in 0..(30.0 / DT) as usize {
            core.step(&mut pos, &c, &t);
        }
        assert!(core.retreating() && pos.y > t.loom.hold_y);
    }

    #[test]
    fn loom_shuttles_alternate_aimed_pairs_and_launch_wisps() {
        let t = types();
        let mut left = LoomShuttle::new(-1.0, &t);
        let mut right = LoomShuttle::new(1.0, &t);
        let c = ctx(Some(Vec2::new(0.0, -80.0)));
        let (mut lt, mut rt) = (Vec::new(), Vec::new());
        let mut launches = 0;
        for frame in 0..(6.0 / DT) as usize {
            let now = frame as f32 * DT;
            let lo = left.step(Vec2::new(-28.0, 60.0), &c, &t);
            let ro = right.step(Vec2::new(28.0, 60.0), &c, &t);
            launches += lo.launch + ro.launch;
            assert!(
                lo.shots
                    .iter()
                    .all(|s| s.y < 0.0 && close(s.length(), 80.0, 0.01))
            );
            lt.extend(lo.shots.iter().map(|_| now));
            rt.extend(ro.shots.iter().map(|_| now));
        }
        assert_eq!(launches, 4, "each shuttle launches at 1 s and 4 s");
        assert_eq!(lt.len() % 2, 0, "pairs");
        assert!(close(lt[1] - lt[0], 0.15, 0.02));
        assert!(
            close(rt[0] - lt[0], t.loom.fire_interval / 2.0, 0.02),
            "they alternate"
        );
    }

    #[test]
    fn a_crawler_fires_along_its_turret_until_the_hero_is_above_it() {
        let t = types();
        let mut b = Brain::Crawler(Crawler::new(0.0, 0, &t));
        let mut pos = Vec2::new(-40.0, 100.0);
        let hero = Some(Vec2::new(100.0, -80.0));
        let outs = run(&mut b, &mut pos, hero, 2.1);
        assert_eq!(shots(&outs), 1);
        let s = outs.iter().find_map(|o| o.shots.first().copied()).unwrap();
        assert!(s.x > 0.0 && s.y < 0.0, "toward the Hero, down-right: {s}");
        assert!(
            close(pos.x, -40.0 + 12.0 * 2.1, 0.5),
            "drives along its road"
        );
        assert!(
            close(pos.y, 100.0 - 24.0 * 2.1, 0.5),
            "and scrolls with the ground"
        );
        let above = Some(Vec2::new(0.0, pos.y + 10.0));
        let outs = run(&mut b, &mut pos, above, 5.0);
        assert_eq!(shots(&outs), 0);
        assert!(!b.telegraph(&t));
    }

    #[test]
    fn convoy_members_fire_a_ripple() {
        let t = types();
        assert!(close(
            Crawler::new(0.0, 2, &t).timer - Crawler::new(0.0, 0, &t).timer,
            0.8,
            1e-5
        ));
    }

    #[test]
    fn a_silo_starts_its_cycle_on_screen_and_rings_as_it_opens() {
        let t = types();
        let mut b = Brain::Silo(Silo::default());
        let mut pos = Vec2::new(0.0, 140.0);
        // Off screen for the first 20 px: the cycle hasn't started.
        run(&mut b, &mut pos, None, 0.8);
        let Brain::Silo(silo) = &b else {
            unreachable!()
        };
        assert_eq!(silo.age, 0.0);
        let outs = run(&mut b, &mut pos, None, 3.0);
        assert_eq!(shots(&outs), 6, "one ring as it opens");
        let Brain::Silo(silo) = &b else {
            unreachable!()
        };
        assert!(silo.open(&t));
    }

    #[test]
    fn a_riser_pod_climbs_on_the_ground_then_drifts_and_bursts() {
        let t = types();
        let mut pod = RiserPod::default();
        let mut pos = Vec2::new(0.0, 60.0);
        let c = ctx(Some(Vec2::new(-100.0, -80.0)));
        for _ in 0..(0.9 / DT) as usize {
            assert!(pod.step(&mut pos, &c, &t).shots.is_empty());
        }
        assert!(!pod.airborne(&t) && pos.x == 0.0, "still on the pad");
        let mut burst = None;
        for frame in 0..(2.0 / DT) as usize {
            let out = pod.step(&mut pos, &c, &t);
            if out.despawn {
                burst = Some((frame, out.shots.len()));
                break;
            }
        }
        let (_, n) = burst.expect("it bursts");
        assert_eq!(n, 6);
        assert!(pos.x < -30.0, "drifted toward the Hero: {pos}");
        // Bursts at climb + fuse + telegraph = 2.5 s.
        assert!(close(pod.age, 2.5, 0.05));
    }

    #[test]
    fn a_riser_launches_only_while_above_the_hero() {
        let mut b = Brain::RiserLauncher(RiserLauncher::default());
        let mut pos = Vec2::new(0.0, 100.0);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, -80.0)), 4.6);
        let launches: u32 = outs.iter().map(|o| o.launch).sum();
        assert_eq!(launches, 2, "at 1.0 s and 4.5 s");
        let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, 100.0)), 8.0);
        assert_eq!(outs.iter().map(|o| o.launch).sum::<u32>(), 0);
    }

    #[test]
    fn a_flak_battery_marks_the_heros_position_every_interval() {
        let t = types();
        let mut b = Brain::Flak(Flak::new(&t));
        let mut pos = Vec2::new(0.0, 100.0);
        let hero = Vec2::new(30.0, -80.0);
        let outs = run(&mut b, &mut pos, Some(hero), 3.7);
        let marks: Vec<_> = outs.iter().filter_map(|o| o.mark).collect();
        assert_eq!(marks, [hero, hero], "at 1.2 s and 3.6 s");
        assert_eq!(shots(&outs), 0, "the burst is not its own shot");
    }

    #[test]
    fn a_mote_homes_then_dissolves() {
        let mut b = Brain::Mote(Mote::new());
        let mut pos = Vec2::new(0.0, 100.0);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, -80.0)), 8.1);
        assert!(outs.last().unwrap().despawn);
        assert!(pos.y < 100.0);
    }

    #[test]
    fn ground_things_scroll_down() {
        let mut b = Brain::Drift;
        let mut pos = Vec2::new(-140.0, 100.0);
        run(&mut b, &mut pos, None, 1.0);
        assert!(close(pos.y, 76.0, 0.1));
    }
}
