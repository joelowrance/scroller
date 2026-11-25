//! The optional CRT look: dark scanlines between the canvas's pixel rows and
//! a slight vignette. Off by default; V turns it on or off anywhere in the
//! game, and the choice is remembered (the `display` stored setting).
//!
//! One overlay holds both, drawn at window resolution over the scaled-up
//! canvas, so the game itself is untouched. Its texture has one texel per
//! window pixel of the canvas and is rebuilt only when the scale changes, so
//! each screen row is either a scanline or not.

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use serde::{Deserialize, Serialize};

use super::PresentationTuning;
use crate::display::{GameCamera, HEIGHT, WIDTH, WINDOW_LAYER, canvas_scale};
use crate::font::PixelText;
use crate::storage::{self, DISPLAY_KEY};
use crate::tuning::Tuning;

pub const TOGGLE_KEY: KeyCode = KeyCode::KeyV;

/// The player's display settings, stored under [`DISPLAY_KEY`].
#[derive(Resource, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Debug)]
#[serde(default)]
pub struct DisplaySettings {
    pub crt: bool,
}

/// The one sprite holding scanlines and vignette (one full-screen blend,
/// which matters with software rendering).
#[derive(Component)]
struct Overlay;

/// "CRT ON" / "CRT OFF" shown briefly after toggling.
#[derive(Component)]
struct Toast(Timer);

/// The overlay's image, and the scale and strengths it was last built for.
#[derive(Resource)]
struct CrtImage {
    overlay: Handle<Image>,
    built_for: Option<(u32, u32, u32)>,
}

