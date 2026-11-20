//! How each enemy moves and fires, as plain state machines with no Bevy
//! systems, so the paths can be tested at fixed time steps.
//!
//! Every frame a [`Brain`] is stepped with the enemy's position and a
//! [`Ctx`]; it moves the position and returns an [`Out`] saying what it
//! wants to fire or launch. The ECS side (`mod.rs`) applies the fairness
//! rules (grace period, minimum distance, bullet cap) before anything is
//! actually fired.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec2;

use crate::cabinets::side_shooter::tuning::EnemyTypes;
use crate::display::{HALF_H, HALF_W};

/// Top of the play area: the HUD uses the top 12 px.
pub const PLAY_TOP: f32 = HALF_H - 12.0;
pub const PLAY_BOTTOM: f32 = -HALF_H;

/// What an enemy can see this frame.
#[derive(Clone, Copy, Debug)]
pub struct Ctx {
    pub dt: f32,
    /// The Hero's position, if there is one.
    pub hero: Option<Vec2>,
    /// Stage scroll speed, px/s. Things fixed to the scenery move left at it.
    pub scroll: f32,
}

impl Ctx {
    /// The Hero, but only while it is still ahead of (to the left of) `pos`.
    /// Aimed shots are never fired backward.
    fn hero_ahead_of(&self, pos: Vec2) -> Option<Vec2> {
        self.hero.filter(|h| h.x < pos.x)
    }
}

/// What a brain asks for this frame.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Out {
    /// Bullet velocities, unscaled (the Stage 2 multiplier comes later).
    pub shots: Vec<Vec2>,
    /// Skiffs to launch from this enemy.
    pub launch: u32,
    /// The enemy is done (a Mote dissolving) and should go without points.
    pub despawn: bool,
}

/// Unit vector at `degrees` (0 = right, 90 = up).
fn dir(degrees: f32) -> Vec2 {
    Vec2::from_angle(degrees.to_radians())
}

/// `count` directions spread evenly over `width` degrees, centred on `aim`.
pub fn fan(aim: Vec2, count: u32, width: f32) -> Vec<Vec2> {
    let centre = aim.to_angle().to_degrees();
    if count <= 1 {
        return vec![dir(centre)];
    }
    let step = width / (count - 1) as f32;
    (0..count)
        .map(|i| dir(centre - width / 2.0 + step * i as f32))
        .collect()
}

/// Rotates `heading` (radians) toward `target` by at most `max` radians.
fn turn_toward(heading: f32, target: f32, max: f32) -> f32 {
    let diff = (target - heading + PI).rem_euclid(TAU) - PI;
    heading + diff.clamp(-max, max)
}

fn bob(amplitude: f32, period: f32, t: f32) -> f32 {
    amplitude * (TAU * t / period.max(0.01)).sin()
}

#[derive(Clone, Debug, PartialEq)]
pub enum Brain {
    Skiff(Skiff),
    Gull(Gull),
    Bellbuoy(Bellbuoy),
    Lamplighter(Lamplighter),
    Hook(Hook),
    Ferry(Ferry),
    Barnacle(Barnacle),
    Clam(Clam),
    Mote(Mote),
    /// Moves with the scenery and does nothing else (Arc Gate pylons).
    Drift,
}

impl Brain {
    pub fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        match self {
            Brain::Skiff(b) => b.step(pos, ctx, t),
            Brain::Gull(b) => b.step(pos, ctx, t),
            Brain::Bellbuoy(b) => b.step(pos, ctx, t),
            Brain::Lamplighter(b) => b.step(pos, ctx, t),
            Brain::Hook(b) => b.step(pos, ctx, t),
            Brain::Ferry(b) => b.step(pos, ctx, t),
            Brain::Barnacle(b) => b.step(pos, ctx, t),
            Brain::Clam(b) => b.step(pos, ctx, t),
            Brain::Mote(b) => b.step(pos, ctx, t),
            Brain::Drift => {
                pos.x -= ctx.scroll * ctx.dt;
                Out::default()
            }
        }
    }

    /// True while the enemy is flashing a warning before it fires.
    pub fn telegraph(&self, t: &EnemyTypes) -> bool {
        match self {
            Brain::Bellbuoy(b) => b.telegraph.is_some(),
            Brain::Lamplighter(b) => b.telegraph(t),
            Brain::Barnacle(b) => !b.silent && b.timer < t.barnacle.telegraph_seconds,
            _ => false,
        }
    }
}

