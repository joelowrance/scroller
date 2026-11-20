//! Turning sprite sources into pixels, without touching Bevy's asset storage.
//!
//! A sprite sheet is a horizontal strip of equally sized frames, RGBA8.

use std::io::Cursor;

use bevy::prelude::*;

use super::palette::Palette;

/// A sprite as embedded by `build.rs`: its text grid and/or an override PNG.
#[derive(Clone, Copy, Debug)]
pub struct SpriteSource {
    pub name: &'static str,
    pub text: Option<&'static str>,
    pub png: Option<&'static [u8]>,
}

/// The line that marks a sprite file as background or terrain. Such sprites
/// may use only the background roles, so nothing behind the action is
/// mistaken for the Hero, an enemy or a bullet (each Cabinet design doc's
/// "Palette" rules); a test enforces it.
#[cfg_attr(not(test), allow(dead_code))]
pub const BACKGROUND_MARK: &str = "#! background";

/// Every sprite under `assets/sprites/`, sorted by name.
pub const SOURCES: &[SpriteSource] = include!(concat!(env!("OUT_DIR"), "/sprite_sources.rs"));

/// A parsed text sprite: palette roles `1`–`8`, `None` for transparent.
#[derive(Clone, Debug, PartialEq)]
pub struct Grid {
    pub width: u32,
    pub height: u32,
    pub frames: Vec<Vec<Option<u8>>>,
}

/// Parses a text sprite. Frames are separated by blank lines; `#` starts a
/// comment line; `.` and `0` are transparent and `1`–`8` are palette roles.
pub fn parse(text: &str) -> Result<Grid, String> {
    let mut frames: Vec<Vec<&str>> = vec![Vec::new()];
    for line in text.lines() {
        let line = line.trim_end();
        if line.starts_with('#') {
            continue;
        }
        if line.is_empty() {
            if !frames.last().unwrap().is_empty() {
                frames.push(Vec::new());
            }
            continue;
        }
        frames.last_mut().unwrap().push(line);
    }
    if frames.last().unwrap().is_empty() {
        frames.pop();
    }
    let first = frames.first().ok_or("no frames")?;
    let height = first.len() as u32;
    let width = first[0].chars().count() as u32;

    let mut out = Vec::with_capacity(frames.len());
    for (f, rows) in frames.iter().enumerate() {
        if rows.len() as u32 != height {
            return Err(format!(
                "frame {} is {} rows tall, expected {height}",
                f + 1,
                rows.len()
            ));
        }
        let mut pixels = Vec::with_capacity((width * height) as usize);
        for (y, row) in rows.iter().enumerate() {
            if row.chars().count() as u32 != width {
                return Err(format!(
                    "frame {} row {} is {} wide, expected {width}",
                    f + 1,
                    y + 1,
                    row.chars().count()
                ));
            }
            for c in row.chars() {
                pixels.push(match c {
                    '.' | '0' => None,
                    '1'..='8' => Some(c as u8 - b'0'),
                    _ => return Err(format!("frame {} row {}: bad pixel {c:?}", f + 1, y + 1)),
                });
            }
        }
        out.push(pixels);
    }
    Ok(Grid {
        width,
        height,
        frames: out,
    })
}

/// RGBA8 pixels for a horizontal strip of frames.
#[derive(Clone, Debug, PartialEq)]
pub struct SheetPixels {
    pub frame_size: UVec2,
    pub frames: u32,
    pub rgba: Vec<u8>,
}

impl SheetPixels {
    pub fn size(&self) -> UVec2 {
        UVec2::new(self.frame_size.x * self.frames, self.frame_size.y)
    }

    /// The RGBA value at a pixel within a frame.
    #[cfg(test)]
    pub fn pixel(&self, frame: u32, x: u32, y: u32) -> [u8; 4] {
        let stride = self.size().x;
        let i = ((y * stride + frame * self.frame_size.x + x) * 4) as usize;
        self.rgba[i..i + 4].try_into().unwrap()
    }
}

/// Colours a grid with a palette.
pub fn bake(grid: &Grid, palette: &Palette) -> SheetPixels {
    let frames = grid.frames.len() as u32;
    let stride = grid.width * frames;
    let mut rgba = vec![0u8; (stride * grid.height * 4) as usize];
    for (f, pixels) in grid.frames.iter().enumerate() {
        for (i, px) in pixels.iter().enumerate() {
            let Some(index) = px else { continue };
            let (x, y) = (i as u32 % grid.width, i as u32 / grid.width);
            let at = ((y * stride + f as u32 * grid.width + x) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&palette.color(*index).to_srgba().to_u8_array());
        }
    }
    SheetPixels {
        frame_size: UVec2::new(grid.width, grid.height),
        frames,
        rgba,
    }
}

