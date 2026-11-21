//! How Cabinet 2's enemies look: #15's `c2_*` sprites in the Cabinet
//! palette, a Dusk shadow under every air enemy, and the frame that shows
//! each enemy's state (a Bellows vent glowing, a Silo opening, a Loom core's
//! shield). Nothing else draws enemies.

use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::Kind;
use super::brain::{Brain, SwiftPhase};
use crate::cabinets::vertical_shooter::CABINET;
use crate::cabinets::vertical_shooter::palette::*;
use crate::cabinets::vertical_shooter::tuning::{EnemyTypes, VerticalShooterTuning, vec2};
use crate::sprites::{SpriteAnimation, Sprites};

/// The sprite size of each kind, from the design doc's enemy table. Hitboxes
/// come from these, not from the art.
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

/// The sprite in `assets/sprites/` for each kind.
pub fn sprite_name(kind: Kind) -> &'static str {
    match kind {
        Kind::Wisp => "c2_wisp",
        Kind::Swift => "c2_swift",
        Kind::Bellows => "c2_bellows",
        Kind::LoomCore => "c2_loom_core",
        Kind::LoomShuttle => "c2_loom_shuttle",
        Kind::Crawler => "c2_crawler",
        Kind::Silo => "c2_silo",
        Kind::RiserLauncher => "c2_riser",
        Kind::RiserPod => "c2_riser_pod",
        Kind::Flak => "c2_flak",
        Kind::Mote => "c2_mote",
        Kind::TetherBalloon => "c2_tether_balloon",
    }
}

/// A Crawler's turret; turned each frame to match its brain.
#[derive(Component)]
pub struct Turret;

/// An air enemy's shadow on the ground (a Riser pod's slides out as it
/// climbs).
#[derive(Component)]
pub struct EnemyShadow;

/// Kinds whose two frames just loop (a shimmer or flicker).
fn shimmers(kind: Kind) -> bool {
    matches!(kind, Kind::Wisp | Kind::Mote)
}

/// Draws an enemy of `kind` on `entity`: its sprite, a turret on a Crawler,
/// and a shadow under anything that flies.
pub fn body(entity: &mut EntityCommands, kind: Kind, sprites: &Sprites, t: &VerticalShooterTuning) {
    let name = sprite_name(kind);
    entity.insert(sprites.sprite(name, CABINET));
    if shimmers(kind) {
        entity.insert(SpriteAnimation::looping([0, 1], 0.12));
    }
    let offset = vec2(t.look.shadow_offset);
    entity.with_children(|c| {
        if kind == Kind::Crawler {
            c.spawn((
                Turret,
                sprites.sprite("c2_crawler_turret", CABINET),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        }
        if !kind.on_ground() || kind == Kind::RiserPod {
            // A pod starts on its pad, its shadow under it.
            let at = if kind == Kind::RiserPod {
                Vec2::ZERO
            } else {
                offset
            };
            c.spawn((
                EnemyShadow,
                sprites.shadow(name, 0, DUSK.with_alpha(0.85)),
                Transform::from_xyz(at.x, at.y, 1.5 - kind.z()),
            ));
        }
    });
}

/// The frame that shows an enemy's state this moment.
pub fn frame(kind: Kind, brain: &Brain, immune: bool, t: &EnemyTypes) -> Option<usize> {
    let warning = brain.telegraph(t);
    Some(match (kind, brain) {
        (Kind::Bellows | Kind::RiserPod, _) => warning as usize,
        (Kind::LoomCore, _) => (!immune) as usize,
        (Kind::Silo, Brain::Silo(silo)) if silo.open(t) => 2,
        (Kind::Silo, _) => warning as usize,
        _ => return None,
    })
}

/// How a Swift is turned: nose up while it climbs, nose down once it dives.
pub fn swift_rotation(brain: &Brain) -> f32 {
    match brain {
        Brain::Swift(s) => match s.phase {
            SwiftPhase::Turn { angle, side, .. } => -side * angle,
            SwiftPhase::Dive { .. } => std::f32::consts::PI,
            _ => 0.0,
        },
        _ => 0.0,
    }
}

/// Turns a Crawler's turret (drawn barrel-up) to `angle` radians.
pub fn aim_turret(transform: &mut Transform, angle: f32) {
    transform.rotation = Quat::from_rotation_z(angle - FRAC_PI_2);
}

/// An enemy bullet: the shared `enemy_bullet` sprite in the Cabinet palette.
pub fn bullet(sprites: &Sprites) -> Sprite {
    sprites.sprite("enemy_bullet", CABINET)
}

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
pub fn chevron(sprites: &Sprites) -> Sprite {
    sprites.sprite("chevron_up", CABINET)
}

/// A flak crosshair, blinking.
pub fn crosshair(sprites: &Sprites) -> impl Bundle {
    (
        sprites.sprite("c2_crosshair", CABINET),
        SpriteAnimation::looping([0, 1], 0.1),
    )
}

/// A flak burst: a Volt flash the size of its hit zone.
pub fn burst(radius: f32) -> impl Bundle {
    Sprite::from_color(VOLT.with_alpha(0.7), Vec2::splat(radius * 2.0))
}
