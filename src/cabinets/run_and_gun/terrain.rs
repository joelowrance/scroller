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
//! The ground is drawn as `scenery:` tiles on the kit's scenery layer
//! ([`StageTerrain::scenery`]). Ledges are drawn here, so crumbling ones can
//! shake and fall away.
//!
//! [`Terrain`], the resource the gameplay reads, adds each crumbling ledge's
//! state and answers the platform physics' [`TileQuery`].

use bevy::prelude::*;
use serde::Deserialize;

use super::tuning::TerrainTuning;
use crate::display::HALF_W;
use crate::flow::Screen;
use crate::kit::platform::TileQuery;
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

    /// The ground as `scenery:` spawn entries, one per tile, at ground
    /// depth. The top tiles get lips at the ground's ends. Ledges are drawn
    /// by [`spawn_ledges`].
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
        out
    }
}

/// Screen x of a world x, with the camera at `camera`.
pub fn screen_x(world_x: f32, camera: f32) -> f32 {
    world_x - camera + HALF_W
}

/// World x of a screen x.
pub fn world_x(screen_x: f32, camera: f32) -> f32 {
    screen_x + camera - HALF_W
}

/// A crumbling ledge's state.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Crumble {
    #[default]
    Whole,
    /// Shaking after the Hero landed; seconds left.
    Shaking(f32),
    /// Fallen away; seconds until it comes back.
    Gone(f32),
}

/// The current Stage's terrain.
#[derive(Resource, Clone, Debug, Default)]
pub struct Terrain {
    pub ground: Vec<Ground>,
    pub ledges: Vec<Ledge>,
    pub crumbles: Vec<Crumble>,
    pub tiles: Tiles,
}

impl Terrain {
    /// Reads and checks a whole Stage file's terrain.
    pub fn parse(stage_file: &str) -> Result<Self, String> {
        StageTerrain::parse(stage_file).map(Self::new)
    }

    pub fn new(file: StageTerrain) -> Self {
        Self {
            crumbles: vec![Crumble::Whole; file.ledges.len()],
            ground: file.ground,
            ledges: file.ledges,
            tiles: file.tiles,
        }
    }

    pub fn ground_y(&self, piece: &Ground) -> f32 {
        surface(piece.top)
    }

    pub fn ledge_y(&self, ledge: &Ledge) -> f32 {
        ledge.surface()
    }

    /// The ground top at one world x (None over a pit).
    pub fn ground_at(&self, x: f32) -> Option<f32> {
        self.ground_top(x, x)
    }

    /// Ground or ledge top a walker at `x` with feet at `feet` stands on.
    pub fn floor_at(&self, x: f32, feet: f32) -> Option<f32> {
        let ground = self.ground_at(x).filter(|g| *g <= feet + 0.5);
        let ledge = self
            .ledge_tops(x, x)
            .into_iter()
            .filter(|t| (*t - feet).abs() <= 0.5)
            .reduce(f32::max);
        match (ground, ledge) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        }
    }

    /// The next ground top at or after `x` going `dir` (for a rescue, or
    /// for the autopilot): the start of the next piece and its top.
    pub fn next_ground(&self, x: f32, dir: f32) -> Option<(f32, f32)> {
        self.ground
            .iter()
            .filter_map(|g| {
                let edge = if dir > 0.0 { g.from } else { g.to };
                ((edge - x) * dir >= 0.0).then(|| (edge, self.ground_y(g)))
            })
            .min_by(|a, b| ((a.0 - x).abs()).total_cmp(&(b.0 - x).abs()))
    }

    /// Steps crumbling ledges: `standing_on` is the ledge the Hero is on.
    pub fn tick_crumbles(&mut self, standing_on: Option<usize>, t: &TerrainTuning, dt: f32) {
        for (i, state) in self.crumbles.iter_mut().enumerate() {
            if !self.ledges[i].crumbles() {
                continue;
            }
            *state = match *state {
                Crumble::Whole if standing_on == Some(i) => Crumble::Shaking(t.crumble_seconds),
                Crumble::Shaking(left) if left - dt <= 0.0 => {
                    Crumble::Gone(t.crumble_respawn_seconds)
                }
                Crumble::Shaking(left) => Crumble::Shaking(left - dt),
                Crumble::Gone(left) if left - dt <= 0.0 => Crumble::Whole,
                Crumble::Gone(left) => Crumble::Gone(left - dt),
                whole => whole,
            };
        }
    }

    /// Every crumbling ledge comes back (a rewind).
    pub fn reset(&mut self) {
        self.crumbles.fill(Crumble::Whole);
    }

    /// The ledge whose top is at `feet` under `x0..x1`, if any.
    pub fn ledge_under(&self, x0: f32, x1: f32, feet: f32) -> Option<usize> {
        self.ledges.iter().enumerate().position(|(i, l)| {
            x1 > l.from
                && x0 < l.to
                && !matches!(self.crumbles[i], Crumble::Gone(_))
                && (self.ledge_y(l) - feet).abs() <= 0.5
        })
    }
}

impl TileQuery for Terrain {
    fn ground_top(&self, x0: f32, x1: f32) -> Option<f32> {
        self.ground
            .iter()
            .filter(|g| x1 >= g.from && x0 < g.to)
            .map(|g| self.ground_y(g))
            .reduce(f32::max)
    }

    fn ledge_tops(&self, x0: f32, x1: f32) -> Vec<f32> {
        self.ledges
            .iter()
            .zip(&self.crumbles)
            .filter(|(l, c)| x1 > l.from && x0 < l.to && !matches!(c, Crumble::Gone(_)))
            .map(|(l, _)| self.ledge_y(l))
            .collect()
    }
}

