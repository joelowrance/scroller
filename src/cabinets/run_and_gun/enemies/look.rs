//! How Cabinet 3's enemies and hazards look: the `c3_*` sprites, with the
//! frame each brain's state calls for (see [`frame_for`]). The Mote is a
//! plain Ice dot; it has no sprite.

use bevy::prelude::*;
use bevy::sprite::Anchor;

use super::Kind;
use super::brain::{Brain, BurrowPhase, SentryPhase};
use crate::cabinets::run_and_gun::CABINET;
use crate::cabinets::run_and_gun::palette::*;
use crate::cabinets::run_and_gun::tuning::EnemyTypes;
use crate::sprites::Sprites;

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

/// The sprite an enemy draws, if it has one: the Mote is a plain dot, the
/// Plough is part of the Hauler's sprite.
pub fn sprite_name(kind: Kind) -> Option<&'static str> {
    Some(match kind {
        Kind::Skitter => "c3_skitter",
        Kind::Tick => "c3_tick",
        Kind::Kite => "c3_kite",
        Kind::Sentry => "c3_sentry",
        Kind::Burrower => "c3_burrower",
        Kind::Dropper => "c3_dropper",
        Kind::Hauler => "c3_hauler",
        Kind::Turret => "c3_turret",
        // The Strider's core is drawn by its cab.
        Kind::Strider => "c3_strider_cab",
        Kind::Mote | Kind::Plough => return None,
    })
}

/// How far the Strider's cab sprite (32x20) is moved up so that its core,
/// 4.5 px below the middle, sits at the enemy's centre.
const CAB_CORE_DROP: f32 = 4.5;

/// Draws an enemy with its sprite (frame 0; [`frame_for`] picks the frame
/// each tick). The sprites carry their own outlines and eyes.
pub fn body(entity: &mut EntityCommands, kind: Kind, sprites: &Sprites) {
    match kind {
        Kind::Mote => {
            entity.insert(Sprite::from_color(ICE, size(kind)));
        }
        Kind::Plough => {}
        Kind::Strider => {
            entity.insert((
                sprites.sprite("c3_strider_cab", CABINET),
                Anchor(Vec2::new(0.0, -CAB_CORE_DROP / 20.0)),
            ));
        }
        _ => {
            if let Some(name) = sprite_name(kind) {
                entity.insert(sprites.sprite(name, CABINET));
            }
        }
    }
}

/// Which frame an enemy shows and whether the sprite is mirrored.
/// `hero_x` is the Hero's world x, `pos_x` the enemy's.
///
/// The sprites face left (the Turret's barrel faces right and is turned
/// instead), so a mirror means "facing right".
pub fn frame_for(
    brain: &Brain,
    clock: f32,
    warning: bool,
    hero_x: f32,
    pos_x: f32,
    t: &EnemyTypes,
) -> (usize, bool) {
    let warn = usize::from(warning);
    match brain {
        Brain::Skitter(b) => ((clock * 8.0) as usize % 2, b.dir > 0.0),
        Brain::Tick(b) => (usize::from(!warning), b.dir > 0.0),
        Brain::Kite(b) => (warn, b.dir > 0.0),
        Brain::Sentry(b) => (
            match b.phase(t) {
                SentryPhase::Down => 0,
                SentryPhase::Rising => 1,
                SentryPhase::Up => 2,
            },
            hero_x > pos_x,
        ),
        Brain::Burrower(b) => (
            usize::from(b.risen() || matches!(b.phase, BurrowPhase::Down(_))),
            false,
        ),
        Brain::Dropper(_) | Brain::Turret(_) => (warn, false),
        Brain::Hauler(b) => (warn, b.dir_x() > 0.0),
        Brain::Strider(b) => (warn, b.dir > 0.0),
        Brain::Mote(_) => (0, false),
    }
}

/// The legs of a Strider (a hazard: they hurt but don't stop shots). The
/// cab is the Strider enemy's own sprite.
pub fn strider_frame(sprites: &Sprites) -> impl Bundle {
    (
        Visibility::default(),
        children![
            (
                sprites.sprite("c3_strider_leg", CABINET),
                Transform::from_xyz(-12.0, -24.0, 0.0),
                Leg,
            ),
            (
                sprites.sprite("c3_strider_leg", CABINET),
                Transform::from_xyz(12.0, -24.0, 0.0),
                Leg,
            ),
        ],
    )
}

/// A Strider's leg: raised while the stomp is telegraphed.
#[derive(Component)]
pub struct Leg;

/// The left-edge chevron warning of an enemy from behind: the up chevron
/// (the spawn turns it to point left).
pub fn chevron(sprites: &Sprites) -> Sprite {
    sprites.sprite("chevron_up", CABINET)
}

/// An enemy bullet: the shared `enemy_bullet`, or a Dropper's canister.
pub fn bullet(sprites: &Sprites, canister: bool) -> Sprite {
    sprites.sprite(
        if canister {
            "c3_canister"
        } else {
            "enemy_bullet"
        },
        CABINET,
    )
}

/// A hazard's plain body (a placeholder for a part with no sprite).
pub fn hazard(size: Vec2) -> impl Bundle {
    Sprite::from_color(IRON, size)
}

/// A crusher: the `c3_crusher` press; frame 1 is its warning light.
pub fn crusher(sprites: &Sprites) -> Sprite {
    sprites.sprite("c3_crusher", CABINET)
}

/// One tile of a hazard strip.
#[derive(Component)]
pub struct Tile;

/// A strip of an 8 px-wide tile sprite laid along `length`, `y` up from the
/// strip's middle; the hazard sets every tile's frame together.
pub fn strip(sprites: &Sprites, name: &str, length: f32, y: f32) -> impl Bundle {
    let width = sprites.frame_size(name).x.max(1) as f32;
    let count = (length / width).ceil().max(1.0) as usize;
    let left = -length / 2.0 + width / 2.0;
    let tiles: Vec<_> = (0..count)
        .map(|i| {
            (
                sprites.sprite(name, CABINET),
                Transform::from_xyz(left + i as f32 * width, y, 0.0),
                Tile,
            )
        })
        .collect();
    (
        Visibility::default(),
        Children::spawn(SpawnIter(tiles.into_iter())),
    )
}
