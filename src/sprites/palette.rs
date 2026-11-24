//! The Cabinet palettes: the one place their colours are defined.
//!
//! Each palette has eight colours in fixed roles, so a sprite drawn with role
//! indices looks right in every Cabinet and changes colour at each Cabinet
//! Transition.

use bevy::prelude::*;

use crate::cabinets::CABINET_COUNT;

/// Number of colours in a palette: sprite characters `1`–`8`.
pub const PALETTE_SIZE: usize = 8;

/// Colour roles, by sprite character. Every Cabinet palette keeps these
/// roles so shared sprites read the same way in each; the names and exact
/// uses per Cabinet are in its design doc.
#[allow(dead_code)]
mod roles {
    /// The Cabinet's background colour.
    pub const BACKDROP: u8 = 1;
    /// Dark shading: far parallax, terrain, outlines on the Hero.
    pub const SHADE: u8 = 2;
    /// Mid tone: water, mid parallax, enemy hull fill.
    pub const FILL: u8 = 3;
    /// The Cabinet's highlight colour.
    pub const ACCENT: u8 = 4;
    /// Near-white: Hero body, enemy outlines, bullet cores, text.
    pub const BRIGHT: u8 = 5;
    /// Hero shots, engines, explosions.
    pub const WARM: u8 = 6;
    /// Enemy bullets and weak points only.
    pub const DANGER: u8 = 7;
    /// A spare colour for the Cabinet to use as its design doc says.
    pub const EXTRA: u8 = 8;
}
pub use roles::*;

/// Roles that backgrounds and terrain may use (backdrop, shade, fill, extra).
#[cfg_attr(not(test), allow(dead_code))]
pub const BACKGROUND_ROLES: [u8; 4] = [BACKDROP, SHADE, FILL, EXTRA];

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub colors: [Color; PALETTE_SIZE],
}

impl Palette {
    /// The colour for a sprite character `1`–`8` (see the roles above).
    pub const fn color(&self, role: u8) -> Color {
        self.colors[role as usize - 1]
    }
}

const fn rgb(hex: u32) -> Color {
    Color::srgb_u8((hex >> 16) as u8, (hex >> 8) as u8, hex as u8)
}

const fn palette(colors: [u32; PALETTE_SIZE]) -> Palette {
    let mut out = [Color::BLACK; PALETTE_SIZE];
    let mut i = 0;
    while i < PALETTE_SIZE {
        out[i] = rgb(colors[i]);
        i += 1;
    }
    Palette { colors: out }
}

/// One palette per Cabinet, in Run order. Each is fixed by its
/// Cabinet's design doc.
#[rustfmt::skip]
pub const PALETTES: [Palette; CABINET_COUNT] = [
    //        backdrop  shade     fill      accent    bright    warm      danger    extra
    // 1: side-scrolling shooter — Night, Slate, Sea, Spark, Foam, Ember, Signal, Rust.
    palette([0x05051a, 0x23305e, 0x2e7a8c, 0x4dd9ff, 0xf4f4f0, 0xffb43c, 0xff3c78, 0x8a3a22]),
    // 2: vertical shooter — Dusk, Plum, Steel, Orchid, Foam, Ember, Volt, Clay.
    palette([0x140019, 0x3a1f4f, 0x5a6e8c, 0xff66cc, 0xf4f4f0, 0xffb43c, 0xc8ff3c, 0x8c5a3c]),
    // 3: run-and-gun platformer — Umber, Bark, Ochre, Sprout, Foam, Ember, Ice, Iron.
    palette([0x1a0d00, 0x4a2a10, 0x8c5a2b, 0x9ce65a, 0xf4f4f0, 0xffb43c, 0x66d9ff, 0x6b6f7a]),
    // 4: vertical climber — Abyss, Indigo, Girder, Mint, Foam, Ember, Rose, Peach.
    palette([0x0b0a24, 0x2b2a5c, 0x5e6a85, 0x66ffc2, 0xf4f4f0, 0xffb43c, 0xff3c8c, 0xc8946e]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_decode() {
        assert_eq!(rgb(0xff8000), Color::srgb_u8(255, 128, 0));
    }

    #[test]
    fn cabinet_palettes_match_their_design_docs() {
        let docs = [
            include_str!("../../docs/cabinets/1-side-shooter.md"),
            include_str!("../../docs/cabinets/2-vertical-shooter.md"),
            include_str!("../../docs/cabinets/3-run-and-gun.md"),
            include_str!("../../docs/cabinets/4-climber.md"),
        ];
        for (cabinet, doc) in docs.iter().enumerate() {
            check_palette_table(cabinet, doc);
        }
    }

    fn check_palette_table(cabinet: usize, doc: &str) {
        for role in 1..=PALETTE_SIZE as u8 {
            let hex = PALETTES[cabinet]
                .color(role)
                .to_srgba()
                .to_hex()
                .to_uppercase();
            let row = doc
                .lines()
                .find(|l| l.starts_with(&format!("| {role} |")))
                .unwrap_or_else(|| panic!("no palette row {role} in the doc"));
            assert!(
                row.contains(&format!("`{hex}`")),
                "row {role} should be {hex}: {row}"
            );
        }
    }

    #[test]
    fn every_palette_is_distinct() {
        for (i, a) in PALETTES.iter().enumerate() {
            for b in &PALETTES[i + 1..] {
                assert_ne!(a.color(ACCENT), b.color(ACCENT));
                assert_ne!(a.color(BACKDROP), b.color(BACKDROP));
            }
        }
    }
}
