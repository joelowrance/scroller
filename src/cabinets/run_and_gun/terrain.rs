//! Cabinet 3's terrain, from the `ground` and `ledges` lists in its Stage
//! files (docs/cabinets/3-run-and-gun.md, "Terrain").
//!
//! Everything is in whole 8 px tiles. x is a scroll position, like a spawn's
//! `at`: a piece at `s` reaches the right edge of the screen when the camera
//! reaches `s`. Rows count up from the bottom of the play area (row 0 is
//! y = −120).
//!
//! - `ground`: solid from the bottom up to `top` rows; anything not covered
//!   is a pit.
//! - `ledges`: one tile thick, occupying row `row` (standing surface at the
//!   top of that row); `kind` is `"thin"` or `"crumble"`.
//! - `tiles` (optional): the sprites to draw it with; Stage 2's deck plates
//!   differ from Stage 1's dirt.
//!
//! Until the Cabinet's own gameplay (#19) draws it, [`StageTerrain::scenery`]
//! turns the terrain into `scenery:` entries for the kit's scenery layer.

use serde::Deserialize;

use crate::display::HALF_W;
use crate::kit::scenery::SCENERY_PREFIX;
use crate::kit::timeline::SpawnEntry;

pub const TILE: f32 = 8.0;
/// The bottom of the play area: the bottom of row 0.
pub const FLOOR_Y: f32 = -120.0;

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Ground {
    pub from: f32,
    pub to: f32,
    /// Solid rows from the bottom: the surface is at `surface(top)`.
    pub top: u32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Ledge {
    pub from: f32,
    pub to: f32,
    /// The tile row the ledge occupies; it is stood on at `surface(row + 1)`.
    pub row: u32,
    /// `"thin"` or `"crumble"`.
    pub kind: String,
}

impl Ledge {
    // The Stage checks use these now; the gameplay (#19) will too.
    #[allow(dead_code)]
    pub fn surface(&self) -> f32 {
        surface(self.row + 1)
    }

    pub fn crumbles(&self) -> bool {
        self.kind == "crumble"
    }
}

/// The sprites a Stage draws its terrain with.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Tiles {
    pub top: String,
    pub fill: String,
    pub ledge: String,
    pub crumble: String,
}

