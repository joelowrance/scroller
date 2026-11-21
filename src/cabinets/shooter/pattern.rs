//! Small, pure helpers for enemy movement and bullet patterns. Angles are in
//! degrees unless a name says radians; 0° points right and 90° up.

use std::f32::consts::{PI, TAU};

use bevy::math::Vec2;

/// Unit vector at `degrees` (0 = right, 90 = up).
pub fn dir(degrees: f32) -> Vec2 {
    Vec2::from_angle(degrees.to_radians())
}

/// `count` directions spread evenly over `width` degrees, centred on `aim`.
pub fn fan(aim: Vec2, count: u32, width: f32) -> Vec<Vec2> {
    let centre = aim.to_angle().to_degrees();
    if count <= 1 {
        return vec![dir(centre)];
    }
    let step = width / (count - 1) as f32;
    (0..count)
        .map(|i| dir(centre - width / 2.0 + step * i as f32))
        .collect()
}

/// `count` directions evenly around a full circle, starting at `offset`.
pub fn ring(count: u32, offset: f32) -> Vec<Vec2> {
    let n = count.max(1);
    let step = 360.0 / n as f32;
    (0..n).map(|i| dir(offset + step * i as f32)).collect()
}

/// Rotates `heading` (radians) toward `target` by at most `max` radians.
pub fn turn_toward(heading: f32, target: f32, max: f32) -> f32 {
    let diff = (target - heading + PI).rem_euclid(TAU) - PI;
    heading + diff.clamp(-max, max)
}

/// A sine bob: `amplitude` px over `period` seconds, at time `t`.
pub fn bob(amplitude: f32, period: f32, t: f32) -> f32 {
    amplitude * (TAU * t / period.max(0.01)).sin()
}

/// One step of homing: turns `heading` (radians) toward `target` at up to
/// `turn_rate` °/s and moves `pos` along it at `speed`.
pub fn home(
    heading: &mut f32,
    pos: &mut Vec2,
    target: Option<Vec2>,
    speed: f32,
    turn_rate: f32,
    dt: f32,
) {
    if let Some(target) = target {
        let want = (target - *pos).to_angle();
        *heading = turn_toward(*heading, want, turn_rate.to_radians() * dt);
    }
    *pos += Vec2::from_angle(*heading) * speed * dt;
}

/// An on/off hazard cycle (an Arc Gate or a Tether line): off, a warning
/// flicker at the end of the off time, then live.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Pulse {
    pub on_seconds: f32,
    pub off_seconds: f32,
    pub telegraph_seconds: f32,
}

impl Pulse {
    fn phase(&self, age: f32) -> f32 {
        age.rem_euclid((self.on_seconds + self.off_seconds).max(0.01))
    }

    pub fn live(&self, age: f32) -> bool {
        self.phase(age) >= self.off_seconds
    }

    pub fn telegraph(&self, age: f32) -> bool {
        let p = self.phase(age);
        p < self.off_seconds && p >= self.off_seconds - self.telegraph_seconds
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_ring_is_evenly_spaced() {
        let r = ring(4, 0.0);
        assert_eq!(r.len(), 4);
        assert!((r[1] - Vec2::Y).length() < 1e-5);
        assert!((r[2] - Vec2::NEG_X).length() < 1e-5);
    }

    #[test]
    fn turning_takes_the_short_way_and_stops_at_the_target() {
        let h = turn_toward(0.0, 3.0 * PI / 2.0, 0.1);
        assert!((h + 0.1).abs() < 1e-5, "turns clockwise toward -90°");
        assert!((turn_toward(0.0, 0.05, 0.1) - 0.05).abs() < 1e-5);
    }

    #[test]
    fn a_pulse_warns_before_it_goes_live() {
        let p = Pulse {
            on_seconds: 1.2,
            off_seconds: 0.8,
            telegraph_seconds: 0.3,
        };
        assert!(!p.live(0.1) && !p.telegraph(0.1));
        assert!(p.telegraph(0.6) && !p.live(0.6));
        assert!(p.live(1.0) && !p.telegraph(1.0));
        assert!(!p.live(2.1), "the cycle repeats");
    }
}
