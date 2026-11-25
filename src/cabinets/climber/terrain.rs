//! Cabinet 4's tower face, from the `ledges`, `ladders`, `gusts` and `route`
//! lists in its Stage files (docs/cabinets/4-climber.md, "Terrain").
//!
//! Heights are scroll positions, like a spawn's `at`: a ledge at `h` has its
//! standing surface on the screen's top edge when the camera reaches `h`, so
//! its screen y is `HALF_H - (camera - h)`. Everything is on the 8 px tile
//! grid, across the face x ∈ [−128, +128].
//!
//! - `ledges`: one tile thick, stood on at `h`. `kind` is `"landing"` (solid
//!   and the full width of the face), `"thin"` (one-way), `"solid"` or
//!   `"crumble"`.
//! - `ladders`: one tile wide, centred on `x`, from one ledge at `from_h` to
//!   another at `to_h`. A ladder that ends at a landing comes up through a
//!   hatch in it; no ladder passes a landing.
//! - `gusts`: zones of wind, by the height of the Hero's feet.
//! - `route`: the footholds a careful player follows, for the Stage checks
//!   and the autopilot. Nothing in the game shows it.
//! - `tiles` (optional): the sprites to draw it with; Stage 2's girders are
//!   lit by the dawn.
//!
//! Until the Cabinet's own gameplay (#24) draws it, [`StageTerrain::scenery`]
//! turns the terrain into `scenery:` entries for the kit's scenery layer.

use serde::Deserialize;

use crate::display::HALF_H;
use crate::kit::scenery::SCENERY_PREFIX;
use crate::kit::timeline::SpawnEntry;

pub const TILE: f32 = 8.0;
/// Half the width of the tower face (`climber.camera.face_half_width`).
pub const FACE_HALF_WIDTH: f32 = 128.0;

pub const KINDS: [&str; 4] = ["landing", "thin", "solid", "crumble"];

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Ledge {
    pub h: f32,
    pub from_x: f32,
    pub to_x: f32,
    pub kind: String,
}

impl Ledge {
    pub fn is_landing(&self) -> bool {
        self.kind == "landing"
    }

    #[allow(dead_code)]
    pub fn crumbles(&self) -> bool {
        self.kind == "crumble"
    }

