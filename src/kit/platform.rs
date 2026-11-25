//! Platform bodies: running, jumping, gravity, landing on ground and
//! one-way ledges, and stopping at walls. Pure functions over a
//! [`TileQuery`], so they can be tested at a fixed step; the on-foot
//! Cabinets (3 and 4) drive them.
//!
//! Positions are a body's *feet*: `x` is its centre, `feet` the bottom of its
//! box. Up is +y.
//!
//! Cabinet 4 adds solid ledges (stood on, never dropped through, and
//! blocking a head from below) and ladders ([`climb`]). Both come from
//! [`TileQuery`] methods that default to none, so Cabinet 3 is unchanged.

/// What a body stands on and runs into.
pub trait TileQuery {
    /// The highest solid ground top anywhere under the span `x0..x1`, or
    /// `None` if the whole span is open (a pit).
    fn ground_top(&self, x0: f32, x1: f32) -> Option<f32>;
    /// The tops of the one-way ledges under the span that are solid now.
    fn ledge_tops(&self, x0: f32, x1: f32) -> Vec<f32>;
    /// The tops of solid ledges under the span: stood on like ledges, but
    /// never dropped through.
    fn solid_tops(&self, _x0: f32, _x1: f32) -> Vec<f32> {
        Vec::new()
    }
    /// The lowest underside of a solid piece over the span that a head
    /// rising from `from` to `to` would cross, if any.
    fn ceiling(&self, _x0: f32, _x1: f32, _from: f32, _to: f32) -> Option<f32> {
        None
    }
    /// The ladder whose centre is within `window` of `x` and which reaches
    /// height `y` (bottom <= y <= top), if any.
    fn ladder_near(&self, _x: f32, _y: f32, _window: f32) -> Option<Ladder> {
        None
    }
}

/// A ladder: one tile wide, centred on `x`, from a ledge at `bottom` to one
/// at `top`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Ladder {
    pub x: f32,
    pub bottom: f32,
    pub top: f32,
}

/// The ladder keys (`climber.ladder` in Cabinet 4).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LadderParams {
    pub climb_speed: f32,
    pub grab_window: f32,
    /// A jump off a ladder, as a fraction of the take-off speed.
    pub jump_speed_fraction: f32,
}

/// The jump keys (`run_and_gun.jump` in Cabinet 3).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct JumpParams {
    pub speed: f32,
    pub gravity_up: f32,
    pub gravity_down: f32,
    pub cut: f32,
    pub max_fall: f32,
    pub coyote_seconds: f32,
    pub buffer_seconds: f32,
}

/// One frame of input for a body.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Control {
    /// Horizontal speed wanted, px/s (signed).
    pub run: f32,
    /// The jump button went down this frame.
    pub jump_pressed: bool,
    /// The jump button is held.
    pub jump_held: bool,
    /// Down + jump on a ledge: fall through it.
    pub drop_through: bool,
    /// A mid-air jump at this speed is allowed (the Jet), if any.
    pub air_jump: Option<f32>,
    /// Up (+1) or Down (-1) held, for ladders.
    pub climb: f32,
}

/// What happened during one [`step`].
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Moved {
    pub jumped: bool,
    pub air_jumped: bool,
    /// Caught a ladder this frame.
    pub grabbed: bool,
    /// Bumped a head on a solid ledge.
    pub bumped: bool,
    pub landed: bool,
    pub blocked: bool,
}

/// A body under the platform physics.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Body {
    pub x: f32,
    pub feet: f32,
    pub vy: f32,
    pub half_width: f32,
    /// Its height, for ceilings (0: no head, as in Cabinet 3).
    pub height: f32,
    pub grounded: bool,
    /// The ladder it is on, if any.
    pub ladder: Option<Ladder>,
    /// Seconds left to jump after running off an edge.
    coyote: f32,
    /// Seconds left in which an early jump press still counts.
    buffer: f32,
    /// This jump's upward speed has already been cut by a release.
    cut_done: bool,
    /// Seconds left falling through ledges after a drop-through.
    dropping: f32,
}

/// How far below its feet a body still counts as standing on something.
const SNAP: f32 = 0.5;
/// How long a drop-through ignores ledges.
const DROP_SECONDS: f32 = 0.2;