// --- Skiff -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Skiff {
    /// Launched by a Ferry: flies straight left, no peel.
    pub launched: bool,
    /// x at which this member starts its peel countdown.
    pub peel_at_x: f32,
    /// Seconds after reaching `peel_at_x` before it turns.
    pub peel_delay: f32,
    /// +1 peels up, -1 down, 0 carries straight on (the tip).
    pub peel_dir: f32,
    heading: f32,
    peel_clock: Option<f32>,
}

impl Skiff {
    pub fn new(peel_at_x: f32, peel_delay: f32, peel_dir: f32) -> Self {
        Self {
            launched: false,
            peel_at_x,
            peel_delay,
            peel_dir,
            heading: PI,
            peel_clock: None,
        }
    }

    pub fn launched() -> Self {
        Self {
            launched: true,
            ..Self::new(f32::NEG_INFINITY, 0.0, 0.0)
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let s = &t.skiff;
        if self.launched {
            pos.x -= s.launched_speed * ctx.dt;
            return Out::default();
        }
        let speed = match self.peel_clock {
            None => {
                if pos.x <= self.peel_at_x {
                    self.peel_clock = Some(0.0);
                }
                s.speed
            }
            Some(ref mut clock) => {
                *clock += ctx.dt;
                if *clock >= self.peel_delay {
                    // Up-left is 120°, down-left 240°; the tip keeps 180°.
                    let target = PI - self.peel_dir * PI / 3.0;
                    let max = s.peel_turn_rate.to_radians() * ctx.dt;
                    self.heading = turn_toward(self.heading, target, max);
                    s.exit_speed
                } else {
                    s.speed
                }
            }
        };
        let heading = if self.heading == PI {
            Vec2::NEG_X
        } else {
            Vec2::from_angle(self.heading)
        };
        *pos += heading * speed * ctx.dt;
        Out::default()
    }
}

// --- Gull ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Gull {
    /// Where the snake entered: every member traces the same wave from here.
    pub entry_x: f32,
    pub base_y: f32,
    pub fires: bool,
    fired: bool,
}

impl Gull {
    pub fn new(entry_x: f32, base_y: f32, fires: bool) -> Self {
        Self {
            entry_x,
            base_y,
            fires,
            fired: false,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let g = &t.gull;
        pos.x -= g.speed * ctx.dt;
        let wavelength = (g.speed * g.period).max(1.0);
        pos.y = self.base_y + g.amplitude * (TAU * (self.entry_x - pos.x) / wavelength).sin();
        let mut out = Out::default();
        if self.fires && !self.fired && pos.x <= g.fire_x {
            self.fired = true;
            out.shots.push(Vec2::NEG_X * g.shot_speed);
        }
        out
    }
}

// --- Bellbuoy --------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Bellbuoy {
    age: f32,
    cooldown: f32,
    /// Seconds left of the ring before it fires.
    telegraph: Option<f32>,
}