/// The same sheet in white, keeping each pixel's alpha, to tint as a shadow.
pub fn silhouette(sheet: &SheetPixels) -> SheetPixels {
    let rgba = sheet
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .flat_map(|p| [255, 255, 255, p[3]])
        .collect();
    SheetPixels {
        rgba,
        ..sheet.clone()
    }
}

/// Decodes a PNG sheet holding `frames` frames side by side.
pub fn decode_png(bytes: &[u8], frames: u32) -> Result<SheetPixels, String> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::normalize_to_color8());
    let mut reader = decoder.read_info().map_err(|e| e.to_string())?;
    let mut buf = vec![0; reader.output_buffer_size().ok_or("PNG too large")?];
    let info = reader.next_frame(&mut buf).map_err(|e| e.to_string())?;
    let pixels = &buf[..info.buffer_size()];
    let rgba: Vec<u8> = match info.color_type {
        png::ColorType::Rgba => pixels.to_vec(),
        png::ColorType::Rgb => pixels
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => pixels
            .as_chunks::<2>()
            .0
            .iter()
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => pixels.iter().flat_map(|&g| [g, g, g, 255]).collect(),
        png::ColorType::Indexed => return Err("indexed PNG was not expanded".into()),
    };
    let frames = frames.max(1);
    if info.width % frames != 0 {
        return Err(format!(
            "PNG is {} wide, which doesn't split into {frames} frames",
            info.width
        ));
    }
    Ok(SheetPixels {
        frame_size: UVec2::new(info.width / frames, info.height),
        frames,
        rgba,
    })
}

/// What a sprite becomes: either one sheet per palette (text) or a single
/// sheet shared by every palette (PNG override).
#[derive(Debug)]
pub enum Baked {
    PerPalette(Vec<SheetPixels>),
    Override(SheetPixels),
}

impl Baked {
    pub fn first(&self) -> &SheetPixels {
        match self {
            Baked::PerPalette(sheets) => &sheets[0],
            Baked::Override(sheet) => sheet,
        }
    }
}

