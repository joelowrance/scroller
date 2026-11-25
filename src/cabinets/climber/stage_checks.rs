//! Checks on Cabinet 4's Stage files against its design doc
//! (docs/cabinets/4-climber.md): lengths, Restart Points and Standoffs on
//! landings, Power-up order, enemy kinds and params, and the route's
//! fairness: every step a base move, and, timed at base speed, always
//! ahead of the rising bottom edge.

use std::collections::BTreeSet;

use ron::Value;

use super::STAGES;
use super::terrain::{FACE_HALF_WIDTH, Foothold, StageTerrain, screen_y};
use crate::display::HALF_H;
use crate::kit::StageTimeline;
use crate::kit::scenery::{SCENERY_PREFIX, pieces};
use crate::kit::timeline::SpawnEntry;
use crate::sprites::{is_background, sprite_text};

// The doc's keys (`climber.*`), until #24 adds them to tuning.ron.
const RUN_SPEED: f32 = 72.0;
const JUMP_SPEED: f32 = 240.0;
const GRAVITY_UP: f32 = 600.0;
const GRAVITY_DOWN: f32 = 900.0;
const CLIMB_SPEED: f32 = 48.0;
const FALL_LINE: f32 = -120.0;
const HERO_HEIGHT: f32 = 20.0;
const HERO_WIDTH: f32 = 10.0;
const MAX_RISE: f32 = 40.0;
const MAX_GAP: f32 = 40.0;
const JUMP_SETUP_SECONDS: f32 = 0.3;
const DROP_MARGIN: f32 = 48.0;
const LOOKAHEAD: f32 = 16.0;
const RESTART_CLEAR_RISE: f32 = 64.0;
const PICKET_LEVEL_WINDOW: f32 = 16.0;
const SWEEPER_LENGTH: f32 = 48.0;
/// A Plumb's spawn x stays this far from the route (the doc's Plumb rule).
const PLUMB_CLEARANCE: f32 = 48.0;
/// A gust zone may only cover crumbling ledges at least this wide.
const GUST_CRUMBLE_MIN_WIDTH: f32 = 32.0;
/// The top of the play area, under the HUD.
const PLAY_TOP: f32 = 108.0;

const LEDGE_ENEMIES: [&str; 4] = ["limpet", "picket", "pitcher", "winch"];
const SCREEN_ENEMIES: [&str; 5] = ["flit", "scrap", "scrapfall", "plumb", "mote"];
const FACE_THINGS: [&str; 2] = ["vent", "sweeper"];
/// Kinds that only appear in Stage 2.
const STAGE_2_ONLY: [&str; 3] = ["mote", "winch", "sweeper"];
const POWER_UPS: [&str; 5] = ["G", "W", "J", "C", "M"];

struct Stage {
    name: &'static str,
    timeline: StageTimeline,
    terrain: StageTerrain,
    standoffs: Vec<(f32, f32)>,
}

fn stages() -> Vec<Stage> {
    ["4-1", "4-2"]
        .into_iter()
        .zip(STAGES)
        .map(|(name, text)| {
            let timeline = StageTimeline::parse(text).unwrap_or_else(|e| panic!("{name}: {e}"));
            let standoffs = timeline
                .spawns
                .iter()
                .filter(|s| s.kind == "standoff")
                .map(|s| {
                    (
                        s.at,
                        param(&s.params, "max_seconds").and_then(number).unwrap(),
                    )
                })
                .collect();
            Stage {
                name,
                terrain: StageTerrain::parse(text).unwrap_or_else(|e| panic!("{name}: {e}")),
                timeline,
                standoffs,
            }
        })
        .collect()
}

fn param<'a>(map: &'a ron::Map, key: &str) -> Option<&'a Value> {
    map.get(&Value::String(key.into()))
}

fn number(v: &Value) -> Option<f32> {
    match v {
        Value::Number(n) => Some(n.into_f64() as f32),
        _ => None,
    }
}

fn text<'a>(map: &'a ron::Map, key: &str) -> Option<&'a str> {
    match param(map, key) {
        Some(Value::String(s)) => Some(s),
        _ => None,
    }
}

/// A spawn, with whether it is one of a Standoff's waves.
struct Enemy {
    entry: SpawnEntry,
    in_standoff: bool,
}

