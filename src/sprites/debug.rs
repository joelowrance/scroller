//! Pages of every sprite in every Cabinet palette, for screenshot review.
//! While `debug_keys` is on, F2 opens the first page on any screen, each
//! press shows the next one, and after the last it closes.
//!
//! Each page shows one palette: its swatches, then the sprites in name order
//! with all their frames side by side, flowed left to right and wrapped.

use bevy::prelude::*;

use super::Sprites;
use super::palette::{BACKDROP, PALETTES};
use crate::display::{HALF_H, HALF_W};
use crate::font::{Align, CELL_W, PixelText, text_at};
use crate::tuning::Tuning;

const MARGIN: f32 = 4.0;
const GAP: f32 = 2.0;
const ITEM_GAP: f32 = 6.0;
const LABEL_H: f32 = 9.0;
const TOP: f32 = HALF_H - 26.0;
const BOTTOM: f32 = -HALF_H + 4.0;

#[derive(Component)]
struct SpriteSheetOverlay;

/// Which page is open, if any.
#[derive(Resource, Default)]
struct OpenPage(Option<usize>);

/// One sprite placed on a page: its top-left corner.
struct Placed {
    name: &'static str,
    left: f32,
    top: f32,
}

/// Sprites laid out for one palette, page by page.
fn layout(sprites: &Sprites) -> Vec<Vec<Placed>> {
    let mut pages = vec![Vec::new()];
    let (mut x, mut y, mut row_h) = (-HALF_W + MARGIN, TOP, 0.0f32);
    for name in sprites.names() {
        let size = sprites.frame_size(name).as_vec2();
        let frames = sprites.frame_count(name) as f32;
        let strip = frames * (size.x + GAP) - GAP;
        let label = name.len() as f32 * CELL_W as f32;
        let w = strip.max(label);
        let h = size.y + LABEL_H;
        if x + w > HALF_W - MARGIN && x > -HALF_W + MARGIN {
            x = -HALF_W + MARGIN;
            y -= row_h + ITEM_GAP;
            row_h = 0.0;
        }
        if y - h < BOTTOM && !pages.last().unwrap().is_empty() {
            pages.push(Vec::new());
            x = -HALF_W + MARGIN;
            y = TOP;
            row_h = 0.0;
        }
        pages.last_mut().unwrap().push(Placed {
            name,
            left: x,
            top: y,
        });
        x += w + ITEM_GAP;
        row_h = row_h.max(h);
    }
    pages
}

fn toggle(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    tuning: Res<Tuning>,
    sprites: Res<Sprites>,
    mut open: ResMut<OpenPage>,
    overlay: Query<Entity, With<SpriteSheetOverlay>>,
) {
    if !tuning.debug_keys || !keys.just_pressed(KeyCode::F2) {
        return;
    }
    for entity in &overlay {
        commands.entity(entity).despawn();
    }
    let pages = layout(&sprites);
    let total = pages.len() * PALETTES.len();
    let next = open.0.map_or(0, |p| p + 1);
    if next >= total {
        open.0 = None;
        return;
    }
    open.0 = Some(next);
    let (cabinet, page) = (next / pages.len(), next % pages.len());
    let palette = &PALETTES[cabinet];

    commands
        .spawn((
            SpriteSheetOverlay,
            Transform::from_xyz(0.0, 0.0, 900.0),
            Visibility::Visible,
        ))
        .with_children(|root| {
            root.spawn(Sprite::from_color(
                palette.color(BACKDROP),
                Vec2::new(HALF_W * 2.0, HALF_H * 2.0),
            ));
            root.spawn(text_at(
                PixelText::new(
                    format!(
                        "SPRITES CAB {} PAGE {}/{}  F2: NEXT",
                        cabinet + 1,
                        page + 1,
                        pages.len()
                    ),
                    Color::WHITE,
                )
                .aligned(Align::Left),
                -HALF_W + MARGIN,
                HALF_H - 8.0,
                1,
            ));
            for (i, &color) in palette.colors.iter().enumerate() {
                root.spawn((
                    Sprite::from_color(color, Vec2::splat(6.0)),
                    Transform::from_xyz(
                        -HALF_W + MARGIN + 3.0 + i as f32 * 8.0,
                        HALF_H - 18.0,
                        1.0,
                    ),
                ));
            }
            for placed in &pages[page] {
                let size = sprites.frame_size(placed.name).as_vec2();
                for f in 0..sprites.frame_count(placed.name) {
                    let x = placed.left + f as f32 * (size.x + GAP) + size.x / 2.0;
                    let y = placed.top - size.y / 2.0;
                    root.spawn((
                        sprites.frame(placed.name, cabinet, f),
                        Transform::from_xyz(x, y, 2.0),
                    ));
                }
                root.spawn(text_at(
                    PixelText::new(placed.name, Color::srgb(0.7, 0.7, 0.7)).aligned(Align::Left),
                    placed.left,
                    placed.top - size.y - 5.0,
                    1,
                ));
            }
        });
}

pub struct SpriteSheetPlugin;

impl Plugin for SpriteSheetPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenPage>().add_systems(Update, toggle);
    }
}
