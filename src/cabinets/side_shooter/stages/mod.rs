//! Cabinet 1's two Stages (docs/cabinets/1-side-shooter.md): their
//! timelines (`assets/stages/1-1.ron`, `1-2.ron`), parallax backgrounds,
//! Stage 2's terrain, and the Stage clear and no-miss bonuses.

use bevy::prelude::*;

use crate::cabinets::cabinet_is;
use crate::cabinets::{LifeLost, ScoreGained, StageCleared};
use crate::flow::{FlowSet, Screen};
use crate::kit::{Hitbox, KitSet, Layer, StageRun, StageTimeline, stage_live};
use crate::run::Session;
use crate::tuning::Tuning;

use super::palette::SLATE;
use crate::sprites::Sprites;

pub mod scenery;
pub mod terrain;

use terrain::{SLOTS, TILE, Terrain};

pub const STAGE_FILES: [&str; 2] = [
    include_str!("../../../../assets/stages/1-1.ron"),
    include_str!("../../../../assets/stages/1-2.ron"),
];

use super::CABINET;

/// The timeline of Stage `stage` (0 or 1).
pub fn timeline(stage: usize) -> StageTimeline {
    StageTimeline::parse(STAGE_FILES[stage.min(1)]).expect("Cabinet 1 Stage file must be valid")
}

/// The terrain of Stage `stage` (empty for Stage 1).
pub fn terrain(stage: usize) -> Terrain {
    Terrain::parse(STAGE_FILES[stage.min(1)]).expect("Cabinet 1 Stage file must be valid")
}

/// Lives lost so far in this Stage, for the no-miss bonus.
#[derive(Resource, Default)]
struct Misses(u32);

/// One 8 px column of terrain: its floor and ceiling blocks and their lips.
#[derive(Component, Clone, Copy)]
struct Column {
    slot: i32,
    ceiling: bool,
}

#[derive(Component, Clone, Copy)]
struct Lip {
    slot: i32,
    ceiling: bool,
}

fn spawn_stage(
    mut commands: Commands,
    session: Res<Session>,
    tuning: Res<Tuning>,
    sprites: Res<Sprites>,
) {
    commands.insert_resource(Misses::default());
    let gallery = tuning.debug_keys && tuning.side_shooter.enemy_gallery;
    if session.stage == 0 || gallery {
        commands.remove_resource::<Terrain>();
        scenery::spawn_low_tide(&mut commands, &sprites);
        return;
    }
    let runout = timeline(1).speed.last().map_or(f32::INFINITY, |k| k.from);
    scenery::spawn_foundry(&mut commands, &sprites, runout);
    for slot in 0..SLOTS {
        for ceiling in [false, true] {
            commands.spawn((
                Column { slot, ceiling },
                Sprite::from_color(SLATE, Vec2::new(TILE, 1.0)),
                Transform::from_xyz(0.0, 0.0, 3.0),
                Visibility::Hidden,
                Hitbox::new(0.0, 0.0),
                Layer::Terrain,
                DespawnOnExit(Screen::Playing),
            ));
            let mut edge = sprites.sprite("foundry-edge", CABINET);
            edge.flip_y = ceiling;
            commands.spawn((
                Lip { slot, ceiling },
                edge,
                Transform::from_xyz(0.0, 0.0, 3.1),
                Visibility::Hidden,
                DespawnOnExit(Screen::Playing),
            ));
        }
    }
    commands.insert_resource(terrain(1));
}

fn scroll_scenery(
    stage: Res<StageRun>,
    pieces: Query<(&scenery::Parallax, &mut Transform)>,
    specks: Query<
        (&scenery::ShaftSpeck, &mut Transform, &mut Visibility),
        Without<scenery::Parallax>,
    >,
) {
    let position = stage.cursor.position;
    scenery::scroll(position, pieces);
    scenery::rise(position, specks);
}

