//! Cabinet 4's colours by their design-doc names
//! (`docs/cabinets/4-climber.md`, "Palette"). The palette itself is
//! defined once, in `sprites::palette`.

#![allow(dead_code)]

use bevy::prelude::Color;

use crate::sprites::palette::{
    ACCENT, BACKDROP, BRIGHT, DANGER, EXTRA, FILL, PALETTES, SHADE, WARM,
};

const fn role(role: u8) -> Color {
    PALETTES[3].color(role)
}

pub const ABYSS: Color = role(BACKDROP);
pub const INDIGO: Color = role(SHADE);
pub const GIRDER: Color = role(FILL);
pub const MINT: Color = role(ACCENT);
pub const FOAM: Color = role(BRIGHT);
pub const EMBER: Color = role(WARM);
pub const ROSE: Color = role(DANGER);
pub const PEACH: Color = role(EXTRA);
