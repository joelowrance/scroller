//! Cabinet 2's score budget (#15's balance note): what a perfect run of each
//! Stage scores, recomputed from the Stage files and tuning, and where the
//! extra lives land across Cabinets 1 and 2.

use super::enemies::{Kind, SpawnParams, kind_for, layout};
use super::power_ups::{Collected, PowerUp, PowerUps};
use super::tuning::VerticalShooterTuning;
use super::*;
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::kit::SpawnRequested;
use crate::kit::timeline::SpawnEntry;

/// The Hero's row, for the Riser's "only while above the Hero" rule.
const HERO_Y: f32 = -80.0;
/// A perfect player's damage per second (the doc's "Damage budget":
/// narrow, large) in Stage 1 and Stage 2. Only used to estimate how long a
/// Loom's shuttles can be left alive to launch Wisps.
const PERFECT_DPS: [(f32, f32); 2] = [(20.0, 60.0), (40.0, 80.0)];

/// What a perfect Stage can score, by source. Bombed bullets are left out:
/// they depend on what is on screen when the Squall goes off.
#[derive(Debug, Default)]
pub struct ScoreBudget {
    pub enemies: u64,
    pub formation_bonus: u64,
    pub clear_and_no_miss: u64,
    /// Power-ups picked up when already maxed, collecting every drop in order
    /// from the start of the Cabinet and never losing a life or a bomb.
    pub medals: u64,
    /// Riser pods shot while climbing, with each launcher left alive as long
    /// as it launches (it stops once it is below the Hero).
    pub riser_pods: u64,
    /// Wisps a Loom's shuttles launch if they are shot off as late as still
    /// leaves time to kill the core before the Loom retreats.
    pub loom_wisps: u64,
}

impl ScoreBudget {
    /// A perfect Stage: every enemy, every Formation and both bonuses.
    pub fn core(&self) -> u64 {
        self.enemies + self.formation_bonus + self.clear_and_no_miss
    }

    /// Plus the medals a perfect player can't avoid.
    pub fn perfect(&self) -> u64 {
        self.core() + self.medals
    }

    /// Plus everything that can be milked.
    pub fn most(&self) -> u64 {
        self.perfect() + self.riser_pods + self.loom_wisps
    }
}

fn params(spawn: &SpawnEntry) -> SpawnParams {
    SpawnRequested {
        kind: spawn.kind.clone(),
        pos: Vec2::ZERO,
        params: spawn.params.clone(),
    }
    .params()
    .unwrap()
}

/// Launches from a timer that starts at `first` and repeats every
/// `interval` while it is under `window` seconds.
fn launches(window: f32, first: f32, interval: f32) -> u64 {
    if window < first {
        0
    } else {
        ((window - first) / interval).floor() as u64 + 1
    }
}

