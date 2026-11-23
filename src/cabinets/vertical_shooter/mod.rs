//! Cabinet 2: SPIREWARD, the vertical shooter.
//! Design: `docs/cabinets/2-vertical-shooter.md`.
//!
//! This module owns the Wren nose up, its Power-ups and the Stage set-up.
//! Enemies, the air and ground layers and Formations live in `enemies/`.
//! The Stages and scenery (`assets/stages/2-*.ron`) are #15's.

use bevy::prelude::*;

use super::{LifeLost, ScoreGained, StageCleared, cabinet_is};
use crate::flow::{FlowSet, Screen};
use crate::kit::{KitSet, SpawnRequested, StageRun, StageTimeline, stage_live};
use crate::run::Session;
use crate::sprites::Sprites;
use crate::tuning::Tuning;

pub mod enemies;
pub mod hero;
pub mod palette;
pub mod power_ups;
pub mod sound;
pub mod tuning;

use power_ups::{PowerUp, PowerUps};

/// This Cabinet's place in the Run.
pub const CABINET: usize = 1;

/// The Stage files (#15): Cloudline and The Outer Yard.
const STAGES: [&str; 2] = [
    include_str!("../../../assets/stages/2-1.ron"),
    include_str!("../../../assets/stages/2-2.ron"),
];

/// The debug enemy gallery when it is switched on, else the Session's Stage.
fn stage_timeline(tuning: &Tuning, stage: usize) -> StageTimeline {
    enemies::gallery_timeline(tuning).unwrap_or_else(|| {
        StageTimeline::parse(STAGES[stage.min(1)]).expect("Cabinet 2 Stage files must be valid")
    })
}

fn spawn_stage(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    sprites: Res<Sprites>,
) {
    let t = &tuning.vertical_shooter;
    let power = PowerUps::from_session(&session.power_ups, t);
    commands.insert_resource(StageRun::new(stage_timeline(&tuning, session.stage)));
    hero::spawn_wren(&mut commands, &sprites, t, &power);
    commands.insert_resource(power);
    commands.insert_resource(Misses::default());
}

/// Spawns the kinds this module owns: Power-up capsules (`power_up:<letter>`).
fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut requests: MessageReader<SpawnRequested>,
) {
    for request in requests.read() {
        if let Some(power_up) = PowerUp::from_kind(&request.kind) {
            power_ups::spawn_capsule(
                &mut commands,
                power_up,
                request.pos,
                &tuning.vertical_shooter,
            );
        }
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
pub fn stage_bonus_points(t: &tuning::VerticalShooterTuning, misses: u32) -> u64 {
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
        score.write(ScoreGained(stage_bonus_points(
            &tuning.vertical_shooter,
            misses.0,
        )));
    }
}

/// Keeps `Session.power_ups` up to date, so a Save at the Stage clear
/// records the Power-ups.
fn save_power_ups(power: Res<PowerUps>, mut session: ResMut<Session>) {
    if power.is_changed() {
        session.power_ups = power.to_json();
    }
}

pub struct VerticalShooterPlugin;

impl Plugin for VerticalShooterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((enemies::EnemiesPlugin, sound::SoundPlugin))
            .add_systems(
                OnEnter(Screen::Playing),
                spawn_stage.run_if(cabinet_is(CABINET)),
            )
            .add_systems(
                Update,
                (
                    spawn_requested.after(KitSet::Timeline),
                    (
                        hero::fly_in,
                        hero::steer,
                        hero::fire,
                        hero::steer_rockets,
                        hero::bomb,
                        hero::sync_equipment,
                        power_ups::move_capsules,
                    )
                        .chain()
                        .after(spawn_requested)
                        .before(KitSet::Move),
                    (
                        hero::sweep_squall,
                        power_ups::collect_capsules,
                        hero::ward_absorbed,
                        hero::after_restart,
                    )
                        .chain()
                        .after(KitSet::Resolve)
                        .before(KitSet::Effects),
                    (hero::pods_blink, hero::show_bombs, save_power_ups).after(KitSet::Effects),
                    stage_bonus.after(KitSet::Timeline).before(FlowSet),
                )
                    .run_if(cabinet_is(CABINET))
                    // The Session moves to this Cabinet one frame before
                    // Cabinet 1's Stage ends; wait for this Cabinet's own.
                    .run_if(resource_exists::<PowerUps>)
                    .run_if(stage_live),
            )
            // Lives can be lost during the hit-stop, so this one always listens.
            .add_systems(
                Update,
                count_misses
                    .run_if(in_state(Screen::Playing))
                    .run_if(cabinet_is(CABINET)),
            )
            .add_systems(OnExit(Screen::Playing), |mut commands: Commands| {
                commands.remove_resource::<PowerUps>();
                commands.remove_resource::<Misses>();
            });
    }
}

#[cfg(test)]
mod balance;
#[cfg(test)]
mod tests;
