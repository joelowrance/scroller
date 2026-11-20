//! Cabinet 1's colours by their design-doc names
//! (`docs/cabinets/1-side-shooter.md`). The palette itself is defined once,
//! in `sprites::palette`.

// Backgrounds and enemies (#8, #9) use the rest.
#![allow(dead_code)]

use bevy::prelude::Color;

use crate::sprites::palette::{
    ACCENT, BACKDROP, BRIGHT, DANGER, EXTRA, FILL, PALETTES, SHADE, WARM,
};

const fn role(role: u8) -> Color {
    PALETTES[0].color(role)
}

pub const NIGHT: Color = role(BACKDROP);
pub const SLATE: Color = role(SHADE);
pub const SEA: Color = role(FILL);
pub const SPARK: Color = role(ACCENT);
pub const FOAM: Color = role(BRIGHT);
pub const EMBER: Color = role(WARM);
pub const SIGNAL: Color = role(DANGER);
pub const RUST: Color = role(EXTRA);
