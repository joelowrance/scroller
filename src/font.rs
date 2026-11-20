//! A generated 5×7 bitmap font, drawn as sprites so text stays pixel-perfect
//! on the low-resolution canvas.
//!
//! Covers ASCII 32–95 (space through `_`); lowercase is drawn as uppercase.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

/// Width of one character cell, including 1px spacing.
pub const CELL_W: u32 = 6;
/// Height of one character cell, including 1px spacing.
pub const CELL_H: u32 = 8;
const GLYPH_W: usize = 5;
const GLYPH_H: usize = 7;
const COLUMNS: u32 = 16;
const ROWS: u32 = 4;
const FIRST: u8 = b' ';

#[rustfmt::skip]
const GLYPHS: &[(char, [&str; GLYPH_H])] = &[
    ('!', ["..#..", "..#..", "..#..", "..#..", "..#..", ".....", "..#.."]),
    ('"', [".#.#.", ".#.#.", ".....", ".....", ".....", ".....", "....."]),
    ('#', [".#.#.", "#####", ".#.#.", ".#.#.", "#####", ".#.#.", "....."]),
    ('%', ["##..#", "##.#.", "..#..", ".#...", "#..##", "...##", "....."]),
    ('&', [".##..", "#..#.", ".##..", "#..#.", "#...#", "#..#.", ".##.#"]),
    ('\'', ["..#..", "..#..", ".....", ".....", ".....", ".....", "....."]),
    ('(', ["...#.", "..#..", ".#...", ".#...", ".#...", "..#..", "...#."]),
    (')', [".#...", "..#..", "...#.", "...#.", "...#.", "..#..", ".#..."]),
    ('*', [".....", "#.#.#", ".###.", "#####", ".###.", "#.#.#", "....."]),
    ('+', [".....", "..#..", "..#..", "#####", "..#..", "..#..", "....."]),
    (',', [".....", ".....", ".....", ".....", "..#..", "..#..", ".#..."]),
    ('-', [".....", ".....", ".....", "#####", ".....", ".....", "....."]),
    ('.', [".....", ".....", ".....", ".....", ".....", ".##..", ".##.."]),
    ('/', ["....#", "...#.", "...#.", "..#..", ".#...", ".#...", "#...."]),
    ('0', [".###.", "#...#", "#..##", "#.#.#", "##..#", "#...#", ".###."]),
    ('1', ["..#..", ".##..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('2', [".###.", "#...#", "....#", "...#.", "..#..", ".#...", "#####"]),
    ('3', ["#####", "...#.", "..#..", "...#.", "....#", "#...#", ".###."]),
    ('4', ["...#.", "..##.", ".#.#.", "#..#.", "#####", "...#.", "...#."]),
    ('5', ["#####", "#....", "####.", "....#", "....#", "#...#", ".###."]),
    ('6', ["..##.", ".#...", "#....", "####.", "#...#", "#...#", ".###."]),
    ('7', ["#####", "....#", "...#.", "..#..", ".#...", ".#...", ".#..."]),
    ('8', [".###.", "#...#", "#...#", ".###.", "#...#", "#...#", ".###."]),
    ('9', [".###.", "#...#", "#...#", ".####", "....#", "...#.", ".##.."]),
    (':', [".....", ".##..", ".##..", ".....", ".##..", ".##..", "....."]),
    (';', [".....", ".##..", ".##..", ".....", ".##..", "..#..", ".#..."]),
    ('<', ["...#.", "..#..", ".#...", "#....", ".#...", "..#..", "...#."]),
    ('=', [".....", ".....", "#####", ".....", "#####", ".....", "....."]),
    ('>', [".#...", "..#..", "...#.", "....#", "...#.", "..#..", ".#..."]),
    ('?', [".###.", "#...#", "....#", "...#.", "..#..", ".....", "..#.."]),
    ('@', [".###.", "#...#", "#.###", "#.#.#", "#.###", "#....", ".###."]),
    ('A', [".###.", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('B', ["####.", "#...#", "#...#", "####.", "#...#", "#...#", "####."]),
    ('C', [".###.", "#...#", "#....", "#....", "#....", "#...#", ".###."]),
    ('D', ["###..", "#..#.", "#...#", "#...#", "#...#", "#..#.", "###.."]),
    ('E', ["#####", "#....", "#....", "####.", "#....", "#....", "#####"]),
    ('F', ["#####", "#....", "#....", "####.", "#....", "#....", "#...."]),
    ('G', [".###.", "#...#", "#....", "#.###", "#...#", "#...#", ".####"]),
    ('H', ["#...#", "#...#", "#...#", "#####", "#...#", "#...#", "#...#"]),
    ('I', [".###.", "..#..", "..#..", "..#..", "..#..", "..#..", ".###."]),
    ('J', ["..###", "...#.", "...#.", "...#.", "...#.", "#..#.", ".##.."]),
    ('K', ["#...#", "#..#.", "#.#..", "##...", "#.#..", "#..#.", "#...#"]),
    ('L', ["#....", "#....", "#....", "#....", "#....", "#....", "#####"]),
    ('M', ["#...#", "##.##", "#.#.#", "#.#.#", "#...#", "#...#", "#...#"]),
    ('N', ["#...#", "#...#", "##..#", "#.#.#", "#..##", "#...#", "#...#"]),
    ('O', [".###.", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('P', ["####.", "#...#", "#...#", "####.", "#....", "#....", "#...."]),
    ('Q', [".###.", "#...#", "#...#", "#...#", "#.#.#", "#..#.", ".##.#"]),
    ('R', ["####.", "#...#", "#...#", "####.", "#.#..", "#..#.", "#...#"]),
    ('S', [".####", "#....", "#....", ".###.", "....#", "....#", "####."]),
    ('T', ["#####", "..#..", "..#..", "..#..", "..#..", "..#..", "..#.."]),
    ('U', ["#...#", "#...#", "#...#", "#...#", "#...#", "#...#", ".###."]),
    ('V', ["#...#", "#...#", "#...#", "#...#", "#...#", ".#.#.", "..#.."]),
    ('W', ["#...#", "#...#", "#...#", "#.#.#", "#.#.#", "#.#.#", ".#.#."]),
    ('X', ["#...#", "#...#", ".#.#.", "..#..", ".#.#.", "#...#", "#...#"]),
    ('Y', ["#...#", "#...#", ".#.#.", "..#..", "..#..", "..#..", "..#.."]),
    ('Z', ["#####", "....#", "...#.", "..#..", ".#...", "#....", "#####"]),
    ('[', [".###.", ".#...", ".#...", ".#...", ".#...", ".#...", ".###."]),
    (']', [".###.", "...#.", "...#.", "...#.", "...#.", "...#.", ".###."]),
    ('_', [".....", ".....", ".....", ".....", ".....", ".....", "#####"]),
];

/// Handles to the generated font atlas.
#[derive(Resource)]
pub struct Font {
    image: Handle<Image>,
    layout: Handle<TextureAtlasLayout>,
}

/// A line of pixel text. Its glyph sprites are rebuilt whenever it changes.
#[derive(Component, Clone)]
#[require(Transform, Visibility)]
pub struct PixelText {
    pub text: String,
    pub color: Color,
    pub align: Align,
}

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

impl PixelText {
    pub fn new(text: impl Into<String>, color: Color) -> Self {
        Self {
            text: text.into(),
            color,
            align: Align::Center,
        }
    }

    pub fn aligned(mut self, align: Align) -> Self {
        self.align = align;
        self
    }
}

/// Bundle for a line of text at a whole-pixel position, optionally scaled up
/// by a whole number for headings.
pub fn text_at(text: PixelText, x: f32, y: f32, scale: u32) -> impl Bundle {
    (
        text,
        Transform::from_xyz(x.round(), y.round(), 10.0).with_scale(Vec3::splat(scale as f32)),
    )
}

fn glyph_index(c: char) -> usize {
    let c = c.to_ascii_uppercase();
    match c as u32 {
        code @ 32..=95 => (code as u8 - FIRST) as usize,
        _ => (b'?' - FIRST) as usize,
    }
}

fn build_atlas_pixels() -> Vec<u8> {
    let width = (COLUMNS * CELL_W) as usize;
    let height = (ROWS * CELL_H) as usize;
    let mut pixels = vec![0u8; width * height * 4];
    for (c, rows) in GLYPHS {
        let index = glyph_index(*c);
        let cell_x = (index % COLUMNS as usize) * CELL_W as usize;
        let cell_y = (index / COLUMNS as usize) * CELL_H as usize;
        for (y, row) in rows.iter().enumerate() {
            for (x, ch) in row.chars().take(GLYPH_W).enumerate() {
                if ch == '#' {
                    let i = ((cell_y + y) * width + cell_x + x) * 4;
                    pixels[i..i + 4].copy_from_slice(&[255, 255, 255, 255]);
                }
            }
        }
    }
    pixels
}

fn setup_font(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut image = Image::new(
        Extent3d {
            width: COLUMNS * CELL_W,
            height: ROWS * CELL_H,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        build_atlas_pixels(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    let layout =
        TextureAtlasLayout::from_grid(UVec2::new(CELL_W, CELL_H), COLUMNS, ROWS, None, None);
    commands.insert_resource(Font {
        image: images.add(image),
        layout: layouts.add(layout),
    });
}

fn rebuild_text(
    mut commands: Commands,
    font: Res<Font>,
    texts: Query<(Entity, &PixelText), Changed<PixelText>>,
) {
    for (entity, text) in &texts {
        let count = text.text.chars().count() as f32;
        let width = count * CELL_W as f32;
        let start = match text.align {
            Align::Left => 0.0,
            Align::Center => (-width / 2.0).floor(),
            Align::Right => -width,
        };
        let mut e = commands.entity(entity);
        e.despawn_related::<Children>();
        e.with_children(|parent| {
            for (i, c) in text.text.chars().enumerate() {
                if c == ' ' {
                    continue;
                }
                let mut sprite = Sprite::from_atlas_image(
                    font.image.clone(),
                    TextureAtlas {
                        layout: font.layout.clone(),
                        index: glyph_index(c),
                    },
                );
                sprite.color = text.color;
                let x = start + i as f32 * CELL_W as f32 + CELL_W as f32 / 2.0;
                parent.spawn((sprite, Transform::from_xyz(x, 0.0, 0.0)));
            }
        });
    }
}

/// Makes an entity blink on and off.
#[derive(Component)]
pub struct Blink(pub Timer);

impl Blink {
    pub fn every(seconds: f32) -> Self {
        Self(Timer::from_seconds(seconds, TimerMode::Repeating))
    }
}

fn blink(time: Res<Time<Real>>, mut blinkers: Query<(&mut Blink, &mut Visibility)>) {
    for (mut blink, mut visibility) in &mut blinkers {
        if blink.0.tick(time.delta()).just_finished() {
            visibility.toggle_visible_hidden();
        }
    }
}

pub struct FontPlugin;

impl Plugin for FontPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup_font)
            .add_systems(PostUpdate, rebuild_text)
            .add_systems(Update, blink);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_glyph_row_is_five_wide_and_fits_the_atlas() {
        for (c, rows) in GLYPHS {
            assert!(rows.iter().all(|r| r.len() == GLYPH_W), "glyph {c:?}");
            assert!(glyph_index(*c) < (COLUMNS * ROWS) as usize);
        }
    }

    #[test]
    fn lowercase_and_unknown_characters_map_sensibly() {
        assert_eq!(glyph_index('a'), glyph_index('A'));
        assert_eq!(glyph_index('é'), glyph_index('?'));
    }
}
