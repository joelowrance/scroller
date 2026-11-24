//! How Cabinet 3's enemies and hazards look, as plain shapes in the Cabinet
//! palette until #20's `c3_*` sprites land: an Ochre (walker) or Iron
//! (flyer) hull with a 1 px Foam outline and an Ice eye; hazards are Iron
//! with an Ice light and no outline. Swap [`body`] for sprites later.

use bevy::prelude::*;

use super::Kind;
use crate::cabinets::run_and_gun::palette::*;

/// The sprite size of each kind (the Sentry and Burrower change size with
/// their state; this is their tallest).
pub fn size(kind: Kind) -> Vec2 {
    let (w, h) = match kind {
        Kind::Skitter => (12.0, 8.0),
        Kind::Tick => (10.0, 10.0),
        Kind::Kite => (14.0, 8.0),
        Kind::Sentry => (14.0, 18.0),
        Kind::Burrower => (12.0, 12.0),
        Kind::Dropper => (24.0, 12.0),
        Kind::Hauler => (24.0, 24.0),
        Kind::Turret => (12.0, 12.0),
        Kind::Mote => (4.0, 4.0),
        Kind::Strider => (16.0, 12.0),
        Kind::Plough => (6.0, 10.0),
    };
    Vec2::new(w, h)
}

/// The eye or core that flashes as a warning.
#[derive(Component)]
pub struct Eye;

/// Draws an enemy: hull, outline, and an Ice eye at the top (high shooters)
/// or the bottom (low shooters).
pub fn body(entity: &mut EntityCommands, kind: Kind) {
    let size = size(kind);
    let hull = if kind.flies() { IRON } else { OCHRE };
    let eye_y = match kind {
        Kind::Burrower => -0.5,
        Kind::Sentry | Kind::Turret => 0.4,
        _ => 0.0,
    } * size.y
        / 2.0;
    entity
        .insert(Sprite::from_color(hull, (size - 2.0).max(Vec2::ONE)))
        .with_children(|c| {
            c.spawn((
                Sprite::from_color(FOAM, size),
                Transform::from_xyz(0.0, 0.0, -0.1),
                Outline,
            ));
            if kind != Kind::Plough {
                c.spawn((
                    Eye,
                    Sprite::from_color(ICE, Vec2::new(3.0, 2.0)),
                    Transform::from_xyz(0.0, eye_y.round(), 0.1),
                ));
            }
        });
}

/// An enemy's outline, resized with it.
#[derive(Component)]
pub struct Outline;

/// The eye's colour: Ice, flashing Foam while warning.
pub fn eye_color(warning: bool, clock: f32) -> Color {
    if warning && ((clock * 20.0) as u32).is_multiple_of(2) {
        FOAM
    } else {
        ICE
    }
}

/// An enemy bullet: an Ice dot with a Foam core.
pub fn bullet(size: Vec2) -> impl Bundle {
    (
        Sprite::from_color(ICE, size),
        children![(
            Sprite::from_color(FOAM, Vec2::ONE),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}

/// A Strider's legs and cab (a hazard: it hurts but doesn't stop shots).
pub fn strider_frame() -> impl Bundle {
    (
        Sprite::from_color(OCHRE, Vec2::new(32.0, 20.0)),
        children![
            (
                Sprite::from_color(OCHRE, Vec2::new(4.0, 30.0)),
                Transform::from_xyz(-12.0, -24.0, 0.0),
            ),
            (
                Sprite::from_color(OCHRE, Vec2::new(4.0, 30.0)),
                Transform::from_xyz(12.0, -24.0, 0.0),
            ),
        ],
    )
}

/// The left-edge chevron warning of an enemy from behind.
pub fn chevron() -> impl Bundle {
    Sprite::from_color(FOAM, Vec2::new(4.0, 6.0))
}

/// A hazard's body: Iron with an Ice light, no outline.
pub fn hazard(size: Vec2) -> impl Bundle {
    (
        Sprite::from_color(IRON, size),
        children![(
            Eye,
            Sprite::from_color(ICE, Vec2::new(4.0, 2.0)),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}
