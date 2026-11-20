//! A sheet of every sprite in every Cabinet palette, for screenshot review.
//! F2 toggles it on any screen while `debug_keys` is on.

use bevy::prelude::*;

use super::Sprites;
use super::palette::{BACKDROP, PALETTES};
use crate::display::{HALF_H, HALF_W};
use crate::font::{Align, PixelText, text_at};
use crate::tuning::Tuning;

const LABEL_W: f32 = 64.0;
const COLUMN_W: f32 = (HALF_W * 2.0 - LABEL_W) / PALETTES.len() as f32;
const GAP: f32 = 2.0;

#[derive(Component)]
struct SpriteSheetOverlay;

fn toggle(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    tuning: Res<Tuning>,
    sprites: Res<Sprites>,
    open: Query<Entity, With<SpriteSheetOverlay>>,
) {
    if !tuning.debug_keys || !keys.just_pressed(KeyCode::F2) {
        return;
    }
    if let Ok(overlay) = open.single() {
        commands.entity(overlay).despawn();
        return;
    }

    let white = Color::WHITE;
    let grey = Color::srgb(0.6, 0.6, 0.6);
    let column_x = |c: usize| -HALF_W + LABEL_W + COLUMN_W * (c as f32 + 0.5);

    commands
        .spawn((
            SpriteSheetOverlay,
            Transform::from_xyz(0.0, 0.0, 900.0),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(Sprite::from_color(
                Color::BLACK,
                Vec2::new(HALF_W * 2.0, HALF_H * 2.0),
            ));
            root.spawn(text_at(
                PixelText::new("SPRITE SHEET   F2: CLOSE", white),
                0.0,
                HALF_H - 10.0,
                1,
            ));

            // Column headers and palette swatches.
            for (c, palette) in PALETTES.iter().enumerate() {
                let x = column_x(c);
                root.spawn(text_at(
                    PixelText::new(format!("CAB {}", c + 1), grey),
                    x,
                    HALF_H - 24.0,
                    1,
                ));
                for (i, &color) in palette.colors.iter().enumerate() {
                    let sx = x - 28.0 + 3.0 + i as f32 * 7.0;
                    root.spawn((
                        Sprite::from_color(color, Vec2::splat(6.0)),
                        Transform::from_xyz(sx, HALF_H - 35.0, 1.0),
                    ));
                }
            }

            // One row per sprite: every frame, in every palette.
            let mut top = HALF_H - 42.0;
            for name in sprites.names() {
                let frames = sprites.frame_count(name);
                let size = sprites.frame_size(name).as_vec2();
                let strip = |scale: f32| frames as f32 * (size.x * scale + GAP) - GAP;
                let scale = if strip(2.0) <= COLUMN_W - 6.0 && size.y <= 24.0 {
                    2.0
                } else {
                    1.0
                };
                let row_h = size.y * scale + 6.0;
                let mid = (top - row_h / 2.0).round();
                root.spawn(text_at(
                    PixelText::new(name, white).aligned(Align::Left),
                    -HALF_W + 4.0,
                    mid,
                    1,
                ));
                for (c, palette) in PALETTES.iter().enumerate() {
                    let x = column_x(c);
                    root.spawn((
                        Sprite::from_color(
                            palette.color(BACKDROP),
                            Vec2::new(COLUMN_W - 2.0, row_h),
                        ),
                        Transform::from_xyz(x, mid, 1.0),
                    ));
                    let left = (x - strip(scale) / 2.0).round();
                    for f in 0..frames {
                        let fx = left + f as f32 * (size.x * scale + GAP) + size.x * scale / 2.0;
                        let fy = mid - (row_h / 2.0).floor() + 3.0 + size.y * scale / 2.0;
                        root.spawn((
                            sprites.frame(name, c, f),
                            Transform::from_xyz(fx, fy, 2.0).with_scale(Vec3::splat(scale)),
                        ));
                    }
                }
                top -= row_h + 2.0;
            }
        });
}

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, toggle);
    }
}
