//! Cabinet 2's colours by their design-doc names
//! (`docs/cabinets/2-vertical-shooter.md`, "Palette"). The palette itself is
//! defined once, in `sprites::palette`.

// The Cabinet's own code (#14) uses the rest.
#![allow(dead_code)]

use bevy::prelude::Color;

use crate::sprites::palette::{
    ACCENT, BACKDROP, BRIGHT, DANGER, EXTRA, FILL, PALETTES, SHADE, WARM,
};

const fn role(role: u8) -> Color {
    PALETTES[1].color(role)
}

/// Backdrop and shadows.
pub const DUSK: Color = role(BACKDROP);
/// Cloud shading, far layer, ground shading.
pub const PLUM: Color = role(SHADE);
/// Clouds, scaffolding, air enemy hull fill.
pub const STEEL: Color = role(FILL);
/// Accent: the Hero's canopy, Power-up rims.
pub const ORCHID: Color = role(ACCENT);
/// Hero body, enemy outlines, bullet cores, text.
pub const FOAM: Color = role(BRIGHT);
/// Hero shots, rockets, the Hero's engine, explosions.
pub const EMBER: Color = role(WARM);
/// Enemy bullets, enemy weak points and flak crosshairs only.
pub const VOLT: Color = role(DANGER);
/// Mesas, the yard floor, ground target hull fill.
pub const CLAY: Color = role(EXTRA);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cabinets::CABINETS;

    const fn hex(rgb: u32) -> Color {
        Color::srgb_u8((rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8)
    }

    /// The table in the design doc, so the two can't drift apart.
    #[test]
    fn matches_the_design_doc() {
        assert_eq!(DUSK, hex(0x140019));
        assert_eq!(PLUM, hex(0x3A1F4F));
        assert_eq!(STEEL, hex(0x5A6E8C));
        assert_eq!(CLAY, hex(0x8C5A3C));
        assert_eq!(ORCHID, hex(0xFF66CC));
        assert_eq!(FOAM, hex(0xF4F4F0));
        assert_eq!(EMBER, hex(0xFFB43C));
        assert_eq!(VOLT, hex(0xC8FF3C));
        assert_eq!(CABINETS[1].backdrop, DUSK);
    }
}