impl Body {
    pub fn new(x: f32, feet: f32, half_width: f32) -> Self {
        Self {
            x,
            feet,
            half_width,
            ..Self::default()
        }
    }

    fn span(&self, x: f32) -> (f32, f32) {
        (x - self.half_width, x + self.half_width)
    }

    /// What it would stand on at `x`: the highest ground or ledge top at or
    /// just below its feet.
    pub fn support(&self, x: f32, world: &impl TileQuery) -> Option<f32> {
        let (x0, x1) = self.span(x);
        // Only ground at the feet holds a body up: lower ground means a
        // fall (walking off a step down).
        let ground = world
            .ground_top(x0, x1)
            .filter(|t| (*t - self.feet).abs() <= SNAP);
        let near = |t: &f32| *t <= self.feet + SNAP && *t >= self.feet - SNAP;
        let solid = world.solid_tops(x0, x1).into_iter().filter(near);
        let ledges = if self.dropping > 0.0 {
            solid.reduce(f32::max)
        } else {
            world
                .ledge_tops(x0, x1)
                .into_iter()
                .filter(near)
                .chain(solid)
                .reduce(f32::max)
        };
        match (ground, ledges) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    /// Standing on a ledge rather than solid ground.
    pub fn on_ledge(&self, world: &impl TileQuery) -> bool {
        let (x0, x1) = self.span(self.x);
        self.grounded
            && world
                .ground_top(x0, x1)
                .is_none_or(|t| t < self.feet - SNAP)
            && !world
                .solid_tops(x0, x1)
                .iter()
                .any(|t| (*t - self.feet).abs() <= SNAP)
    }
}

/// Moves a body by one frame: run (stopping at walls), jump, gravity and
/// landing.
pub fn step(
    body: &mut Body,
    input: Control,
    p: &JumpParams,
    world: &impl TileQuery,
    dt: f32,
) -> Moved {
    let mut moved = Moved::default();
    body.buffer = (body.buffer - dt).max(0.0);
    body.coyote = (body.coyote - dt).max(0.0);
    body.dropping = (body.dropping - dt).max(0.0);
    if input.jump_pressed {
        body.buffer = p.buffer_seconds;
    }

    // Run, stopping at anything taller than the feet.
    let want = body.x + input.run * dt;
    let blocked = |x: f32| {
        let (x0, x1) = body.span(x);
        world
            .ground_top(x0, x1)
            .is_some_and(|t| t > body.feet + SNAP)
    };
    if blocked(want) {
        moved.blocked = true;
        // Creep up to the wall in small steps.
        let dir = (want - body.x).signum();
        let mut x = body.x;
        while (want - x).abs() > 0.0 {
            let next = x + dir * (want - x).abs().min(0.25);
            if blocked(next) {
                break;
            }
            x = next;
        }
        body.x = x;
    } else {
        body.x = want;
    }

    // Walking off an edge starts a fall, with coyote time to still jump.
    if body.grounded && body.support(body.x, world).is_none() {
        body.grounded = false;
        body.coyote = p.coyote_seconds;
    }
    if body.grounded {
        body.coyote = p.coyote_seconds;
        if input.drop_through && body.on_ledge(world) {
            body.dropping = DROP_SECONDS;
            body.grounded = false;
            body.coyote = 0.0;
            body.buffer = 0.0;
        }
    }

    // Jump: a press now or just before landing, on the ground or just after
    // leaving it.
    if body.buffer > 0.0 && (body.grounded || body.coyote > 0.0) {
        body.vy = p.speed;
        body.grounded = false;
        body.coyote = 0.0;
        body.buffer = 0.0;
        body.cut_done = false;
        moved.jumped = true;
    } else if input.jump_pressed
        && !body.grounded
        && let Some(speed) = input.air_jump
    {
        body.vy = speed;
        body.buffer = 0.0;
        body.cut_done = false;
        moved.air_jumped = true;
    }

    if body.grounded {
        body.vy = 0.0;
        return moved;
    }

    // Letting go while rising cuts the jump short, once.
    if body.vy > 0.0 && !input.jump_held && !body.cut_done {
        body.vy *= p.cut;
        body.cut_done = true;
    }
    let gravity = if body.vy > 0.0 && input.jump_held {
        p.gravity_up
    } else {
        p.gravity_down
    };
    body.vy = (body.vy - gravity * dt).max(-p.max_fall);
    let mut next = body.feet + body.vy * dt;
    if body.vy > 0.0 && body.height > 0.0 {
        let (x0, x1) = body.span(body.x);
        let head = body.feet + body.height;
        if let Some(under) = world.ceiling(x0, x1, head, next + body.height) {
            next = (under - body.height).max(body.feet);
            body.vy = 0.0;
            body.cut_done = true;
            moved.bumped = true;
        }
    }
    if body.vy <= 0.0 {
        let (x0, x1) = body.span(body.x);
        let mut floor = world
            .ground_top(x0, x1)
            .filter(|t| *t <= body.feet + SNAP && *t >= next);
        let mut tops = world.solid_tops(x0, x1);
        if body.dropping <= 0.0 {
            tops.extend(world.ledge_tops(x0, x1));
        }
        for t in tops {
            if t <= body.feet + SNAP && t >= next {
                floor = Some(floor.map_or(t, |f| f.max(t)));
            }
        }
        if let Some(top) = floor {
            body.feet = top;
            body.vy = 0.0;
            body.grounded = true;
            moved.landed = true;
            return moved;
        }
    }
    body.feet = next;
    moved
}

/// Ladders: catching one, climbing it, and leaving it. Call before [`step`]
/// each frame; while it returns `Some`, the body is on a ladder and `step`
/// is skipped.
///
/// - Up or Down with the body's centre within the grab window of a ladder
///   catches it, on a ledge or in the air (Down from the ledge at its top).
/// - On it, the body climbs at the climb speed, snapped to its centre.
///   Past the top it stands on the ledge there; at the bottom, on the ledge
///   below.
/// - The jump button with left or right jumps off at a fraction of the
///   take-off speed; with no direction it lets go and falls.
pub fn climb(
    body: &mut Body,
    input: Control,
    lp: &LadderParams,
    p: &JumpParams,
    world: &impl TileQuery,
    dt: f32,
) -> Option<Moved> {
    let mut moved = Moved::default();
    if body.ladder.is_none() {
        if input.climb == 0.0 || input.jump_pressed {
            return None;
        }
        // Look a pixel the way it wants to go, so where one ladder ends and
        // the next begins, Up takes the one above and Down the one below.
        let ladder = world.ladder_near(body.x, body.feet + input.climb.signum(), lp.grab_window)?;
        let at_top = (body.feet - ladder.top).abs() <= SNAP;
        let at_bottom = (body.feet - ladder.bottom).abs() <= SNAP;
        // Up climbs from below its top; Down climbs from above its bottom.
        let wants = if input.climb > 0.0 {
            !at_top || !body.grounded
        } else {
            !at_bottom || !body.grounded
        };
        if !wants || (body.grounded && input.climb > 0.0 && at_top) {
            return None;
        }
        body.ladder = Some(ladder);
        body.grounded = false;
        body.vy = 0.0;
        body.coyote = 0.0;
        body.buffer = 0.0;
        moved.grabbed = true;
    }
    let ladder = body.ladder.expect("on a ladder");
    body.x = ladder.x;
    if input.jump_pressed {
        body.ladder = None;
        if input.run != 0.0 {
            body.vy = p.speed * lp.jump_speed_fraction;
            body.cut_done = false;
            body.x += input.run * dt;
            moved.jumped = true;
        } else {
            body.vy = 0.0;
        }
        return Some(moved);
    }
    body.feet += input.climb.signum() * (input.climb != 0.0) as u8 as f32 * lp.climb_speed * dt;
    if body.feet >= ladder.top {
        body.feet = ladder.top;
        body.ladder = None;
        body.grounded = true;
        moved.landed = true;
    } else if body.feet <= ladder.bottom {
        body.feet = ladder.bottom;
        body.ladder = None;
        body.grounded = true;
        moved.landed = true;
    }
    Some(moved)
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 240.0;

    /// Cabinet 3's jump keys.
    fn p() -> JumpParams {
        JumpParams {
            speed: 240.0,
            gravity_up: 600.0,
            gravity_down: 900.0,
            cut: 0.4,
            max_fall: 300.0,
            coyote_seconds: 0.08,
            buffer_seconds: 0.1,
        }
    }

    /// Ground at y = 0 everywhere except `pits`, plus ledges.
    struct World {
        pits: Vec<(f32, f32)>,
        walls: Vec<(f32, f32, f32)>,
        ledges: Vec<(f32, f32, f32)>,
    }

    fn flat() -> World {
        World {
            pits: vec![],
            walls: vec![],
            ledges: vec![],
        }
    }

    impl TileQuery for World {
        fn ground_top(&self, x0: f32, x1: f32) -> Option<f32> {
            let mut top: Option<f32> = None;
            // Sample the span every pixel: open only where every sample is.
            let mut x = x0;
            while x <= x1 {
                let pit = self.pits.iter().any(|&(a, b)| x > a && x < b);
                let wall = self
                    .walls
                    .iter()
                    .filter(|&&(a, b, _)| x >= a && x <= b)
                    .map(|w| w.2)
                    .reduce(f32::max);
                let here = match (pit, wall) {
                    (_, Some(h)) => Some(h),
                    (true, None) => None,
                    (false, None) => Some(0.0),
                };
                top = match (top, here) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    (a, b) => a.or(b),
                };
                x += 1.0;
            }
            top
        }

        fn ledge_tops(&self, x0: f32, x1: f32) -> Vec<f32> {
            self.ledges
                .iter()
                .filter(|&&(a, b, _)| x1 > a && x0 < b)
                .map(|l| l.2)
                .collect()
        }
    }