/// Budgets both Stages, carrying the Power-ups from Stage 1 into Stage 2.
pub fn score_budgets(t: &VerticalShooterTuning) -> [ScoreBudget; 2] {
    let e = &t.enemies;
    let mut power = PowerUps::fresh(t);
    [0, 1].map(|stage| {
        let timeline = stage_timeline(&Tuning::default(), stage);
        let stage2 = stage == 1;
        let (narrow, large) = PERFECT_DPS[stage];
        let mut b = ScoreBudget {
            clear_and_no_miss: stage_bonus_points(t, 0),
            ..default()
        };
        for spawn in &timeline.spawns {
            let Some(kind) = kind_for(&spawn.kind) else {
                continue;
            };
            let p = params(spawn);
            let pos = Vec2::from(spawn.pos);
            let members = layout(kind, pos, &p, None, e).len() as u64;
            b.enemies += members
                * match kind {
                    Kind::Wisp => e.wisp.score,
                    Kind::Swift => e.swift.score,
                    Kind::Bellows => e.bellows.score,
                    Kind::LoomCore => e.loom.core_score + 2 * e.loom.shuttle_score,
                    Kind::Crawler => e.crawler.score,
                    Kind::Silo => e.silo.score,
                    Kind::RiserLauncher => e.riser.score,
                    Kind::Flak => e.flak.score,
                    Kind::Mote => e.mote.score,
                    Kind::TetherBalloon => e.tether.score,
                    Kind::LoomShuttle | Kind::RiserPod => unreachable!("never in a Stage file"),
                };
            // Formations as `spawn_requested` makes them: a drop, or more
            // than one member (a Tether only with a drop).
            if let Some(letter) = p.drop.as_deref().and_then(|d| d.chars().next()) {
                let power_up = PowerUp::from_letter(letter).expect("a known Power-up");
                if power.collect(power_up, t) == Collected::Medal {
                    b.medals += t.score.maxed_power_up;
                }
            } else if members > 1 && kind != Kind::TetherBalloon {
                b.formation_bonus += members * t.score.formation_bonus_per_enemy;
            }
            let speed = timeline.speed_at(spawn.at);
            match kind {
                Kind::RiserLauncher => {
                    // Fully on screen (its 14 px top below the HUD) until
                    // it scrolls below the Hero.
                    let on = (pos.y + 7.0 - PLAY_TOP).max(0.0) / speed;
                    let above = (pos.y - HERO_Y) / speed;
                    let pods = launches(
                        above - on,
                        e.riser.first_launch_delay,
                        e.riser.launch_interval,
                    );
                    b.riser_pods += pods * e.riser.pod_score_climbing;
                }
                Kind::LoomCore => {
                    let l = &e.loom;
                    let (shuttle_hp, core_hp) = if stage2 {
                        (l.shuttle_hp_stage2, l.core_hp_stage2)
                    } else {
                        (l.shuttle_hp, l.core_hp)
                    };
                    let held = (pos.y - l.hold_y) / l.speed + l.hold_seconds;
                    let kill = 2.0 * shuttle_hp as f32 / narrow + core_hp as f32 / large;
                    let each = launches(held - kill, l.first_launch_delay, l.launch_interval);
                    b.loom_wisps += 2 * each * e.wisp.score;
                }
                _ => {}
            }
        }
        b
    })
}

#[test]
fn extra_lives_land_where_the_design_doc_says() {
    let tuning = Tuning::default();
    let [one, two] = score_budgets(&tuning.vertical_shooter);
    println!(
        "Stage 1: core {}, perfect {}, most {} = {one:?}",
        one.core(),
        one.perfect(),
        one.most()
    );
    println!(
        "Stage 2: core {}, perfect {}, most {} = {two:?}",
        two.core(),
        two.perfect(),
        two.most()
    );
    // The totals the balance note on #15 quotes.
    assert_eq!((one.core(), one.perfect()), (CORE[0], PERFECT[0]));
    assert_eq!((two.core(), two.perfect()), (CORE[1], PERFECT[1]));

    // Cabinet 1's perfect core (#11), then this Cabinet.
    use crate::cabinets::side_shooter::stages::tests::score_budget;
    let cabinet_1 = score_budget(0).core() + score_budget(1).core();
    let cabinet_1_most = score_budget(0).total() + score_budget(1).total();
    let third = tuning.first_extra_life + 2 * tuning.extra_life_every;
    // A strong player reaches the third extra life late in Cabinet 2: not
    // in Stage 1, even with everything milked in both Cabinets so far...
    assert!(cabinet_1_most + one.most() < third);
    // ...but by the end of a perfect Stage 2.
    assert!(cabinet_1 + one.perfect() + two.perfect() >= third);
}

/// Core totals (enemies, Formations, clear and no-miss) for Stages 1 and 2.
const CORE: [u64; 2] = [33_600, 39_000];
/// Core plus the medals a perfect player collects.
const PERFECT: [u64; 2] = [33_600, 44_000];