/// Builds a sprite. A PNG wins over the text grid; the grid (if any) still
/// says how many frames the PNG holds. A broken PNG falls back to the grid.
pub fn build(source: &SpriteSource, palettes: &[Palette]) -> Result<Baked, String> {
    let grid = match source.text.map(parse).transpose() {
        Ok(grid) => grid,
        // A PNG can stand on its own; it needs the grid only to count frames.
        Err(err) if source.png.is_some() => {
            warn!(
                "{}.txt ignored, using {}.png as one frame: {err}",
                source.name, source.name
            );
            None
        }
        Err(err) => return Err(err),
    };
    if let Some(png) = source.png {
        let frames = grid.as_ref().map_or(1, |g| g.frames.len() as u32);
        match decode_png(png, frames) {
            Ok(sheet) => return Ok(Baked::Override(sheet)),
            Err(err) if grid.is_some() => {
                warn!(
                    "{}.png ignored, using {}.txt: {err}",
                    source.name, source.name
                );
            }
            Err(err) => return Err(err),
        }
    }
    let grid = grid.ok_or("no sprite text or PNG")?;
    Ok(Baked::PerPalette(
        palettes.iter().map(|p| bake(&grid, p)).collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::super::palette::{ACCENT, BRIGHT, PALETTES};
    use super::*;

    fn encode_png(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        let mut encoder = png::Encoder::new(&mut out, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header().unwrap();
        writer.write_image_data(rgba).unwrap();
        writer.finish().unwrap();
        out
    }

    #[test]
    fn parses_frames_comments_and_transparency() {
        let grid = parse("# a comment\n\n.4\n50\n\n\n44\n55\n\n").unwrap();
        assert_eq!((grid.width, grid.height), (2, 2));
        assert_eq!(grid.frames.len(), 2);
        assert_eq!(grid.frames[0], vec![None, Some(4), Some(5), None]);
    }

    #[test]
    fn rejects_ragged_frames_and_unknown_pixels() {
        assert!(parse("11\n1\n").is_err());
        assert!(parse("11\n11\n\n111\n111\n").is_err());
        assert!(parse("19\n").is_err());
        assert!(parse("1x\n").is_err());
        assert!(parse("# only a comment\n").is_err());
    }

    #[test]
    fn bakes_frames_side_by_side_in_palette_colours() {
        let grid = parse(".4\n\n5.\n").unwrap();
        let sheet = bake(&grid, &PALETTES[0]);
        assert_eq!(sheet.size(), UVec2::new(4, 1));
        assert_eq!(sheet.pixel(0, 0, 0), [0, 0, 0, 0]);
        let accent = PALETTES[0].color(ACCENT).to_srgba().to_u8_array();
        let bright = PALETTES[0].color(BRIGHT).to_srgba().to_u8_array();
        assert_eq!(sheet.pixel(0, 1, 0), accent);
        assert_eq!(sheet.pixel(1, 0, 0), bright);
    }

    #[test]
    fn a_png_overrides_the_text_grid() {
        // Two 1×1 frames: red, then translucent blue.
        let png = encode_png(2, 1, &[255, 0, 0, 255, 0, 0, 255, 128]);
        let png: &'static [u8] = Box::leak(png.into_boxed_slice());
        let source = SpriteSource {
            name: "test",
            text: Some("4\n\n5\n"),
            png: Some(png),
        };
        let Baked::Override(sheet) = build(&source, &PALETTES).unwrap() else {
            panic!("the PNG should win over the text");
        };
        assert_eq!(sheet.frames, 2);
        assert_eq!(sheet.frame_size, UVec2::new(1, 1));
        assert_eq!(sheet.pixel(0, 0, 0), [255, 0, 0, 255]);
        assert_eq!(sheet.pixel(1, 0, 0), [0, 0, 255, 128]);
    }

    #[test]
    fn a_png_can_be_bigger_than_the_grid() {
        let png = encode_png(4, 2, &[200; 4 * 2 * 4]);
        let png: &'static [u8] = Box::leak(png.into_boxed_slice());
        let source = SpriteSource {
            name: "test",
            text: Some("4\n\n5\n"),
            png: Some(png),
        };
        assert_eq!(
            build(&source, &PALETTES).unwrap().first().frame_size,
            UVec2::new(2, 2)
        );
    }

    #[test]
    fn a_png_alone_is_one_frame_and_a_bad_png_falls_back_to_text() {
        let png = encode_png(3, 2, &[9; 3 * 2 * 4]);
        let png: &'static [u8] = Box::leak(png.into_boxed_slice());
        let alone = SpriteSource {
            name: "a",
            text: None,
            png: Some(png),
        };
        assert_eq!(build(&alone, &PALETTES).unwrap().first().frames, 1);

        let broken = SpriteSource {
            name: "b",
            text: Some("4\n"),
            png: Some(b"not a png"),
        };
        assert!(matches!(
            build(&broken, &PALETTES).unwrap(),
            Baked::PerPalette(_)
        ));
    }

    #[test]
    fn a_silhouette_is_white_with_the_same_alpha() {
        let sheet = bake(&parse(".4\n").unwrap(), &PALETTES[0]);
        let shadow = silhouette(&sheet);
        assert_eq!(shadow.pixel(0, 0, 0), [255, 255, 255, 0]);
        assert_eq!(shadow.pixel(0, 1, 0), [255, 255, 255, 255]);
    }

    #[test]
    fn a_broken_grid_still_lets_its_png_win() {
        let png = encode_png(2, 2, &[7; 2 * 2 * 4]);
        let png: &'static [u8] = Box::leak(png.into_boxed_slice());
        let source = SpriteSource {
            name: "c",
            text: Some("1x\n"),
            png: Some(png),
        };
        assert!(matches!(
            build(&source, &PALETTES).unwrap(),
            Baked::Override(_)
        ));
    }

    #[test]
    fn background_sprites_use_only_background_roles() {
        use super::super::palette::BACKGROUND_ROLES;
        let marked = |s: &&SpriteSource| {
            s.text
                .is_some_and(|t| t.lines().any(|l| l.trim_end() == BACKGROUND_MARK))
        };
        let backgrounds: Vec<_> = SOURCES.iter().filter(marked).collect();
        assert!(backgrounds.iter().any(|s| s.name == "stripe"));
        for source in backgrounds {
            let name = source.name;
            let grid = parse(source.text.unwrap()).unwrap();
            for role in grid.frames.iter().flatten().flatten() {
                assert!(BACKGROUND_ROLES.contains(role), "{name} uses role {role}");
            }
        }
    }

    #[test]
    fn every_embedded_sprite_builds() {
        assert!(!SOURCES.is_empty());
        for source in SOURCES {
            build(source, &PALETTES).unwrap_or_else(|e| panic!("{}: {e}", source.name));
        }
    }

    /// Ties the build script to the folder: every `.txt` and `.png` in
    /// `assets/sprites/` is embedded, so dropping in a PNG overrides a sprite.
    #[test]
    fn embedded_sources_match_the_sprite_folder() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/sprites");
        for entry in std::fs::read_dir(dir).unwrap().flatten() {
            let path = entry.path();
            let name = path.file_stem().unwrap().to_str().unwrap();
            let source = SOURCES.iter().find(|s| s.name == name);
            match path.extension().and_then(|e| e.to_str()) {
                Some("txt") => assert!(source.and_then(|s| s.text).is_some(), "{name}.txt"),
                Some("png") => assert!(source.and_then(|s| s.png).is_some(), "{name}.png"),
                _ => {}
            }
        }
    }
}