    fn standing(world: &World) -> Body {
        let mut b = Body::new(0.0, 0.0, 5.0);
        step(&mut b, Control::default(), &p(), world, DT);
        assert!(b.grounded);
        b
    }

    /// Jumps (holding the button for `hold` s) and returns the peak height,
    /// the airtime and the distance at `run` px/s.
    fn jump(hold: f32, run: f32) -> (f32, f32, f32) {
        let w = flat();
        let mut b = standing(&w);
        let mut t = 0.0;
        let mut peak: f32 = 0.0;
        loop {
            let input = Control {
                run,
                jump_pressed: t == 0.0,
                jump_held: t < hold,
                ..Control::default()
            };
            let m = step(&mut b, input, &p(), &w, DT);
            t += DT;
            peak = peak.max(b.feet);
            if m.landed {
                return (peak, t, b.x);
            }
            assert!(t < 3.0, "never landed");
        }
    }

    #[test]
    fn a_full_jump_matches_the_design_doc() {
        let (height, air, across) = jump(10.0, 72.0);
        assert!((height - 48.0).abs() < 1.0, "48 px high: {height}");
        assert!((air - 0.73).abs() < 0.02, "0.73 s in the air: {air}");
        assert!((across - 52.0).abs() < 1.5, "52 px across: {across}");
    }