impl Bellbuoy {
    pub fn new() -> Self {
        Self {
            age: 0.0,
            cooldown: 0.0,
            telegraph: None,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let b = &t.bellbuoy;
        self.age += ctx.dt;
        pos.x -= ctx.scroll * ctx.dt;
        pos.y = b.wave_y + bob(b.bob_amplitude, b.bob_period, self.age);
        let mut out = Out::default();
        let target = ctx.hero_ahead_of(*pos);
        if let Some(left) = &mut self.telegraph {
            *left -= ctx.dt;
            if *left <= 0.0 {
                self.telegraph = None;
                self.cooldown = b.cooldown;
                if let Some(hero) = target {
                    let aim = (hero - *pos).normalize_or(Vec2::NEG_X);
                    out.shots = fan(aim, b.fan_count, b.fan_degrees * 2.0)
                        .into_iter()
                        .map(|d| d * b.shot_speed)
                        .collect();
                }
            }
        } else if self.cooldown > 0.0 {
            self.cooldown -= ctx.dt;
        } else if target.is_some_and(|h| pos.x - h.x <= b.trigger_range) {
            self.telegraph = Some(b.telegraph_seconds);
        }
        out
    }
}

impl Default for Bellbuoy {
    fn default() -> Self {
        Self::new()
    }
}

// --- Lamplighter -----------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum LampPhase {
    Enter,
    Hover { t: f32, base_y: f32, fired: u32 },
    Exit { dir: f32 },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Lamplighter {
    pub phase: LampPhase,
}

impl Lamplighter {
    pub fn new() -> Self {
        Self {
            phase: LampPhase::Enter,
        }
    }

    /// Seconds into the hover at which shot `k` fires.
    fn shot_time(k: u32, t: &EnemyTypes) -> f32 {
        let l = &t.lamplighter;
        l.telegraph_seconds + k as f32 * l.fire_interval
    }

