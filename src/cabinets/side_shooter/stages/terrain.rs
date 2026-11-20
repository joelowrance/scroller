//! Stage 2's floor and ceiling.
//!
//! A Stage file can list `terrain` keys: floor and ceiling heights (canvas
//! y) at scroll positions, joined by straight ramps and snapped to the 8 px
//! tile grid measured from the play-area edges. A key at scroll position
//! `s` reaches the right edge of the screen when the Stage has scrolled to
//! `s`, the same as a spawn at `s`.
//!
//! On screen the terrain is a ring of 8 px columns, each placed from the
//! scroll position every frame, so a rewind to a Restart Point needs no
//! special handling.

use bevy::prelude::*;
use serde::Deserialize;

use crate::cabinets::side_shooter::enemies::brain::{PLAY_BOTTOM, PLAY_TOP};
use crate::display::HALF_W;

/// One terrain key from a Stage file.
#[derive(Clone, Copy, Debug, PartialEq, Deserialize)]
pub struct TerrainKey {
    pub at: f32,
    pub floor: f32,
    pub ceiling: f32,
}

/// The terrain section of a Stage file. Other fields belong to the kit.
#[derive(Clone, Debug, Default, Deserialize, Resource)]
#[serde(default)]
pub struct Terrain {
    pub terrain: Vec<TerrainKey>,
}

impl Terrain {
    pub fn parse(stage_file: &str) -> Result<Self, String> {
        let mut t: Self = ron::from_str(stage_file).map_err(|e| e.to_string())?;
        t.terrain.sort_by(|a, b| a.at.total_cmp(&b.at));
        Ok(t)
    }

    pub fn is_empty(&self) -> bool {
        self.terrain.is_empty()
    }

    /// Floor and ceiling at scroll position `s`, on the tile grid. With no
    /// terrain, the play-area edges.
    pub fn heights_at(&self, s: f32) -> (f32, f32) {
        let keys = &self.terrain;
        let (floor, ceiling) = match keys.iter().position(|k| k.at > s) {
            _ if keys.is_empty() => (PLAY_BOTTOM, PLAY_TOP),
            Some(0) => (keys[0].floor, keys[0].ceiling),
            None => {
                let k = keys[keys.len() - 1];
                (k.floor, k.ceiling)
            }
            Some(i) => {
                let (a, b) = (keys[i - 1], keys[i]);
                let f = ((s - a.at) / (b.at - a.at).max(0.001)).clamp(0.0, 1.0);
                (a.floor.lerp(b.floor, f), a.ceiling.lerp(b.ceiling, f))
            }
        };
        (snap_floor(floor), snap_ceiling(ceiling))
    }

    /// The tightest corridor over scroll positions `from..to`: the highest
    /// floor and the lowest ceiling.
    pub fn corridor(&self, from: f32, to: f32) -> (f32, f32) {
        let mut tightest = (PLAY_BOTTOM, PLAY_TOP);
        let mut s = from;
        while s <= to {
            let (f, c) = self.heights_at(s);
            tightest = (tightest.0.max(f), tightest.1.min(c));
            s += TILE / 2.0;
        }
        tightest
    }
}

/// The tile size; terrain heights snap to it.
pub const TILE: f32 = 8.0;

fn snap_floor(y: f32) -> f32 {
    PLAY_BOTTOM + ((y - PLAY_BOTTOM) / TILE).round().max(0.0) * TILE
}

fn snap_ceiling(y: f32) -> f32 {
    PLAY_TOP - ((PLAY_TOP - y) / TILE).round().max(0.0) * TILE
}

/// Screen x of scroll position `s` when the Stage has scrolled to `position`.
pub fn screen_x(s: f32, position: f32) -> f32 {
    s - position + HALF_W
}

/// Scroll position under screen x `x`.
pub fn scroll_at(x: f32, position: f32) -> f32 {
    x + position - HALF_W
}

/// Number of column slots: enough to cover the screen plus one each side.
pub const SLOTS: i32 = 44;

/// The tile column a slot shows at a scroll position: the one in view whose
/// index is congruent to the slot.
pub fn column_for_slot(slot: i32, position: f32) -> i32 {
    let first = (scroll_at(-HALF_W, position) / TILE).floor() as i32 - 1;
    first + (slot - first).rem_euclid(SLOTS)
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAGE: &str = r#"(
        length: 1000.0,
        speed: [(from: 0.0, speed: 30.0)],
        terrain: [
            (at: 100.0, floor: -56.0, ceiling: 44.0),
            (at: 0.0, floor: -120.0, ceiling: 108.0),
        ],
    )"#;

    #[test]
    fn heights_ramp_between_keys_on_the_grid() {
        let t = Terrain::parse(STAGE).unwrap();
        assert_eq!(t.heights_at(-50.0), (-120.0, 108.0));
        assert_eq!(t.heights_at(0.0), (-120.0, 108.0));
        assert_eq!(t.heights_at(100.0), (-56.0, 44.0));
        assert_eq!(t.heights_at(500.0), (-56.0, 44.0));
        let (f, c) = t.heights_at(50.0);
        assert_eq!((f, c), (-88.0, 76.0));
        let (f, _) = t.heights_at(37.0);
        assert_eq!((f - PLAY_BOTTOM) % TILE, 0.0);
    }

    #[test]
    fn no_terrain_means_the_play_area_edges() {
        let t = Terrain::parse("(length: 1.0, speed: [])").unwrap();
        assert!(t.is_empty());
        assert_eq!(t.heights_at(10.0), (PLAY_BOTTOM, PLAY_TOP));
    }

    #[test]
    fn corridor_is_the_tightest_span() {
        let t = Terrain::parse(STAGE).unwrap();
        assert_eq!(t.corridor(-100.0, 0.0), (-120.0, 108.0));
        assert_eq!(t.corridor(-100.0, 200.0), (-56.0, 44.0));
    }

    #[test]
    fn slots_cover_every_visible_column_once() {
        for position in [0.0, 3.5, 812.25, 5000.0] {
            let mut cols: Vec<i32> = (0..SLOTS).map(|s| column_for_slot(s, position)).collect();
            cols.sort();
            cols.dedup();
            assert_eq!(cols.len(), SLOTS as usize);
            let left = scroll_at(-HALF_W, position);
            let right = scroll_at(HALF_W, position);
            for c in (left / TILE).floor() as i32..=(right / TILE).floor() as i32 {
                assert!(cols.contains(&c), "column {c} at {position}");
            }
        }
    }

    #[test]
    fn a_key_reaches_the_right_edge_when_the_scroll_reaches_it() {
        assert_eq!(screen_x(500.0, 500.0), HALF_W);
        assert_eq!(scroll_at(HALF_W, 500.0), 500.0);
    }
}
