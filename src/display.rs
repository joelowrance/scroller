//! The 320×240 arcade screen.
//!
//! Everything in the game is drawn on render layer 0 by a camera that renders
//! into a 320×240 image (the canvas). A second camera shows that canvas in the
//! window, scaled up by a whole number so pixels stay crisp, with black bars
//! filling the rest.
//!
//! Game coordinates: the origin is the centre of the screen, x runs from -160
//! to 160 and y from -120 to 120.

use bevy::camera::RenderTarget;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;

pub const WIDTH: u32 = 320;
pub const HEIGHT: u32 = 240;
pub const HALF_W: f32 = WIDTH as f32 / 2.0;
pub const HALF_H: f32 = HEIGHT as f32 / 2.0;

/// Layer for things drawn at window resolution (just the canvas itself).
const WINDOW_LAYER: RenderLayers = RenderLayers::layer(1);

/// The camera that draws the game onto the canvas.
#[derive(Component)]
pub struct GameCamera;

#[derive(Component)]
struct WindowCamera;

/// Colour drawn behind everything on the canvas. Each Cabinet sets its own.
#[derive(Resource, Clone, Copy)]
pub struct Backdrop(pub Color);

fn setup(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let canvas = images.add(Image::new_target_texture(
        WIDTH,
        HEIGHT,
        TextureFormat::Rgba8UnormSrgb,
        None,
    ));

    commands.spawn((
        Camera2d,
        Camera {
            order: -1,
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        RenderTarget::Image(canvas.clone().into()),
        Msaa::Off,
        GameCamera,
        RenderLayers::layer(0),
    ));

    commands.spawn((Sprite::from_image(canvas), WINDOW_LAYER));

    commands.spawn((
        Camera2d,
        Camera {
            clear_color: ClearColorConfig::Custom(Color::BLACK),
            ..default()
        },
        Msaa::Off,
        WindowCamera,
        WINDOW_LAYER,
    ));
}

/// Scales the canvas by the largest whole number that fits the window.
/// Windows smaller than 320×240 shrink the canvas instead.
fn fit_canvas(
    window: Single<&Window>,
    mut projection: Single<&mut Projection, With<WindowCamera>>,
) {
    let Projection::Orthographic(projection) = &mut **projection else {
        return;
    };
    let fit = (window.width() / WIDTH as f32).min(window.height() / HEIGHT as f32);
    let scale = if fit >= 1.0 {
        fit.floor()
    } else {
        fit.max(0.01)
    };
    let wanted = 1.0 / scale;
    if projection.scale != wanted {
        projection.scale = wanted;
    }
}

fn apply_backdrop(backdrop: Res<Backdrop>, mut camera: Single<&mut Camera, With<GameCamera>>) {
    if backdrop.is_changed() {
        camera.clear_color = ClearColorConfig::Custom(backdrop.0);
    }
}

pub struct DisplayPlugin;

impl Plugin for DisplayPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Backdrop(Color::BLACK))
            .add_systems(Startup, setup)
            .add_systems(Update, (fit_canvas, apply_backdrop));
    }
}
