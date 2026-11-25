//! The title logo: "SCROLLER" in chunky slanted letters with a sunset-chrome
//! gradient, a dark outline and a drop shadow, generated as pixel art at the
//! canvas's own resolution. A glint sweeps across it now and then.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

use crate::sprites::SpriteAnimation;

const WORD: &str = "SCROLLER";
/// Each letter is drawn on a 7×9 grid; every grid cell becomes CELL×CELL pixels.
const GRID_W: usize = 7;
const GRID_H: usize = 9;
const CELL: usize = 3;
const LETTER_W: usize = GRID_W * CELL;
const LETTER_H: usize = GRID_H * CELL;
const GAP: usize = CELL;
/// Rows of letter per pixel of rightward slant (bottom row unslanted).
const SLANT_EVERY: usize = 4;
const SLANT: usize = (LETTER_H - 1) / SLANT_EVERY;
const SHADOW: usize = 2;
/// Glint positions in the animation, plus one frame with no glint.
const GLINT_STEPS: usize = 14;
const GLINT_FRAMES: usize = GLINT_STEPS + 1;
/// Seconds per glint frame, and frames of rest between sweeps.
const GLINT_FRAME_SECONDS: f32 = 0.035;
const REST_FRAMES: usize = 90;

#[rustfmt::skip]
const LETTERS: &[(char, [&str; GRID_H])] = &[
    ('S', [".#####.", "##...##", "##.....", "##.....", ".#####.", ".....##", ".....##", "##...##", ".#####."]),
    ('C', [".#####.", "##...##", "##.....", "##.....", "##.....", "##.....", "##.....", "##...##", ".#####."]),
    ('R', ["######.", "##...##", "##...##", "##...##", "######.", "##.##..", "##..##.", "##...##", "##...##"]),
    ('O', [".#####.", "##...##", "##...##", "##...##", "##...##", "##...##", "##...##", "##...##", ".#####."]),
    ('L', ["##.....", "##.....", "##.....", "##.....", "##.....", "##.....", "##.....", "##.....", "#######"]),
    ('E', ["#######", "##.....", "##.....", "##.....", "######.", "##.....", "##.....", "##.....", "#######"]),
];

/// Logo size in pixels.
pub const fn size() -> (usize, usize) {
    let letters = WORD.len();
    let w = letters * LETTER_W + (letters - 1) * GAP + SLANT + SHADOW + 2;
    let h = LETTER_H + SHADOW + 2;
    (w, h)
}

/// Top to bottom: pale highlight, gold, orange, a dark "horizon" seam, then
/// hot pink fading to violet.
fn fill(y: usize) -> [u8; 3] {
    let t = y as f32 / (LETTER_H - 1) as f32;
    let stops: [(f32, [f32; 3]); 6] = [
        (0.0, [255.0, 250.0, 210.0]),
        (0.30, [255.0, 205.0, 60.0]),
        (0.48, [255.0, 130.0, 40.0]),
        (0.52, [110.0, 20.0, 80.0]),
        (0.58, [255.0, 70.0, 160.0]),
        (1.0, [150.0, 40.0, 200.0]),
    ];
    let (mut a, mut b) = (stops[0], stops[stops.len() - 1]);
    for pair in stops.windows(2) {
        if t >= pair[0].0 && t <= pair[1].0 {
            (a, b) = (pair[0], pair[1]);
            break;
        }
    }
    let k = if b.0 > a.0 {
        (t - a.0) / (b.0 - a.0)
    } else {
        0.0
    };
    // The seam is a hard edge, not a blend.
    let k = if a.0 >= 0.48 && b.0 <= 0.58 {
        k.round()
    } else {
        k
    };
    std::array::from_fn(|i| (a.1[i] + (b.1[i] - a.1[i]) * k) as u8)
}

const OUTLINE: [u8; 4] = [20, 6, 40, 255];
const SHADOW_RGBA: [u8; 4] = [60, 10, 90, 255];
const GLINT: [u8; 4] = [255, 255, 255, 255];

/// Which pixels of the logo are letter, before outline and shadow.
fn mask() -> (usize, usize, Vec<bool>) {
    let (w, h) = size();
    let mut m = vec![false; w * h];
    for (i, c) in WORD.chars().enumerate() {
        let rows = LETTERS
            .iter()
            .find(|(l, _)| *l == c)
            .map(|(_, r)| r)
            .expect("every logo letter is drawn");
        let left = 1 + i * (LETTER_W + GAP);
        for py in 0..LETTER_H {
            let slant = (LETTER_H - 1 - py) / SLANT_EVERY;
            let row = rows[py / CELL].as_bytes();
            for px in 0..LETTER_W {
                if row[px / CELL] == b'#' {
                    m[(1 + py) * w + left + slant + px] = true;
                }
            }
        }
    }
    (w, h, m)
}

