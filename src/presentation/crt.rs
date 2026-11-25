//! The optional CRT look: dark scanlines between the canvas's pixel rows and
//! a slight vignette. Off by default; V turns it on or off anywhere in the
//! game, and the choice is remembered (the `display` stored setting).
//!
//! Both overlays are drawn at window resolution over the scaled-up canvas, so
//! the game itself is untouched. The scanline texture is one texel wide and
//! exactly as tall as the canvas on screen, rebuilt only when the scale
//! changes, so each screen row is either a scanline or not.

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

#[derive(Component)]
struct Scanlines;

#[derive(Component)]
struct Vignette;

/// "CRT ON" / "CRT OFF" shown briefly after toggling.
#[derive(Component)]
struct Toast(Timer);

/// Handles and the scale the scanline texture was last built for.
#[derive(Resource)]
struct CrtImages {
    scanlines: Handle<Image>,
    vignette: Handle<Image>,
    built_for: Option<(u32, u32)>,
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

const VIGNETTE_W: u32 = 64;
const VIGNETTE_H: u32 = 48;

fn vignette_image(strength: f32) -> Image {
    let mut pixels = Vec::with_capacity((VIGNETTE_W * VIGNETTE_H * 4) as usize);
    for y in 0..VIGNETTE_H {
        for x in 0..VIGNETTE_W {
            let fx = (x as f32 + 0.5) / VIGNETTE_W as f32 * 2.0 - 1.0;
            let fy = (y as f32 + 0.5) / VIGNETTE_H as f32 * 2.0 - 1.0;
            let a = (vignette_alpha(fx, fy, strength) * 255.0).round() as u8;
            pixels.extend_from_slice(&[0, 0, 0, a]);
        }
    }
    let mut image = image(VIGNETTE_W, VIGNETTE_H, pixels);
    // Smooth, not pixelated: it is a soft darkening, not part of the art.
    image.sampler = ImageSampler::linear();
    image
}

fn scanline_image(scale: u32, alpha: f32) -> Image {
    let pixels = scanline_rows(scale, alpha)
        .into_iter()
        .flat_map(|a| [0, 0, 0, a])
        .collect();
    let mut image = image(1, HEIGHT * scale, pixels);
    image.sampler = ImageSampler::nearest();
    image
}

fn setup(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    tuning: Res<Tuning>,
    settings: Res<DisplaySettings>,
) {
    let p = &tuning.presentation;
    let scanlines = images.add(scanline_image(1, p.crt_scanline_alpha));
    let vignette = images.add(vignette_image(p.crt_vignette_alpha));
    let visibility = if settings.crt {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    let size = Vec2::new(WIDTH as f32, HEIGHT as f32);
    commands.spawn((
        Scanlines,
        Sprite {
            image: scanlines.clone(),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 1.0),
        visibility,
        WINDOW_LAYER,
    ));
    commands.spawn((
        Vignette,
        Sprite {
            image: vignette.clone(),
            custom_size: Some(size),
            ..default()
        },
        Transform::from_xyz(0.0, 0.0, 2.0),
        visibility,
        WINDOW_LAYER,
    ));
    commands.insert_resource(CrtImages {
        scanlines,
        vignette,
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

/// Shows or hides the overlays, and rebuilds them for a new window scale or
/// new tuning values.
fn apply(
    window: Single<&Window>,
    settings: Res<DisplaySettings>,
    tuning: Res<Tuning>,
    mut crt: ResMut<CrtImages>,
    mut images: ResMut<Assets<Image>>,
    mut overlays: Query<&mut Visibility, Or<(With<Scanlines>, With<Vignette>)>>,
) {
    let wanted = if settings.crt {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    };
    for mut visibility in &mut overlays {
        visibility.set_if_neq(wanted);
    }
    if !settings.crt {
        return;
    }
    let p: &PresentationTuning = &tuning.presentation;
    let scale = canvas_scale(&window).max(1.0) as u32;
    let key = (
        scale,
        p.crt_scanline_alpha.to_bits() ^ p.crt_vignette_alpha.to_bits(),
    );
    if crt.built_for == Some(key) {
        return;
    }
    crt.built_for = Some(key);
    if let Some(mut image) = images.get_mut(&crt.scanlines) {
        *image = scanline_image(scale, p.crt_scanline_alpha);
    }
    if let Some(mut image) = images.get_mut(&crt.vignette) {
        *image = vignette_image(p.crt_vignette_alpha);
    }
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