/// A drawn ledge: its index in `Terrain::ledges`.
#[derive(Component)]
pub(super) struct LedgePiece(usize);

/// Draws each ledge as a row of its tile sprites, placed from the camera by
/// [`place_ledges`]. Ground is on the scenery layer.
pub(super) fn spawn_ledges(
    commands: &mut Commands,
    terrain: &Terrain,
    sprites: &crate::sprites::Sprites,
    cabinet: usize,
) {
    for (index, l) in terrain.ledges.iter().enumerate() {
        let name = if l.crumbles() {
            &terrain.tiles.crumble
        } else {
            &terrain.tiles.ledge
        };
        let width = l.to - l.from;
        let mut parent = commands.spawn((
            LedgePiece(index),
            Transform::from_xyz(0.0, surface(l.row) + TILE / 2.0, 1.2),
            Visibility::Hidden,
            DespawnOnExit(Screen::Playing),
        ));
        parent.with_children(|row| {
            let mut x = 0.0;
            while x < width {
                row.spawn((
                    sprites.sprite(name, cabinet),
                    Transform::from_xyz(x + TILE / 2.0 - width / 2.0, 0.0, 0.0),
                ));
                x += TILE;
            }
        });
    }
}

/// Places ledges from the camera, showing only those on screen, and shakes
/// or hides crumbling ledges.
pub(super) fn place_ledges(
    terrain: Res<Terrain>,
    stage: Res<crate::kit::StageRun>,
    time: Res<Time>,
    mut pieces: Query<(&LedgePiece, &mut Transform, &mut Visibility)>,
) {
    let camera = stage.cursor.position;
    for (piece, mut transform, mut visibility) in &mut pieces {
        let l = &terrain.ledges[piece.0];
        let state = terrain.crumbles[piece.0];
        let left = screen_x(l.from, camera);
        let right = screen_x(l.to, camera);
        let on = right > -HALF_W && left < HALF_W && !matches!(state, Crumble::Gone(_));
        let shake = match state {
            Crumble::Shaking(_) => ((time.elapsed_secs() * 30.0) as i32 % 2) as f32,
            _ => 0.0,
        };
        transform.translation.x = ((left + right) / 2.0).round() + shake;
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
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
        // 115 + 71 + 25 columns of ground, 3 or 6 rows deep (ledges are
        // drawn by the gameplay).
        assert_eq!(tiles.len(), (115 + 71) * 3 + 25 * 6);
        let first = &tiles[0];
        assert_eq!(first.at, -320.0);
        assert_eq!(first.pos, (HALF_W + 4.0, -116.0));
        let pieces = crate::kit::scenery::pieces(&tiles).unwrap();
        // At the start the first column is at the left edge of the screen.
        let at = pieces[0].place(0.0, crate::cabinets::Scroll::Sideways);
        assert_eq!(at.x, -HALF_W + 4.0);
        assert_eq!(pieces[0].frame, 1, "a lip at the ground's start");
    }

    fn terrain() -> Terrain {
        Terrain::parse(
            r#"(
                length: 1000.0,
                speed: [(from: 0.0, speed: 40.0)],
                ground: [
                    (from: -320.0, to: 600.0, top: 3),
                    (from: 632.0, to: 1200.0, top: 3),
                    (from: 1200.0, to: 1400.0, top: 6),
                ],
                ledges: [
                    (from: 900.0, to: 948.0, row: 7, kind: "thin"),
                    (from: 1500.0, to: 1532.0, row: 7, kind: "crumble"),
                ],
            )"#,
        )
        .unwrap()
    }

    #[test]
    fn the_stage_file_gives_ground_pits_and_ledges() {
        let t = terrain();
        assert_eq!(t.ground_at(0.0), Some(-96.0));
        assert_eq!(t.ground_at(610.0), None, "a pit");
        assert_eq!(t.ground_at(1300.0), Some(-72.0), "a step up");
        assert_eq!(t.ledge_tops(920.0, 930.0), [-56.0]);
        assert_eq!(t.ground_top(595.0, 640.0), Some(-96.0), "spans the pit");
    }

    #[test]
    fn crumbling_ledges_fall_and_come_back() {
        let mut t = terrain();
        let tt = super::super::tuning::TerrainTuning::default();
        t.tick_crumbles(Some(1), &tt, 0.1);
        assert!(matches!(t.crumbles[1], Crumble::Shaking(_)));
        t.tick_crumbles(None, &tt, 0.6);
        assert!(t.ledge_tops(1510.0, 1520.0).is_empty(), "fallen");
        for _ in 0..31 {
            t.tick_crumbles(None, &tt, 0.1);
        }
        assert_eq!(t.ledge_tops(1510.0, 1520.0), [-56.0], "back after 3 s");
        t.tick_crumbles(Some(1), &tt, 0.1);
        t.reset();
        assert_eq!(t.crumbles[1], Crumble::Whole);
        // Thin ledges never crumble.
        t.tick_crumbles(Some(0), &tt, 1.0);
        assert_eq!(t.crumbles[0], Crumble::Whole);
    }

    #[test]
    fn world_and_screen_x_round_trip() {
        assert_eq!(screen_x(500.0, 500.0), HALF_W, "reaches the right edge");
        assert_eq!(world_x(screen_x(123.0, 77.0), 77.0), 123.0);
    }
}
