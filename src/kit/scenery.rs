//! Scenery from the Stage file: backgrounds and terrain art that scrolls with
//! the Stage but is never hit.
//!
//! A spawn entry whose kind is `scenery:<sprite>` places that sprite (from
//! `assets/sprites/`, in the Cabinet's palette) at `pos` when the scroll
//! reaches `at`. From there it moves against the scroll direction at `depth`
//! times the scroll speed. Optional params:
//!
//! - `depth` (default 1.0): 1.0 moves with the ground, so it stays lined up
//!   with ground targets spawned at the same `at`; less is a slower, farther
//!   layer.
//! - `frame` (default 0): which frame of the sprite to show.
//! - `anim` (seconds per frame): loop through every frame instead.
//! - `flip` (default false): mirror the sprite left to right.
//! - `z` (default from `depth`): draw order; scenery stays under the action.
//! - `until`: show the piece only from `at` until the scroll reaches this
//!   position (for something that appears and goes, like a burst in the sky,
//!   or one Spire swapped for a bigger one).
//! - `repeat` and `every`: `repeat` more copies, each `every` scroll pixels
//!   after the last (for a long cloud bank or a fence).
//!
//! Pieces are placed from the scroll position every frame rather than moved,
//! so a rewind to a Restart Point puts every piece back exactly. Only pieces
//! on screen have entities.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::Deserialize;

use super::timeline::{SpawnEntry, StageRun};
use crate::cabinets::{CABINETS, Scroll};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::run::Session;
use crate::sprites::{SpriteAnimation, Sprites};

pub const SCENERY_PREFIX: &str = "scenery:";

#[derive(Deserialize)]
#[serde(default)]
struct SceneryParams {
    depth: f32,
    frame: usize,
    flip: bool,
    #[serde(deserialize_with = "some")]
    z: Option<f32>,
    #[serde(deserialize_with = "some")]
    until: Option<f32>,
    #[serde(deserialize_with = "some")]
    anim: Option<f32>,
    repeat: u32,
    every: f32,
}

/// Reads a plain number in a Stage file as `Some(number)`.
fn some<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<f32>, D::Error> {
    f32::deserialize(d).map(Some)
}

impl Default for SceneryParams {
    fn default() -> Self {
        Self {
            depth: 1.0,
            frame: 0,
            anim: None,
            flip: false,
            z: None,
            until: None,
            repeat: 0,
            every: 0.0,
        }
    }
}

/// One piece of scenery, from a Stage file entry.
#[derive(Clone, Debug, PartialEq)]
pub struct Piece {
    pub sprite: String,
    /// Scroll position at which the piece is at `pos`.
    pub at: f32,
    pub pos: Vec2,
    pub depth: f32,
    pub frame: usize,
    pub anim: Option<f32>,
    pub flip: bool,
    pub z: f32,
    /// With `until`, the piece shows only for scroll positions in `at..until`.
    pub until: Option<f32>,
}

impl Piece {
    /// Whether the piece exists at this scroll position (see `until`).
    pub fn shown_at(&self, position: f32) -> bool {
        self.until
            .is_none_or(|until| (self.at..until).contains(&position))
    }

    /// Where the piece is when the scroll is at `position`.
    pub fn place(&self, position: f32, scroll: Scroll) -> Vec2 {
        let moved = (position - self.at) * self.depth;
        match scroll {
            Scroll::Sideways => self.pos - Vec2::new(moved, 0.0),
            Scroll::Upward => self.pos - Vec2::new(0.0, moved),
        }
    }
}

/// Default draw order: far layers at the back, the ground in front of them,
/// all well behind enemies, shots and the Hero.
fn default_z(depth: f32) -> f32 {
    -100.0 + depth.clamp(0.0, 1.5) * 40.0
}

/// The scenery pieces in a Stage's spawn list, with repeats expanded.
pub fn pieces(spawns: &[SpawnEntry]) -> Result<Vec<Piece>, String> {
    let mut out = Vec::new();
    for entry in spawns {
        let Some(sprite) = entry.kind.strip_prefix(SCENERY_PREFIX) else {
            continue;
        };
        let params: SceneryParams = ron::Value::Map(entry.params.clone())
            .into_rust()
            .map_err(|e| format!("{} at {}: {e}", entry.kind, entry.at))?;
        if params.repeat > 0 && params.every <= 0.0 {
            return Err(format!(
                "{} at {}: repeat needs a positive every",
                entry.kind, entry.at
            ));
        }
        for i in 0..=params.repeat {
            out.push(Piece {
                sprite: sprite.to_string(),
                at: entry.at + i as f32 * params.every,
                pos: Vec2::new(entry.pos.0, entry.pos.1),
                depth: params.depth,
                frame: params.frame,
                anim: params.anim,
                flip: params.flip,
                z: params.z.unwrap_or_else(|| default_z(params.depth)),
                until: params.until.map(|u| u + i as f32 * params.every),
            });
        }
    }
    Ok(out)
}

/// Whether a piece of this size at `at` overlaps the screen.
fn on_screen(at: Vec2, size: Vec2) -> bool {
    at.x.abs() < HALF_W + size.x / 2.0 && at.y.abs() < HALF_H + size.y / 2.0
}

#[derive(Component)]
pub struct SceneryPiece(usize);

/// The current Stage's scenery and which pieces have entities.
#[derive(Default)]
pub(super) struct SceneryState {
    pieces: Vec<Piece>,
    sizes: Vec<Vec2>,
    live: HashMap<usize, Entity>,
}