fn image(width: u32, height: u32, pixels: Vec<u8>) -> Image {
    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

/// Alpha of each screen row of the scanline overlay at a whole-number
/// `scale`: the last third (at least one row) of every canvas row is dark.
/// Below 2× there is no room for a scanline, so nothing is darkened.
pub fn scanline_rows(scale: u32, alpha: f32) -> Vec<u8> {
    let a = (alpha.clamp(0.0, 1.0) * 255.0).round() as u8;
    let dark = if scale >= 2 { (scale / 3).max(1) } else { 0 };
    (0..HEIGHT * scale)
        .map(|row| if row % scale >= scale - dark { a } else { 0 })
        .collect()
}

/// Alpha of the vignette at (x, y), each from -1 to 1 across the screen.
pub fn vignette_alpha(x: f32, y: f32, strength: f32) -> f32 {
    let r = (x * x + y * y).sqrt();
    let k = ((r - 0.6) / 0.8).clamp(0.0, 1.0);
    strength * k * k
}

/// Largest scale the overlay is built at; past it (a canvas over 2560×1920)
/// scanlines are slightly uneven rather than the texture growing huge.
const MAX_SCALE: u32 = 8;

/// RGBA8 pixels of the whole overlay at a whole-number `scale`: scanlines and
/// vignette combined, one texel per window pixel of the canvas.
pub fn overlay_pixels(scale: u32, scanline_alpha: f32, vignette_strength: f32) -> Vec<u8> {
    let (w, h) = (WIDTH * scale, HEIGHT * scale);
    let rows = scanline_rows(scale, scanline_alpha);
    let mut pixels = Vec::with_capacity((w * h * 4) as usize);
    for (y, row) in rows.iter().enumerate() {
        let scan = *row as f32 / 255.0;
        let fy = (y as f32 + 0.5) / h as f32 * 2.0 - 1.0;
        for x in 0..w {
            let fx = (x as f32 + 0.5) / w as f32 * 2.0 - 1.0;
            let vignette = vignette_alpha(fx, fy, vignette_strength);
            let a = 1.0 - (1.0 - scan) * (1.0 - vignette);
            pixels.extend_from_slice(&[0, 0, 0, (a * 255.0).round() as u8]);
        }
    }
    pixels
}

fn overlay_image(scale: u32, p: &PresentationTuning) -> Image {
    let scale = scale.clamp(1, MAX_SCALE);
    let pixels = overlay_pixels(scale, p.crt_scanline_alpha, p.crt_vignette_alpha);
    let mut image = image(WIDTH * scale, HEIGHT * scale, pixels);
    image.sampler = ImageSampler::nearest();
    image
}

fn setup(mut commands: Commands) {
    // No image until `apply` builds one for the window's scale.
    let overlay = Handle::default();
    commands.spawn((
        Overlay,
        Sprite {
            image: overlay.clone(),
            custom_size: Some(Vec2::new(WIDTH as f32, HEIGHT as f32)),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        Visibility::Hidden,
        WINDOW_LAYER,
    ));
    commands.insert_resource(CrtImage {
        overlay,
        built_for: None,
    });
}

fn toggle(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<DisplaySettings>,
    toasts: Query<Entity, With<Toast>>,
    camera: Single<Entity, With<GameCamera>>,
) {
    if !keys.just_pressed(TOGGLE_KEY) {
        return;
    }
    settings.crt = !settings.crt;
    storage::store(DISPLAY_KEY, &*settings);
    for toast in &toasts {
        commands.entity(toast).despawn();
    }
    let label = if settings.crt { "CRT ON" } else { "CRT OFF" };
    // On the camera, so it shows wherever a Cabinet has scrolled to, above
    // everything on the canvas, screen fades included.
    commands.entity(*camera).with_child((
        Toast(Timer::from_seconds(1.2, TimerMode::Once)),
        PixelText::new(label, Color::WHITE),
        Transform::from_xyz(0.0, -100.0, 990.0),
    ));
}

fn expire_toasts(
    mut commands: Commands,
    time: Res<Time<Real>>,
    mut toasts: Query<(Entity, &mut Toast)>,
) {
    for (entity, mut toast) in &mut toasts {
        if toast.0.tick(time.delta()).is_finished() {
            commands.entity(entity).despawn();
        }
    }
}

/// Shows or hides the overlay, and rebuilds it for a new window scale or new
/// tuning values.
fn apply(
    window: Single<&Window>,
    settings: Res<DisplaySettings>,
    tuning: Res<Tuning>,
    mut crt: ResMut<CrtImage>,
    mut images: ResMut<Assets<Image>>,
    mut overlay: Single<(&mut Visibility, &mut Sprite), With<Overlay>>,
) {
    let (visibility, sprite) = &mut *overlay;
    let wanted = if settings.crt {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    visibility.set_if_neq(wanted);
    if !settings.crt {
        return;
    }
    let p: &PresentationTuning = &tuning.presentation;
    let scale = canvas_scale(&window).max(1.0) as u32;
    let key = (
        scale,
        p.crt_scanline_alpha.to_bits(),
        p.crt_vignette_alpha.to_bits(),
    );
    if crt.built_for == Some(key) {
        return;
    }
    // Before the first build the handle is the default (shared) image, which
    // must never be removed.
    if crt.built_for.is_some() {
        images.remove(&crt.overlay);
    }
    crt.built_for = Some(key);
    // A new asset rather than editing the old one: a render-world-only image
    // keeps no pixels in the main world to edit once it has been uploaded.
    crt.overlay = images.add(overlay_image(scale, p));
    sprite.image = crt.overlay.clone();
}

pub(super) fn plugin(app: &mut App) {
    app.insert_resource(storage::load::<DisplaySettings>(DISPLAY_KEY).unwrap_or_default())
        .add_systems(Startup, setup)
        .add_systems(Update, (toggle, apply, expire_toasts).chain());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scanlines_darken_a_third_of_each_canvas_row() {
        let rows = scanline_rows(3, 0.5);
        assert_eq!(rows.len(), (HEIGHT * 3) as usize);
        assert_eq!(&rows[..6], &[0, 0, 128, 0, 0, 128]);
        let rows = scanline_rows(6, 1.0);
        assert_eq!(&rows[..6], &[0, 0, 0, 0, 255, 255]);
    }

    #[test]
    fn no_scanlines_at_one_to_one() {
        assert!(scanline_rows(1, 1.0).iter().all(|a| *a == 0));
        assert_eq!(scanline_rows(2, 1.0)[..4], [0, 255, 0, 255]);
    }

    #[test]
    fn the_vignette_leaves_the_middle_alone_and_darkens_corners() {
        assert_eq!(vignette_alpha(0.0, 0.0, 0.4), 0.0);
        assert_eq!(vignette_alpha(0.5, 0.0, 0.4), 0.0);
        let corner = vignette_alpha(1.0, 1.0, 0.4);
        assert!(corner > 0.2 && corner <= 0.4, "{corner}");
    }

    #[test]
    fn the_overlay_combines_scanlines_and_vignette() {
        let px = overlay_pixels(2, 1.0, 0.5);
        let w = (WIDTH * 2) as usize;
        let alpha = |x: usize, y: usize| px[(y * w + x) * 4 + 3];
        assert_eq!(px.len(), w * (HEIGHT * 2) as usize * 4);
        // The middle: clear on a pixel row, fully dark on its scanline.
        assert_eq!(alpha(w / 2, HEIGHT as usize), 0);
        assert_eq!(alpha(w / 2, HEIGHT as usize + 1), 255);
        // A corner on a pixel row is darkened by the vignette alone.
        assert!(alpha(0, 0) > 64 && alpha(0, 0) < 255);
    }

    #[test]
    fn the_setting_defaults_off_and_round_trips() {
        assert!(!DisplaySettings::default().crt);
        let on = DisplaySettings { crt: true };
        let text = serde_json::to_string(&on).unwrap();
        assert_eq!(serde_json::from_str::<DisplaySettings>(&text).unwrap(), on);
        assert_eq!(
            serde_json::from_str::<DisplaySettings>("{}").unwrap(),
            DisplaySettings::default()
        );
    }
}
