//! The tower face as the gameplay reads it: the Stage file's terrain
//! (`terrain.rs`) plus the state that changes in play (crumbling ledges and
//! the gust clock), answering the platform physics' [`TileQuery`].
//!
//! **Tower space.** Gameplay on the face works in tower space: x as on
//! screen, and y as a *height*, the scroll position at which that point is on
//! the screen's top edge (see `terrain.rs`). The terrain never moves there;
//! the camera rises past it. [`screen_y`] and [`tower_y`] convert.

use bevy::prelude::*;

use super::terrain::{Foothold, Gust, Ledge, StageTerrain, TILE};
use super::tuning::{ClimberTuning, TerrainTuning};
use crate::display::HALF_H;
use crate::flow::Screen;
use crate::kit::platform::{Ladder, TileQuery};

pub use super::terrain::screen_y;

/// The height (tower y) of screen y `y` with the camera at `camera`.
pub fn tower_y(y: f32, camera: f32) -> f32 {
    y - HALF_H + camera
}

/// Beyond the face's edges is a wall this high.
const WALL: f32 = 1.0e6;

/// A crumbling ledge's state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Crumble {
    #[default]
    Whole,
    /// Shaking after the Hero landed; seconds left.
    Shaking(f32),
    /// Fallen away; seconds until it comes back.
    Gone(f32),
}

/// Where a gust zone is in its cycle.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GustPhase {
    Calm,
    /// The arrows blink at the upwind edge.
    Warning,
    Blowing,
}

/// The current Stage's tower face.
#[derive(Resource, Clone, Debug, Default)]
pub struct Tower {
    pub ledges: Vec<Ledge>,
    pub crumbles: Vec<Crumble>,
    pub ladders: Vec<Ladder>,
    pub gusts: Vec<Gust>,
    pub route: Vec<Foothold>,
    /// Half the face's width: its edges are walls.
    pub face: f32,
    /// Seconds of play since the Stage began (or the last rewind), for the
    /// gust cycles.
    pub clock: f32,
}

impl Tower {
    pub fn new(file: StageTerrain, face: f32) -> Self {
        Self {
            crumbles: vec![Crumble::Whole; file.ledges.len()],
            ladders: file
                .ladders
                .iter()
                .map(|d| Ladder {
                    x: d.x,
                    bottom: d.from_h,
                    top: d.to_h,
                })
                .collect(),
            ledges: file.ledges,
            gusts: file.gusts,
            route: file.route,
            face,
            clock: 0.0,
        }
    }

    /// Reads a whole Stage file's terrain.
    pub fn parse(stage_file: &str, t: &ClimberTuning) -> Result<Self, String> {
        StageTerrain::parse(stage_file).map(|f| Self::new(f, t.camera.face_half_width))
    }

    fn overlaps(l: &Ledge, x0: f32, x1: f32) -> bool {
        x1 > l.from_x && x0 < l.to_x
    }

    /// Solid ledges: landings and `solid` ones. They block from below and
    /// stop bullets.
    pub fn is_solid(l: &Ledge) -> bool {
        matches!(l.kind.as_str(), "landing" | "solid")
    }

    fn standing(&self, i: usize) -> bool {
        !matches!(self.crumbles[i], Crumble::Gone(_))
    }

    /// Whether the point (x, h) is inside a solid ledge's tile.
    pub fn in_solid(&self, x: f32, h: f32) -> bool {
        self.ledges
            .iter()
            .any(|l| Self::is_solid(l) && l.covers(x) && h <= l.h && h > l.h - TILE)
    }

    /// The highest standing ledge top (of any kind) at or below `h` under x.
    pub fn floor_below(&self, x: f32, h: f32) -> Option<f32> {
        self.ledges
            .iter()
            .enumerate()
            .filter(|(i, l)| l.covers(x) && l.h <= h && self.standing(*i))
            .map(|(_, l)| l.h)
            .reduce(f32::max)
    }

