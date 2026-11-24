//! Checks on the Stage files against the design doc and the fairness rules.

use super::*;
use crate::cabinets::side_shooter::enemies::{SpawnParams, kind_for};
use crate::display::HALF_W;
use crate::kit::SpawnRequested;
use crate::kit::timeline::SpawnEntry;

fn params(spawn: &SpawnEntry) -> SpawnParams {
    SpawnRequested {
        kind: spawn.kind.clone(),
        pos: Vec2::ZERO,
        params: spawn.params.clone(),
    }
    .params()
    .unwrap()
}

/// Scroll position under screen x `x` when a spawn at `at` fires.
fn under(at: f32, x: f32) -> f32 {
    terrain::scroll_at(x, at)
}

#[test]
fn both_stages_last_three_to_four_minutes() {
    let one = timeline(0).duration();
    let two = timeline(1).duration();
    assert!((205.0..=215.0).contains(&one), "Stage 1 takes {one} s");
    assert!((220.0..=230.0).contains(&two), "Stage 2 takes {two} s");
}

#[test]
fn every_spawn_is_a_known_enemy_with_valid_params() {
    for stage in 0..2 {
        for spawn in &timeline(stage).spawns {
            assert!(kind_for(&spawn.kind).is_some(), "{}", spawn.kind);
            let p = params(spawn);
            if let Some(drop) = &p.drop {
                assert!(
                    ["G", "D", "T", "S", "W", "B"].contains(&drop.as_str()),
                    "{drop}"
                );
            }
        }
    }
}

#[test]
fn power_ups_arrive_in_the_designed_order() {
    let drops = |stage| -> String {
        timeline(stage)
            .spawns
            .iter()
            .filter_map(|s| params(s).drop)
            .collect()
    };
    assert_eq!(drops(0), "GDSGBWDG");
    assert_eq!(drops(1), "TSGWDB");
}

#[test]
fn restart_points_match_the_pacing_outline() {
    let at = |stage| -> Vec<f32> {
        let t = timeline(stage);
        t.restart_points
            .iter()
            .map(|r| {
                let mut cut = t.clone();
                cut.length = r.at.max(0.001);
                (cut.duration() * 10.0).round() / 10.0
            })
            .collect()
    };
    assert_eq!(at(0), [0.0, 70.0, 140.0]);
    assert_eq!(at(1), [0.0, 60.0, 120.0, 175.0]);
}

#[test]
fn the_corridor_is_never_narrower_than_the_minimum() {
    let ground = terrain(1);
    let min_gap = Tuning::default().side_shooter.terrain.min_gap;
    let length = timeline(1).length;
    let mut s = -2.0 * HALF_W;
    while s <= length + 2.0 * HALF_W {
        let (floor, ceiling) = ground.heights_at(s);
        assert!(ceiling - floor >= min_gap, "gap {} at {s}", ceiling - floor);
        s += 2.0;
    }
}

#[test]
fn the_hero_reappears_clear_of_terrain() {
    let ground = terrain(1);
    for point in &timeline(1).restart_points {
        let hero = point.hero_pos();
        // Where the Hero stands, and the next 3 s ahead of it at 32 px/s.
        let from = under(point.at, hero.x - 8.0);
        let (floor, ceiling) = ground.corridor(from, from + 100.0);
        assert!(
            hero.y - 12.0 > floor && hero.y + 12.0 < ceiling,
            "Restart Point at {} puts the Hero at {} in {floor}..{ceiling}",
            point.at,
            hero.y
        );
    }
}

#[test]
fn barnacles_and_pylons_sit_on_the_terrain() {
    let ground = terrain(1);
    for spawn in &timeline(1).spawns {
        let (floor, ceiling) = ground.heights_at(under(spawn.at, spawn.pos.0));
        match spawn.kind.as_str() {
            "barnacle" => {
                let want = if params(spawn).ceiling {
                    ceiling - 5.0
                } else {
                    floor + 5.0
                };
                assert_eq!(spawn.pos.1, want, "Barnacle at {}", spawn.at);
                // The ground under it stays put while it is on screen.
                let gone = under(spawn.at, -HALF_W);
                let (f2, c2) = ground.heights_at(gone + 2.0 * HALF_W + 16.0);
                assert_eq!((f2, c2), (floor, ceiling), "Barnacle at {}", spawn.at);
            }
            "arc_gate" => {
                let p = params(spawn);
                assert_eq!(p.floor_y, Some(floor + 8.0), "Arc Gate at {}", spawn.at);
                assert_eq!(p.ceiling_y, Some(ceiling - 8.0), "Arc Gate at {}", spawn.at);
            }
            _ => {}
        }
    }
}

#[test]
fn hooks_only_come_where_the_corridor_is_wide() {
    let ground = terrain(1);
    for spawn in timeline(1).spawns.iter().filter(|s| s.kind == "hook") {
        let left = under(spawn.at, -HALF_W);
        let (floor, ceiling) = ground.corridor(left, left + 2.0 * HALF_W + 120.0);
        let wide = Tuning::default().side_shooter.terrain.hook_min_gap;
        assert!(ceiling - floor >= wide, "Hook at {}", spawn.at);
    }
}