    #[test]
    fn releasing_early_makes_a_short_hop() {
        let (tap, _, _) = jump(0.1, 0.0);
        assert!((tap - 25.0).abs() < 2.5, "about 25 px: {tap}");
        let (first_frame, _, _) = jump(0.0, 0.0);
        assert!(first_frame < 10.0, "about 8 px: {first_frame}");
    }

    #[test]
    fn coyote_time_allows_a_late_jump_off_an_edge() {
        let w = World {
            pits: vec![(10.0, 200.0)],
            ..flat()
        };
        let run = |late: f32| {
            let mut b = standing(&w);
            // Run right until the body has left the edge `late` s ago.
            let mut since_edge = None;
            for _ in 0..2000 {
                let input = Control {
                    run: 72.0,
                    jump_pressed: since_edge.is_some_and(|s: f32| s >= late),
                    jump_held: true,
                    ..Control::default()
                };
                let m = step(&mut b, input, &p(), &w, DT);
                if m.jumped {
                    return true;
                }
                if !b.grounded {
                    since_edge = Some(since_edge.map_or(0.0, |s| s + DT));
                    if since_edge.unwrap() > late + 0.01 {
                        return false;
                    }
                }
            }
            false
        };
        assert!(run(0.06), "0.06 s after the edge still jumps");
        assert!(!run(0.12), "0.12 s is too late");
    }

