//! A fade from black whenever the screen changes, so the cut from one screen
//! to the next is never abrupt.
//!
//! It fades in only: a fade out would have to hold the old screen alive
//! while every screen's own logic waits, which each screen would need to
//! know about. Fading the new screen in from black reads the same at arcade
//! speed and keeps every screen change instant underneath.

use bevy::prelude::*;
use bevy::state::state::StateTransitionEvent;

use crate::display::GameCamera;
use crate::flow::Screen;
use crate::tuning::Tuning;

/// The black overlay on the game camera.
#[derive(Component)]
struct FadeOverlay;

/// Seconds of fade left, and how long the whole fade is.
#[derive(Resource, Default, Debug, PartialEq)]
pub struct Fade {
    pub left: f32,
    pub total: f32,
}

impl Fade {
    /// Opacity of the black overlay: 1 at the cut, easing to 0.
    pub fn alpha(&self) -> f32 {
        if self.total <= 0.0 {
            return 0.0;
        }
        let t = (self.left / self.total).clamp(0.0, 1.0);
        // Smoothstep, so the picture eases in rather than ramping.
        t * t * (3.0 - 2.0 * t)
    }
}

fn setup(mut commands: Commands, camera: Single<Entity, With<GameCamera>>) {
    // On the camera, so shake, scrolling or a turning camera never uncover
    // an edge; big enough to cover the canvas at any rotation.
    commands.entity(*camera).with_child((
        FadeOverlay,
        Sprite::from_color(Color::BLACK, Vec2::splat(480.0)),
        Transform::from_xyz(0.0, 0.0, 980.0),
        Visibility::Hidden,
    ));
}

fn start(
    tuning: Res<Tuning>,
    mut changes: MessageReader<StateTransitionEvent<Screen>>,
    mut fade: ResMut<Fade>,
) {
    let changed = changes
        .read()
        .any(|change| change.exited.is_some() && change.exited != change.entered);
    if changed && tuning.presentation.fade_seconds > 0.0 {
        fade.total = tuning.presentation.fade_seconds;
        fade.left = fade.total;
    }
}

fn run(
    time: Res<Time<Real>>,
    mut fade: ResMut<Fade>,
    mut overlay: Single<(&mut Sprite, &mut Visibility), With<FadeOverlay>>,
) {
    let (sprite, visibility) = &mut *overlay;
    let alpha = fade.alpha();
    if alpha <= 0.0 {
        visibility.set_if_neq(Visibility::Hidden);
        return;
    }
    visibility.set_if_neq(Visibility::Inherited);
    sprite.color = Color::BLACK.with_alpha(alpha);
    fade.left = (fade.left - time.delta_secs()).max(0.0);
}

pub(super) fn plugin(app: &mut App) {
    app.init_resource::<Fade>()
        .add_systems(PostStartup, setup)
        .add_systems(Update, (start, run).chain());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fade_starts_black_and_eases_to_clear() {
        let mut fade = Fade {
            left: 0.4,
            total: 0.4,
        };
        assert_eq!(fade.alpha(), 1.0);
        fade.left = 0.2;
        assert_eq!(fade.alpha(), 0.5);
        fade.left = 0.0;
        assert_eq!(fade.alpha(), 0.0);
        assert_eq!(Fade::default().alpha(), 0.0);
    }
}