pub(super) fn sync_scenery(
    mut commands: Commands,
    mut state: Local<SceneryState>,
    stage: Res<StageRun>,
    session: Res<Session>,
    sprites: Option<Res<Sprites>>,
    mut placed: Query<&mut Transform, With<SceneryPiece>>,
) {
    let Some(sprites) = sprites else { return };
    if stage.is_added() {
        // A new Stage: the old pieces went with the last screen.
        state.live.clear();
        state.pieces = pieces(&stage.timeline.spawns).unwrap_or_else(|e| {
            error!("Stage scenery: {e}");
            Vec::new()
        });
        state.sizes = state
            .pieces
            .iter()
            .map(|p| sprites.frame_size(&p.sprite).as_vec2())
            .collect();
    }
    let scroll = CABINETS[session.cabinet].scroll;
    let position = stage.cursor.position;
    let SceneryState {
        pieces,
        sizes,
        live,
    } = &mut *state;
    for (i, piece) in pieces.iter().enumerate() {
        let at = piece.place(position, scroll);
        let visible = piece.shown_at(position) && on_screen(at, sizes[i]);
        // Whole pixels, offset by half a pixel for odd sizes, so art stays crisp.
        let snap = |v: f32, size: f32| {
            if size as i32 % 2 == 0 {
                v.round()
            } else {
                v.floor() + 0.5
            }
        };
        let at = Vec3::new(snap(at.x, sizes[i].x), snap(at.y, sizes[i].y), piece.z);
        match (visible, live.get(&i)) {
            (true, Some(&entity)) => {
                if let Ok(mut transform) = placed.get_mut(entity) {
                    transform.translation = at;
                }
            }
            (true, None) => {
                let mut sprite = sprites.frame(&piece.sprite, session.cabinet, piece.frame);
                sprite.flip_x = piece.flip;
                let mut entity = commands.spawn((
                    sprite,
                    Transform::from_translation(at),
                    SceneryPiece(i),
                    DespawnOnExit(Screen::Playing),
                ));
                if let Some(seconds) = piece.anim {
                    let frames: Vec<usize> = (0..sprites.frame_count(&piece.sprite)).collect();
                    entity.insert(SpriteAnimation::looping(frames, seconds));
                }
                live.insert(i, entity.id());
            }
            (false, Some(&entity)) => {
                commands.entity(entity).try_despawn();
                live.remove(&i);
            }
            (false, None) => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kit::StageTimeline;

    const STAGE: &str = r#"(
        length: 1000.0,
        speed: [(from: 0.0, speed: 20.0)],
        spawns: [
            (at: 0.0, kind: "wisp", pos: (0.0, 130.0)),
            (at: 100.0, kind: "scenery:mesa", pos: (40.0, 150.0)),
            (at: 0.0, kind: "scenery:cloud", pos: (0.0, 0.0),
             params: {"depth": 0.5, "repeat": 2, "every": 300.0, "flip": true}),
        ],
    )"#;

    fn stage_pieces() -> Vec<Piece> {
        pieces(&StageTimeline::parse(STAGE).unwrap().spawns).unwrap()
    }

    #[test]
    fn only_scenery_entries_become_pieces_and_repeats_expand() {
        let p = stage_pieces();
        let names: Vec<(&str, f32)> = p.iter().map(|p| (p.sprite.as_str(), p.at)).collect();
        assert_eq!(
            names,
            [
                ("cloud", 0.0),
                ("cloud", 300.0),
                ("cloud", 600.0),
                ("mesa", 100.0)
            ]
        );
        assert!(p[0].flip && !p[3].flip);
        assert!(p[0].z < p[3].z, "far layers draw behind the ground");
    }

    #[test]
    fn ground_moves_with_the_scroll_and_far_layers_slower() {
        let p = stage_pieces();
        let mesa = &p[3];
        assert_eq!(mesa.place(100.0, Scroll::Upward), Vec2::new(40.0, 150.0));
        assert_eq!(mesa.place(160.0, Scroll::Upward), Vec2::new(40.0, 90.0));
        // Before its `at` it is further up, so a rewind puts it back.
        assert_eq!(mesa.place(50.0, Scroll::Upward), Vec2::new(40.0, 200.0));
        assert_eq!(p[0].place(100.0, Scroll::Upward), Vec2::new(0.0, -50.0));
        assert_eq!(mesa.place(160.0, Scroll::Sideways), Vec2::new(-20.0, 150.0));
    }

    #[test]
    fn until_limits_a_piece_to_its_window() {
        let stage = StageTimeline::parse(
            r#"(length: 10.0, speed: [(from: 0.0, speed: 1.0)],
                spawns: [(at: 5.0, kind: "scenery:x", pos: (0.0, 0.0), params: {"until": 8.0})])"#,
        )
        .unwrap();
        let p = &pieces(&stage.spawns).unwrap()[0];
        assert!(!p.shown_at(4.0) && p.shown_at(5.0) && p.shown_at(7.9) && !p.shown_at(8.0));
    }

    #[test]
    fn a_repeat_without_spacing_is_an_error() {
        let stage = StageTimeline::parse(
            r#"(length: 10.0, speed: [(from: 0.0, speed: 1.0)],
                spawns: [(at: 0.0, kind: "scenery:x", pos: (0.0, 0.0), params: {"repeat": 3})])"#,
        )
        .unwrap();
        assert!(pieces(&stage.spawns).is_err());
    }

    #[test]
    fn on_screen_counts_partly_visible_pieces() {
        assert!(on_screen(Vec2::new(0.0, 125.0), Vec2::new(10.0, 12.0)));
        assert!(!on_screen(Vec2::new(0.0, 127.0), Vec2::new(10.0, 12.0)));
    }
}