    fn telegraph(&self, t: &EnemyTypes) -> bool {
        let l = &t.lamplighter;
        match self.phase {
            LampPhase::Hover { t: now, fired, .. } if fired < l.shots => {
                now >= Self::shot_time(fired, t) - l.telegraph_seconds
            }
            _ => false,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let l = &t.lamplighter;
        let mut out = Out::default();
        match &mut self.phase {
            LampPhase::Enter => {
                // Even braking over the last `brake_distance` px.
                let left = pos.x - l.stop_x;
                let brake = (left / l.brake_distance.max(1.0)).sqrt().clamp(0.15, 1.0);
                pos.x -= l.speed * brake * ctx.dt;
                if pos.x <= l.stop_x {
                    pos.x = l.stop_x;
                    self.phase = LampPhase::Hover {
                        t: 0.0,
                        base_y: pos.y,
                        fired: 0,
                    };
                }
            }
            LampPhase::Hover {
                t: now,
                base_y,
                fired,
            } => {
                *now += ctx.dt;
                pos.y = *base_y + bob(l.bob_amplitude, l.bob_period, *now);
                if *fired < l.shots && *now >= Self::shot_time(*fired, t) {
                    *fired += 1;
                    if let Some(hero) = ctx.hero_ahead_of(*pos) {
                        out.shots
                            .push((hero - *pos).normalize_or(Vec2::NEG_X) * l.shot_speed);
                    }
                }
                if *now >= l.hover_seconds {
                    // Leave away from the Hero's side of the screen.
                    let hero_y = ctx.hero.map_or(0.0, |h| h.y);
                    let dir = if hero_y >= pos.y { -1.0 } else { 1.0 };
                    self.phase = LampPhase::Exit { dir };
                }
            }
            LampPhase::Exit { dir } => pos.y += *dir * l.exit_speed * ctx.dt,
        }
        out
    }
}

impl Default for Lamplighter {
    fn default() -> Self {
        Self::new()
    }
}

// --- Hook ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum HookPhase {
    /// Waiting off the left edge while the warning chevron blinks.
    Warn {
        left: f32,
    },
    Pass,
    /// Half-circle U-turn; `angle` runs 0 to π.
    Turn {
        angle: f32,
        centre: Vec2,
        side: f32,
    },
    Charge {
        row: f32,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct Hook {
    /// The row it flies along on its pass.
    pub row: f32,
    pub phase: HookPhase,
}

/// x where a Hook waits during its warning, just off the left edge.
pub const HOOK_WAIT_X: f32 = -HALF_W - 12.0;

impl Hook {
    /// A Hook that passes `offset` px above (positive) or below the Hero's row,
    /// kept clear of the play-area edges.
    pub fn new(hero_y: f32, offset: f32, t: &EnemyTypes) -> Self {
        Self {
            row: clamp_row(hero_y + offset, t),
            phase: HookPhase::Warn {
                left: t.hook.warning_seconds,
            },
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let h = &t.hook;
        match &mut self.phase {
            HookPhase::Warn { left } => {
                *pos = Vec2::new(HOOK_WAIT_X, self.row);
                *left -= ctx.dt;
                if *left <= 0.0 {
                    self.phase = HookPhase::Pass;
                }
            }
            HookPhase::Pass => {
                pos.x += h.speed * ctx.dt;
                pos.y = self.row;
                if pos.x >= h.turn_x {
                    // Turn toward the Hero's side of the row.
                    let hero_y = ctx.hero.map_or(0.0, |hero| hero.y);
                    let side = if hero_y >= self.row { 1.0 } else { -1.0 };
                    let r = h.turn_radius.max(1.0);
                    self.phase = HookPhase::Turn {
                        angle: 0.0,
                        centre: Vec2::new(h.turn_x, self.row + side * r),
                        side,
                    };
                }
            }
            HookPhase::Turn {
                angle,
                centre,
                side,
            } => {
                let r = h.turn_radius.max(1.0);
                *angle = (*angle + h.speed / r * ctx.dt).min(PI);
                *pos = *centre + Vec2::new(angle.sin(), -*side * angle.cos()) * r;
                if *angle >= PI {
                    let row = clamp_row(ctx.hero.map_or(pos.y, |hero| hero.y), t);
                    self.phase = HookPhase::Charge { row };
                }
            }
            HookPhase::Charge { row } => {
                pos.x -= h.charge_speed * ctx.dt;
                let max = h.charge_steer_speed * ctx.dt;
                pos.y += (*row - pos.y).clamp(-max, max);
            }
        }
        Out::default()
    }
}

/// Keeps a Hook's row `terrain_clearance` px inside the play area.
fn clamp_row(y: f32, t: &EnemyTypes) -> f32 {
    let c = t.hook.terrain_clearance;
    y.clamp(PLAY_BOTTOM + c, PLAY_TOP - c)
}

// --- Ferry -----------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum FerryPhase {
    Enter,
    Hold { t: f32 },
    Sink,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Ferry {
    pub phase: FerryPhase,
    age: f32,
    next_launch: f32,
    next_ring: f32,
    rings: u32,
}

impl Ferry {
    pub fn new(t: &EnemyTypes) -> Self {
        Self {
            phase: FerryPhase::Enter,
            age: 0.0,
            next_launch: t.ferry.first_launch_delay,
            next_ring: t.ferry.first_ring_delay,
            rings: 0,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let f = &t.ferry;
        let mut out = Out::default();
        self.age += ctx.dt;
        match &mut self.phase {
            FerryPhase::Enter => {
                pos.x = (pos.x - f.speed * ctx.dt).max(f.hold_x);
                if pos.x <= f.hold_x {
                    self.phase = FerryPhase::Hold { t: 0.0 };
                }
            }
            FerryPhase::Hold { t: held } => {
                *held += ctx.dt;
                if *held >= f.hold_seconds {
                    self.phase = FerryPhase::Sink;
                }
            }
            FerryPhase::Sink => pos.y -= f.sink_speed * ctx.dt,
        }
        if self.phase == FerryPhase::Sink {
            return out;
        }
        if self.age >= self.next_launch {
            self.next_launch += f.launch_interval.max(0.1);
            out.launch = f.launch_count;
        }
        if self.age >= self.next_ring {
            self.next_ring += f.ring_interval.max(0.1);
            // Alternate rings are turned by half a step, so the gaps move.
            let n = f.ring_count.max(1);
            let step = 360.0 / n as f32;
            let offset = if self.rings % 2 == 1 { step / 2.0 } else { 0.0 };
            out.shots = (0..n)
                .map(|i| dir(offset + step * i as f32) * f.ring_speed)
                .collect();
            self.rings += 1;
        }
        out
    }
}

// --- Barnacle --------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Barnacle {
    /// Barrel direction, radians.
    pub barrel: f32,
    /// Seconds to the next shot.
    pub timer: f32,
    /// The Hero has passed it: it never fires again.
    pub silent: bool,
}

impl Barnacle {
    pub fn new(ceiling: bool, t: &EnemyTypes) -> Self {
        Self {
            barrel: if ceiling { -PI / 2.0 } else { PI / 2.0 },
            timer: t.barnacle.fire_interval,
            silent: false,
        }
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let b = &t.barnacle;
        pos.x -= ctx.scroll * ctx.dt;
        let mut out = Out::default();
        let Some(hero) = ctx.hero else {
            return out;
        };
        if hero.x >= pos.x {
            self.silent = true;
        }
        if self.silent {
            return out;
        }
        let want = (hero - *pos).to_angle();
        self.barrel = turn_toward(self.barrel, want, b.turn_rate.to_radians() * ctx.dt);
        self.timer -= ctx.dt;
        if self.timer <= 0.0 {
            self.timer += b.fire_interval.max(0.1);
            out.shots
                .push((hero - *pos).normalize_or(Vec2::NEG_X) * b.shot_speed);
        }
        out
    }
}

// --- Clam ------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub struct Clam {
    pub base_y: f32,
    age: f32,
}

impl Clam {
    pub fn new(base_y: f32) -> Self {
        Self { base_y, age: 0.0 }
    }

    /// Whether the shell is open (and so can be hurt) at this age.
    pub fn open_at(age: f32, t: &EnemyTypes) -> bool {
        let c = &t.clam;
        let cycle = (c.closed_seconds + c.open_seconds).max(0.01);
        age.rem_euclid(cycle) >= c.closed_seconds
    }

    pub fn is_open(&self, t: &EnemyTypes) -> bool {
        Self::open_at(self.age, t)
    }

    fn step(&mut self, pos: &mut Vec2, ctx: &Ctx, t: &EnemyTypes) -> Out {
        let c = &t.clam;
        let was_open = self.is_open(t);
        self.age += ctx.dt;
        pos.x -= ctx.scroll * ctx.dt;
        pos.y = self.base_y + bob(c.bob_amplitude, c.bob_period, self.age);
        let mut out = Out::default();
        if self.is_open(t) && !was_open {
            out.shots = fan(Vec2::NEG_X, c.fan_count, c.fan_degrees)
                .into_iter()
                .map(|d| d * c.shot_speed)
                .collect();
        }
        out
    }
}

// --- Mote ------------------------------------------------------------------

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
        if let Some(hero) = ctx.hero {
            let want = (hero - *pos).to_angle();
            self.heading = turn_toward(self.heading, want, m.turn_rate.to_radians() * ctx.dt);
        }
        *pos += Vec2::from_angle(self.heading) * m.speed * ctx.dt;
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

// --- Arc Gate --------------------------------------------------------------

/// The arc between two Arc Gate pylons: off, a flicker, then live.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Arc {
    pub age: f32,
}

impl Arc {
    fn phase(&self, t: &EnemyTypes) -> f32 {
        let a = &t.arc_gate;
        self.age
            .rem_euclid((a.on_seconds + a.off_seconds).max(0.01))
    }

