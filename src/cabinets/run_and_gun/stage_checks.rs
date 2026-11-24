//! Checks on Cabinet 3's Stage files against its design doc
//! (docs/cabinets/3-run-and-gun.md): lengths, Restart Points, Power-up order,
//! enemy kinds and params, and the terrain's fairness test, every gap and
//! step against the base jump.

use std::collections::BTreeSet;

use ron::Value;

use super::STAGES;
use super::terrain::{StageTerrain, surface};
use crate::display::HALF_W;
use crate::kit::StageTimeline;
use crate::kit::scenery::{SCENERY_PREFIX, pieces};
use crate::kit::timeline::SpawnEntry;
use crate::sprites::{is_background, sprite_text};

// The doc's keys (`run_and_gun.*`), until #19 adds them to tuning.ron.
const RUN_SPEED: f32 = 72.0;
const JUMP_SPEED: f32 = 240.0;
const GRAVITY_UP: f32 = 600.0;
const GRAVITY_DOWN: f32 = 900.0;
const MAX_FALL: f32 = 300.0;
const MAX_GAP: f32 = 40.0;
const MAX_STEP: f32 = 40.0;
/// Room to spare at take-off and landing for a gap with a rise.
const SPARE: f32 = 8.0;
const RESTART_PIT_CLEARANCE: f32 = 48.0;
const RESTART_HAZARD_CLEARANCE: f32 = 48.0;
const PIT_FREE_RUN: f32 = 64.0;
/// The camera's push line: where the Hero stands while the camera follows.
const PUSH_X: f32 = -16.0;
/// The Hero's terrain box is 20 px tall.
const HERO_HEIGHT: f32 = 20.0;

const WALKERS: [&str; 6] = ["skitter", "tick", "sentry", "burrower", "hauler", "strider"];
const FLYERS: [&str; 3] = ["kite", "dropper", "mote"];
const HAZARDS: [&str; 3] = ["crusher", "arc_floor", "conveyor"];
/// Kinds that only appear in Stage 2.
const STAGE_2_ONLY: [&str; 6] = [
    "turret",
    "mote",
    "strider",
    "crusher",
    "arc_floor",
    "conveyor",
];
/// Not scenery colours, on purpose: the wreck's Ember flicker is a
/// foreground effect (see the doc's Palette rules).
const FOREGROUND_SCENERY: [&str; 1] = ["c3_flame"];

struct Stage {
    name: &'static str,
    timeline: StageTimeline,
    terrain: StageTerrain,
}