#[test]
fn the_tile_grid_matches_the_tuning() {
    assert_eq!(Tuning::default().side_shooter.terrain.tile_size, TILE);
}

#[test]
fn stage_one_has_no_terrain() {
    assert!(terrain(0).is_empty());
    assert!(!terrain(1).is_empty());
}

mod playthrough {
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::cabinets::side_shooter::enemies::EnemiesPlugin;
    use crate::display::GameCamera;
    use crate::flow::Paused;
    use crate::kit::{Hero, Invulnerable, KitPlugin, projectile};

    #[derive(Resource, Default)]
    struct Done {
        cleared: bool,
        score: u64,
    }

    fn watch(
        mut done: ResMut<Done>,
        mut cleared: MessageReader<StageCleared>,
        mut score: MessageReader<ScoreGained>,
    ) {
        done.cleared |= cleared.read().count() > 0;
        done.score += score.read().map(|s| s.0).sum::<u64>();
    }

    /// An invincible Hero that fires straight ahead every frame.
    fn fire(mut commands: Commands, hero: Single<&Transform, With<Hero>>) {
        let at = hero.translation.truncate() + Vec2::X * 8.0;
        commands.spawn(projectile(
            Layer::HeroShot,
            at,
            Vec2::X * 360.0,
            Vec2::new(8.0, 2.0),
            Color::WHITE,
            2.0,
        ));
    }

    /// Plays a whole Stage with the enemies, terrain and scenery, at a frame
    /// step of `dt`, and returns the points scored and the game seconds it
    /// took to clear.
    fn play(stage: usize, dt: f32, time_scale: f32) -> (u64, f32) {
        let tuning = Tuning {
            debug_keys: true,
            invincible_hero: true,
            time_scale,
            ..Tuning::default()
        };
        let mut session = Session::new_run(&tuning);
        session.stage = stage;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                dt,
            )))
            .insert_resource(tuning)
            .insert_resource(session)
            .init_resource::<Paused>()
            .init_resource::<Done>()
            .init_resource::<crate::sprites::Sprites>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_state(Screen::Playing)
            .add_message::<ScoreGained>()
            .add_message::<LifeLost>()
            .add_message::<StageCleared>()
            .add_message::<crate::audio::PlaySfx>()
            .add_plugins((KitPlugin, EnemiesPlugin, StagesPlugin))
            .add_systems(Update, fire.before(KitSet::Move))
            .add_systems(Last, watch)
            .insert_resource(StageRun::new(timeline(stage)));
        app.world_mut().spawn((GameCamera, Transform::default()));
        app.world_mut().spawn((
            Hero,
            Invulnerable(0.0),
            Hitbox::new(4.0, 4.0),
            Layer::Hero,
            Transform::from_xyz(-110.0, 0.0, 0.0),
        ));
        // Generous: enough for the whole Stage even if the scale is ignored.
        let frames = (300.0 / dt.min(0.25)) as usize;
        for _ in 0..frames {
            app.update();
            if app.world().resource::<Done>().cleared {
                break;
            }
        }
        let done = app.world().resource::<Done>();
        assert!(done.cleared, "Stage {} did not clear", stage + 1);
        (
            done.score,
            app.world().resource::<crate::kit::StageClock>().seconds,
        )
    }

    #[test]
    fn both_stages_play_through_at_normal_speed() {
        // Measured, not computed: the clear comes when the designed length
        // is up, within a couple of frames.
        for (stage, length) in [(0, 210.0), (1, 225.0)] {
            let (score, seconds) = play(stage, 1.0 / 60.0, 1.0);
            assert!(score >= 3000, "Stage {} scored {score}", stage + 1);
            assert!(
                (seconds - length).abs() < 0.05,
                "Stage {} took {seconds} s",
                stage + 1
            );
        }
    }

    #[test]
    fn stage_two_follows_stage_one_through_the_stage_card() {
        let tuning = Tuning {
            debug_keys: true,
            invincible_hero: true,
            time_scale: 8.0,
            ..Tuning::default()
        };
        let session = Session::new_run(&tuning);
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                0.1,
            )))
            .insert_resource(tuning)
            .insert_resource(session)
            .init_resource::<Paused>()
            .init_resource::<Done>()
            .init_resource::<crate::sprites::Sprites>()
            .init_resource::<ButtonInput<KeyCode>>()
            .insert_state(Screen::Playing)
            .add_message::<ScoreGained>()
            .add_message::<LifeLost>()
            .add_message::<StageCleared>()
            .add_message::<crate::audio::PlaySfx>()
            .add_plugins((KitPlugin, EnemiesPlugin, StagesPlugin))
            .add_systems(Update, fire.before(KitSet::Move))
            .add_systems(Last, watch)
            .insert_resource(StageRun::new(timeline(0)));
        app.world_mut().spawn((GameCamera, Transform::default()));
        app.world_mut().spawn((
            Hero,
            Invulnerable(0.0),
            Hitbox::new(4.0, 4.0),
            Layer::Hero,
            Transform::from_xyz(-110.0, 0.0, 0.0),
        ));
        for stage in 0..2 {
            for _ in 0..3000 {
                app.update();
                if app.world().resource::<Done>().cleared {
                    break;
                }
            }
            assert!(
                app.world().resource::<Done>().cleared,
                "Stage {}",
                stage + 1
            );
            // The flow: Stage Card, then the next Stage.
            app.world_mut()
                .resource_mut::<NextState<Screen>>()
                .set(Screen::StageCard);
            app.update();
            app.update();
            app.world_mut().resource_mut::<Session>().stage = 1;
            app.world_mut().insert_resource(StageRun::new(timeline(1)));
            app.world_mut().resource_mut::<Done>().cleared = false;
            app.world_mut()
                .resource_mut::<NextState<Screen>>()
                .set(Screen::Playing);
            app.update();
        }
    }

    #[test]
    fn both_stages_play_through_in_big_steps() {
        // What a slow browser does at time_scale 8: game frames of up to 2 s.
        for dt in [0.05, 0.1, 0.25] {
            for stage in 0..2 {
                play(stage, dt, 8.0);
            }
        }
    }
}

