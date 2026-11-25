//! Fixed-step tests of each brain against docs/cabinets/4-climber.md.

use super::*;
use crate::cabinets::climber::tuning::ClimberTuning;

const DT: f32 = 1.0 / 120.0;

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

fn tower() -> Tower {
    Tower::parse(STAGE, &ClimberTuning::default()).unwrap()
}

fn t() -> ClimberTuning {
    ClimberTuning::default()
}

/// Steps a brain for `seconds`, returning every `Out` it gave.
fn run(
    brain: &mut Brain,
    pos: &mut Vec2,
    hero: Option<Vec2>,
    camera: f32,
    seconds: f32,
) -> Vec<Out> {
    let w = tower();
    let t = t();
    let ctx = Ctx {
        dt: DT,
        camera,
        hero,
        tower: &w,
    };
    (0..(seconds / DT).round() as usize)
        .map(|_| brain.step(pos, &ctx, &t.enemies, &t.enemy))
        .collect()
}

fn shots(outs: &[Out]) -> Vec<Shot> {
    outs.iter().flat_map(|o| o.shots.clone()).collect()
}

#[test]
fn a_flit_crosses_at_60_px_a_second_in_a_12_px_wave() {
    let mut b = Brain::Flit(Flit::crossing(-1.0));
    let mut pos = Vec2::new(168.0, 40.0);
    let mut low: f32 = f32::MAX;
    let mut high: f32 = f32::MIN;
    for _ in 0..240 {
        run(&mut b, &mut pos, None, 0.0, DT);
        low = low.min(pos.y);
        high = high.max(pos.y);
    }
    assert!((pos.x - (168.0 - 120.0)).abs() < 0.5, "{}", pos.x);
    assert!(
        (high - 52.0).abs() < 0.2 && (low - 28.0).abs() < 0.2,
        "{low}..{high}"
    );
    let outs = run(&mut b, &mut pos, None, 0.0, 10.0);
    assert!(shots(&outs).is_empty(), "it never fires");
    assert!(outs.iter().any(|o| o.despawn), "gone off the far side");
}

#[test]
fn a_flit_from_below_rises_then_leaves_through_the_nearer_side() {
    let mut b = Brain::Flit(Flit::from_below());
    let mut pos = Vec2::new(-40.0, -128.0);
    run(&mut b, &mut pos, None, 0.0, 1.0);
    assert!(
        (pos.x + 40.0).abs() < 0.01 && pos.y > -70.0,
        "straight up: {pos}"
    );
    let outs = run(&mut b, &mut pos, None, 0.0, 6.0);
    assert!(outs.iter().any(|o| o.despawn));
    assert!(pos.x < -HALF_W, "out through the left: {pos}");
}

#[test]
fn a_limpet_crawls_all_the_way_round_its_ledge() {
    // The ledge at 40 is 64 wide; with the Limpet's 3 px lift its path is
    // 70 by 14, so once round is 168 px: 7 s at 24 px/s.
    assert_eq!(Limpet::round(-64.0, 0.0), 168.0);
    let start = Limpet::place(-64.0, 0.0, 40.0, 0.0);
    assert_eq!(start, Vec2::new(-67.0, 43.0), "on top, at the left end");
    assert_eq!(
        Limpet::place(-64.0, 0.0, 40.0, 77.0),
        Vec2::new(3.0, 36.0),
        "round the right end"
    );
    assert_eq!(
        Limpet::place(-64.0, 0.0, 40.0, 122.0),
        Vec2::new(-35.0, 29.0),
        "along the underside"
    );
    let mut b = Brain::Limpet(Limpet::new(1, 0.0));
    let mut pos = start;
    let mut lowest: f32 = f32::MAX;
    let mut biggest_step: f32 = 0.0;
    for _ in 0..(7.0 / DT) as usize {
        let before = pos;
        run(&mut b, &mut pos, None, 0.0, DT);
        lowest = lowest.min(pos.y);
        biggest_step = biggest_step.max(pos.distance(before));
    }
    assert!(pos.distance(start) < 0.5, "back where it began: {pos}");
    assert!(lowest < 40.0 - 8.0, "it went under: {lowest}");
    assert!(
        biggest_step < 0.5,
        "no jumps at the corners: {biggest_step}"
    );
}

