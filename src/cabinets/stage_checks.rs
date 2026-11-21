//! Checks on the Stage files against their Cabinet design docs, so a Stage
//! edit that breaks a rule fails `cargo test` rather than a playtest.

use crate::kit::StageTimeline;
use crate::kit::scenery::{SCENERY_PREFIX, pieces};
use crate::sprites::{is_background, sprite_text};

const CABINET_2: [(&str, &str); 2] = [
    ("2-1", include_str!("../../assets/stages/2-1.ron")),
    ("2-2", include_str!("../../assets/stages/2-2.ron")),
];

/// Enemy kinds Cabinet 2's Stage files may use (docs/cabinets/2-vertical-shooter.md).
const CABINET_2_KINDS: &[&str] = &[
    "wisp", "swift", "bellows", "loom", "crawler", "silo", "riser", "flak", "mote", "tether",
];
const POWER_UPS: &str = "GPRSWB";
const RESTART_HERO: (f32, f32) = (0.0, -80.0);
/// Scroll a ground target covers from entering at the top until it has left
/// the bottom: the play area plus its own height, with room to spare.
const GROUND_ON_SCREEN: f32 = 262.0;

fn stage(text: &str) -> StageTimeline {
    StageTimeline::parse(text).unwrap()
}

/// Seconds the Stage takes at normal speed.
fn duration(t: &StageTimeline) -> f32 {
    let mut keys: Vec<(f32, f32)> = t.speed.iter().map(|k| (k.from, k.speed)).collect();
    keys.push((t.length, 0.0));
    keys.windows(2)
        .map(|w| (w[1].0.min(t.length) - w[0].0) / w[0].1)
        .sum()
}

fn param<'a>(entry: &'a crate::kit::timeline::SpawnEntry, key: &str) -> Option<&'a ron::Value> {
    entry.params.get(&ron::Value::String(key.into()))
}

#[test]
fn each_cabinet_2_stage_lasts_three_to_four_minutes() {
    for (name, text) in CABINET_2 {
        let seconds = duration(&stage(text));
        assert!((180.0..=240.0).contains(&seconds), "{name}: {seconds} s");
    }
    // The doc's own lengths: 3:20 and 3:36.
    assert!((duration(&stage(CABINET_2[0].1)) - 200.0).abs() < 1.0);
    assert!((duration(&stage(CABINET_2[1].1)) - 216.0).abs() < 1.0);
}

#[test]
fn cabinet_2_restart_points_match_the_doc() {
    for (name, text) in CABINET_2 {
        let t = stage(text);
        assert_eq!(t.restart_points.len(), 4, "{name}");
        for point in &t.restart_points {
            assert_eq!(point.hero, RESTART_HERO, "{name} at {}", point.at);
        }
    }
}

#[test]
fn cabinet_2_spawns_use_known_kinds_and_valid_drops() {
    for (name, text) in CABINET_2 {
        for entry in &stage(text).spawns {
            if let Some(sprite) = entry.kind.strip_prefix(SCENERY_PREFIX) {
                assert!(sprite_text(sprite).is_some(), "{name}: no sprite {sprite}");
                assert!(
                    is_background(sprite),
                    "{name}: {sprite} isn't background art"
                );
                continue;
            }
            assert!(
                CABINET_2_KINDS.contains(&entry.kind.as_str()),
                "{name}: unknown kind {}",
                entry.kind
            );
            if let Some(drop) = param(entry, "drop") {
                let ron::Value::String(d) = drop else {
                    panic!("{name}: drop must be a letter")
                };
                assert!(
                    d.len() == 1 && POWER_UPS.contains(d.as_str()),
                    "{name}: drop {d}"
                );
            }
            if entry.kind == "tether" {
                assert!(param(entry, "left_x").is_some() && param(entry, "right_x").is_some());
            }
            if entry.kind == "bellows" {
                assert!(
                    param(entry, "side").is_some(),
                    "{name}: Bellows needs a side"
                );
            }
        }
        pieces(&stage(text).spawns).unwrap();
    }
}

#[test]
fn never_more_than_two_flak_batteries_on_screen() {
    for (name, text) in CABINET_2 {
        let flak: Vec<f32> = stage(text)
            .spawns
            .iter()
            .filter(|s| s.kind == "flak")
            .map(|s| s.at)
            .collect();
        for &at in &flak {
            let near = flak
                .iter()
                .filter(|&&o| (o - at).abs() < GROUND_ON_SCREEN)
                .count();
            assert!(near <= 2, "{name}: {near} Flak Batteries around {at}");
        }
    }
}

#[test]
fn power_ups_follow_the_doc() {
    // Stage 1: G, P, R, G, B, S, W, P, G. Stage 2: G, S, W, B, P, G, G.
    let expected = ["GPRGBSWPG", "GSWBPGG"];
    for ((name, text), want) in CABINET_2.iter().zip(expected) {
        let got: String = stage(text)
            .spawns
            .iter()
            .filter_map(|s| match param(s, "drop") {
                Some(ron::Value::String(d)) => Some(d.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(got, want, "{name}");
    }
}