/// RGBA8 pixels of one logo frame. `glint` is the glint's x position along
/// the logo, if it shows in this frame.
pub fn pixels(glint: Option<f32>) -> Vec<u8> {
    let (w, h, m) = mask();
    let on = |x: isize, y: isize| {
        x >= 0 && y >= 0 && (x as usize) < w && (y as usize) < h && m[y as usize * w + x as usize]
    };
    let mut out = vec![0u8; w * h * 4];
    for y in 0..h {
        for x in 0..w {
            let (xi, yi) = (x as isize, y as isize);
            let px = if on(xi, yi) {
                // A diagonal glint band, 3px wide, leaning with the slant.
                let lean = x as f32 + (y as f32) / SLANT_EVERY as f32;
                if glint.is_some_and(|g| (lean - g).abs() < 1.5) {
                    GLINT
                } else {
                    let [r, g, b] = fill(y - 1);
                    [r, g, b, 255]
                }
            } else if (-1..=1).any(|dy| (-1..=1).any(|dx| on(xi + dx, yi + dy))) {
                OUTLINE
            } else if (0..=SHADOW as isize).any(|d| on(xi - d, yi - d) || on(xi - d + 1, yi - d)) {
                SHADOW_RGBA
            } else {
                [0, 0, 0, 0]
            };
            out[(y * w + x) * 4..][..4].copy_from_slice(&px);
        }
    }
    out
}

/// A horizontal strip of every animation frame: frame 0 has no glint, the
/// rest sweep it from left to right.
fn strip() -> Vec<u8> {
    let (w, h) = size();
    let frames: Vec<Vec<u8>> = (0..GLINT_FRAMES)
        .map(|f| (f > 0).then(|| (f - 1) as f32 / (GLINT_STEPS - 1) as f32 * (w + LETTER_H) as f32))
        .map(|g| pixels(g.map(|g| g - LETTER_H as f32 / 2.0)))
        .collect();
    let mut out = vec![0u8; w * GLINT_FRAMES * h * 4];
    for (f, frame) in frames.iter().enumerate() {
        for y in 0..h {
            let dst = (y * w * GLINT_FRAMES + f * w) * 4;
            out[dst..dst + w * 4].copy_from_slice(&frame[y * w * 4..(y + 1) * w * 4]);
        }
    }
    out
}

/// The logo's sprite and its animation, ready to spawn.
pub fn bundle(
    images: &mut Assets<Image>,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle + use<> {
    let (w, h) = size();
    let mut image = Image::new(
        Extent3d {
            width: (w * GLINT_FRAMES) as u32,
            height: h as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        strip(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    let layout = TextureAtlasLayout::from_grid(
        UVec2::new(w as u32, h as u32),
        GLINT_FRAMES as u32,
        1,
        None,
        None,
    );
    let frames: Vec<usize> = std::iter::repeat_n(0, REST_FRAMES)
        .chain(1..GLINT_FRAMES)
        .collect();
    (
        Sprite::from_atlas_image(
            images.add(image),
            TextureAtlas {
                layout: layouts.add(layout),
                index: 0,
            },
        ),
        SpriteAnimation::looping(frames, GLINT_FRAME_SECONDS),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_letter_row_is_seven_wide() {
        for (c, rows) in LETTERS {
            assert!(rows.iter().all(|r| r.len() == GRID_W), "letter {c}");
        }
    }

    #[test]
    fn the_logo_fits_the_canvas_with_room_to_spare() {
        let (w, h) = size();
        assert!(w <= 240 && h <= 40, "{w}×{h}");
    }

    #[test]
    fn letters_are_outlined_and_the_border_is_clear() {
        let (w, h) = size();
        let px = pixels(None);
        let at = |x: usize, y: usize| &px[(y * w + x) * 4..][..4];
        // Nothing touches the image's top row except outline.
        assert!((0..w).all(|x| at(x, 0)[3] == 0 || at(x, 0) == OUTLINE));
        assert!(px.chunks(4).any(|p| p == OUTLINE));
        assert!(px.chunks(4).any(|p| p == SHADOW_RGBA));
        assert!(px.chunks(4).filter(|p| p[3] == 255).count() > w * h / 4);
    }

    #[test]
    fn the_glint_only_lights_letter_pixels() {
        let plain = pixels(None);
        let lit = pixels(Some(60.0));
        let changed: Vec<usize> = (0..plain.len() / 4)
            .filter(|i| plain[i * 4..][..4] != lit[i * 4..][..4])
            .collect();
        assert!(!changed.is_empty());
        assert!(changed.iter().all(|i| lit[i * 4..][..4] == GLINT));
        assert!(changed.iter().all(|i| plain[i * 4 + 3] == 255));
    }
}
