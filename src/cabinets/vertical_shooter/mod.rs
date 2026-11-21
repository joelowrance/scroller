//! Cabinet 2: SPIREWARD, the vertical shooter.
//! Design: `docs/cabinets/2-vertical-shooter.md`.
//!
//! This module owns the Wren nose up, its Power-ups and the Stage set-up.
//! Enemies, the air and ground layers and Formations live in `enemies/`.
//! The Stages and scenery (`assets/stages/2-*.ron`) are #15's.

use bevy::prelude::*;

use super::cabinet_is;
use crate::flow::Screen;
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
                )
                    .run_if(cabinet_is(CABINET))
                    // The Session moves to this Cabinet one frame before
                    // Cabinet 1's Stage ends; wait for this Cabinet's own.
                    .run_if(resource_exists::<PowerUps>)
                    .run_if(stage_live),
            )
            .add_systems(OnExit(Screen::Playing), |mut commands: Commands| {
                commands.remove_resource::<PowerUps>();
            });
    }
}

#[cfg(test)]
mod tests;