#[test]
fn scrap_waits_out_its_warning_then_falls_through_thin_ledges_onto_solid_ones() {
    let t = t();
    let mut b = Brain::Scrap(Scrap::new(&t.enemies));
    // Camera 200: the landing at 200 is at the top edge, the thin ledge at
    // 120 at y = 40, the landing at 0 at y = -80.
    let mut pos = Vec2::new(-30.0, 116.0);
    run(&mut b, &mut pos, None, 200.0, 0.7);
    assert_eq!(
        pos.y,
        PLAY_TOP + 8.0,
        "held above the top during the warning"
    );
    let outs = run(&mut b, &mut pos, None, 200.0, 0.5);
    assert!(
        pos.y < 116.0 - 50.0 && pos.y > 40.0,
        "falling at 120 px/s: {}",
        pos.y
    );
    assert!(outs.iter().all(|o| !o.shattered));
    let outs = run(&mut b, &mut pos, None, 200.0, 1.5);
    let at = outs.iter().position(|o| o.shattered).expect("shattered");
    assert!(at > 0, "not on the thin ledge");
    assert!(pos.y < -70.0, "on the landing at y = -80: {}", pos.y);
}

#[test]
fn a_scrapfall_drops_a_piece_every_0_8_s_never_within_24_px_of_the_last() {
    let t = t();
    let mut fall = Scrapfall::new(-112.0, -24.0, 5.0, 77);
    let mut xs = Vec::new();
    let mut steps = 0;
    loop {
        let (pieces, done) = fall.step(0.1, &t.enemies);
        xs.extend(pieces);
        steps += 1;
        if done {
            break;
        }
    }
    assert!((50..=51).contains(&steps), "5 s: {steps}");
    assert_eq!(xs.len(), 7, "0, 0.8, ... 4.8 s: {xs:?}");
    for pair in xs.windows(2) {
        assert!((pair[1] - pair[0]).abs() >= 24.0, "{xs:?}");
    }
    assert!(xs.iter().all(|x| (-112.0..=-24.0).contains(x)));
    // The same fall every time.
    let mut again = Scrapfall::new(-112.0, -24.0, 5.0, 77);
    assert_eq!(again.step(0.1, &t.enemies).0, xs[..1]);
}

#[test]
fn a_picket_paces_its_ledge_and_turns_at_the_ends() {
    let mut b = Brain::Picket(Picket::new(2, 1.0));
    let mut pos = Vec2::new(0.0, 120.0);
    // The Hero far below: it just walks. 26 px to the end at 20 px/s, then
    // back.
    run(&mut b, &mut pos, Some(Vec2::new(100.0, 0.0)), 200.0, 2.0);
    assert!(pos.x < 32.0 - 6.0 + 0.01 && pos.x > 10.0, "{}", pos.x);
    let Brain::Picket(p) = &b else { unreachable!() };
    assert_eq!(p.dir, -1.0, "turned at the end");
}

#[test]
fn a_picket_fires_a_two_shot_burst_at_chest_height_along_its_level() {
    let t = t();
    let mut b = Brain::Picket(Picket::new(2, 1.0));
    let mut pos = Vec2::new(-40.0, 120.0);
    // Level (12 px below, inside 16) and in front.
    let hero = Some(Vec2::new(60.0, 108.0));
    let outs = run(&mut b, &mut pos, hero, 200.0, 1.0);
    let s = shots(&outs);
    assert_eq!(s.len(), 2);
    assert_eq!(s[0].offset, Vec2::new(6.0, 14.0), "chest height, in front");
    assert_eq!(s[0].velocity, Vec2::new(60.0, 0.0));
    let first = outs.iter().position(|o| !o.shots.is_empty()).unwrap() as f32 * DT;
    assert!(
        (first - t.enemies.picket.telegraph_seconds).abs() < 0.02,
        "{first}"
    );
    assert!(outs.iter().any(|o| o.warned));
    // Then it waits 2 s before it can fire again.
    let outs = run(&mut b, &mut pos, hero, 200.0, 1.5);
    assert!(shots(&outs).is_empty());
}

#[test]
fn a_picket_ignores_a_hero_behind_it_or_off_its_level() {
    for hero in [Vec2::new(-100.0, 120.0), Vec2::new(60.0, 100.0)] {
        let mut b = Brain::Picket(Picket::new(2, 1.0));
        let mut pos = Vec2::new(-40.0, 120.0);
        // Stop it walking round to face the Hero behind: a short look.
        let outs = run(&mut b, &mut pos, Some(hero), 200.0, 0.5);
        assert!(outs.iter().all(|o| !o.warned), "{hero}");
    }
}