    pub fn live(&self, t: &EnemyTypes) -> bool {
        self.phase(t) >= t.arc_gate.off_seconds
    }

    pub fn telegraph(&self, t: &EnemyTypes) -> bool {
        let a = &t.arc_gate;
        let p = self.phase(t);
        p < a.off_seconds && p >= a.off_seconds - a.telegraph_seconds
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
            scroll: 30.0,
        }
    }

    /// Steps a brain for `seconds`, returning every Out and the path.
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
    fn fan_spreads_evenly_around_the_aim() {
        let f = fan(Vec2::NEG_X, 5, 60.0);
        assert_eq!(f.len(), 5);
        assert!(close(f[2].x, -1.0, 1e-5));
        let a0 = f[0].to_angle().to_degrees().rem_euclid(360.0);
        let a4 = f[4].to_angle().to_degrees().rem_euclid(360.0);
        assert!(
            close(a0, 150.0, 0.01) && close(a4, 210.0, 0.01),
            "{a0} {a4}"
        );
        assert_eq!(fan(Vec2::X, 1, 90.0), vec![Vec2::X]);
    }

    #[test]
    fn skiff_holds_its_line_then_peels_up() {
        let t = types();
        let mut b = Brain::Skiff(Skiff::new(t.skiff.peel_x, 0.0, 1.0));
        let mut pos = Vec2::new(170.0, 0.0);
        // 130 px at 90 px/s: still flat after 1.4 s.
        run(&mut b, &mut pos, None, 1.4);
        assert!(pos.x > t.skiff.peel_x && pos.y == 0.0, "{pos}");
        run(&mut b, &mut pos, None, 1.0);
        assert!(pos.y > 20.0, "peeled up: {pos}");
        assert!(pos.x < t.skiff.peel_x);
    }