    /// The ledge (by index) a body over `x0..x1` with its feet at `feet`
    /// stands on, if any.
    pub fn ledge_under(&self, x0: f32, x1: f32, feet: f32) -> Option<usize> {
        self.ledges.iter().enumerate().position(|(i, l)| {
            Self::overlaps(l, x0, x1) && self.standing(i) && (l.h - feet).abs() <= 0.5
        })
    }

    /// Steps crumbling ledges: `standing_on` is the ledge the Hero is on;
    /// `factor` stretches the time before one falls (Mag boots).
    pub fn tick_crumbles(
        &mut self,
        standing_on: Option<usize>,
        factor: f32,
        t: &TerrainTuning,
        dt: f32,
    ) {
        for (i, state) in self.crumbles.iter_mut().enumerate() {
            if !self.ledges[i].crumbles() {
                continue;
            }
            *state = match *state {
                Crumble::Whole if standing_on == Some(i) => {
                    Crumble::Shaking(t.crumble_seconds * factor.max(0.0))
                }
                Crumble::Shaking(left) if left - dt <= 0.0 => {
                    Crumble::Gone(t.crumble_respawn_seconds)
                }
                Crumble::Shaking(left) => Crumble::Shaking(left - dt),
                Crumble::Gone(left) if left - dt <= 0.0 => Crumble::Whole,
                Crumble::Gone(left) => Crumble::Gone(left - dt),
                whole => whole,
            };
        }
    }

    /// A rewind: every crumbling ledge is back and the gusts start over.
    pub fn reset(&mut self) {
        self.crumbles.fill(Crumble::Whole);
        self.clock = 0.0;
    }

    /// The gust zone a body with its feet at height `feet` is in, if any.
    pub fn gust_at(&self, feet: f32) -> Option<&Gust> {
        self.gusts
            .iter()
            .find(|g| g.from_h <= feet && feet < g.to_h)
    }

    /// Where a gust zone is in its cycle now: arrows for `warning_seconds`,
    /// then wind for `blow_seconds`, then calm until `period` is up. The
    /// zone's `offset` delays its first warning.
    pub fn gust_phase(&self, gust: &Gust, h: &super::tuning::Gust) -> GustPhase {
        let p = (self.clock - gust.offset).rem_euclid(gust.period.max(0.01));
        if self.clock < gust.offset {
            GustPhase::Calm
        } else if p < h.warning_seconds {
            GustPhase::Warning
        } else if p < h.warning_seconds + h.blow_seconds {
            GustPhase::Blowing
        } else {
            GustPhase::Calm
        }
    }

    /// The wind's push at height `feet`, px/s (signed; 0 when calm).
    pub fn push_at(&self, feet: f32, h: &super::tuning::Gust) -> f32 {
        match self.gust_at(feet) {
            Some(g) if self.gust_phase(g, h) == GustPhase::Blowing => {
                if g.dir == "left" {
                    -h.push_speed
                } else {
                    h.push_speed
                }
            }
            _ => 0.0,
        }
    }
}

impl TileQuery for Tower {
    /// No ground in a tower, only the face's edges as walls.
    fn ground_top(&self, x0: f32, x1: f32) -> Option<f32> {
        (x0 < -self.face || x1 > self.face).then_some(WALL)
    }

    fn ledge_tops(&self, x0: f32, x1: f32) -> Vec<f32> {
        self.ledges
            .iter()
            .enumerate()
            .filter(|(i, l)| !Self::is_solid(l) && Self::overlaps(l, x0, x1) && self.standing(*i))
            .map(|(_, l)| l.h)
            .collect()
    }

    fn solid_tops(&self, x0: f32, x1: f32) -> Vec<f32> {
        self.ledges
            .iter()
            .filter(|l| Self::is_solid(l) && Self::overlaps(l, x0, x1))
            .map(|l| l.h)
            .collect()
    }