#[test]
fn a_vent_starts_on_screen_then_cycles_closed_glowing_open() {
    let t = t();
    let v = &t.enemies.vent;
    let mut b = Brain::Vent(Vent::default());
    // Height 330: off the top until the camera passes 330 + 12 + 6.
    let mut pos = Vec2::new(0.0, 330.0);
    run(&mut b, &mut pos, None, 300.0, 1.0);
    let Brain::Vent(vent) = &b else {
        unreachable!()
    };
    assert_eq!(vent.phase, VentPhase::Waiting);
    let mut phases = Vec::new();
    let mut outs = Vec::new();
    for _ in 0..(3.0 / DT) as usize {
        outs.extend(run(&mut b, &mut pos, None, 360.0, DT));
        let Brain::Vent(vent) = &b else {
            unreachable!()
        };
        if phases.last() != Some(&vent.phase) {
            phases.push(vent.phase);
        }
    }
    assert_eq!(
        phases,
        [VentPhase::Closed, VentPhase::Glowing, VentPhase::Open]
    );
    let s = shots(&outs);
    assert_eq!(s.len(), 3, "one fan per opening");
    let mut angles: Vec<_> = s
        .iter()
        .map(|s| s.velocity.to_angle().to_degrees().round())
        .collect();
    angles.sort_by(f32::total_cmp);
    assert_eq!(angles, [-120.0, -90.0, -60.0], "±30° from straight down");
    assert!((s[0].velocity.length() - v.shot_speed).abs() < 0.01);
    // Open for 1.4 s, then closed again.
    let outs = run(&mut b, &mut pos, None, 360.0, 1.0);
    let Brain::Vent(vent) = &b else {
        unreachable!()
    };
    assert!(!vent.open());
    assert!(shots(&outs).is_empty());
}

#[test]
fn a_plumb_drops_to_chest_height_fires_both_ways_then_reels_away() {
    let t = t();
    let mut b = Brain::Plumb(Plumb::default());
    let mut pos = Vec2::new(60.0, PLAY_TOP + 5.0);
    // Hero's feet at screen y -80 (height 0, camera 200): chest at -66.
    let hero = Some(Vec2::new(-60.0, 0.0));
    let drop_time = (PLAY_TOP + 5.0 + 66.0) / t.enemies.plumb.drop_speed;
    let outs = run(&mut b, &mut pos, hero, 200.0, drop_time + 0.05);
    assert!(
        (pos.y + 66.0).abs() < 0.01,
        "level with the chest: {}",
        pos.y
    );
    assert!(outs.iter().any(|o| o.warned));
    let outs = run(&mut b, &mut pos, hero, 200.0, 0.4);
    let s = shots(&outs);
    assert_eq!(s.len(), 2, "one each way");
    assert_eq!(s[0].velocity, Vec2::new(-60.0, 0.0));
    assert_eq!(s[1].velocity, Vec2::new(60.0, 0.0));
    let outs = run(&mut b, &mut pos, hero, 200.0, 7.0);
    assert!(outs.iter().any(|o| o.despawn), "reeled away");
}

#[test]
fn a_plumb_skips_the_side_where_the_hero_is_close() {
    let mut b = Brain::Plumb(Plumb::default());
    let mut pos = Vec2::new(30.0, PLAY_TOP + 5.0);
    let hero = Some(Vec2::new(10.0, 0.0));
    let outs = run(&mut b, &mut pos, hero, 200.0, 4.0);
    let s = shots(&outs);
    assert_eq!(s.len(), 1);
    assert!(s[0].velocity.x > 0.0, "only away from the Hero");
}

#[test]
fn a_pitcher_throws_only_at_a_hero_below_and_in_range() {
    let t = t();
    let p = &t.enemies.pitcher;
    let mut b = Brain::Pitcher(Pitcher::new(&t.enemies));
    let mut pos = Vec2::new(-40.0, 120.0);
    for hero in [Vec2::new(-40.0, 200.0), Vec2::new(100.0, 0.0)] {
        let outs = run(&mut b, &mut pos, Some(hero), 200.0, 5.0);
        assert!(outs.iter().all(|o| o.canister.is_none()), "{hero}");
    }
    let outs = run(&mut b, &mut pos, Some(Vec2::new(0.0, 0.0)), 200.0, 5.0);
    let throws: Vec<_> = outs.iter().filter_map(|o| o.canister).collect();
    assert_eq!(throws.len(), 2, "every 2.4 s");
    let wind = outs.iter().position(|o| o.warned).unwrap() as f32 * DT;
    let first = outs.iter().position(|o| o.canister.is_some()).unwrap() as f32 * DT;
    assert!((first - wind - p.telegraph_seconds).abs() < 0.02);
    assert!(throws[0].1.length() <= p.launch_speed + 0.01);
}

