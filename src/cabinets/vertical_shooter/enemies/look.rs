//! How Cabinet 2's enemies look. Until #15's sprites land they are plain
//! shapes in the Cabinet palette: a hull (Steel in the air, Clay on the
//! ground) with a 1 px Foam outline and a Volt core, and a Dusk shadow under
//! every air enemy. Swap [`body`] for real sprites later; nothing else draws
//! enemies.

use bevy::prelude::*;

use super::Kind;
use crate::cabinets::vertical_shooter::hero::shadow;
use crate::cabinets::vertical_shooter::palette::*;
use crate::cabinets::vertical_shooter::tuning::VerticalShooterTuning;

/// The sprite size of each kind, from the design doc's enemy table.
pub fn size(kind: Kind) -> Vec2 {
    let (w, h) = match kind {
        Kind::Wisp => (8.0, 8.0),
        Kind::Swift => (10.0, 12.0),
        Kind::Bellows => (20.0, 16.0),
        Kind::LoomCore => (32.0, 28.0),
        Kind::LoomShuttle => (16.0, 20.0),
        Kind::Crawler => (12.0, 12.0),
        Kind::Silo => (16.0, 16.0),
        Kind::RiserLauncher => (16.0, 14.0),
        Kind::RiserPod => (6.0, 6.0),
        Kind::Flak => (16.0, 12.0),
        Kind::Mote => (4.0, 4.0),
        Kind::TetherBalloon => (10.0, 12.0),
    };
    Vec2::new(w, h)
}

/// The glowing core that lights up as a warning (and, on a Silo or a
/// shielded Loom core, shows only while it can be hurt).
#[derive(Component)]
pub struct Core;

/// A Crawler's turret; turned each frame to match its brain.
#[derive(Component)]
pub struct Turret;

const TURRET_SIZE: Vec2 = Vec2::new(6.0, 2.0);

/// Places a turret so it sticks out from the hull's centre at `angle`
/// (radians).
pub fn aim_turret(transform: &mut Transform, angle: f32) {
    let reach = Vec2::from_angle(angle) * TURRET_SIZE.x / 2.0;
    transform.translation.x = reach.x;
    transform.translation.y = reach.y;
    transform.rotation = Quat::from_rotation_z(angle);
}

/// Draws an enemy of `kind` on `entity`: the hull as its own sprite (so kit
/// hit flashes and explosions use the hull colour), with the outline behind
/// it, the core in front, and a shadow under air enemies.
pub fn body(entity: &mut EntityCommands, kind: Kind, t: &VerticalShooterTuning) {
    let size = size(kind);
    let ground = kind.on_ground();
    let hull = if ground { CLAY } else { STEEL };
    let inner = (size - 2.0).max(Vec2::ONE);
    // Where the core sits, as a fraction of the half-size, and its size.
    let (core_at, core_size) = match kind {
        Kind::Wisp | Kind::Swift => (Vec2::new(0.0, -0.4), Vec2::new(2.0, 2.0)),
        Kind::Bellows => (Vec2::new(0.0, -0.6), Vec2::new(4.0, 2.0)),
        Kind::LoomCore => (Vec2::ZERO, Vec2::new(8.0, 8.0)),
        Kind::LoomShuttle => (Vec2::new(0.0, -0.5), Vec2::new(4.0, 3.0)),
        Kind::Mote | Kind::RiserPod => (Vec2::ZERO, Vec2::ONE * 2.0),
        _ => (Vec2::ZERO, Vec2::new(4.0, 4.0)),
    };
    let core_pos = core_at * size / 2.0;
    let hidden_core = matches!(kind, Kind::Silo | Kind::LoomCore);
    entity
        .insert(Sprite::from_color(hull, inner))
        .with_children(|c| {
            c.spawn((
                Sprite::from_color(FOAM, size),
                Transform::from_xyz(0.0, 0.0, -0.1),
            ));
            c.spawn((
                Core,
                Sprite::from_color(VOLT, core_size),
                Transform::from_translation(core_pos.round().extend(0.1)),
                if hidden_core {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                },
            ));
            if kind == Kind::Crawler {
                let mut transform = Transform::from_xyz(0.0, 0.0, 0.05);
                aim_turret(&mut transform, -std::f32::consts::FRAC_PI_2);
                c.spawn((Turret, Sprite::from_color(FOAM, TURRET_SIZE), transform));
            }
            if !ground {
                c.spawn(shadow((size - 2.0).max(Vec2::ONE), kind.z(), t));
            }
            if kind == Kind::RiserPod {
                // It starts on the pad; its shadow slides out as it climbs.
                c.spawn(shadow(Vec2::splat(4.0), kind.z(), t))
                    .insert(PodShadow);
            }
        });
}

/// A Riser pod's shadow, moved out as the pod climbs.
#[derive(Component)]
pub struct PodShadow;

/// The colour of a core: Volt normally, flashing Foam while warning.
pub fn core_color(warning: bool, clock: f32) -> Color {
    if warning && ((clock * 20.0) as u32).is_multiple_of(2) {
        FOAM
    } else {
        VOLT
    }
}

/// An enemy bullet: a Volt dot with a Foam centre.
pub fn bullet_core() -> impl Bundle {
    (
        Sprite::from_color(FOAM, Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, 0.1),
    )
}

pub const BULLET: Color = VOLT;

/// The line between Tether balloons: solid Foam while live, a faint Foam
/// flicker while it warns, otherwise nothing (no Hero colours on a hazard).
pub fn line_look(live: bool, warning: bool, clock: f32) -> (Color, Visibility) {
    if live {
        (FOAM, Visibility::Inherited)
    } else if warning && ((clock * 20.0) as u32).is_multiple_of(2) {
        (FOAM.with_alpha(0.6), Visibility::Inherited)
    } else {
        (FOAM, Visibility::Hidden)
    }
}

/// The chevron that warns of a Swift coming from below.
pub fn chevron() -> impl Bundle {
    Sprite::from_color(FOAM, Vec2::new(6.0, 4.0))
}

/// A flak crosshair: four Volt ticks around the marked point.
pub fn crosshair() -> impl Bundle {
    let tick = |w: f32, h: f32, x: f32, y: f32| {
        (
            Sprite::from_color(VOLT, Vec2::new(w, h)),
            Transform::from_xyz(x, y, 0.0),
        )
    };
    (
        Visibility::default(),
        children![
            tick(1.0, 4.0, 0.0, 6.0),
            tick(1.0, 4.0, 0.0, -6.0),
            tick(4.0, 1.0, 6.0, 0.0),
            tick(4.0, 1.0, -6.0, 0.0),
        ],
    )
}

/// A flak burst: a Volt flash the size of its hit zone.
pub fn burst(radius: f32) -> impl Bundle {
    Sprite::from_color(VOLT.with_alpha(0.7), Vec2::splat(radius * 2.0))
}