    #[test]
    fn skiff_peel_waits_for_its_delay() {
        let t = types();
        let mut b = Brain::Skiff(Skiff::new(t.skiff.peel_x, 0.6, -1.0));
        let mut pos = Vec2::new(t.skiff.peel_x + 1.0, 0.0);
        run(&mut b, &mut pos, None, 0.5);
        assert_eq!(pos.y, 0.0);
        run(&mut b, &mut pos, None, 0.5);
        assert!(pos.y < 0.0, "peeled down: {pos}");
    }

    #[test]
    fn launched_skiff_flies_straight_left() {
        let mut b = Brain::Skiff(Skiff::launched());
        let mut pos = Vec2::new(60.0, 10.0);
        run(&mut b, &mut pos, None, 1.0);
        assert!(close(pos.x, 60.0 - 120.0, 0.5) && pos.y == 10.0, "{pos}");
    }

    #[test]
    fn gull_members_trace_the_same_wave() {
        let t = types();
        let wavelength = t.gull.speed * t.gull.period;
        let mut lead = Brain::Gull(Gull::new(170.0, 0.0, false));
        let mut tail = Brain::Gull(Gull::new(170.0, 0.0, false));
        let mut a = Vec2::new(170.0, 0.0);
        let mut b = Vec2::new(170.0 + wavelength / 4.0, 0.0);
        run(&mut lead, &mut a, None, t.gull.period / 4.0);
        // The lead is a quarter wave in: at the crest.
        assert!(close(a.y, t.gull.amplitude, 0.5), "{a}");
        run(&mut lead, &mut a, None, 2.0);
        run(&mut tail, &mut b, None, 2.0 + t.gull.period / 2.0);
        // Both are at the same x now, so on the same height.
        assert!(close(a.x, b.x, 0.5) && close(a.y, b.y, 0.5), "{a} {b}");
    }

    #[test]
    fn firing_gull_shoots_once_at_fire_x() {
        let mut b = Brain::Gull(Gull::new(50.0, 0.0, true));
        let mut pos = Vec2::new(50.0, 0.0);
        let outs = run(&mut b, &mut pos, None, 3.0);
        assert_eq!(shots(&outs), 1);
        let first = outs.iter().position(|o| !o.shots.is_empty()).unwrap();
        // 50 px at 70 px/s.
        assert!(close(first as f32 * DT, 50.0 / 70.0, 0.05));
        assert_eq!(outs[first].shots[0], Vec2::new(-80.0, 0.0));
    }

    #[test]
    fn bellbuoy_rings_then_fires_a_fan_at_a_hero_ahead() {
        let t = types();
        let mut b = Brain::Bellbuoy(Bellbuoy::new());
        let mut pos = Vec2::new(100.0, t.bellbuoy.wave_y);
        let hero = Some(Vec2::new(40.0, 0.0));
        let outs = run(&mut b, &mut pos, hero, 0.3);
        assert_eq!(shots(&outs), 0);
        assert!(b.telegraph(&t));
        let outs = run(&mut b, &mut pos, hero, 0.2);
        assert_eq!(shots(&outs), 3);
        let fan = outs.iter().find(|o| !o.shots.is_empty()).unwrap();
        assert!(fan.shots.iter().all(|v| v.y > 0.0), "aimed upward");
        // Cooldown: nothing more for a while.
        assert_eq!(shots(&run(&mut b, &mut pos, hero, 2.0)), 0);
    }

