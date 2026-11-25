//! Cabinet 3: UNDERFIELD, the run-and-gun platformer.
//! Design: `docs/cabinets/3-run-and-gun.md`.
//!
//! The Hero on foot (`hero.rs`) runs on the kit's platform physics over the
//! Stage file's terrain (`terrain.rs`) and pushes the camera. Enemies and
//! hazards are in `enemies/`, Standoffs in `standoff.rs`. The Stages,
//! scenery, sprites and sounds are #20's.

use bevy::prelude::*;

use super::cabinet_is;
use crate::flow::Screen;
use crate::kit::{DrivenScroll, KitSet, SpawnRequested, StageRun, StageTimeline, stage_live};
use crate::run::Session;
use crate::tuning::Tuning;

pub mod enemies;
pub mod hero;
pub mod palette;
pub mod power_ups;
pub mod standoff;
pub mod terrain;
pub mod tuning;

use power_ups::{PowerUp, PowerUps};
use standoff::{Standoff, StandoffParams};
use terrain::Terrain;

/// This Cabinet's place in the Run.
pub const CABINET: usize = 2;

/// Cabinet 3's two Stage files.
pub const STAGES: [&str; 2] = [
    include_str!("../../../assets/stages/3-1.ron"),
    include_str!("../../../assets/stages/3-2.ron"),
];

#[cfg(test)]
mod stage_checks;

/// The x (world, a scroll position) of something fixed to the ground: its
/// screen x follows the camera.
#[derive(Component, Debug, Clone, Copy)]
pub struct WorldX(pub f32);

/// The Stage file to play: the gallery when it is switched on.
fn stage_file(tuning: &Tuning, session: &Session) -> &'static str {
    enemies::gallery_stage(tuning).unwrap_or(STAGES[session.stage.min(1)])
}

fn spawn_stage(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    sprites: Res<crate::sprites::Sprites>,
) {
    let t = &tuning.run_and_gun;
    let file = stage_file(&tuning, &session);
    let mut timeline = StageTimeline::parse(file).expect("Cabinet 3 Stage file must be valid");
    let terrain = Terrain::parse(file).expect("Cabinet 3 terrain must be valid");
    // The ground is drawn as tiles on the scenery layer; the timeline plays
    // its spawns in order.
    let ground = terrain::StageTerrain::parse(file).expect("checked above");
    timeline.spawns.extend(ground.scenery());
    timeline.spawns.sort_by(|a, b| a.at.total_cmp(&b.at));
    let power = PowerUps::from_session(&session.power_ups, t);
    let stage = StageRun::new(timeline);
    let start = stage.start_point().hero_pos();
    hero::spawn_hero(
        &mut commands,
        t,
        &power,
        start.x,
        stage.cursor.position,
        &sprites,
    );
    terrain::spawn_ledges(&mut commands, &terrain, &sprites, CABINET);
    commands.insert_resource(stage);
    commands.insert_resource(terrain);
    commands.insert_resource(power);
    commands.insert_resource(DrivenScroll(0.0));
    commands.insert_resource(crate::flow::WalkOffSeconds(t.hero.exit_seconds));
    commands.insert_resource(hero::Rescues::default());
}

/// Spawns the kinds this module owns: Power-up capsules, and Standoffs.
fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut requests: MessageReader<SpawnRequested>,
    mut sfx: MessageWriter<crate::audio::PlaySfx>,
) {
    for request in requests.read() {
        if let Some(power_up) = PowerUp::from_kind(&request.kind) {
            power_ups::spawn_capsule(
                &mut commands,
                power_up,
                request.pos,
                stage.cursor.position,
                &tuning.run_and_gun,
            );
        } else if request.kind == standoff::KIND {
            let params: StandoffParams = request.params().unwrap_or_else(|e| {
                warn!("bad Standoff params: {e}");
                StandoffParams::default()
            });
            commands.insert_resource(Standoff::new(params));
            sfx.write(crate::audio::PlaySfx(crate::audio::Sfx::StandoffLock));
        }
    }
}

/// The Stage's designed length is its pace time: the scroll plus each
/// Standoff at its full length, as the doc's tables count it. Runs in
/// `PostUpdate`, after the kit has started the clock for the new Stage.
fn clock_counts_standoffs(stage: Res<StageRun>, mut clock: ResMut<crate::kit::StageClock>) {
    if stage.is_added() {
        clock.designed += crate::cabinets::on_foot::standoff::standoff_seconds(&stage.timeline);
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
    commands.remove_resource::<Terrain>();
    commands.remove_resource::<Standoff>();
    commands.remove_resource::<DrivenScroll>();
}

pub struct RunAndGunPlugin;

impl Plugin for RunAndGunPlugin {
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
                        standoff::run_standoff,
                        hero::run,
                        hero::fire,
                        hero::fly_flares,
                        hero::sync_equipment,
                        hero::animate,
                        power_ups::move_capsules,
                        terrain::place_ledges,
                    )
                        .chain()
                        .after(KitSet::Timeline)
                        .before(enemies::spawn_requested)
                        .before(KitSet::Move),
                    (
                        power_ups::collect_capsules,
                        hero::ward_absorbed,
                        hero::after_restart,
                        standoff::end_on_restart,
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
                    .run_if(resource_exists::<Terrain>)
                    .run_if(stage_live),
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