/// Places every terrain column for the current scroll position.
#[allow(clippy::type_complexity)]
fn place_terrain(
    stage: Res<StageRun>,
    terrain: Res<Terrain>,
    mut columns: Query<
        (
            &Column,
            &mut Transform,
            &mut Sprite,
            &mut Hitbox,
            &mut Visibility,
        ),
        Without<Lip>,
    >,
    mut lips: Query<(&Lip, &mut Transform, &mut Visibility), Without<Column>>,
) {
    let position = stage.cursor.position;
    let top = super::enemies::brain::PLAY_TOP;
    let bottom = super::enemies::brain::PLAY_BOTTOM;
    let place = |slot: i32, ceiling: bool| {
        let c = terrain::column_for_slot(slot, position);
        let s = c as f32 * TILE + TILE / 2.0;
        let x = terrain::screen_x(s, position).round();
        let (floor, roof) = terrain.heights_at(s);
        let (from, to) = if ceiling {
            (roof, top)
        } else {
            (bottom, floor)
        };
        (x, from, to)
    };
    for (column, mut t, mut sprite, mut hitbox, mut vis) in &mut columns {
        let (x, from, to) = place(column.slot, column.ceiling);
        let height = (to - from).max(0.0);
        t.translation.x = x;
        t.translation.y = (from + to) / 2.0;
        sprite.custom_size = Some(Vec2::new(TILE, height));
        hitbox.size = Vec2::new(TILE, height);
        vis.set_if_neq(if height > 0.0 {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (lip, mut t, mut vis) in &mut lips {
        let (x, from, to) = place(lip.slot, lip.ceiling);
        t.translation.x = x;
        // The surface tile sits just inside the terrain.
        t.translation.y = if lip.ceiling {
            from + TILE / 2.0
        } else {
            to - TILE / 2.0
        };
        vis.set_if_neq(if to > from {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

fn count_misses(mut lost: MessageReader<LifeLost>, misses: Option<ResMut<Misses>>) {
    let n = lost.read().count() as u32;
    if let Some(mut misses) = misses {
        misses.0 += n;
    }
}

/// The Stage clear and no-miss bonuses, paid as the scroll ends (before the
/// flow saves the score and shows the Stage Card).
fn stage_bonus(
    tuning: Res<Tuning>,
    misses: Option<Res<Misses>>,
    mut cleared: MessageReader<StageCleared>,
    mut score: MessageWriter<ScoreGained>,
) {
    if cleared.read().count() == 0 {
        return;
    }
    let s = &tuning.side_shooter.score;
    let mut points = s.stage_clear;
    if misses.is_none_or(|m| m.0 == 0) {
        points += s.no_miss;
    }
    score.write(ScoreGained(points));
}

/// Game seconds played in this Stage (stops while paused or in hit-stop).
#[derive(Resource, Default, Clone, Copy, serde::Serialize)]
pub struct StageClock {
    pub stage: usize,
    pub seconds: f32,
    /// Fraction of the Stage scrolled, 0 to 1.
    pub progress: f32,
}

fn tick_clock(
    time: Res<Time>,
    stage: Res<StageRun>,
    session: Res<Session>,
    mut clock: ResMut<StageClock>,
) {
    clock.stage = session.stage;
    clock.seconds += time.delta_secs();
    clock.progress = stage.progress();
}

/// Debug aid for the browser tests: the Stage's real progress, in
/// localStorage as `scroller.debug.stage`, so a test can wait on the game
/// rather than the wall clock (which a slow browser can't keep up with).
#[cfg(target_arch = "wasm32")]
fn report_clock(
    tuning: Res<Tuning>,
    real: Res<Time<Real>>,
    clock: Res<StageClock>,
    mut every: Local<f32>,
) {
    *every -= real.delta_secs();
    if tuning.debug_keys && *every <= 0.0 {
        *every = 0.2;
        crate::storage::store("debug.stage", &*clock);
    }
}

/// The final reading as the Stage ends. By then the Session may already be on
/// the next Cabinet, so this one isn't gated on Cabinet 1.
#[cfg(target_arch = "wasm32")]
fn report_final_clock(mut commands: Commands, tuning: Res<Tuning>, clock: Res<StageClock>) {
    if tuning.debug_keys {
        crate::storage::store("debug.stage", &*clock);
    }
    commands.remove_resource::<StageClock>();
}

pub struct StagesPlugin;

impl Plugin for StagesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::Playing),
            (spawn_stage, |mut commands: Commands| {
                commands.insert_resource(StageClock::default());
            })
                .run_if(cabinet_is(CABINET)),
        )
        .add_systems(
            Update,
            tick_clock
                .after(KitSet::Timeline)
                .run_if(resource_exists::<StageClock>)
                .run_if(cabinet_is(CABINET))
                .run_if(stage_live),
        )
        .add_systems(OnExit(Screen::Playing), |mut commands: Commands| {
            commands.remove_resource::<Terrain>();
            commands.remove_resource::<Misses>();
        })
        .add_systems(
            Update,
            (
                (
                    scroll_scenery,
                    place_terrain.run_if(resource_exists::<Terrain>),
                )
                    .after(KitSet::Timeline)
                    .before(KitSet::Move),
                stage_bonus.after(KitSet::Timeline).before(FlowSet),
            )
                .run_if(cabinet_is(CABINET))
                .run_if(stage_live),
        )
        // Lives can be lost during the hit-stop, so this one always listens.
        .add_systems(
            Update,
            count_misses
                .run_if(in_state(Screen::Playing))
                .run_if(cabinet_is(CABINET)),
        );
        #[cfg(target_arch = "wasm32")]
        app.add_systems(
            Update,
            report_clock
                .run_if(resource_exists::<StageClock>)
                .run_if(cabinet_is(CABINET)),
        )
        .add_systems(
            OnExit(Screen::Playing),
            report_final_clock.run_if(resource_exists::<StageClock>),
        );
    }
}

#[cfg(test)]
pub(crate) mod tests;
