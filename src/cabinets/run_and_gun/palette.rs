//! Cabinet 3's colours by their design-doc names
//! (`docs/cabinets/3-run-and-gun.md`, "Palette"). The palette itself is
//! defined once, in `sprites::palette`.

#![allow(dead_code)]

use bevy::prelude::Color;

use crate::sprites::palette::{
    ACCENT, BACKDROP, BRIGHT, DANGER, EXTRA, FILL, PALETTES, SHADE, WARM,
};

const fn role(role: u8) -> Color {
    PALETTES[2].color(role)
}

pub const UMBER: Color = role(BACKDROP);
pub const BARK: Color = role(SHADE);
pub const OCHRE: Color = role(FILL);
pub const SPROUT: Color = role(ACCENT);
pub const FOAM: Color = role(BRIGHT);
pub const EMBER: Color = role(WARM);
pub const ICE: Color = role(DANGER);
pub const IRON: Color = role(EXTRA);