/// The Stage's spawns, with a Standoff's waves as entries of their own at
/// the Standoff's `at` (the camera holds there).
fn enemies(spawns: &[SpawnEntry]) -> Vec<Enemy> {
    let mut out = Vec::new();
    for s in spawns {
        if s.kind.starts_with(SCENERY_PREFIX) {
            continue;
        }
        if s.kind != "standoff" {
            out.push(Enemy {
                entry: s.clone(),
                in_standoff: false,
            });
            continue;
        }
        let Some(Value::Seq(waves)) = param(&s.params, "waves") else {
            panic!("Standoff at {} has no waves", s.at);
        };
        for w in waves {
            let Value::Map(w) = w else {
                panic!("Standoff at {}: a wave isn't a map", s.at)
            };
            let kind = text(w, "kind")
                .unwrap_or_else(|| panic!("Standoff at {}: a wave has no kind", s.at))
                .to_string();
            let pos = match param(w, "pos") {
                Some(Value::Seq(p)) if p.len() == 2 => {
                    (number(&p[0]).unwrap(), number(&p[1]).unwrap())
                }
                _ => panic!("Standoff at {}: wave {kind} has no pos", s.at),
            };
            assert!(
                param(w, "after").and_then(number).is_some_and(|a| a >= 0.0),
                "Standoff at {}: wave {kind} needs an after",
                s.at
            );
            let params = match param(w, "params") {
                Some(Value::Map(m)) => m.clone(),
                _ => ron::Map::new(),
            };
            out.push(Enemy {
                entry: SpawnEntry {
                    at: s.at,
                    kind,
                    pos,
                    params,
                },
                in_standoff: true,
            });
        }
    }
    out
}

impl Stage {
    /// Seconds at pace: the scroll, plus each Standoff's full timer.
    fn pace_seconds(&self) -> f32 {
        self.timeline.duration() + self.standoffs.iter().map(|s| s.1).sum::<f32>()
    }

    fn speed(&self) -> f32 {
        assert_eq!(
            self.timeline.speed.len(),
            1,
            "{}: one scroll speed",
            self.name
        );
        self.timeline.speed[0].speed
    }

    /// The camera at `t` seconds, holding at each Standoff for its timer.
    fn camera_at(&self, t: f32) -> f32 {
        let (mut camera, mut left) = (0.0, t);
        for &(at, seconds) in &self.standoffs {
            let travel = (at - camera) / self.speed();
            if left <= travel {
                return camera + left * self.speed();
            }
            left -= travel;
            camera = at;
            if left <= seconds {
                return camera;
            }
            left -= seconds;
        }
        (camera + left * self.speed()).min(self.timeline.length)
    }

    /// The first time the camera reaches `camera`.
    fn time_of(&self, camera: f32) -> f32 {
        camera / self.speed()
            + self
                .standoffs
                .iter()
                .filter(|s| camera > s.0)
                .map(|s| s.1)
                .sum::<f32>()
    }

    /// The landing under x whose surface is on screen, low down, when the
    /// camera is at `camera`.
    fn landing_under(&self, x: f32, camera: f32) -> Option<f32> {
        self.terrain
            .ledges
            .iter()
            .filter(|l| l.is_landing() && l.covers(x))
            .map(|l| l.h)
            .filter(|&h| (FALL_LINE..=0.0).contains(&screen_y(h, camera)))
            .reduce(f32::max)
    }

    /// The ledge a ledge enemy spawned at `at` stands on: the highest under
    /// x at or below the top edge.
    fn footing(&self, x: f32, at: f32) -> Option<f32> {
        self.terrain
            .ledges
            .iter()
            .filter(|l| l.covers(x) && l.h <= at && screen_y(l.h, at) >= FALL_LINE)
            .map(|l| l.h)
            .reduce(f32::max)
    }
}

/// Seconds from take-off to landing `rise` px higher, on the way down.
fn airtime(rise: f32) -> f32 {
    let apex = JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY_UP);
    JUMP_SPEED / GRAVITY_UP + (2.0 * (apex - rise) / GRAVITY_DOWN).sqrt()
}

#[derive(Debug, PartialEq)]
enum Move {
    Walk,
    Jump,
    Drop,
    Ladder,
}

