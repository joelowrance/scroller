//! Cabinet 1's palette, from `docs/cabinets/1-side-shooter.md`.

// Backgrounds and enemies (#8, #9) use the rest.
#![allow(dead_code)]

use bevy::prelude::Color;

const fn hex(rgb: u32) -> Color {
    Color::srgb(
        ((rgb >> 16) & 0xFF) as f32 / 255.0,
        ((rgb >> 8) & 0xFF) as f32 / 255.0,
        (rgb & 0xFF) as f32 / 255.0,
    )
}

pub const NIGHT: Color = hex(0x05051A);
pub const SLATE: Color = hex(0x23305E);
pub const SEA: Color = hex(0x2E7A8C);
pub const SPARK: Color = hex(0x4DD9FF);
pub const FOAM: Color = hex(0xF4F4F0);
pub const EMBER: Color = hex(0xFFB43C);
pub const SIGNAL: Color = hex(0xFF3C78);
pub const RUST: Color = hex(0x8A3A22);
