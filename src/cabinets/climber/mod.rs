//! Cabinet 4: HIGHWIRE, the vertical climber (docs/cabinets/4-climber.md).
//!
//! The Hero climbs the tower face (`hero.rs`) on the kit's platform physics
//! with ladders, over the Stage file's terrain (`terrain.rs`, read into the
//! [`Tower`] the gameplay uses, `tower.rs`). The camera rises on its own at
//! the Stage's scroll speed, and stops only for a Standoff or while the Hero
//! drops back in after a lost life. Enemies and hazards are in `enemies/`;
//! the Standoff clock is shared with Cabinet 3 (`on_foot::standoff`). The
//! Stages, scenery, sprites and sounds are #25's.

use bevy::prelude::*;

use super::cabinet_is;
use super::on_foot::standoff::{self, Outcome, Standoff, StandoffParams};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::flow::Screen;
use crate::kit::scenery::SCENERY_PREFIX;
use crate::kit::{
    DrivenScroll, KitSet, SpawnRequested, StageRestarted, StageRun, StageTimeline, stage_live,
};
use crate::run::Session;
use crate::sprites::Sprites;
use crate::tuning::Tuning;

pub mod enemies;
pub mod hero;
pub mod palette;
pub mod power_ups;
pub mod terrain;
pub mod tower;
pub mod tuning;

use power_ups::{PowerUp, PowerUps};
use tower::Tower;

/// The Cabinet's index in `CABINETS`.
pub const CABINET: usize = 3;

/// Cabinet 4's two Stage files.
pub const STAGES: [&str; 2] = [
    include_str!("../../../assets/stages/4-1.ron"),
    include_str!("../../../assets/stages/4-2.ron"),
];

#[cfg(test)]
mod stage_checks;

/// The Stage file to play: the gallery when it is switched on.
fn stage_file(tuning: &Tuning, session: &Session) -> &'static str {
    enemies::gallery_stage(tuning).unwrap_or(STAGES[session.stage.min(1)])
}

/// The Stage's timeline, with its terrain added as scenery (except
/// crumbling ledges, which the gameplay draws so they can fall).
pub fn timeline(file: &str) -> StageTimeline {
    let mut timeline = StageTimeline::parse(file).expect("Cabinet 4 Stage file must be valid");
    let ground = terrain::StageTerrain::parse(file).expect("Cabinet 4 terrain must be valid");
    let crumble = format!("{SCENERY_PREFIX}{}", ground.tiles.crumble);
    timeline
        .spawns
        .extend(ground.scenery().into_iter().filter(|s| s.kind != crumble));
    timeline.spawns.sort_by(|a, b| a.at.total_cmp(&b.at));
    timeline
}

fn spawn_stage(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    sprites: Res<Sprites>,
) {
    let t = &tuning.climber;
    let file = stage_file(&tuning, &session);
    let tower = Tower::parse(file, t).expect("Cabinet 4 terrain must be valid");
    let crumble = terrain::StageTerrain::parse(file)
        .expect("checked above")
        .tiles
        .crumble;
    let power = PowerUps::from_session(&session.power_ups, t);
    let stage = StageRun::new(timeline(file));
    // The Stage starts with the Hero standing on its first landing.
    let start = stage.start_point().hero_pos();
    let mut climber = hero::Climber::standing(start, 0.0, t);
    climber.route_next = hero::route_index_at(&tower, climber.body.x, climber.body.feet);
    hero::spawn_hero(&mut commands, t, &sprites, &power, climber);
    tower::spawn_crumbles(&mut commands, &tower, &crumble, &sprites, CABINET);
    commands.insert_resource(stage);
    commands.insert_resource(tower);
    commands.insert_resource(power);
    commands.insert_resource(DrivenScroll(f32::INFINITY));
    commands.insert_resource(crate::flow::WalkOffSeconds(t.hero.exit_seconds));
    commands.insert_resource(hero::Rescues::default());
    commands.insert_resource(Misses::default());
}

/// Spawns the kinds this module owns: Power-up capsules, and Standoffs.
fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut requests: MessageReader<SpawnRequested>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for request in requests.read() {
        if let Some(power_up) = PowerUp::from_kind(&request.kind) {
            power_ups::spawn_capsule(
                &mut commands,
                power_up,
                request.pos,
                stage.cursor.position,
                &tuning.climber,
            );
        } else if request.kind == standoff::KIND {
            let params: StandoffParams = request.params().unwrap_or_else(|e| {
                warn!("bad Standoff params: {e}");
                StandoffParams::default()
            });
            commands.insert_resource(Standoff::new(params));
            sfx.write(PlaySfx(Sfx::StandoffLock));
        }
    }
}