/// What kind of base move takes the Hero from `a` to `b`, or why none does.
fn base_move(terrain: &StageTerrain, a: Foothold, b: Foothold) -> Result<Move, String> {
    let rise = b.h - a.h;
    let dx = (b.x - a.x).abs();
    if a.x == b.x && rise != 0.0 && terrain.ladder_between(a.x, a.h, b.h).is_some() {
        return Ok(Move::Ladder);
    }
    if terrain.ledge_at(b.x, b.h).is_none() {
        return Err("lands on nothing".into());
    }
    if rise == 0.0 {
        return Ok(Move::Walk);
    }
    if rise < 0.0 {
        let through = terrain
            .ledges
            .iter()
            .any(|l| l.kind != "thin" && b.h < l.h && l.h < a.h && l.covers(b.x));
        return if through {
            Err("drops through a solid ledge".into())
        } else {
            Ok(Move::Drop)
        };
    }
    if rise > MAX_RISE {
        return Err(format!("a {rise} px rise"));
    }
    if dx > MAX_GAP {
        return Err(format!("a {dx} px jump across"));
    }
    if dx > RUN_SPEED * airtime(rise) + HERO_WIDTH {
        return Err("out of the jump's reach".into());
    }
    // A full jump's head must clear every landing above (they're solid).
    let apex_head = a.h + JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY_UP) + HERO_HEIGHT;
    if terrain.ledges.iter().any(|l| {
        matches!(l.kind.as_str(), "landing" | "solid")
            && (l.covers(a.x) || l.covers(b.x))
            && l.h > a.h
            && l.h - 8.0 < apex_head
    }) {
        return Err("a landing above is in the way".into());
    }
    Ok(Move::Jump)
}

/// Follows the route from foothold `from` at time `t` with the base moves,
/// waiting for each higher foothold to be on screen first. Returns the
/// smallest gap between the Hero's feet and the bottom edge, and where.
fn follow_route(stage: &Stage, from: usize, mut t: f32) -> (f32, String) {
    let route = &stage.terrain.route;
    let mut worst = (f32::MAX, String::new());
    let mut note = |feet: f32, t: f32, at: &str| {
        let gap = feet - (stage.camera_at(t) - HALF_H + FALL_LINE);
        if gap < worst.0 {
            worst = (gap, format!("{at} at {t:.1} s"));
        }
    };
    for i in from..route.len() - 1 {
        let (a, b) = (route[i], route[i + 1]);
        if b.h > a.h {
            // Wait until the foothold is LOOKAHEAD px below the top.
            while stage.camera_at(t) - HALF_H + PLAY_TOP - LOOKAHEAD < b.h
                && stage.camera_at(t) < stage.timeline.length
            {
                t += 0.01;
            }
        }
        let walk = (b.x - a.x).abs() / RUN_SPEED;
        let (leave, seconds) = match base_move(&stage.terrain, a, b).unwrap() {
            Move::Walk => (walk, walk),
            Move::Ladder => (0.0, (b.h - a.h).abs() / CLIMB_SPEED),
            Move::Jump => (JUMP_SETUP_SECONDS, JUMP_SETUP_SECONDS + airtime(b.h - a.h)),
            Move::Drop => (walk, walk + (2.0 * (a.h - b.h) / GRAVITY_DOWN).sqrt()),
        };
        note(a.h, t + leave, &format!("leaving ({}, {})", a.x, a.h));
        t += seconds;
        note(b.h, t, &format!("reaching ({}, {})", b.x, b.h));
    }
    worst
}

#[test]
fn the_jump_matches_the_doc() {
    // 48 px high, and the 40 px rise still reaches 40 px across.
    assert_eq!(JUMP_SPEED * JUMP_SPEED / (2.0 * GRAVITY_UP), 48.0);
    assert!(RUN_SPEED * airtime(MAX_RISE) + HERO_WIDTH >= MAX_GAP);
}

#[test]
fn both_stages_last_the_doc_lengths_at_pace() {
    // 2:45 and 2:55, the short Stages the doc plans after #65 (see #67).
    for (stage, want) in stages().iter().zip([165.0, 175.0]) {
        let seconds = stage.pace_seconds();
        assert!((seconds - want).abs() < 1.0, "{}: {seconds}", stage.name);
    }
}

#[test]
fn restart_points_and_standoffs_stand_on_landings() {
    for stage in stages() {
        let t = &stage.timeline;
        assert_eq!(t.restart_points.len(), 4, "{}", stage.name);
        for p in &t.restart_points {
            let at = format!("{}: Restart Point at {}", stage.name, p.at);
            let h = stage
                .landing_under(p.hero.0, p.at)
                .unwrap_or_else(|| panic!("{at}: no landing"));
            let feet = screen_y(h, p.at);
            assert_eq!(p.hero.1, feet + HERO_HEIGHT / 2.0, "{at}: stand on it");
            assert!(feet - FALL_LINE >= DROP_MARGIN, "{at}: too near the edge");
            assert!(
                stage.standoffs.iter().all(|s| s.0 != p.at),
                "{at}: a Standoff"
            );
            // Nothing pushes the Hero in the first climb above it.
            let clear = (h, h + RESTART_CLEAR_RISE);
            for g in &stage.terrain.gusts {
                assert!(g.to_h <= clear.0 || g.from_h >= clear.1, "{at}: a gust");
            }
            for s in enemies(&t.spawns)
                .iter()
                .filter(|e| e.entry.kind == "sweeper")
            {
                let hub = face_height(&s.entry);
                assert!(
                    hub + SWEEPER_LENGTH <= clear.0 || hub - SWEEPER_LENGTH >= clear.1,
                    "{at}: a Sweeper"
                );
            }
        }
        for &(at, _) in &stage.standoffs {
            let h = stage.landing_under(0.0, at);
            assert!(
                h.is_some_and(|h| screen_y(h, at) - FALL_LINE >= DROP_MARGIN),
                "{}: the Standoff at {at} isn't on a landing",
                stage.name
            );
        }
    }
}

