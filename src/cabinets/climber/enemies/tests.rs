//! Spawning and fairness for Cabinet 4's enemies (docs/cabinets/4-climber.md).

use super::*;
use crate::cabinets::climber::STAGES;

const STAGE: &str = r#"(
    length: 1000.0,
    speed: [(from: 0.0, speed: 14.0)],
    ledges: [
        (h: 0.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
        (h: 40.0, from_x: -64.0, to_x: 0.0, kind: "thin"),
        (h: 120.0, from_x: -96.0, to_x: 32.0, kind: "thin"),
        (h: 200.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
    ],
    route: [(x: 0.0, h: 0.0)],
)"#;

fn t() -> ClimberTuning {
    ClimberTuning::default()
}

fn tower() -> Tower {
    Tower::parse(STAGE, &t()).unwrap()
}

fn params(text: &str) -> SpawnParams {
    ron::from_str(text).unwrap()
}

#[test]
fn ledge_enemies_stand_on_the_highest_ledge_at_or_below_the_top_edge() {
    let w = tower();
    // Camera 130: the ledge at 120 is just on screen, 200 isn't yet.
    assert_eq!(footing(&w, -40.0, 130.0), Some(2));
    // Camera 100: 120 is above the top edge; the ledge at 40 is highest.
    assert_eq!(footing(&w, -40.0, 100.0), Some(1));
    // Over x = 60 only the landing is there.
    assert_eq!(footing(&w, 60.0, 100.0), Some(0));
    // Camera 300: the landing at 0 is below the bottom edge.
    assert_eq!(footing(&w, 60.0, 300.0), Some(3));
}

#[test]
fn a_picket_stands_on_its_ledge_facing_the_middle() {
    let w = tower();
    let members = layout(
        Kind::Picket,
        Vec2::new(-90.0, 0.0),
        &params("()"),
        130.0,
        &w,
        &t(),
    );
    assert_eq!(members.len(), 1);
    let (pos, brain) = &members[0];
    assert_eq!(*pos, Vec2::new(-90.0, 120.0), "feet on the ledge at 120");
    let Brain::Picket(p) = brain else { panic!() };
    assert_eq!((p.ledge, p.dir), (2, 1.0));
    // Placed inside the ledge, never hanging off its end.
    let members = layout(
        Kind::Picket,
        Vec2::new(-95.0, 0.0),
        &params("()"),
        130.0,
        &w,
        &t(),
    );
    assert_eq!(members[0].0.x, -90.0);
}

#[test]
fn a_limpet_pair_starts_at_opposite_ends_of_its_ledge() {
    let w = tower();
    let members = layout(
        Kind::Limpet,
        Vec2::new(-30.0, 0.0),
        &params("()"),
        100.0,
        &w,
        &t(),
    );
    assert_eq!(members.len(), 2);
    let a = members[0].0;
    let b = members[1].0;
    assert!(
        a.y > 40.0 && b.y < 40.0 - 8.0,
        "one on top, one under: {a} {b}"
    );
    assert!(
        (a.x + 67.0).abs() < 0.01 && (b.x - 3.0).abs() < 0.01,
        "{a} {b}"
    );
}

#[test]
fn flits_come_in_trios_from_their_side_or_from_below() {
    let w = tower();
    let side = layout(
        Kind::Flit,
        Vec2::new(176.0, 40.0),
        &params(r#"(side: "right")"#),
        0.0,
        &w,
        &t(),
    );
    assert_eq!(side.len(), 3);
    assert!(side.iter().all(|(p, _)| p.x > HALF_W && p.y == 40.0));
    assert_eq!(side[1].0.x - side[0].0.x, 24.0, "0.4 s apart at 60 px/s");
    let below = layout(
        Kind::Flit,
        Vec2::new(-48.0, -128.0),
        &params(r#"(from: "below")"#),
        0.0,
        &w,
        &t(),
    );
    assert_eq!(below.len(), 3);
    assert!(below.iter().all(|(p, _)| p.x == -48.0 && p.y < -HALF_H));
}

#[test]
fn things_in_the_face_are_placed_like_scenery() {
    let w = tower();
    let vent = layout(
        Kind::Vent,
        Vec2::new(-24.0, 128.0),
        &params("()"),
        880.0,
        &w,
        &t(),
    );
    assert_eq!(vent[0].0, Vec2::new(-24.0, 888.0), "height = at + y - 120");
    assert_eq!(
        screen_centre(Kind::Vent, vent[0].0, 880.0),
        Vec2::new(-24.0, 128.0)
    );
    assert_eq!(
        screen_centre(Kind::Vent, vent[0].0, 900.0),
        Vec2::new(-24.0, 108.0)
    );
}

#[test]
fn the_winch_hangs_from_its_rail_over_the_landing_on_screen() {
    let w = tower();
    // Camera 280: the landing at 200 is at y = 40, too high for a floor
    // under the rail; the one at 0 is at y = -160, off screen. At 330 the
    // landing at 200 is at y = -10.
    let members = layout(
        Kind::Winch,
        Vec2::new(0.0, 80.0),
        &params("()"),
        330.0,
        &w,
        &t(),
    );
    let Brain::Winch(winch) = &members[0].1 else {
        panic!()
    };
    assert_eq!(winch.floor, -10.0);
    assert_eq!(members[0].0.y, 80.0 - brain::HOOK_HANG);
}

#[test]
fn a_sweepers_arm_reaches_48_px_from_its_hub() {
    let s = Sweeper {
        hub: Vec2::new(10.0, 100.0),
        angle: std::f32::consts::FRAC_PI_2,
        turn: -1.0,
    };
    let arm = s.arm(48.0, ARM_BOXES);
    assert_eq!(arm.len(), ARM_BOXES);
    assert!((arm.last().unwrap().y - 148.0).abs() < 0.01);
    assert!((arm[0].y - 108.0).abs() < 0.01);
}

#[test]
fn the_fairness_rules_hold_back_a_shot() {
    let r = t().enemy;
    let hero = Some(Vec2::new(0.0, -60.0));
    let far = [Vec2::new(0.0, 40.0)];
    // In the open, above the Hero: it fires.
    assert!(may_fire(&r, &far, 1.0, hero, 100.0, Some(-70.0), 0));
    // In its first 0.5 s on screen.
    assert!(!may_fire(&r, &far, 0.4, hero, 100.0, Some(-70.0), 0));
    // Within 40 px of the Hero.
    assert!(!may_fire(
        &r,
        &[Vec2::new(0.0, -30.0)],
        1.0,
        hero,
        100.0,
        Some(-70.0),
        0
    ));
    // Over the bullet cap.
    assert!(!may_fire(&r, &far, 1.0, hero, 100.0, Some(-70.0), 32));
    // From below the Hero's feet: it has been passed.
    assert!(!may_fire(&r, &far, 1.0, hero, -80.0, Some(-70.0), 0));
}

#[test]
fn stage_two_bullets_are_15_percent_faster() {
    let t = t();
    let tuning = crate::tuning::Tuning::default();
    let mut session = Session::new_run(&tuning);
    assert_eq!(bullet_scale(&session, &t), 1.0);
    session.stage = 1;
    assert_eq!(bullet_scale(&session, &t), 1.15);
    // The fastest straight shot stays under rule 5's 70 px/s.
    assert!(t.enemies.picket.shot_speed * 1.15 < 70.0);
}

#[test]
fn every_kind_in_the_stage_files_and_the_gallery_is_known() {
    let owned = ["sweeper", "scrapfall", "standoff"];
    for file in STAGES.iter().chain([&gallery_file()]) {
        let timeline = crate::kit::StageTimeline::parse(file).unwrap();
        let mut entries: Vec<(String, ron::Map)> = timeline
            .spawns
            .iter()
            .map(|s| (s.kind.clone(), s.params.clone()))
            .collect();
        // The Standoffs' waves too.
        for s in &timeline.spawns {
            if s.kind == "standoff" {
                let p: crate::cabinets::on_foot::standoff::StandoffParams =
                    ron::Value::Map(s.params.clone()).into_rust().unwrap();
                entries.extend(p.waves.into_iter().map(|w| (w.kind, w.params)));
            }
        }
        for (kind, params) in entries {
            if kind.starts_with(crate::kit::scenery::SCENERY_PREFIX) {
                continue;
            }
            if let Some(letter) = kind.strip_prefix("power_up:") {
                assert!(PowerUp::from_kind(&kind).is_some(), "power_up:{letter}");
                continue;
            }
            assert!(
                kind_for(&kind).is_some() || owned.contains(&kind.as_str()),
                "unknown kind {kind}"
            );
            let p: SpawnParams = ron::Value::Map(params).into_rust().unwrap();
            if let Some(d) = p.drop {
                assert!(
                    PowerUp::from_letter(d.chars().next().unwrap()).is_some(),
                    "drop {d}"
                );
            }
        }
    }
}

#[test]
fn the_gallery_has_each_ledge_kind_a_gust_zone_and_a_sweeper() {
    let file = gallery_file();
    let tower = Tower::parse(file, &t()).unwrap();
    for kind in ["landing", "thin", "solid", "crumble"] {
        assert!(tower.ledges.iter().any(|l| l.kind == kind), "{kind}");
    }
    assert!(!tower.ladders.is_empty() && !tower.gusts.is_empty());
    assert!(file.contains("kind: \"sweeper\""));
}
