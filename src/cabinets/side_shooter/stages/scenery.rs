//! Parallax backgrounds for both Stages: sprites from `assets/sprites/`
//! (sea stacks, the lighthouse, wave crests, pipes) plus plain lines for the
//! sky bands, furnace glow and girders. Backgrounds use only Night, Slate,
//! Sea and Rust (docs/cabinets/1-side-shooter.md, Palette).
//!
//! Every piece is placed from the scroll position each frame (a [`Parallax`]
//! factor and a wrap width), so it needs no state and survives rewinds.

use bevy::prelude::*;

use crate::cabinets::side_shooter::palette::{RUST, SEA, SLATE};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::sprites::Sprites;

/// Moves at `factor` × the scroll speed, wrapping every `wrap` px.
#[derive(Component, Clone, Copy, Debug)]
pub struct Parallax {
    pub base_x: f32,
    pub factor: f32,
    pub wrap: f32,
}

impl Parallax {
    /// Screen x at a scroll position. Pieces wrap so they are always within
    /// `wrap / 2` of the centre.
    pub fn x_at(&self, position: f32) -> f32 {
        let raw = self.base_x - position * self.factor;
        (raw + self.wrap / 2.0).rem_euclid(self.wrap) - self.wrap / 2.0
    }
}

/// A Stage 2 speck that rises up the shaft once the run-out begins.
#[derive(Component, Clone, Copy, Debug)]
pub struct ShaftSpeck {
    pub base: Vec2,
    /// Scroll position where the run-out starts.
    pub from: f32,
}

/// Horizon of the Stage 1 sea.
const HORIZON: f32 = -40.0;
/// Top of the Stage 1 wave strip; the Bellbuoys ride on it.
const WAVE_TOP: f32 = -104.0;

fn piece(commands: &mut Commands, color: Color, size: Vec2, at: Vec2, z: f32, p: Parallax) {
    commands.spawn((
        Sprite::from_color(color, size),
        Transform::from_translation(at.extend(z)),
        p,
        DespawnOnExit(Screen::Playing),
    ));
}

fn sprite_piece(commands: &mut Commands, sprite: Sprite, at: Vec2, z: f32, p: Parallax) {
    commands.spawn((
        sprite,
        Transform::from_translation(at.extend(z)),
        p,
        DespawnOnExit(Screen::Playing),
    ));
}

fn fixed(commands: &mut Commands, color: Color, size: Vec2, at: Vec2, z: f32) {
    let still = Parallax {
        base_x: at.x,
        factor: 0.0,
        wrap: 4.0 * HALF_W,
    };
    piece(commands, color, size, at, z, still);
}

/// Low Tide: a dusk sky in thinning bands, sea stacks and a lighthouse on
/// the horizon, and a wave strip along the bottom.
pub fn spawn_low_tide(commands: &mut Commands, sprites: &Sprites) {
    let full = 2.0 * HALF_W;
    // Sky: Slate lines, closer together toward the horizon.
    let mut gap = 2.0;
    let mut y = HORIZON + 2.0;
    while y < HALF_H {
        fixed(
            commands,
            SLATE,
            Vec2::new(full, 1.0),
            Vec2::new(0.0, y),
            0.1,
        );
        y += gap;
        gap += 1.5;
    }
    // Sea: Sea dashes in rows, faster the nearer they are.
    for (row, y) in [-44.0, -50.0, -58.0, -68.0, -80.0, -94.0]
        .into_iter()
        .enumerate()
    {
        let factor = 0.2 + 0.12 * row as f32;
        let spacing = 30.0 - 3.0 * row as f32;
        let mut x = -HALF_W + (row as f32 * 7.0) % spacing;
        while x < HALF_W + 32.0 {
            let p = Parallax {
                base_x: x,
                factor,
                wrap: full + 64.0,
            };
            piece(
                commands,
                SEA,
                Vec2::new(4.0 + row as f32, 1.0),
                Vec2::new(x, y),
                0.2,
                p,
            );
            x += spacing;
        }
    }
    // Sea stacks and the lighthouse, on the horizon.
    let wrap = 3.0 * full;
    for (i, x) in [-120.0, 40.0, 190.0, 330.0].into_iter().enumerate() {
        let p = Parallax {
            base_x: x,
            factor: 0.35,
            wrap,
        };
        let stack = sprites.frame("sea-stack", 0, i % 3);
        sprite_piece(commands, stack, Vec2::new(x, HORIZON + 16.0), 0.5, p);
    }
    let lighthouse = Parallax {
        base_x: 260.0,
        factor: 0.35,
        wrap,
    };
    let at = Vec2::new(260.0, HORIZON + 18.0);
    sprite_piece(
        commands,
        sprites.sprite("lighthouse", 0),
        at,
        0.5,
        lighthouse,
    );
    // Wave strip: a Sea band with Slate crests moving at the scroll speed.
    let band = Vec2::new(full, WAVE_TOP + HALF_H);
    fixed(
        commands,
        SEA,
        band,
        Vec2::new(0.0, (WAVE_TOP - HALF_H) / 2.0),
        2.5,
    );
    let mut x = -HALF_W;
    while x < HALF_W + 16.0 {
        let p = Parallax {
            base_x: x,
            factor: 1.0,
            wrap: full + 32.0,
        };
        let crest = Vec2::new(x, WAVE_TOP + 0.5);
        sprite_piece(commands, sprites.sprite("wave-crest", 0), crest, 2.6, p);
        x += 16.0;
    }
}