    /// Whether the ledge is under scroll-space x `x`.
    #[allow(dead_code)]
    pub fn covers(&self, x: f32) -> bool {
        self.from_x <= x && x <= self.to_x
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Ladder {
    pub x: f32,
    pub from_h: f32,
    pub to_h: f32,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Gust {
    pub from_h: f32,
    pub to_h: f32,
    /// `"left"` or `"right"`: the way it blows.
    pub dir: String,
    pub period: f32,
    pub offset: f32,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
pub struct Foothold {
    pub x: f32,
    pub h: f32,
}

/// The sprites a Stage draws its terrain with.
#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct Tiles {
    pub ledge: String,
    pub crumble: String,
    pub landing: String,
    pub ladder: String,
}

impl Default for Tiles {
    fn default() -> Self {
        Self {
            ledge: "c4_catwalk".into(),
            crumble: "c4_catwalk_crumble".into(),
            landing: "c4_landing".into(),
            ladder: "c4_ladder".into(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq)]
pub struct StageTerrain {
    #[serde(default)]
    pub ledges: Vec<Ledge>,
    #[serde(default)]
    pub ladders: Vec<Ladder>,
    #[serde(default)]
    pub gusts: Vec<Gust>,
    #[serde(default)]
    pub route: Vec<Foothold>,
    #[serde(default)]
    pub tiles: Tiles,
}

/// Screen y of height `h` when the camera is at `camera`.
// The Stage checks use this now; the gameplay (#24) will too.
#[allow(dead_code)]
pub fn screen_y(h: f32, camera: f32) -> f32 {
    HALF_H - (camera - h)
}

fn on_grid(v: f32) -> bool {
    (v / TILE).fract() == 0.0
}

impl StageTerrain {
    /// Reads the terrain from a whole Stage file (other fields are ignored)
    /// and checks its shape: everything on the tile grid and on the face,
    /// known kinds, landings full width, and ladders from ledge to ledge.
    pub fn parse(text: &str) -> Result<Self, String> {
        let terrain: Self = ron::from_str(text).map_err(|e| e.to_string())?;
        for l in &terrain.ledges {
            let at = format!("ledge at {} ({}..{})", l.h, l.from_x, l.to_x);
            if !(on_grid(l.h) && on_grid(l.from_x) && on_grid(l.to_x)) || l.to_x <= l.from_x {
                return Err(format!("{at} is not whole tiles"));
            }
            if l.from_x < -FACE_HALF_WIDTH || l.to_x > FACE_HALF_WIDTH {
                return Err(format!("{at} is off the face"));
            }
            if !KINDS.contains(&l.kind.as_str()) {
                return Err(format!("{at}: unknown kind {:?}", l.kind));
            }
            if l.is_landing() && (l.from_x, l.to_x) != (-FACE_HALF_WIDTH, FACE_HALF_WIDTH) {
                return Err(format!("{at}: a landing is the full width"));
            }
        }
        for d in &terrain.ladders {
            let at = format!("ladder at x {} ({}..{})", d.x, d.from_h, d.to_h);
            if !on_grid(d.x - TILE / 2.0) || !on_grid(d.from_h) || !on_grid(d.to_h) {
                return Err(format!("{at} is off the tile grid"));
            }
            if d.to_h <= d.from_h {
                return Err(format!("{at} goes down"));
            }
            for h in [d.from_h, d.to_h] {
                if terrain.ledge_at(d.x, h).is_none() {
                    return Err(format!("{at} has no ledge at {h}"));
                }
            }
            if terrain
                .ledges
                .iter()
                .any(|l| l.is_landing() && d.from_h < l.h && l.h < d.to_h)
            {
                return Err(format!("{at} passes a landing"));
            }
        }
        for g in &terrain.gusts {
            if g.to_h <= g.from_h || g.period <= 0.0 || !["left", "right"].contains(&g.dir.as_str())
            {
                return Err(format!("bad gust zone at {}", g.from_h));
            }
        }
        if terrain.route.is_empty() {
            return Err("no route".into());
        }
        Ok(terrain)
    }

    /// The ledge at height `h` under x, if any.
    pub fn ledge_at(&self, x: f32, h: f32) -> Option<&Ledge> {
        self.ledges.iter().find(|l| l.h == h && l.covers(x))
    }

    /// The ladder at x that spans heights `a` to `b`, if any.
    #[allow(dead_code)]
    pub fn ladder_between(&self, x: f32, a: f32, b: f32) -> Option<&Ladder> {
        let (lo, hi) = (a.min(b), a.max(b));
        self.ladders
            .iter()
            .find(|d| d.x == x && d.from_h <= lo && hi <= d.to_h)
    }

    /// The terrain as `scenery:` spawn entries, one per tile, at the face's
    /// depth. Ladders draw over ledges, so a hatch in a landing shows them.
    pub fn scenery(&self) -> Vec<SpawnEntry> {
        let mut out = Vec::new();
        // A tile whose top is at height `top` is on the screen's top edge
        // when the camera reaches `top`.
        let mut tile = |top: f32, x: f32, sprite: &str, frame: u32, z: f32| {
            let mut params = ron::Map::new();
            if frame > 0 {
                params.insert(
                    ron::Value::String("frame".into()),
                    ron::Value::Number(frame.into()),
                );
            }
            params.insert(
                ron::Value::String("z".into()),
                ron::Value::Number(ron::Number::new(z as f64)),
            );
            out.push(SpawnEntry {
                at: top,
                kind: format!("{SCENERY_PREFIX}{sprite}"),
                pos: (x, HALF_H - TILE / 2.0),
                params,
            });
        };
        for l in &self.ledges {
            let sprite = match l.kind.as_str() {
                "landing" | "solid" => &self.tiles.landing,
                "crumble" => &self.tiles.crumble,
                _ => &self.tiles.ledge,
            };
            let columns = ((l.to_x - l.from_x) / TILE) as u32;
            for c in 0..columns {
                // Catwalks get end posts; landings run wall to wall.
                let frame = if l.is_landing() || columns == 1 {
                    0
                } else if c == 0 {
                    1
                } else if c + 1 == columns {
                    2
                } else {
                    0
                };
                let frame = if l.crumbles() { 0 } else { frame };
                tile(
                    l.h,
                    l.from_x + (c as f32 + 0.5) * TILE,
                    sprite,
                    frame,
                    -60.0,
                );
            }
        }
        for d in &self.ladders {
            let mut top = d.from_h + TILE;
            while top <= d.to_h {
                tile(top, d.x, &self.tiles.ladder, 0, -59.0);
                top += TILE;
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
        speed: [(from: 0.0, speed: 14.0)],
        ledges: [
            (h: 0.0, from_x: -128.0, to_x: 128.0, kind: "landing"),
            (h: 40.0, from_x: -128.0, to_x: -64.0, kind: "thin"),
            (h: 72.0, from_x: -24.0, to_x: 40.0, kind: "crumble"),
            (h: 80.0, from_x: 80.0, to_x: 112.0, kind: "thin"),
        ],
        ladders: [(x: 92.0, from_h: 0.0, to_h: 80.0)],
        gusts: [(from_h: 900.0, to_h: 1200.0, dir: "left", period: 6.0, offset: 0.0)],
        route: [(x: -100.0, h: 0.0), (x: -96.0, h: 40.0)],
        spawns: [],
    )"#;

    #[test]
    fn reads_the_doc_example() {
        let t = StageTerrain::parse(STAGE).unwrap();
        assert_eq!(t.ledges.len(), 4);
        assert!(t.ledges[0].is_landing() && t.ledges[2].crumbles());
        assert_eq!(t.ledge_at(-100.0, 40.0).unwrap().kind, "thin");
        assert!(t.ladder_between(92.0, 80.0, 8.0).is_some());
        assert_eq!(t.tiles, Tiles::default());
        assert_eq!(screen_y(0.0, 184.0), -64.0);
    }

    #[test]
    fn bad_terrain_is_refused() {
        for (from, to) in [
            ("to_x: -64.0", "to_x: -63.0"),
            ("h: 40.0", "h: 41.0"),
            ("\"crumble\"", "\"rope\""),
            ("from_x: -128.0, to_x: 128.0", "from_x: -120.0, to_x: 128.0"),
            ("x: 92.0", "x: 96.0"),
            ("to_h: 80.0", "to_h: 72.0"),
            ("dir: \"left\"", "dir: \"up\""),
            (
                "route: [(x: -100.0, h: 0.0), (x: -96.0, h: 40.0)]",
                "route: []",
            ),
        ] {
            let bad = STAGE.replacen(from, to, 1);
            assert_ne!(bad, STAGE);
            assert!(StageTerrain::parse(&bad).is_err(), "{to}");
        }
        let through = STAGE.replace(
            "(h: 80.0, from_x: 80.0",
            "(h: 40.0, from_x: -128.0, to_x: 128.0, kind: \"landing\"), (h: 80.0, from_x: 80.0",
        );
        assert!(StageTerrain::parse(&through).is_err());
    }

    #[test]
    fn tiles_reach_the_top_edge_at_their_height() {
        let t = StageTerrain::parse(STAGE).unwrap();
        let tiles = t.scenery();
        // 32 + 8 + 8 + 4 ledge tiles and a 10-tile ladder.
        assert_eq!(tiles.len(), 32 + 8 + 8 + 4 + 10);
        let pieces = crate::kit::scenery::pieces(&tiles).unwrap();
        let landing = &pieces[0];
        assert_eq!(landing.pos.x, -124.0);
        // At camera 184 the landing's surface is at y = -64.
        let at = landing.place(184.0, crate::cabinets::Scroll::Upward);
        assert_eq!(at.y + TILE / 2.0, -64.0);
        let catwalk = &pieces[32];
        assert_eq!(catwalk.frame, 1, "a post at the catwalk's start");
        let ladder = pieces.last().unwrap();
        assert_eq!((ladder.at, ladder.pos.x), (80.0, 92.0));
    }
}