    #[test]
    fn bellbuoy_is_silent_once_passed() {
        let t = types();
        let mut b = Brain::Bellbuoy(Bellbuoy::new());
        let mut pos = Vec2::new(0.0, t.bellbuoy.wave_y);
        let outs = run(&mut b, &mut pos, Some(Vec2::new(30.0, 0.0)), 5.0);
        assert_eq!(shots(&outs), 0);
    }

    #[test]
    fn lamplighter_stops_fires_twice_then_leaves_away_from_the_hero() {
        let t = types();
        let mut b = Brain::Lamplighter(Lamplighter::new());
        let mut pos = Vec2::new(170.0, 20.0);
        let hero = Some(Vec2::new(-100.0, 60.0));
        let outs = run(&mut b, &mut pos, hero, 1.3);
        assert_eq!(pos.x, t.lamplighter.stop_x, "stopped within about 1.2 s");
        assert!(b.telegraph(&t), "lamp lit as soon as it stops");
        let first = outs.len()
            + run(&mut b, &mut pos, hero, 0.5)
                .iter()
                .position(|o| !o.shots.is_empty())
                .expect("first shot");
        assert!(first as f32 * DT < 1.8, "first shot before 1.8 s");
        let rest = run(&mut b, &mut pos, hero, 3.0);
        assert_eq!(shots(&rest), 1, "two shots in all");
        assert!(pos.y < 20.0, "left downward, away from a Hero above");
        let shot = rest.iter().find(|o| !o.shots.is_empty()).unwrap().shots[0];
        assert!(close(shot.length(), t.lamplighter.shot_speed, 0.01));
        assert!(shot.x < 0.0);
    }

    #[test]
    fn hook_warns_passes_turns_and_charges_at_the_hero_row() {
        let t = types();
        let hero = Vec2::new(-80.0, 0.0);
        let mut b = Brain::Hook(Hook::new(hero.y, t.hook.pass_offset, &t));
        let mut pos = Vec2::ZERO;
        run(&mut b, &mut pos, Some(hero), t.hook.warning_seconds - 0.1);
        assert_eq!(pos, Vec2::new(HOOK_WAIT_X, 50.0));
        // Across the screen at 150 px/s to x = 110.
        run(&mut b, &mut pos, Some(hero), 2.0);
        assert!(matches!(
            b,
            Brain::Hook(Hook {
                phase: HookPhase::Turn { .. },
                ..
            })
        ));
        assert!(pos.x >= t.hook.turn_x);
        run(&mut b, &mut pos, Some(hero), 0.6);
        let Brain::Hook(h) = &b else { unreachable!() };
        assert_eq!(h.phase, HookPhase::Charge { row: 0.0 });
        run(&mut b, &mut pos, Some(hero), 0.5);
        assert!(close(pos.y, 0.0, 0.01), "on the Hero's row: {pos}");
        assert!(pos.x < 60.0);
    }

    #[test]
    fn hook_row_stays_clear_of_the_edges() {
        let t = types();
        let Brain::Hook(h) = Brain::Hook(Hook::new(100.0, 50.0, &t)) else {
            unreachable!()
        };
        assert_eq!(h.row, PLAY_TOP - t.hook.terrain_clearance);
    }

