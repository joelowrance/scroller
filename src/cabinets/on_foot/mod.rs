//! What the on-foot Cabinets (3 and 4) share: the Hero's aim, the gun
//! levels, where shots leave from, and the Standoff clock. Each Cabinet
//! passes its own keys, so the two can be tuned apart.
//!
//! The platform physics they both run on is the kit's (`kit::platform`).

use std::f32::consts::FRAC_PI_4;

use bevy::math::Vec2;

pub mod standoff;

/// A gun's keys (`run_and_gun.shot` or `climber.shot`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gun {
    pub fire_interval: f32,
    pub fire_interval_fast: f32,
    pub damage: u32,
    pub size: Vec2,
    pub spread_degrees: f32,
    pub bolt_size: Vec2,
    pub bolt_damage: u32,
}

/// One shot of a volley, relative to the Hero's feet.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotSpec {
    pub offset: Vec2,
    pub direction: Vec2,
    pub size: Vec2,
    pub damage: u32,
    /// A bolt passes through the first enemy it hits.
    pub pierce: bool,
}

/// Where shots leave from, relative to the feet: chest height (`muzzle`), a
/// little in front, or knee height when crouching (`crouch_muzzle`).
pub fn muzzle(
    aim: Vec2,
    facing: f32,
    crouching: bool,
    muzzle: (f32, f32),
    crouch_muzzle: (f32, f32),
) -> Vec2 {
    if crouching {
        return Vec2::new(crouch_muzzle.0 * facing, crouch_muzzle.1);
    }
    let (x, y) = muzzle;
    Vec2::new(aim.x * x, y + aim.y * x)
}

/// The shots of one volley along `aim` (a unit vector) at gun level `gun`:
/// one shot at 1–2, a spread of three at 3, and a piercing bolt in the
/// middle at 4.
pub fn volley(gun: u32, aim: Vec2, from: Vec2, g: &Gun) -> Vec<ShotSpec> {
    let straight = |direction: Vec2| ShotSpec {
        offset: from,
        direction,
        size: g.size,
        damage: g.damage,
        pierce: false,
    };
    let mut shots = vec![if gun >= 4 {
        ShotSpec {
            size: g.bolt_size,
            damage: g.bolt_damage,
            pierce: true,
            ..straight(aim)
        }
    } else {
        straight(aim)
    }];
    if gun >= 3 {
        for sign in [1.0, -1.0] {
            shots.push(straight(Vec2::from_angle(
                aim.to_angle() + (sign * g.spread_degrees).to_radians(),
            )));
        }
    }
    shots
}

/// Seconds between volleys at gun level `gun`.
pub fn fire_interval(gun: u32, g: &Gun) -> f32 {
    if gun >= 2 {
        g.fire_interval_fast
    } else {
        g.fire_interval
    }
}

/// The aim for a held direction: up, up-diagonal, forward; down and
/// down-diagonal only in the air (down on the ground crouches).
pub fn aim_for(dir: Vec2, facing: f32, grounded: bool) -> Vec2 {
    let up = dir.y > 0.5;
    let down = dir.y < -0.5 && !grounded;
    let side = if dir.x.abs() > 0.5 {
        dir.x.signum()
    } else {
        0.0
    };
    match (up, down, side) {
        (true, _, 0.0) => Vec2::Y,
        (true, _, s) => Vec2::from_angle(if s > 0.0 { FRAC_PI_4 } else { 3.0 * FRAC_PI_4 }),
        (_, true, 0.0) => Vec2::NEG_Y,
        (_, true, s) => Vec2::from_angle(if s > 0.0 {
            -FRAC_PI_4
        } else {
            -3.0 * FRAC_PI_4
        }),
        _ => Vec2::new(facing, 0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn gun() -> Gun {
        Gun {
            fire_interval: 0.125,
            fire_interval_fast: 0.1,
            damage: 1,
            size: Vec2::new(6.0, 2.0),
            spread_degrees: 12.0,
            bolt_size: Vec2::new(10.0, 3.0),
            bolt_damage: 2,
        }
    }

    #[test]
    fn gun_levels_add_a_spread_then_a_bolt() {
        let g = gun();
        assert_eq!(volley(1, Vec2::Y, Vec2::ZERO, &g).len(), 1);
        assert_eq!(volley(2, Vec2::Y, Vec2::ZERO, &g).len(), 1);
        assert_eq!(fire_interval(1, &g), 0.125);
        assert_eq!(fire_interval(2, &g), 0.1);
        let g3 = volley(3, Vec2::Y, Vec2::ZERO, &g);
        let angles: Vec<_> = g3
            .iter()
            .map(|s| s.direction.to_angle().to_degrees().round())
            .collect();
        assert_eq!(angles, [90.0, 102.0, 78.0]);
        let g4 = volley(4, Vec2::Y, Vec2::ZERO, &g);
        assert!(g4[0].pierce && g4[0].damage == 2 && g4[0].size == Vec2::new(10.0, 3.0));
        assert!(g4[1..].iter().all(|s| !s.pierce && s.damage == 1));
    }

    #[test]
    fn the_muzzle_follows_the_aim() {
        assert_eq!(
            muzzle(Vec2::Y, 1.0, false, (8.0, 14.0), (8.0, 6.0)),
            Vec2::new(0.0, 22.0)
        );
        assert_eq!(
            muzzle(Vec2::X, -1.0, true, (8.0, 14.0), (8.0, 6.0)),
            Vec2::new(-8.0, 6.0)
        );
    }
}