/// Runs the Standoff: sends its waves, and ends it.
#[allow(clippy::too_many_arguments)]
fn run_standoff(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    standoff: Option<ResMut<Standoff>>,
    members: Query<Entity, With<enemies::StandoffMember>>,
    hero: Single<&Transform, With<crate::kit::Hero>>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut score: MessageWriter<ScoreGained>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let Some(mut standoff) = standoff else {
        return;
    };
    let (due, outcome) = standoff.tick(time.delta_secs(), members.iter().count());
    for wave in due {
        spawns.write(Standoff::request(wave));
    }
    if outcome.is_some() {
        sfx.write(PlaySfx(Sfx::StandoffRelease));
    }
    match outcome {
        Some(Outcome::Cleared { seconds_left }) => {
            let bonus = seconds_left as u64 * tuning.climber.score.standoff_time_bonus;
            if bonus > 0 {
                score.write(ScoreGained(bonus));
            }
            if let Some(drop) = standoff.drop.and_then(PowerUp::from_letter) {
                let at = hero.translation.truncate() + Vec2::new(24.0, 48.0);
                spawns.write(SpawnRequested::new(drop.kind(), at));
            }
            commands.remove_resource::<Standoff>();
        }
        Some(Outcome::TimedOut) => {
            for e in &members {
                commands
                    .entity(e)
                    .try_insert(enemies::Leaving)
                    .try_remove::<enemies::StandoffMember>();
            }
            commands.remove_resource::<Standoff>();
        }
        None => {}
    }
}

/// A rewind goes back to before the Standoff, which replays from the start.
fn end_standoff_on_restart(mut commands: Commands, mut restarted: MessageReader<StageRestarted>) {
    if restarted.read().count() > 0 {
        commands.remove_resource::<Standoff>();
    }
}

/// The Stage's designed length is its pace time: the scroll plus each
/// Standoff at its full length, as the doc's tables count it. Runs in
/// `PostUpdate`, after the kit has started the clock for the new Stage.
fn clock_counts_standoffs(stage: Res<StageRun>, mut clock: ResMut<crate::kit::StageClock>) {
    if stage.is_added() {
        clock.designed += standoff::standoff_seconds(&stage.timeline);
    }
}

/// Lives lost so far in this Stage, for the no-miss bonus.
#[derive(Resource, Default)]
struct Misses(u32);

fn count_misses(mut lost: MessageReader<LifeLost>, misses: Option<ResMut<Misses>>) {
    let n = lost.read().count() as u32;
    if let Some(mut misses) = misses {
        misses.0 += n;
    }
}

/// The Stage clear bonus, plus the no-miss bonus if no life was lost.
pub fn stage_bonus_points(t: &tuning::ClimberTuning, misses: u32) -> u64 {
    t.score.stage_clear + if misses == 0 { t.score.no_miss } else { 0 }
}

/// The Stage clear and no-miss bonuses, paid as the scroll ends (before the
/// flow saves the score and shows the Stage Card), as in Cabinet 1.
fn stage_bonus(
    tuning: Res<Tuning>,
    misses: Res<Misses>,
    mut cleared: MessageReader<StageCleared>,
    mut score: MessageWriter<ScoreGained>,
) {
    if cleared.read().count() > 0 {
        score.write(ScoreGained(stage_bonus_points(&tuning.climber, misses.0)));
    }
}

/// Keeps `Session.power_ups` up to date, so a Save at the Stage clear
/// records the Power-ups.
fn save_power_ups(power: Res<PowerUps>, mut session: ResMut<Session>) {
    if power.is_changed() {
        session.power_ups = power.to_json();
    }
}

fn end_stage(mut commands: Commands) {
    commands.remove_resource::<PowerUps>();
    commands.remove_resource::<Tower>();
    commands.remove_resource::<Standoff>();
    commands.remove_resource::<DrivenScroll>();
    commands.remove_resource::<Misses>();
}

pub struct ClimberPlugin;

impl Plugin for ClimberPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(enemies::EnemiesPlugin)
            .add_systems(
                OnEnter(Screen::Playing),
                spawn_stage.run_if(cabinet_is(CABINET)),
            )
            .add_systems(
                Update,
                (
                    (
                        spawn_requested,
                        run_standoff,
                        hero::run,
                        hero::fire,
                        hero::sync_equipment,
                        power_ups::move_capsules,
                        tower::place_crumbles,
                    )
                        .chain()
                        .after(KitSet::Timeline)
                        .before(enemies::spawn_requested)
                        .before(KitSet::Move),
                    (
                        power_ups::collect_capsules,
                        hero::ward_absorbed,
                        hero::after_restart,
                        end_standoff_on_restart,
                    )
                        .chain()
                        .after(KitSet::Resolve)
                        .before(KitSet::Effects),
                    save_power_ups.after(KitSet::Effects),
                )
                    .run_if(cabinet_is(CABINET))
                    // The Session can move to this Cabinet a frame before the
                    // last one's Stage ends; wait for this Cabinet's own.
                    .run_if(resource_exists::<PowerUps>)
                    .run_if(resource_exists::<Tower>)
                    .run_if(stage_live),
            )
            .add_systems(
                Update,
                (
                    count_misses,
                    stage_bonus
                        .after(KitSet::Timeline)
                        .before(crate::flow::FlowSet),
                )
                    .run_if(cabinet_is(CABINET))
                    .run_if(resource_exists::<Misses>)
                    .run_if(in_state(Screen::Playing)),
            )
            .add_systems(
                PostUpdate,
                clock_counts_standoffs
                    .run_if(cabinet_is(CABINET))
                    .run_if(in_state(Screen::Playing))
                    .run_if(resource_exists::<StageRun>),
            )
            .add_systems(OnExit(Screen::Playing), end_stage);
    }
}

#[cfg(test)]
mod tests;
