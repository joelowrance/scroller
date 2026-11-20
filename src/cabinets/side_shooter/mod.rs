//! Cabinet 1: STATIC TIDE, the side-scrolling shooter.
//! Design: `docs/cabinets/1-side-shooter.md`.
//!
//! This module owns the Hero's ship (the Wren), its Power-ups and the Stage
//! set-up. Enemies and Formations live in `enemies/` (#8).

use bevy::prelude::*;

use super::cabinet_is;
use crate::flow::Screen;
use crate::kit::{KitSet, SpawnRequested, StageRun, StageTimeline, stage_live};
use crate::run::Session;
use crate::tuning::Tuning;

pub mod enemies;
pub mod hero;
pub mod palette;
pub mod power_ups;
pub mod tuning;

use power_ups::{PowerUp, PowerUps};

/// This Cabinet's place in the Run.
pub const CABINET: usize = 0;

/// Stand-in Stage used for both Stages until #9 writes `1-1.ron` and
/// `1-2.ron`.
const STAND_IN_STAGE: &str = include_str!("../../../assets/stages/side-shooter-stand-in.ron");

/// The debug enemy gallery when it is switched on, else the stand-in Stage.
fn stage_timeline(tuning: &Tuning) -> StageTimeline {
    enemies::gallery_timeline(tuning).unwrap_or_else(|| {
        StageTimeline::parse(STAND_IN_STAGE).expect("Cabinet 1 Stage file must be valid")
    })
}

fn spawn_stage(mut commands: Commands, tuning: Res<Tuning>, session: Res<Session>) {
    let t = &tuning.side_shooter;
    let power = PowerUps::from_session(&session.power_ups, t);
    commands.insert_resource(StageRun::new(stage_timeline(&tuning)));
    hero::spawn_wren(&mut commands, t, &power);
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
            power_ups::spawn_capsule(&mut commands, power_up, request.pos, &tuning.side_shooter);
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

pub struct SideShooterPlugin;

impl Plugin for SideShooterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(enemies::EnemiesPlugin)
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
                        hero::bomb,
                        hero::sync_equipment,
                        power_ups::move_capsules,
                    )
                        .chain()
                        .after(spawn_requested)
                        .before(KitSet::Move),
                    (
                        hero::grow_undertow,
                        power_ups::collect_capsules,
                        hero::ward_absorbed,
                        hero::after_restart,
                    )
                        .chain()
                        .after(KitSet::Resolve)
                        .before(KitSet::Effects),
                    (
                        hero::drones_blink,
                        hero::flicker_engine,
                        hero::show_bombs,
                        save_power_ups,
                    )
                        .after(KitSet::Effects),
                )
                    .run_if(cabinet_is(CABINET))
                    .run_if(stage_live),
            )
            .add_systems(OnExit(Screen::Playing), |mut commands: Commands| {
                commands.remove_resource::<PowerUps>();
            });
    }
}

#[cfg(test)]
mod tests;
