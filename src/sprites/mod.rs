//! Generated pixel art: text sprites under `assets/sprites/`, coloured with
//! each Cabinet's palette and turned into textures at startup.
//!
//! - Sprite files are character grids, one character per palette index, with
//!   frames separated by blank lines (format in `assets/sprites/README.md`).
//! - Every sprite is baked once per Cabinet palette, so
//!   `sprites.sprite("hero", cabinet)` returns it in that Cabinet's colours.
//! - A `name.png` beside `name.txt` (frames side by side) replaces the grid in
//!   every palette; no code change is needed, only a rebuild.
//! - [`SpriteAnimation`] steps through atlas frames.
//! - With `debug_keys` on, F2 toggles a sheet showing every sprite in every
//!   palette, for screenshot review.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};

mod debug;
pub mod palette;
mod sheet;

use palette::PALETTES;
use sheet::{Baked, SOURCES, SheetPixels};

/// Textures and atlas layout for one sprite.
struct SheetHandles {
    frames: usize,
    frame_size: UVec2,
    layout: Handle<TextureAtlasLayout>,
    /// One image per Cabinet palette, or a single image for a PNG override.
    images: Vec<Handle<Image>>,
}

/// Every sprite, ready to draw.
#[derive(Resource, Default)]
pub struct Sprites {
    sheets: HashMap<&'static str, SheetHandles>,
}

impl Sprites {
    /// The first frame of a sprite in a Cabinet's palette. An unknown name
    /// logs an error and draws a magenta square so it stands out.
    pub fn sprite(&self, name: &str, cabinet: usize) -> Sprite {
        self.frame(name, cabinet, 0)
    }

    /// A given frame of a sprite in a Cabinet's palette.
    pub fn frame(&self, name: &str, cabinet: usize, frame: usize) -> Sprite {
        let Some(sheet) = self.sheets.get(name) else {
            error!("no sprite named {name:?} in assets/sprites/");
            return Sprite::from_color(Color::srgb(1.0, 0.0, 1.0), Vec2::splat(8.0));
        };
        let image = sheet.images[cabinet.min(sheet.images.len() - 1)].clone();
        Sprite::from_atlas_image(
            image,
            TextureAtlas {
                layout: sheet.layout.clone(),
                index: frame % sheet.frames,
            },
        )
    }

    /// How many frames a sprite has (0 if unknown).
    pub fn frame_count(&self, name: &str) -> usize {
        self.sheets.get(name).map_or(0, |s| s.frames)
    }

    /// Size of one frame in pixels.
    pub fn frame_size(&self, name: &str) -> UVec2 {
        self.sheets.get(name).map_or(UVec2::ZERO, |s| s.frame_size)
    }

    /// Sprite names in alphabetical order.
    pub fn names(&self) -> impl Iterator<Item = &'static str> {
        SOURCES
            .iter()
            .map(|s| s.name)
            .filter(|n| self.sheets.contains_key(n))
    }
}

fn to_image(sheet: &SheetPixels) -> Image {
    let size = sheet.size();
    let mut image = Image::new(
        Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        sheet.rgba.clone(),
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

fn load_sprites(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let mut sprites = Sprites::default();
    for source in SOURCES {
        let baked = match sheet::build(source, &PALETTES) {
            Ok(baked) => baked,
            Err(err) => {
                error!("sprite {:?}: {err}", source.name);
                continue;
            }
        };
        let first = baked.first();
        let layout = TextureAtlasLayout::from_grid(first.frame_size, first.frames, 1, None, None);
        let handles = match &baked {
            Baked::PerPalette(sheets) => sheets.iter().map(|s| images.add(to_image(s))).collect(),
            Baked::Override(sheet) => vec![images.add(to_image(sheet))],
        };
        sprites.sheets.insert(
            source.name,
            SheetHandles {
                frames: first.frames as usize,
                frame_size: first.frame_size,
                layout: layouts.add(layout),
                images: handles,
            },
        );
    }
    commands.insert_resource(sprites);
}

// Play-once animations (explosions, pickups) arrive with the real Cabinets.
#[allow(dead_code)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimationMode {
    /// Start again after the last frame.
    Loop,
    /// Stop on the last frame.
    Once,
}

/// Steps a sprite's atlas index through a list of frames. Runs on virtual
/// time, so it freezes while the game is paused.
#[derive(Component, Clone, Debug)]
pub struct SpriteAnimation {
    pub frames: Vec<usize>,
    pub frame_time: f32,
    pub mode: AnimationMode,
    elapsed: f32,
}

#[allow(dead_code)]
impl SpriteAnimation {
    pub fn new(frames: impl Into<Vec<usize>>, frame_time: f32, mode: AnimationMode) -> Self {
        Self {
            frames: frames.into(),
            frame_time,
            mode,
            elapsed: 0.0,
        }
    }

    pub fn looping(frames: impl Into<Vec<usize>>, frame_time: f32) -> Self {
        Self::new(frames, frame_time, AnimationMode::Loop)
    }

    pub fn once(frames: impl Into<Vec<usize>>, frame_time: f32) -> Self {
        Self::new(frames, frame_time, AnimationMode::Once)
    }

    fn step(&self) -> usize {
        (self.elapsed / self.frame_time.max(f32::EPSILON)) as usize
    }

    /// Advances time; returns the atlas index to show.
    pub fn tick(&mut self, seconds: f32) -> usize {
        self.elapsed += seconds;
        if self.mode == AnimationMode::Loop {
            let length = self.frame_time * self.frames.len() as f32;
            if length > 0.0 {
                self.elapsed %= length;
            }
        }
        self.current()
    }

    /// The atlas index currently showing.
    pub fn current(&self) -> usize {
        let Some(&last) = self.frames.last() else {
            return 0;
        };
        self.frames.get(self.step()).copied().unwrap_or(last)
    }

    /// A `Once` animation has reached its last frame's end.
    pub fn finished(&self) -> bool {
        self.mode == AnimationMode::Once && self.step() >= self.frames.len()
    }
}

fn animate(time: Res<Time>, mut animated: Query<(&mut SpriteAnimation, &mut Sprite)>) {
    for (mut animation, mut sprite) in &mut animated {
        let index = animation.tick(time.delta_secs());
        if let Some(atlas) = &mut sprite.texture_atlas
            && atlas.index != index
        {
            atlas.index = index;
        }
    }
}

pub struct SpritesPlugin;

impl Plugin for SpritesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, load_sprites)
            .add_systems(Update, animate)
            .add_plugins(debug::SpriteSheetPlugin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn looping_animation_wraps() {
        let mut a = SpriteAnimation::looping([3, 4, 5], 0.1);
        assert_eq!(a.current(), 3);
        assert_eq!(a.tick(0.15), 4);
        assert_eq!(a.tick(0.1), 5);
        assert_eq!(a.tick(0.1), 3);
        assert!(!a.finished());
    }

    #[test]
    fn once_animation_holds_the_last_frame() {
        let mut a = SpriteAnimation::once([0, 1], 0.1);
        assert_eq!(a.tick(0.05), 0);
        assert!(!a.finished());
        assert_eq!(a.tick(1.0), 1);
        assert!(a.finished());
    }

    #[test]
    fn empty_animation_is_harmless() {
        let mut a = SpriteAnimation::looping(Vec::new(), 0.1);
        assert_eq!(a.tick(1.0), 0);
    }
}