/// The most a perfect Stage can score, by source (bombed bullets left out:
/// they depend on how much is on screen when the Undertow goes off).
#[derive(Debug, Default)]
pub(crate) struct ScoreBudget {
    enemies: u64,
    formation_bonus: u64,
    /// Skiffs a Ferry launches if it is left alive until it sinks.
    ferry_skiffs: u64,
    /// Every Power-up picked up when already maxed.
    maxed_power_ups: u64,
    clear_and_no_miss: u64,
}

impl ScoreBudget {
    /// A perfect Stage without the extras (maxed Power-ups, milked Ferries).
    pub(crate) fn core(&self) -> u64 {
        self.enemies + self.formation_bonus + self.clear_and_no_miss
    }

    pub(crate) fn total(&self) -> u64 {
        self.enemies
            + self.formation_bonus
            + self.ferry_skiffs
            + self.maxed_power_ups
            + self.clear_and_no_miss
    }
}

pub(crate) fn score_budget(stage: usize) -> ScoreBudget {
    use crate::cabinets::side_shooter::enemies::{Kind, layout};
    let tuning = Tuning::default();
    let s = &tuning.side_shooter;
    let t = &s.enemies;
    let mut budget = ScoreBudget {
        clear_and_no_miss: s.score.stage_clear + s.score.no_miss,
        ..default()
    };
    for spawn in &timeline(stage).spawns {
        let kind = kind_for(&spawn.kind).unwrap();
        let params = params(spawn);
        let pos = Vec2::from(spawn.pos);
        let members = layout(kind, pos, &params, None, t).len() as u64;
        let score = match kind {
            Kind::Skiff => t.skiff.score,
            Kind::Gull => t.gull.score,
            Kind::Bellbuoy => t.bellbuoy.score,
            Kind::Lamplighter => t.lamplighter.score,
            Kind::Hook => t.hook.score,
            Kind::Ferry => t.ferry.score,
            Kind::Barnacle => t.barnacle.score,
            Kind::Clam => t.clam.score,
            Kind::Mote => t.mote.score,
            Kind::ArcPylon => t.arc_gate.score,
        };
        budget.enemies += members * score;
        if params.drop.is_some() {
            budget.maxed_power_ups += s.score.maxed_power_up;
        } else if members > 1 && kind != Kind::ArcPylon {
            budget.formation_bonus += members * s.score.formation_bonus_per_enemy;
        }
        if kind == Kind::Ferry {
            let f = &t.ferry;
            let alive = (pos.x - f.hold_x) / f.speed + f.hold_seconds;
            let launches = ((alive - f.first_launch_delay) / f.launch_interval).floor() as u64 + 1;
            budget.ferry_skiffs += launches * u64::from(f.launch_count) * t.skiff.score;
        }
    }
    budget
}

#[test]
fn extra_lives_land_where_the_design_doc_says() {
    let tuning = Tuning::default();
    let [one, two] = [score_budget(0), score_budget(1)];
    println!("Stage 1: {} = {one:?}", one.total());
    println!("Stage 2: {} = {two:?}", two.total());
    // The measured totals the design doc's Scoring section quotes.
    assert!((35_000..=45_000).contains(&one.total()), "Stage 1");
    assert!((32_000..=42_000).contains(&two.total()), "Stage 2");
    // Without the extras (maxed Power-ups, Ferries milked for Skiffs), a
    // flawless Stage 1 still passes the first extra life...
    let core = ScoreBudget::core;
    assert!(core(&one) >= tuning.first_extra_life);
    // ...and only a player squeezing out the extras gets the second one
    // before Cabinet 2.
    let second = tuning.first_extra_life + tuning.extra_life_every;
    assert!(core(&one) + core(&two) < second);
    assert!(one.total() + two.total() >= second);
}