    fn ceiling(&self, x0: f32, x1: f32, from: f32, to: f32) -> Option<f32> {
        self.ledges
            .iter()
            .filter(|l| Self::is_solid(l) && Self::overlaps(l, x0, x1))
            .map(|l| l.h - TILE)
            .filter(|under| *under >= from - 0.01 && *under < to)
            .reduce(f32::min)
    }

    fn ladder_near(&self, x: f32, y: f32, window: f32) -> Option<Ladder> {
        self.ladders
            .iter()
            .filter(|d| (d.x - x).abs() <= window && d.bottom - 0.5 <= y && y <= d.top + 0.5)
            .min_by(|a, b| (a.x - x).abs().total_cmp(&(b.x - x).abs()))
            .copied()
    }
}

/// A drawn crumbling ledge: its index in `Tower::ledges`. Other ledges and
/// ladders are drawn on the kit's scenery layer.
#[derive(Component)]
pub(super) struct CrumblePiece(usize);

/// Draws each crumbling ledge as a row of tiles, placed from the camera by
/// [`place_crumbles`], so it can shake and fall away.
pub(super) fn spawn_crumbles(
    commands: &mut Commands,
    tower: &Tower,
    sprite: &str,
    sprites: &crate::sprites::Sprites,
    cabinet: usize,
) {
    for (index, l) in tower.ledges.iter().enumerate() {
        if !l.crumbles() {
            continue;
        }
        let width = l.to_x - l.from_x;
        let mut parent = commands.spawn((
            CrumblePiece(index),
            Transform::from_xyz((l.from_x + l.to_x) / 2.0, 0.0, -60.0),
            Visibility::Hidden,
            DespawnOnExit(Screen::Playing),
        ));
        parent.with_children(|row| {
            let mut x = 0.0;
            while x < width {
                row.spawn((
                    sprites.sprite(sprite, cabinet),
                    Transform::from_xyz(x + TILE / 2.0 - width / 2.0, 0.0, 0.0),
                ));
                x += TILE;
            }
        });
    }
}

