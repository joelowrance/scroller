//! The Cabinet palettes: the one place their colours are defined.
//!
//! Each palette has eight colours in fixed roles, so a sprite drawn with role
//! indices looks right in every Cabinet and changes colour at each Cabinet
//! Transition.

use bevy::prelude::*;

use crate::cabinets::CABINET_COUNT;

/// Number of colours in a palette (sprite characters `0`–`7`).
pub const PALETTE_SIZE: usize = 8;

/// Colour roles, by palette index. Not all are used by code yet; sprite files
/// use them by number.
#[allow(dead_code)]
mod roles {
    pub const BACKDROP: usize = 0;
    pub const SHADOW: usize = 1;
    pub const BASE: usize = 2;
    pub const LIGHT: usize = 3;
    pub const ACCENT: usize = 4;
    pub const HOT: usize = 5;
    pub const BRIGHT: usize = 6;
    pub const ALT: usize = 7;
}
pub use roles::*;

#[derive(Clone, Copy, Debug)]
pub struct Palette {
    pub colors: [Color; PALETTE_SIZE],
}

impl Palette {
    pub const fn color(&self, role: usize) -> Color {
        self.colors[role]
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

/// One palette per Cabinet, in Run order.
#[rustfmt::skip]
pub const PALETTES: [Palette; CABINET_COUNT] = [
    //        backdrop  shadow    base      light     accent    hot       bright    alt
    // 1: side-scrolling shooter — deep space blue with cyan.
    palette([0x05051a, 0x1c2a5a, 0x3a6ea5, 0x7fb8e6, 0x4dd9ff, 0xff8c3a, 0xf0f8ff, 0xc050ff]),
    // 2: vertical shooter — night violet with magenta.
    palette([0x140019, 0x3d1450, 0x8a2d8f, 0xd070c8, 0xff66cc, 0xffd23f, 0xfff0fa, 0x40e0d0]),
    // 3: run-and-gun platformer — dusk earth with amber.
    palette([0x1a0d00, 0x4a2a10, 0x8c5a2b, 0xc89650, 0xffbf33, 0xe83a2a, 0xfff6e0, 0x6aa84f]),
    // 4: vertical climber — jungle green.
    palette([0x00140a, 0x144a2a, 0x2e8a4a, 0x7fd08a, 0x66ff80, 0xff5a5a, 0xeffff0, 0xf0d040]),
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_colours_decode() {
        assert_eq!(rgb(0xff8000), Color::srgb_u8(255, 128, 0));
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