#[test]
fn power_ups_come_in_the_doc_order() {
    for (stage, want) in stages().iter().zip(["GCGJWG", "MGWCJWG"]) {
        let mut drops: Vec<(f32, &str)> = stage
            .timeline
            .spawns
            .iter()
            .filter_map(|s| text(&s.params, "drop").map(|d| (s.at, d)))
            .collect();
        drops.sort_by(|a, b| a.0.total_cmp(&b.0));
        let order: String = drops.iter().map(|d| d.1).collect();
        assert_eq!(order, want, "{}", stage.name);
    }
}

/// The tower height of something fixed in the face: it is at `pos` when the
/// camera is at `at`.
fn face_height(s: &SpawnEntry) -> f32 {
    s.at - HALF_H + s.pos.1
}

#[test]
fn kinds_and_params_are_known_and_ledge_enemies_have_footing() {
    for (i, stage) in stages().iter().enumerate() {
        for e in enemies(&stage.timeline.spawns) {
            let s = &e.entry;
            let kind = s.kind.as_str();
            let at = format!("{} {kind} at {}", stage.name, s.at);
            assert!(
                LEDGE_ENEMIES.contains(&kind)
                    || SCREEN_ENEMIES.contains(&kind)
                    || FACE_THINGS.contains(&kind),
                "{at}: unknown kind"
            );
            assert!(
                i == 1 || !STAGE_2_ONLY.contains(&kind),
                "{at}: a Stage 2 kind"
            );
            assert!(
                kind != "winch" || e.in_standoff,
                "{at}: the Winch only comes in a Standoff"
            );
            if let Some(size) = param(&s.params, "size") {
                assert!(
                    matches!(size, Value::Number(n) if n.into_f64() >= 1.0 && n.into_f64().fract() == 0.0),
                    "{at}: size must be a whole number"
                );
            }
            if let Some(drop) = param(&s.params, "drop") {
                assert!(
                    matches!(drop, Value::String(d) if POWER_UPS.contains(&d.as_str())),
                    "{at}: unknown drop"
                );
            }
            let x = s.pos.0;
            match kind {
                "flit" => {
                    let side = text(&s.params, "side");
                    let below = text(&s.params, "from") == Some("below");
                    assert!(
                        below ^ matches!(side, Some("left" | "right")),
                        "{at}: a side or from below"
                    );
                    if let Some(side) = side {
                        assert_eq!(side == "left", x < 0.0, "{at}: enters on its side");
                    }
                }
                "scrapfall" => {
                    let from = param(&s.params, "from_x").and_then(number).unwrap();
                    let to = param(&s.params, "to_x").and_then(number).unwrap();
                    let seconds = param(&s.params, "seconds").and_then(number).unwrap();
                    assert!(
                        -FACE_HALF_WIDTH <= from && from < to && to <= FACE_HALF_WIDTH,
                        "{at}: span"
                    );
                    assert!(seconds > 0.0, "{at}: seconds");
                }
                "sweeper" => assert!(
                    matches!(text(&s.params, "dir"), Some("cw" | "ccw")),
                    "{at}: dir"
                ),
                _ => {}
            }
            if LEDGE_ENEMIES.contains(&kind) && kind != "winch" {
                assert!(stage.footing(x, s.at).is_some(), "{at}: no ledge under it");
            }
            if !matches!(kind, "flit" | "mote" | "scrapfall") {
                assert!(x.abs() <= FACE_HALF_WIDTH, "{at}: off the face");
            }
        }
    }
}