    #[test]
    fn a_press_just_before_landing_still_jumps() {
        let w = flat();
        let mut b = Body::new(0.0, 20.0, 5.0);
        let mut jumped_on_landing = false;
        let mut pressed = false;
        for _ in 0..1000 {
            // Press once, when about 0.05 s from the ground.
            let near = !pressed && b.feet < 0.05 * 200.0 && b.vy < 0.0;
            pressed |= near;
            let m = step(
                &mut b,
                Control {
                    jump_pressed: near,
                    jump_held: true,
                    ..Control::default()
                },
                &p(),
                &w,
                DT,
            );
            if m.jumped {
                jumped_on_landing = true;
                break;
            }
        }
        assert!(jumped_on_landing);
    }

    #[test]
    fn holding_the_button_does_not_jump_again_on_landing() {
        let w = flat();
        let mut b = standing(&w);
        let mut jumps = 0;
        for i in 0..800 {
            let m = step(
                &mut b,
                Control {
                    jump_pressed: i == 0,
                    jump_held: true,
                    ..Control::default()
                },
                &p(),
                &w,
                DT,
            );
            jumps += m.jumped as u32;
        }
        assert_eq!(jumps, 1);
    }

    #[test]
    fn walls_stop_a_runner_and_a_jump_clears_a_step() {
        let w = World {
            walls: vec![(20.0, 400.0, 24.0)],
            ..flat()
        };
        let mut b = standing(&w);
        for _ in 0..240 {
            step(
                &mut b,
                Control {
                    run: 72.0,
                    ..Control::default()
                },
                &p(),
                &w,
                DT,
            );
        }
        assert!(
            (b.x + b.half_width - 20.0).abs() < 0.3,
            "against the wall: {}",
            b.x
        );
        // Jump onto it.
        for i in 0..400 {
            let input = Control {
                run: 72.0,
                jump_pressed: i == 0,
                jump_held: true,
                ..Control::default()
            };
            step(&mut b, input, &p(), &w, DT);
        }
        assert!(b.grounded && b.feet == 24.0, "on the step: {b:?}");
    }

    #[test]
    fn walking_off_a_step_down_falls_to_the_lower_ground() {
        let w = World {
            walls: vec![(-100.0, 0.0, 24.0)],
            ..flat()
        };
        let mut b = Body::new(-20.0, 24.0, 5.0);
        b.grounded = true;
        for _ in 0..240 {
            step(
                &mut b,
                Control {
                    run: 72.0,
                    ..Control::default()
                },
                &p(),
                &w,
                DT,
            );
        }
        assert!(
            b.grounded && b.feet == 0.0,
            "down on the lower ground: {b:?}"
        );
    }

    #[test]
    fn ledges_are_one_way_and_can_be_dropped_through() {
        let w = World {
            ledges: vec![(-50.0, 50.0, 30.0)],
            ..flat()
        };
        let mut b = standing(&w);
        // Jump up through it and land on top.
        for i in 0..400 {
            let input = Control {
                jump_pressed: i == 0,
                jump_held: true,
                ..Control::default()
            };
            step(&mut b, input, &p(), &w, DT);
        }
        assert!(b.grounded && b.feet == 30.0, "on the ledge: {b:?}");
        assert!(b.on_ledge(&w));
        step(
            &mut b,
            Control {
                drop_through: true,
                ..Control::default()
            },
            &p(),
            &w,
            DT,
        );
        for _ in 0..400 {
            step(&mut b, Control::default(), &p(), &w, DT);
        }
        assert!(b.grounded && b.feet == 0.0, "back on the ground: {b:?}");
    }

    #[test]
    fn a_body_falls_into_a_pit() {
        let w = World {
            pits: vec![(-100.0, 100.0)],
            ..flat()
        };
        let mut b = Body::new(0.0, 0.0, 5.0);
        for _ in 0..240 {
            step(&mut b, Control::default(), &p(), &w, DT);
        }
        assert!(b.feet < -100.0 && !b.grounded);
        assert_eq!(b.vy, -300.0, "at the top falling speed");
    }

    #[test]
    fn an_air_jump_works_once_in_mid_air() {
        let w = flat();
        let mut b = standing(&w);
        let mut air = 0;
        for i in 0..400 {
            let input = Control {
                jump_pressed: i == 0 || i == 60 || i == 80,
                jump_held: true,
                air_jump: (i < 70).then_some(168.0),
                ..Control::default()
            };
            air += step(&mut b, input, &p(), &w, DT).air_jumped as u32;
        }
        assert_eq!(air, 1);
    }
}