/// Places crumbling ledges from the camera; shaking ones jitter and fallen
/// ones are hidden.
pub(super) fn place_crumbles(
    tower: Res<Tower>,
    stage: Res<crate::kit::StageRun>,
    time: Res<Time>,
    mut pieces: Query<(&CrumblePiece, &mut Transform, &mut Visibility)>,
) {
    let camera = stage.cursor.position;
    for (piece, mut transform, mut visibility) in &mut pieces {
        let l = &tower.ledges[piece.0];
        let state = tower.crumbles[piece.0];
        let y = screen_y(l.h, camera) - TILE / 2.0;
        let on = y.abs() < HALF_H + TILE && !matches!(state, Crumble::Gone(_));
        let shake = match state {
            Crumble::Shaking(_) => ((time.elapsed_secs() * 30.0) as i32 % 2) as f32,
            _ => 0.0,
        };
        transform.translation.x = (l.from_x + l.to_x) / 2.0 + shake;
        transform.translation.y = y.round();
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::platform::{self, Body, Control};

    const STAGE: &str = r#"(
        length: 1000.0,
        speed: [(from: 0.0, speed: 14.0)],
        ledges: [
            (h: 0.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
            (h: 40.0, from_x: -128.0, to_x: -64.0, kind: "thin"),
            (h: 72.0, from_x: -24.0, to_x: 40.0, kind: "crumble"),
            (h: 80.0, from_x: 80.0, to_x: 112.0, kind: "thin"),
            (h: 160.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
        ],
        ladders: [(x: 92.0, from_h: 0.0, to_h: 80.0), (x: 100.0, from_h: 80.0, to_h: 160.0)],
        gusts: [(from_h: 100.0, to_h: 150.0, dir: "left", period: 6.0, offset: 1.0)],
        route: [(x: -100.0, h: 0.0), (x: -96.0, h: 40.0)],
    )"#;

    fn tower() -> Tower {
        Tower::parse(STAGE, &ClimberTuning::default()).unwrap()
    }

    const DT: f32 = 1.0 / 240.0;

    fn t() -> ClimberTuning {
        ClimberTuning::default()
    }

    fn body(x: f32, feet: f32) -> Body {
        let mut b = Body::new(x, feet, 5.0);
        b.height = 20.0;
        b.grounded = true;
        b
    }

    /// Runs `frames` frames of the Hero's physics: ladder first, then the
    /// platform step.
    fn run(b: &mut Body, w: &Tower, input: impl Fn(usize) -> Control, frames: usize) {
        let t = t();
        for i in 0..frames {
            let c = input(i);
            if platform::climb(b, c, &t.ladder.params(), &t.jump.params(), w, DT).is_none() {
                platform::step(b, c, &t.jump.params(), w, DT);
            }
        }
    }

    #[test]
    fn heights_and_screen_y_round_trip() {
        assert_eq!(screen_y(0.0, 184.0), -64.0);
        assert_eq!(tower_y(screen_y(37.0, 512.0), 512.0), 37.0);
    }

    #[test]
    fn the_face_edges_are_walls() {
        let w = tower();
        let mut b = body(100.0, 0.0);
        run(
            &mut b,
            &w,
            |_| Control {
                run: 72.0,
                ..Control::default()
            },
            480,
        );
        assert!((b.x + 5.0 - 128.0).abs() < 0.3, "at the edge: {}", b.x);
        assert!(b.grounded && b.feet == 0.0);
    }

    #[test]
    fn a_landing_is_never_dropped_through_but_a_thin_ledge_is() {
        let w = tower();
        let drop = |_| Control {
            drop_through: true,
            ..Control::default()
        };
        let mut on_landing = body(0.0, 0.0);
        run(&mut on_landing, &w, drop, 120);
        assert!(on_landing.grounded && on_landing.feet == 0.0);
        let mut on_thin = body(-100.0, 40.0);
        run(&mut on_thin, &w, drop, 1);
        run(&mut on_thin, &w, |_| Control::default(), 240);
        assert!(on_thin.grounded && on_thin.feet == 0.0, "{on_thin:?}");
    }

    #[test]
    fn a_landing_bumps_a_head_from_below() {
        let w = tower();
        // On the thin ledge at 80, the landing at 160 has its underside at
        // 152: a full jump (48 px) would reach 148 + 20 = 168 at the head.
        let mut b = body(96.0, 80.0);
        b.x = 108.0;
        let mut peak: f32 = 0.0;
        let t = t();
        for i in 0..300 {
            let c = Control {
                jump_pressed: i == 0,
                jump_held: true,
                ..Control::default()
            };
            platform::step(&mut b, c, &t.jump.params(), &w, DT);
            peak = peak.max(b.feet);
        }
        assert!(peak <= 152.0 - 20.0 + 0.01, "head stopped at 152: {peak}");
        assert!(b.grounded && b.feet == 80.0);
    }

    #[test]
    fn a_ladder_is_climbed_to_the_ledge_at_its_top_and_back_down() {
        let w = tower();
        let mut b = body(88.0, 0.0);
        let up = |_| Control {
            climb: 1.0,
            ..Control::default()
        };
        run(&mut b, &w, up, 10);
        assert!(b.ladder.is_some() && b.x == 92.0, "snapped on: {b:?}");
        // 80 px at 48 px/s is 1.67 s.
        run(&mut b, &w, up, 400);
        assert!(b.grounded && b.ladder.is_none() && b.feet == 80.0, "{b:?}");
        let down = |_| Control {
            climb: -1.0,
            ..Control::default()
        };
        run(&mut b, &w, down, 10);
        assert!(b.ladder.is_some() && b.feet < 80.0, "back on: {b:?}");
        run(&mut b, &w, down, 600);
        assert!(b.grounded && b.feet == 0.0, "down at the bottom: {b:?}");
    }

    #[test]
    fn the_grab_window_is_six_pixels_and_works_in_the_air() {
        let w = tower();
        let up = |_| Control {
            climb: 1.0,
            ..Control::default()
        };
        let mut far = body(85.0, 0.0);
        run(&mut far, &w, up, 1);
        assert!(far.ladder.is_none(), "7 px away");
        let mut falling = body(95.0, 50.0);
        falling.grounded = false;
        falling.vy = -100.0;
        run(&mut falling, &w, up, 1);
        assert!(falling.ladder.is_some(), "caught in the air");
    }

    #[test]
    fn jumping_off_a_ladder_sideways_or_letting_go() {
        let w = tower();
        let mut b = body(92.0, 0.0);
        run(
            &mut b,
            &w,
            |_| Control {
                climb: 1.0,
                ..Control::default()
            },
            120,
        );
        let mut side = b;
        run(
            &mut side,
            &w,
            |i| Control {
                run: -72.0,
                jump_pressed: i == 0,
                jump_held: true,
                ..Control::default()
            },
            1,
        );
        assert!(side.ladder.is_none() && side.vy == 240.0 * 0.6);
        let mut go = b;
        run(
            &mut go,
            &w,
            |i| Control {
                jump_pressed: i == 0,
                ..Control::default()
            },
            1,
        );
        assert!(go.ladder.is_none() && go.vy == 0.0);
        run(&mut go, &w, |_| Control::default(), 240);
        assert!(go.grounded && go.feet == 0.0, "fell to the landing");
    }

    #[test]
    fn crumbling_ledges_fall_later_with_mag_boots_and_come_back() {
        let tt = TerrainTuning::default();
        let mut w = tower();
        w.tick_crumbles(Some(2), 2.0, &tt, 0.1);
        assert_eq!(w.crumbles[2], Crumble::Shaking(1.0), "0.5 s doubled");
        w.tick_crumbles(None, 2.0, &tt, 0.95);
        assert!(!w.ledge_tops(0.0, 1.0).is_empty(), "still there at 0.95 s");
        w.tick_crumbles(None, 2.0, &tt, 0.1);
        assert!(w.ledge_tops(0.0, 1.0).is_empty(), "fallen");
        for _ in 0..31 {
            w.tick_crumbles(None, 1.0, &tt, 0.1);
        }
        assert_eq!(w.ledge_tops(0.0, 1.0), [72.0], "back after 3 s");
        w.tick_crumbles(Some(2), 1.0, &tt, 0.1);
        w.reset();
        assert_eq!(w.crumbles[2], Crumble::Whole);
    }

    #[test]
    fn gusts_warn_then_blow_on_their_own_cycle() {
        let h = super::super::tuning::Gust::default();
        let mut w = tower();
        let g = w.gusts[0].clone();
        let at = |w: &mut Tower, s: f32| {
            w.clock = s;
            w.gust_phase(&g, &h)
        };
        assert_eq!(at(&mut w, 0.5), GustPhase::Calm, "before its offset");
        assert_eq!(at(&mut w, 1.2), GustPhase::Warning);
        assert_eq!(at(&mut w, 2.0), GustPhase::Blowing);
        assert_eq!(w.push_at(120.0, &h), -36.0, "blowing left");
        assert_eq!(w.push_at(90.0, &h), 0.0, "below the zone");
        assert_eq!(at(&mut w, 4.0), GustPhase::Calm);
        assert_eq!(at(&mut w, 7.3), GustPhase::Warning, "every 6 s");
    }

    #[test]
    fn solid_ledges_stop_falling_things_and_thin_ones_do_not() {
        let w = tower();
        assert_eq!(w.floor_below(-100.0, 60.0), Some(40.0));
        assert!(w.in_solid(0.0, -3.0) && !w.in_solid(-100.0, 38.0));
    }
}