#[test]
fn the_route_is_made_of_base_moves() {
    for stage in stages() {
        let route = &stage.terrain.route;
        assert!(
            route[0].h < 0.0 && stage.terrain.ledge_at(route[0].x, route[0].h).is_some(),
            "{}: starts on the first landing",
            stage.name
        );
        for pair in route.windows(2) {
            if let Err(why) = base_move(&stage.terrain, pair[0], pair[1]) {
                panic!("{}: {:?} to {:?}: {why}", stage.name, pair[0], pair[1]);
            }
        }
        // It ends on a landing that is on screen when the scroll ends.
        let end = route.last().unwrap();
        assert!(
            stage.landing_under(end.x, stage.timeline.length) == Some(end.h),
            "{}: the route doesn't end on the last landing",
            stage.name
        );
    }
}

#[test]
fn the_route_keeps_ahead_of_the_bottom_edge() {
    for stage in stages() {
        let route = &stage.terrain.route;
        let mut starts = vec![(0, 0.0)];
        for p in &stage.timeline.restart_points {
            let h = stage.landing_under(p.hero.0, p.at).unwrap();
            let i = (0..route.len())
                .filter(|&i| route[i].h == h)
                .min_by(|&a, &b| {
                    (route[a].x - p.hero.0)
                        .abs()
                        .total_cmp(&(route[b].x - p.hero.0).abs())
                })
                .unwrap_or_else(|| {
                    panic!("{}: Restart Point at {} is off the route", stage.name, p.at)
                });
            assert_eq!(
                route[i].x, p.hero.0,
                "{}: Restart Point at {}",
                stage.name, p.at
            );
            starts.push((i, stage.time_of(p.at)));
        }
        for (i, t) in starts {
            let (gap, at) = follow_route(&stage, i, t);
            assert!(
                gap >= DROP_MARGIN,
                "{}: from foothold {i}, only {gap} px above the edge {at}",
                stage.name
            );
        }
    }
}

#[test]
fn plumbs_miss_the_route_and_ladders_miss_picket_lines() {
    for stage in stages() {
        let route = &stage.terrain.route;
        for e in enemies(&stage.timeline.spawns) {
            let s = &e.entry;
            if s.kind == "plumb" && !e.in_standoff {
                // Every route step on screen when it drops.
                let (lo, hi) = (s.at - 2.0 * HALF_H, s.at - HALF_H + PLAY_TOP);
                for pair in route.windows(2) {
                    let (a, b) = (pair[0], pair[1]);
                    if a.h.max(b.h) < lo || a.h.min(b.h) > hi {
                        continue;
                    }
                    let near = if (a.x.min(b.x)..=a.x.max(b.x)).contains(&s.pos.0) {
                        0.0
                    } else {
                        (a.x - s.pos.0).abs().min((b.x - s.pos.0).abs())
                    };
                    assert!(
                        near >= PLUMB_CLEARANCE,
                        "{}: the Plumb at {} drops {near} px from the route",
                        stage.name,
                        s.at
                    );
                }
            }
            if s.kind == "picket" {
                let level = stage.footing(s.pos.0, s.at).unwrap();
                for d in &stage.terrain.ladders {
                    assert!(
                        d.to_h <= level - PICKET_LEVEL_WINDOW
                            || d.from_h >= level + PICKET_LEVEL_WINDOW,
                        "{}: the ladder at x {} is in the Picket at {}'s line",
                        stage.name,
                        d.x,
                        s.at
                    );
                }
            }
        }
    }
}

#[test]
fn gusts_and_crumbling_ledges_are_stage_2_and_never_combine() {
    for (i, stage) in stages().iter().enumerate() {
        let terrain = &stage.terrain;
        if i == 0 {
            assert!(terrain.gusts.is_empty(), "{}: gusts", stage.name);
            assert!(
                !terrain.ledges.iter().any(|l| l.crumbles()),
                "{}: crumbling",
                stage.name
            );
            continue;
        }
        for g in &terrain.gusts {
            for l in terrain.ledges.iter().filter(|l| l.crumbles()) {
                assert!(
                    l.h < g.from_h || l.h >= g.to_h || l.to_x - l.from_x >= GUST_CRUMBLE_MIN_WIDTH,
                    "{}: a narrow crumbling ledge at {} in a gust",
                    stage.name,
                    l.h
                );
            }
        }
    }
}

#[test]
fn scenery_sprites_exist_and_are_background_art() {
    let mut names = BTreeSet::new();
    for stage in stages() {
        for p in pieces(&stage.timeline.spawns).unwrap() {
            names.insert(p.sprite);
        }
        for p in pieces(&stage.terrain.scenery()).unwrap() {
            names.insert(p.sprite);
        }
    }
    for name in names {
        assert!(sprite_text(&name).is_some(), "no sprite {name}");
        assert!(
            is_background(&name),
            "{name} isn't marked as background art"
        );
    }
}
