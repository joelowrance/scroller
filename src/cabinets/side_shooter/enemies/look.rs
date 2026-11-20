//! How enemies look. Until the sprite pipeline (#4) lands they are plain
//! shapes in the Cabinet palette: a hull with a 1 px Foam outline and a
//! Signal core. Swap [`body`] for real sprites later; nothing else draws
//! enemies.

use bevy::prelude::*;

use super::Kind;

/// Cabinet 1 palette (docs/cabinets/1-side-shooter.md). Only the colours
/// enemies use are here; the sprite pipeline will own the full palette.
pub mod palette {
    use bevy::color::Color;

    pub const SEA: Color = Color::srgb(0.180, 0.478, 0.549); // #2E7A8C
    pub const FOAM: Color = Color::srgb(0.957, 0.957, 0.941); // #F4F4F0
    pub const SIGNAL: Color = Color::srgb(1.0, 0.235, 0.471); // #FF3C78
    pub const RUST: Color = Color::srgb(0.541, 0.227, 0.133); // #8A3A22
}

use palette::*;

/// The sprite size of each kind, from the design doc's enemy table.
pub fn size(kind: Kind) -> Vec2 {
    let (w, h) = match kind {
        Kind::Skiff => (10.0, 6.0),
        Kind::Gull => (12.0, 8.0),
        Kind::Bellbuoy => (12.0, 12.0),
        Kind::Lamplighter => (20.0, 14.0),
        Kind::Hook => (14.0, 8.0),
        Kind::Ferry => (48.0, 32.0),
        Kind::Barnacle => (12.0, 10.0),
        Kind::Clam => (16.0, 14.0),
        Kind::Mote => (4.0, 4.0),
        Kind::ArcPylon => (10.0, 16.0),
    };
    Vec2::new(w, h)
}

/// The glowing eye or core that lights up as a warning (and, on a Clam,
/// shows only while the shell is open).
#[derive(Component)]
pub struct Core;

/// A Barnacle's barrel; [`aim_barrel`] turns it each frame.
#[derive(Component)]
pub struct Barrel;

const BARREL_SIZE: Vec2 = Vec2::new(7.0, 2.0);

/// Places a barrel so it sticks out from the dome's centre at `angle`
/// (radians).
pub fn aim_barrel(transform: &mut Transform, angle: f32) {
    let reach = Vec2::from_angle(angle) * BARREL_SIZE.x / 2.0;
    transform.translation.x = reach.x;
    transform.translation.y = reach.y;
    transform.rotation = Quat::from_rotation_z(angle);
}

/// Draws an enemy of `kind` on `entity`: the hull as its own sprite (so
/// kit hit flashes and explosions use the hull colour), with the outline
/// behind it and the core in front as children.
pub fn body(entity: &mut EntityCommands, kind: Kind, flipped: bool) {
    let size = size(kind);
    let hull = match kind {
        Kind::Barnacle | Kind::ArcPylon | Kind::Clam => RUST,
        _ => SEA,
    };
    let inner = (size - 2.0).max(Vec2::ONE);
    // Where the core sits, as a fraction of the half-size, and its size.
    let (core_at, core_size) = match kind {
        Kind::Skiff | Kind::Gull | Kind::Hook => (Vec2::new(-0.5, 0.0), Vec2::new(2.0, 2.0)),
        Kind::Lamplighter => (Vec2::new(-0.75, 0.0), Vec2::new(3.0, 3.0)),
        Kind::Ferry => (Vec2::new(-0.6, 0.3), Vec2::new(6.0, 4.0)),
        Kind::Barnacle => (Vec2::new(0.0, 0.4), Vec2::new(4.0, 2.0)),
        Kind::Mote => (Vec2::ZERO, Vec2::ONE),
        _ => (Vec2::ZERO, Vec2::new(4.0, 4.0)),
    };
    let flip = if flipped { -1.0 } else { 1.0 };
    let core_pos = core_at * size / 2.0 * Vec2::new(1.0, flip);
    entity
        .insert(Sprite::from_color(hull, inner))
        .with_children(|c| {
            c.spawn((
                Sprite::from_color(FOAM, size),
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
            c.spawn((
                Core,
                Sprite::from_color(SIGNAL, core_size),
                Transform::from_translation(core_pos.round().extend(0.1)),
                if kind == Kind::Clam {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ));
            if kind == Kind::Barnacle {
                // Starts pointing away from the wall it sits on.
                let mut transform = Transform::from_xyz(0.0, 0.0, 0.05);
                aim_barrel(&mut transform, flip * std::f32::consts::FRAC_PI_2);
                c.spawn((Barrel, Sprite::from_color(FOAM, BARREL_SIZE), transform));
            }
        });
}

/// The colour of a core: Signal normally, flashing Foam while warning.
pub fn core_color(warning: bool, clock: f32) -> Color {
    if warning && ((clock * 20.0) as u32).is_multiple_of(2) {
        FOAM
    } else {
        SIGNAL
    }
}

/// An enemy bullet: a Signal dot with a Foam centre.
pub fn bullet_core() -> impl Bundle {
    (
        Sprite::from_color(FOAM, Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 0.1),
    )
}

pub const BULLET: Color = SIGNAL;

/// The arc between Arc Gate pylons: solid Foam while live, a faint Foam
/// flicker while it warns, otherwise nothing. Foam, not Spark: Spark is the
/// Hero's colour, and backgrounds never use Foam, so the arc reads as enemy.
pub fn arc_look(live: bool, warning: bool, clock: f32) -> (Color, Visibility) {
    if live {
        (FOAM, Visibility::Inherited)
    } else if warning && ((clock * 20.0) as u32).is_multiple_of(2) {
        (FOAM.with_alpha(0.6), Visibility::Inherited)
    } else {
        (FOAM, Visibility::Hidden)
    }
}

/// The chevron that warns of a Hook coming from behind.
pub fn chevron() -> impl Bundle {
    Sprite::from_color(FOAM, Vec2::new(4.0, 6.0))
}