#[test]
fn a_lob_lands_where_the_hero_stood_when_it_can() {
    let p = t().enemies.pitcher;
    let from = Vec2::new(-40.0, 134.0);
    let to = Vec2::new(0.0, 0.0);
    let v = Pitcher::lob(from, to, p.launch_speed, p.canister_gravity);
    assert!(v.length() <= 60.0 + 0.01);
    // Fly it.
    let (mut at, mut vel) = (from, v);
    while at.y > to.y {
        vel.y -= p.canister_gravity * DT;
        at += vel * DT;
    }
    assert!((at.x - to.x).abs() < 3.0, "lands at {at}");
}

#[test]
fn a_canister_falls_through_thin_ledges_and_bursts_on_a_solid_one() {
    let mut b = Brain::Canister(Canister {
        vel: Vec2::new(0.0, 0.0),
        gravity: 300.0,
    });
    let mut pos = Vec2::new(-30.0, 150.0);
    let outs = run(&mut b, &mut pos, None, 200.0, 2.0);
    let at = outs.iter().position(|o| o.shattered).expect("burst");
    // It passed the thin ledge at 120 and the one at 40.
    let fallen = 0.5 * 300.0 * (at as f32 * DT).powi(2);
    assert!(
        fallen > 120.0,
        "on the landing at 0, not a thin ledge: {fallen}"
    );
}

#[test]
fn motes_home_slowly_then_dissolve() {
    let mut b = Brain::Mote(Mote::default());
    let mut pos = Vec2::new(0.0, 100.0);
    let hero = Some(Vec2::new(0.0, 0.0));
    let outs = run(&mut b, &mut pos, hero, 200.0, 7.9);
    assert!(outs.iter().all(|o| !o.despawn));
    assert!(pos.y < 100.0, "drawn down toward the Hero");
    let outs = run(&mut b, &mut pos, hero, 200.0, 0.2);
    assert!(outs.iter().any(|o| o.despawn), "dissolved at 8 s");
}

#[test]
fn the_winch_slides_shakes_drops_rests_and_reels_on_one_clock() {
    let t = t();
    let w = &t.enemies.winch;
    let floor = -80.0;
    let hang = w.rail_y - HOOK_HANG;
    let mut b = Brain::Winch(Winch::new(-60.0, floor));
    let mut pos = Vec2::new(-60.0, hang);
    let hero = Some(Vec2::new(0.0, 0.0));
    let mut phases = Vec::new();
    let mut outs = Vec::new();
    let mut cycle = None;
    for i in 0..(12.0 / DT) as usize {
        outs.extend(run(&mut b, &mut pos, hero, 200.0, DT));
        let Brain::Winch(winch) = &b else {
            unreachable!()
        };
        let name = std::mem::discriminant(&winch.phase);
        if phases.last() != Some(&name) {
            phases.push(name);
            if phases.len() == 6 && cycle.is_none() {
                cycle = Some(i as f32 * DT);
            }
        }
    }
    let Brain::Winch(winch) = &b else {
        unreachable!()
    };
    assert!((winch.carriage_x - 0.0).abs() < 0.01, "above the Hero");
    let cycle = cycle.expect("a full cycle");
    assert!((5.0..=6.0).contains(&cycle), "5-6 s a cycle: {cycle}");
    let s = shots(&outs);
    assert!(s.len() >= 2);
    assert_eq!(s[0].velocity, Vec2::new(-60.0, 0.0));
    assert_eq!(s[1].velocity, Vec2::new(60.0, 0.0));
    // From the floor at ankle height: the hook's centre is 8 px up.
    assert_eq!(s[0].offset.y + floor + 8.0, floor + 4.0);
}

#[test]
fn only_warnings_flash() {
    let t = t();
    assert!(!Brain::Flit(Flit::crossing(1.0)).telegraph());
    let mut p = Picket::new(0, 1.0);
    p.phase = PicketPhase::Flash(0.1);
    assert!(Brain::Picket(p).telegraph());
    let mut pitcher = Pitcher::new(&t.enemies);
    pitcher.winding = true;
    assert!(Brain::Pitcher(pitcher).telegraph());
}