impl Default for Tiles {
    fn default() -> Self {
        Self {
            top: "c3_ground_top".into(),
            fill: "c3_ground_fill".into(),
            ledge: "c3_ledge".into(),
            crumble: "c3_ledge_crumble".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct StageTerrain {
    #[serde(default)]
    pub ground: Vec<Ground>,
    #[serde(default)]
    pub ledges: Vec<Ledge>,
    #[serde(default)]
    pub tiles: Tiles,
}

/// The y of the top of `rows` solid rows.
pub fn surface(rows: u32) -> f32 {
    FLOOR_Y + rows as f32 * TILE
}

/// Whether a piece from `from` to `to` is forwards and a whole number of tiles.
fn whole_tiles(from: f32, to: f32) -> bool {
    to > from && ((to - from) / TILE).fract() == 0.0
}

impl StageTerrain {
    /// Reads the terrain from a whole Stage file (other fields are ignored)
    /// and checks it: each piece forwards and a whole number of tiles, ground in order with
    /// no overlaps, known ledge kinds.
    pub fn parse(text: &str) -> Result<Self, String> {
        let terrain: Self = ron::from_str(text).map_err(|e| e.to_string())?;
        for g in &terrain.ground {
            if !whole_tiles(g.from, g.to) {
                return Err(format!(
                    "ground {}..{} is not a whole number of tiles",
                    g.from, g.to
                ));
            }
        }
        for pair in terrain.ground.windows(2) {
            if pair[1].from < pair[0].to {
                return Err(format!(
                    "ground at {} overlaps the one before",
                    pair[1].from
                ));
            }
        }
        for l in &terrain.ledges {
            if !whole_tiles(l.from, l.to) {
                return Err(format!(
                    "ledge {}..{} is not a whole number of tiles",
                    l.from, l.to
                ));
            }
            if l.kind != "thin" && l.kind != "crumble" {
                return Err(format!("ledge at {}: unknown kind {:?}", l.from, l.kind));
            }
        }
        Ok(terrain)
    }

    /// The ground under scroll position `x`, if it isn't a pit.
    #[allow(dead_code)]
    pub fn ground_at(&self, x: f32) -> Option<&Ground> {
        self.ground.iter().find(|g| g.from <= x && x < g.to)
    }

    /// The pits between ground pieces, as `(from, to)`.
    #[allow(dead_code)]
    pub fn pits(&self) -> Vec<(f32, f32)> {
        self.ground
            .windows(2)
            .filter(|p| p[1].from > p[0].to)
            .map(|p| (p[0].to, p[1].from))
            .collect()
    }

    /// The terrain as `scenery:` spawn entries, one per tile, at ground
    /// depth. The top tiles get lips at the ground's ends.
    pub fn scenery(&self) -> Vec<SpawnEntry> {
        let mut out = Vec::new();
        // A tile at scroll position x (its left edge) is at the right edge of
        // the screen when the camera reaches x.
        let mut tile = |at: f32, row: u32, sprite: &str, frame: usize| {
            let mut params = ron::Map::new();
            if frame > 0 {
                params.insert(
                    ron::Value::String("frame".into()),
                    ron::Value::Number((frame as u32).into()),
                );
            }
            out.push(SpawnEntry {
                at,
                kind: format!("{SCENERY_PREFIX}{sprite}"),
                pos: (HALF_W + TILE / 2.0, surface(row) + TILE / 2.0),
                params,
            });
        };
        for g in &self.ground {
            let columns = ((g.to - g.from) / TILE) as usize;
            for c in 0..columns {
                let at = g.from + c as f32 * TILE;
                let lip = if c == 0 {
                    1
                } else if c + 1 == columns {
                    2
                } else {
                    0
                };
                for row in 0..g.top {
                    let sprite = if row + 1 == g.top {
                        &self.tiles.top
                    } else {
                        &self.tiles.fill
                    };
                    tile(at, row, sprite, lip);
                }
            }
        }
        for l in &self.ledges {
            let sprite = if l.crumbles() {
                &self.tiles.crumble
            } else {
                &self.tiles.ledge
            };
            let mut at = l.from;
            while at < l.to {
                tile(at, l.row, sprite, 0);
                at += TILE;
            }
        }
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const STAGE: &str = r#"(
        length: 1000.0,
        speed: [(from: 0.0, speed: 40.0)],
        ground: [
            (from: -320.0, to: 600.0, top: 3),
            (from: 632.0, to: 1200.0, top: 3),
            (from: 1200.0, to: 1400.0, top: 6),
        ],
        ledges: [(from: 900.0, to: 948.0, row: 7, kind: "thin")],
        spawns: [],
    )"#;

    #[test]
    fn reads_the_doc_example_and_finds_its_pit() {
        let t = StageTerrain::parse(STAGE).unwrap();
        assert_eq!(t.pits(), [(600.0, 632.0)]);
        assert_eq!(t.ground_at(610.0), None);
        assert_eq!(t.ground_at(1200.0).unwrap().top, 6);
        assert_eq!(surface(3), -96.0);
        assert_eq!(t.ledges[0].surface(), -56.0);
        assert_eq!(t.tiles, Tiles::default());
    }

    #[test]
    fn bad_terrain_is_refused() {
        let off_grid = STAGE.replace("to: 600.0", "to: 601.0");
        assert!(StageTerrain::parse(&off_grid).is_err());
        let overlap = STAGE.replace("from: 632.0", "from: 592.0");
        assert!(StageTerrain::parse(&overlap).is_err());
        let kind = STAGE.replace("\"thin\"", "\"rope\"");
        assert!(StageTerrain::parse(&kind).is_err());
    }

    #[test]
    fn tiles_reach_the_right_edge_at_their_scroll_position() {
        let t = StageTerrain::parse(STAGE).unwrap();
        let tiles = t.scenery();
        // 115 + 71 + 25 columns of ground, 3 or 6 rows deep, and a 6-tile ledge.
        assert_eq!(tiles.len(), (115 + 71) * 3 + 25 * 6 + 6);
        let first = &tiles[0];
        assert_eq!(first.at, -320.0);
        assert_eq!(first.pos, (HALF_W + 4.0, -116.0));
        let pieces = crate::kit::scenery::pieces(&tiles).unwrap();
        // At the start the first column is at the left edge of the screen.
        let at = pieces[0].place(0.0, crate::cabinets::Scroll::Sideways);
        assert_eq!(at.x, -HALF_W + 4.0);
        assert_eq!(pieces[0].frame, 1, "a lip at the ground's start");
    }
}