fn stages() -> Vec<Stage> {
    ["3-1", "3-2"]
        .into_iter()
        .zip(STAGES)
        .map(|(name, text)| Stage {
            name,
            timeline: StageTimeline::parse(text).unwrap_or_else(|e| panic!("{name}: {e}")),
            terrain: StageTerrain::parse(text).unwrap_or_else(|e| panic!("{name}: {e}")),
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

/// An entry's spawns, with a Standoff's waves as entries of their own
/// placed where they appear (the camera holds at the Standoff's `at`).
fn enemy_entries(spawns: &[SpawnEntry]) -> Vec<SpawnEntry> {
    let mut out = Vec::new();
    for s in spawns {
        if s.kind.starts_with(SCENERY_PREFIX) {
            continue;
        }
        if s.kind == "standoff" {
            let Some(Value::Seq(waves)) = param(&s.params, "waves") else {
                panic!("Standoff at {} has no waves", s.at);
            };
            for w in waves {
                let Value::Map(w) = w else {
                    panic!("Standoff at {}: a wave isn't a map", s.at)
                };
                let kind = match param(w, "kind") {
                    Some(Value::String(k)) => k.clone(),
                    _ => panic!("Standoff at {}: a wave has no kind", s.at),
                };
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
                out.push(SpawnEntry {
                    at: s.at,
                    kind,
                    pos,
                    params,
                });
            }
        } else {
            out.push(s.clone());
        }
    }
    out
}

/// Scroll position of the screen x `x` when the camera is at `at`.
fn world_x(at: f32, x: f32) -> f32 {
    at + x - HALF_W
}

fn standoffs(t: &StageTimeline) -> Vec<&SpawnEntry> {
    t.spawns.iter().filter(|s| s.kind == "standoff").collect()
}

/// Seconds at pace: scrolling at full speed, and each Standoff its full time.
fn pace_seconds(t: &StageTimeline) -> f32 {
    t.duration()
        + standoffs(t)
            .iter()
            .map(|s| param(&s.params, "max_seconds").and_then(number).unwrap())
            .sum::<f32>()
}

/// The highest surface at scroll position x: ground or a ledge, or the
/// bottom of the play area over a pit.
fn surface_at(terrain: &StageTerrain, x: f32) -> Option<f32> {
    let ground = terrain.ground_at(x).map(|g| surface(g.top));
    let ledge = terrain
        .ledges
        .iter()
        .filter(|l| l.from <= x && x < l.to)
        .map(|l| l.surface())
        .reduce(f32::max);
    match (ground, ledge) {
        (Some(g), Some(l)) => Some(g.max(l)),
        (g, l) => g.or(l),
    }
}

/// How high a full jump is (above take-off) after travelling `d` px at run
/// speed, stepped at 240 Hz with the doc's jump keys.
fn jump_height_at(d: f32) -> f32 {
    let dt = 1.0 / 240.0;
    let (mut x, mut y, mut vy) = (0.0f32, 0.0f32, JUMP_SPEED);
    while x < d {
        let g = if vy > 0.0 { GRAVITY_UP } else { GRAVITY_DOWN };
        vy = (vy - g * dt).max(-MAX_FALL);
        y += vy * dt;
        x += RUN_SPEED * dt;
    }
    y
}

#[test]
fn the_jump_matches_the_doc() {
    // 48 px high, 52 px across.
    let top = (1..=60)
        .map(|d| jump_height_at(d as f32))
        .fold(0.0, f32::max);
    assert!((top - 48.0).abs() < 1.0, "{top}");
    assert!(jump_height_at(51.0) > 0.0 && jump_height_at(54.0) < 0.0);
}

#[test]
fn both_stages_last_three_to_four_minutes_at_pace_and_the_short_end() {
    for (stage, want) in stages().iter().zip([190.0, 195.0]) {
        let seconds = pace_seconds(&stage.timeline);
        assert!(
            (180.0..=240.0).contains(&seconds),
            "{}: {seconds}",
            stage.name
        );
        // #65: Cabinet 1's Stages felt long, so these aim at the short end.
        assert!((seconds - want).abs() < 1.0, "{}: {seconds}", stage.name);
    }
}

#[test]
fn restart_points_stand_on_safe_ground_outside_standoffs() {
    for stage in stages() {
        let t = &stage.timeline;
        let terrain = &stage.terrain;
        assert_eq!(t.restart_points.len(), 4, "{}", stage.name);
        let hazards: Vec<(f32, f32)> = enemy_entries(&t.spawns)
            .iter()
            .filter(|s| s.kind == "crusher" || s.kind == "arc_floor")
            .map(|s| {
                let x = world_x(s.at, s.pos.0);
                let len = param(&s.params, "length").and_then(number).unwrap_or(24.0);
                (x - len / 2.0, x + len / 2.0)
            })
            .collect();
        for p in &t.restart_points {
            assert_eq!(p.hero, (-96.0, -86.0), "{} at {}", stage.name, p.at);
            let x = world_x(p.at, p.hero.0);
            assert!(
                terrain.ground_at(x).is_some(),
                "{}: Restart Point at {} over a pit",
                stage.name,
                p.at
            );
            for (from, to) in terrain.pits() {
                assert!(
                    to <= x - RESTART_PIT_CLEARANCE || from >= x + PIT_FREE_RUN,
                    "{}: pit {from}..{to} too near the Restart Point at {}",
                    stage.name,
                    p.at
                );
            }
            for &(from, to) in &hazards {
                assert!(
                    to <= x - RESTART_HAZARD_CLEARANCE || from >= x + RESTART_HAZARD_CLEARANCE,
                    "{}: a hazard at {from}..{to} too near the Restart Point at {}",
                    stage.name,
                    p.at
                );
            }
            assert!(
                standoffs(t).iter().all(|s| s.at != p.at),
                "{}: Restart Point at {} is a Standoff",
                stage.name,
                p.at
            );
        }
        for s in standoffs(t) {
            let x = world_x(s.at, PUSH_X);
            for (from, to) in terrain.pits() {
                assert!(
                    to <= x - PIT_FREE_RUN || from >= x + PIT_FREE_RUN,
                    "{}: pit {from}..{to} within {PIT_FREE_RUN} px of the Standoff at {}",
                    stage.name,
                    s.at
                );
            }
        }
    }
}

#[test]
fn every_gap_and_step_is_cleared_by_the_base_jump() {
    for stage in stages() {
        for pair in stage.terrain.ground.windows(2) {
            let (a, b) = (&pair[0], &pair[1]);
            let gap = b.from - a.to;
            let rise = surface(b.top) - surface(a.top);
            let at = format!("{}: {}..{}", stage.name, a.to, b.from);
            assert!(rise <= MAX_STEP, "{at}: a {rise} px step");
            if gap == 0.0 {
                continue;
            }
            if rise == 0.0 {
                assert!(gap <= MAX_GAP, "{at}: a {gap} px pit");
            } else {
                let height = jump_height_at(gap + 2.0 * SPARE);
                assert!(
                    height >= rise,
                    "{at}: the jump is {height} px up there, needs {rise}"
                );
            }
        }
    }
}

#[test]
fn power_ups_come_in_the_doc_order() {
    for (stage, want) in stages().iter().zip(["GFGJWG", "GWJGFW"]) {
        let mut drops: Vec<(f32, String)> = stage
            .timeline
            .spawns
            .iter()
            .filter_map(|s| match param(&s.params, "drop") {
                Some(Value::String(d)) => Some((s.at, d.clone())),
                _ => None,
            })
            .collect();
        drops.sort_by(|a, b| a.0.total_cmp(&b.0));
        let order: String = drops.iter().map(|d| d.1.as_str()).collect();
        assert_eq!(order, want, "{}", stage.name);
    }
}

#[test]
fn kinds_and_params_are_known_and_walkers_have_footing() {
    for (i, stage) in stages().iter().enumerate() {
        let terrain = &stage.terrain;
        for s in enemy_entries(&stage.timeline.spawns) {
            let kind = s.kind.as_str();
            let at = format!("{} {kind} at {}", stage.name, s.at);
            let known = WALKERS.contains(&kind)
                || FLYERS.contains(&kind)
                || HAZARDS.contains(&kind)
                || kind == "turret";
            assert!(known, "{at}: unknown kind");
            assert!(
                i == 1 || !STAGE_2_ONLY.contains(&kind),
                "{at}: a Stage 2 kind"
            );
            if let Some(size) = param(&s.params, "size") {
                assert!(
                    matches!(size, Value::Number(n) if n.into_f64() >= 1.0 && n.into_f64().fract() == 0.0),
                    "{at}: size must be a whole number"
                );
            }
            if let Some(drop) = param(&s.params, "drop") {
                assert!(
                    matches!(drop, Value::String(d) if ["G", "F", "J", "W", "H"].contains(&d.as_str())),
                    "{at}: unknown drop"
                );
            }
            let x = world_x(s.at, s.pos.0);
            match kind {
                "turret" => {
                    let mount = param(&s.params, "mount");
                    assert!(
                        matches!(mount, Some(Value::String(m)) if ["floor", "ceiling", "wall"].contains(&m.as_str())),
                        "{at}: needs a mount"
                    );
                    // The aimed-shot rule: always above the Hero's head.
                    let ground = terrain.ground_at(x).map_or(-120.0, |g| surface(g.top));
                    assert!(
                        s.pos.1 - 6.0 > ground + HERO_HEIGHT,
                        "{at}: below a standing Hero's head"
                    );
                }
                "crusher" => {
                    assert!(
                        param(&s.params, "offset").and_then(number).is_some(),
                        "{at}: offset"
                    );
                }
                "arc_floor" | "conveyor" => {
                    let len = param(&s.params, "length").and_then(number).unwrap_or(0.0);
                    assert!((32.0..=128.0).contains(&len), "{at}: length {len}");
                    if kind == "arc_floor" {
                        assert!(len <= 64.0, "{at}: arc floors are 32-64 px");
                        assert!(
                            param(&s.params, "offset").and_then(number).is_some(),
                            "{at}: offset"
                        );
                    } else {
                        assert!(
                            matches!(param(&s.params, "dir"), Some(Value::String(d)) if d == "left" || d == "right"),
                            "{at}: dir"
                        );
                    }
                    for edge in [x - len / 2.0, x + len / 2.0 - 1.0] {
                        assert!(terrain.ground_at(edge).is_some(), "{at}: not all on ground");
                    }
                }
                "strider" => {
                    assert!(
                        stage
                            .timeline
                            .spawns
                            .iter()
                            .any(|t| t.kind == "standoff" && t.at == s.at),
                        "{at}: the Strider only comes in a Standoff"
                    );
                }
                _ => {}
            }
            if WALKERS.contains(&kind) || kind == "crusher" {
                assert!(
                    surface_at(terrain, x).is_some(),
                    "{at}: nothing to stand on at {x}"
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
            is_background(&name) || FOREGROUND_SCENERY.contains(&name.as_str()),
            "{name} isn't marked as background art"
        );
    }
}