    #[test]
    fn ferry_holds_launches_and_rings_then_sinks() {
        let t = types();
        let mut b = Brain::Ferry(Ferry::new(&t));
        let mut pos = Vec2::new(184.0, 0.0);
        let outs = run(&mut b, &mut pos, None, 8.0);
        // Launches at 1, 4, 7 s; rings at 2, 5, 8 s (the last lands on the
        // boundary, so allow 2 or 3).
        let launches: u32 = outs.iter().map(|o| o.launch).sum();
        assert_eq!(launches, 3 * t.ferry.launch_count);
        let rings = outs.iter().filter(|o| !o.shots.is_empty()).count();
        assert!((2..=3).contains(&rings), "{rings}");
        assert_eq!(pos.x, t.ferry.hold_x);
        let ring = outs.iter().find(|o| !o.shots.is_empty()).unwrap();
        assert_eq!(ring.shots.len(), t.ferry.ring_count as usize);
        run(&mut b, &mut pos, None, t.ferry.hold_seconds);
        let outs = run(&mut b, &mut pos, None, 2.0);
        assert!(pos.y < -30.0, "sinking: {pos}");
        assert!(outs.iter().all(|o| o.shots.is_empty() && o.launch == 0));
    }

    #[test]
    fn barnacle_fires_every_interval_until_passed() {
        let t = types();
        let mut b = Brain::Barnacle(Barnacle::new(false, &t));
        let mut pos = Vec2::new(150.0, -100.0);
        let hero = Some(Vec2::new(-100.0, 0.0));
        let outs = run(&mut b, &mut pos, hero, 3.3);
        assert_eq!(shots(&outs), 2);
        let s = outs.iter().find(|o| !o.shots.is_empty()).unwrap().shots[0];
        assert!(s.x < 0.0 && s.y > 0.0, "aimed up-left at the Hero");
        assert!(
            close(pos.x, 150.0 - 30.0 * 3.3, 0.5),
            "moves with the scroll"
        );
        // The Hero flies past it: silent from then on.
        let passed = Some(Vec2::new(pos.x + 10.0, 0.0));
        let outs = run(&mut b, &mut pos, passed, 5.0);
        assert_eq!(shots(&outs), 0);
        assert!(!b.telegraph(&t));
    }

    #[test]
    fn clam_fires_only_as_it_opens() {
        let t = types();
        let mut b = Brain::Clam(Clam::new(0.0));
        let mut pos = Vec2::new(100.0, 0.0);
        let outs = run(&mut b, &mut pos, None, 6.05);
        let fires: Vec<f32> = outs
            .iter()
            .enumerate()
            .filter(|(_, o)| !o.shots.is_empty())
            .map(|(i, _)| (i + 1) as f32 * DT)
            .collect();
        assert_eq!(fires.len(), 2, "{fires:?}");
        assert!(close(fires[0], 2.0, DT * 1.5) && close(fires[1], 5.0, DT * 1.5));
        assert_eq!(outs[fires.len()].shots.len(), 0);
        assert!(!Clam::open_at(1.9, &t));
        assert!(Clam::open_at(2.5, &t));
        assert!(!Clam::open_at(3.1, &t));
    }

    #[test]
    fn mote_homes_slowly_and_dissolves() {
        let t = types();
        let mut b = Brain::Mote(Mote::new());
        let mut pos = Vec2::new(100.0, 0.0);
        // The Hero is straight below: after 0.5 s the Mote has turned only 45°.
        run(&mut b, &mut pos, Some(Vec2::new(100.0, -200.0)), 0.5);
        let Brain::Mote(m) = &b else { unreachable!() };
        assert!(
            close(m.heading.to_degrees(), 225.0, 1.0),
            "{}",
            m.heading.to_degrees()
        );
        let outs = run(&mut b, &mut pos, None, t.mote.lifetime);
        assert!(outs.last().unwrap().despawn);
        assert!(!outs[0].despawn);
    }

    #[test]
    fn arc_cycles_off_flicker_live() {
        let t = types();
        let at = |age: f32| Arc { age };
        assert!(!at(0.1).live(&t) && !at(0.1).telegraph(&t));
        assert!(at(0.6).telegraph(&t) && !at(0.6).live(&t));
        assert!(at(1.0).live(&t) && !at(1.0).telegraph(&t));
        assert!(!at(2.1).live(&t));
    }
}