/// The Foundry: furnace glow in Rust bands, Slate gantries, then chains
/// and pipes. The terrain is drawn separately.
pub fn spawn_foundry(commands: &mut Commands, sprites: &Sprites, runout_from: f32) {
    let full = 2.0 * HALF_W;
    // Furnace glow: Rust lines, thinning upward.
    let mut gap = 2.0;
    let mut y = -HALF_H + 1.0;
    while y < 20.0 {
        fixed(commands, RUST, Vec2::new(full, 1.0), Vec2::new(0.0, y), 0.1);
        y += gap;
        gap += 1.5;
    }
    // Gantries: girders and cross-beams.
    let wrap = full + 160.0;
    let mut x = -HALF_W;
    while x < HALF_W + 160.0 {
        let p = Parallax {
            base_x: x,
            factor: 0.25,
            wrap,
        };
        piece(
            commands,
            SLATE,
            Vec2::new(4.0, 2.0 * HALF_H),
            Vec2::new(x, 0.0),
            0.2,
            p,
        );
        for beam_y in [44.0, -24.0] {
            let at = Vec2::new(x + 40.0, beam_y);
            let beam = Parallax { base_x: at.x, ..p };
            piece(commands, SLATE, Vec2::new(80.0, 3.0), at, 0.2, beam);
        }
        x += 80.0;
    }
    // Chains hanging from above, and pipes.
    let wrap = full + 80.0;
    for (i, x) in [-140.0, -60.0, 30.0, 110.0, 190.0].into_iter().enumerate() {
        let length = 30.0 + (i as f32 * 23.0) % 50.0;
        let p = Parallax {
            base_x: x,
            factor: 0.5,
            wrap,
        };
        let mut y = HALF_H - 14.0;
        while y > HALF_H - 14.0 - length {
            piece(
                commands,
                SLATE,
                Vec2::new(2.0, 3.0),
                Vec2::new(x, y),
                0.5,
                p,
            );
            y -= 5.0;
        }
        let pipe = Vec2::new(x + 30.0, -40.0 - (i as f32 * 17.0) % 40.0);
        let pipe_sprite = sprites.sprite("pipe", 0);
        let along = Parallax {
            base_x: pipe.x,
            ..p
        };
        sprite_piece(commands, pipe_sprite, pipe, 0.5, along);
    }
    // Specks rising up the shaft in the run-out.
    for i in 0..24 {
        let base = Vec2::new(
            -HALF_W + (i as f32 * 53.0) % full,
            -HALF_H + (i as f32 * 37.0) % (2.0 * HALF_H),
        );
        commands.spawn((
            Sprite::from_color(SEA, Vec2::splat(2.0)),
            Transform::from_translation(base.extend(0.4)),
            Visibility::Hidden,
            ShaftSpeck {
                base,
                from: runout_from,
            },
            DespawnOnExit(Screen::Playing),
        ));
    }
}

pub fn scroll(position: f32, mut pieces: Query<(&Parallax, &mut Transform)>) {
    for (p, mut t) in &mut pieces {
        t.translation.x = p.x_at(position).round();
    }
}

pub fn rise(
    position: f32,
    mut specks: Query<(&ShaftSpeck, &mut Transform, &mut Visibility), Without<Parallax>>,
) {
    for (speck, mut t, mut vis) in &mut specks {
        let into = position - speck.from;
        if into <= 0.0 {
            vis.set_if_neq(Visibility::Hidden);
            continue;
        }
        vis.set_if_neq(Visibility::Inherited);
        let y = (speck.base.y + HALF_H + into * 1.5).rem_euclid(2.0 * HALF_H) - HALF_H;
        let x = (speck.base.x + HALF_W - into * 0.3).rem_euclid(2.0 * HALF_W) - HALF_W;
        t.translation.x = x.round();
        t.translation.y = y.round();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parallax_moves_at_its_factor_and_wraps() {
        let p = Parallax {
            base_x: 0.0,
            factor: 0.5,
            wrap: 400.0,
        };
        assert_eq!(p.x_at(0.0), 0.0);
        assert_eq!(p.x_at(100.0), -50.0);
        // 0.5 × 500 = 250 px left: wrapped back in from the right.
        assert_eq!(p.x_at(500.0), 150.0);
        let still = Parallax { factor: 0.0, ..p };
        assert_eq!(still.x_at(12345.0), 0.0);
    }
}
