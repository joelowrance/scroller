//! The Wren's Power-ups: their state, the capsules that carry them, and
//! collecting them.
//!
//! The state lives in `Session.power_ups` as JSON, so it carries from Stage 1
//! to Stage 2 and survives a Save and Continue:
//! `{ "gun": 1, "drones": 0, "tail": false, "speed": 0, "ward": false, "bombs": 2 }`.
//!
//! A capsule appears when something asks the timeline for a spawn of kind
//! `power_up:<letter>` (a cleared Formation does this, or a Stage file
//! directly).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::palette;
use super::tuning::{SideShooterTuning, vec2};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::font::{PixelText, text_at};
use crate::kit::{DespawnOffscreen, Hitbox, Layer, Pickup, PickupCollected};
use crate::tuning::Tuning;

/// Spawn and pickup kinds for capsules start with this.
pub const KIND_PREFIX: &str = "power_up:";

/// Top of the play area (the HUD uses the strip above it).
pub const PLAY_TOP: f32 = HALF_H - 12.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Gun,
    Drone,
    Tail,
    Speed,
    Ward,
    Bomb,
}

impl PowerUp {
    pub const ALL: [PowerUp; 6] = [
        PowerUp::Gun,
        PowerUp::Drone,
        PowerUp::Tail,
        PowerUp::Speed,
        PowerUp::Ward,
        PowerUp::Bomb,
    ];

    /// The pickup chirp: the same rise, a different top note per type.
    pub fn sfx(self) -> Sfx {
        match self {
            PowerUp::Gun => Sfx::PowerUpGun,
            PowerUp::Drone => Sfx::PowerUpDrone,
            PowerUp::Tail => Sfx::PowerUpTail,
            PowerUp::Speed => Sfx::PowerUpSpeed,
            PowerUp::Ward => Sfx::PowerUpWard,
            PowerUp::Bomb => Sfx::PowerUpBomb,
        }
    }

    pub fn letter(self) -> char {
        match self {
            PowerUp::Gun => 'G',
            PowerUp::Drone => 'D',
            PowerUp::Tail => 'T',
            PowerUp::Speed => 'S',
            PowerUp::Ward => 'W',
            PowerUp::Bomb => 'B',
        }
    }

    /// Reads a spawn or pickup kind such as `power_up:G`.
    pub fn from_kind(kind: &str) -> Option<Self> {
        let mut letters = kind.strip_prefix(KIND_PREFIX)?.chars();
        let letter = letters.next()?.to_ascii_uppercase();
        if letters.next().is_some() {
            return None;
        }
        Self::ALL.into_iter().find(|p| p.letter() == letter)
    }

    pub fn kind(self) -> String {
        format!("{KIND_PREFIX}{}", self.letter())
    }
}

/// What a collected Power-up did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collected {
    Upgraded,
    /// It was already at its maximum: a medal instead.
    Medal,
}

/// Everything the Wren has picked up this Cabinet.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerUps {
    /// Gun level, 1 to `gun_max`.
    pub gun: u32,
    pub drones: u32,
    pub tail: bool,
    /// Speed steps.
    pub speed: u32,
    pub ward: bool,
    pub bombs: u32,
}

impl Default for PowerUps {
    fn default() -> Self {
        Self::fresh(&SideShooterTuning::default())
    }
}

impl PowerUps {
    /// The state at the start of the Cabinet (and of Practice).
    pub fn fresh(t: &SideShooterTuning) -> Self {
        Self {
            gun: 1,
            drones: 0,
            tail: false,
            speed: 0,
            ward: false,
            bombs: t.bomb.starting,
        }
    }

    /// Reads `Session.power_ups`. Null (a new Cabinet) or anything unreadable
    /// gives the fresh state.
    pub fn from_session(value: &serde_json::Value, t: &SideShooterTuning) -> Self {
        if value.is_null() {
            return Self::fresh(t);
        }
        serde_json::from_value(value.clone()).unwrap_or_else(|_| Self::fresh(t))
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).expect("PowerUps always serializes")
    }

    pub fn collect(&mut self, power_up: PowerUp, t: &SideShooterTuning) -> Collected {
        let p = &t.power_up;
        let raised = match power_up {
            PowerUp::Gun => raise(&mut self.gun, p.gun_max),
            PowerUp::Drone => raise(&mut self.drones, p.drone_max),
            PowerUp::Speed => raise(&mut self.speed, p.speed_max),
            PowerUp::Bomb => raise(&mut self.bombs, t.bomb.max),
            PowerUp::Tail => !std::mem::replace(&mut self.tail, true),
            PowerUp::Ward => !std::mem::replace(&mut self.ward, true),
        };
        if raised {
            Collected::Upgraded
        } else {
            Collected::Medal
        }
    }

    /// One step of power lost with a life: gun down one, the Ward and the
    /// lower drone gone, bombs topped back up to the starting count.
    pub fn lose_life(&mut self, t: &SideShooterTuning) {
        self.gun = self.gun.saturating_sub(1).max(1);
        self.ward = false;
        self.drones = self.drones.saturating_sub(1);
        self.bombs = self.bombs.max(t.bomb.starting);
    }

    pub fn hero_speed(&self, t: &SideShooterTuning) -> f32 {
        t.hero.speed + t.hero.speed_step * self.speed as f32
    }
}

fn raise(value: &mut u32, max: u32) -> bool {
    if *value < max {
        *value += 1;
        true
    } else {
        false
    }
}

/// A capsule drifting left and bobbing.
#[derive(Component)]
pub(super) struct Capsule {
    base_y: f32,
    age: f32,
}

/// Where a capsule appears: at least `edge_inset` inside the play area, so
/// one dropped at an edge can still be collected.
pub fn capsule_position(at: Vec2, t: &SideShooterTuning) -> Vec2 {
    let inset = t.power_up.edge_inset;
    Vec2::new(
        at.x.clamp(-HALF_W + inset, HALF_W - inset),
        at.y.clamp(-HALF_H + inset, PLAY_TOP - inset),
    )
}

/// The capsule's look: a Spark rim and a Foam letter on Night.
/// Swap this for the sprite from #4 when it lands.
fn capsule_visual(size: Vec2, letter: char) -> impl Bundle {
    (
        Sprite::from_color(palette::SPARK, size),
        children![
            (
                Sprite::from_color(palette::NIGHT, size - 2.0),
                Transform::from_xyz(0.0, 0.0, 0.1),
            ),
            text_at(
                PixelText::new(letter.to_string(), palette::FOAM),
                0.0,
                0.0,
                1
            ),
        ],
    )
}

pub(super) fn spawn_capsule(
    commands: &mut Commands,
    power_up: PowerUp,
    at: Vec2,
    t: &SideShooterTuning,
) {
    let at = capsule_position(at, t);
    let size = vec2(t.power_up.size);
    // The kit tests the capsule's box against the Hero's small hitbox. A box
    // this size overlaps it exactly when the capsule overlaps the generous
    // pickup box around the Wren.
    let hitbox = size + vec2(t.power_up.pickup_box) - vec2(t.hero.hitbox);
    commands.spawn((
        capsule_visual(size, power_up.letter()),
        Transform::from_translation(at.extend(6.0)),
        Pickup {
            kind: power_up.kind(),
        },
        Layer::Pickup,
        Hitbox::new(hitbox.x, hitbox.y),
        Capsule {
            base_y: at.y,
            age: 0.0,
        },
        DespawnOffscreen::new(size.x),
        DespawnOnExit(Screen::Playing),
    ));
}

pub(super) fn move_capsules(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut capsules: Query<(&mut Capsule, &mut Transform)>,
) {
    let p = &tuning.side_shooter.power_up;
    let dt = time.delta_secs();
    for (mut capsule, mut transform) in &mut capsules {
        capsule.age += dt;
        transform.translation.x -= p.drift_speed * dt;
        let phase = capsule.age / p.bob_period.max(0.01) * std::f32::consts::TAU;
        transform.translation.y = (capsule.base_y + p.bob_amplitude * phase.sin()).round();
    }
}

pub(super) fn collect_capsules(
    tuning: Res<Tuning>,
    mut collected: MessageReader<PickupCollected>,
    mut power_ups: ResMut<PowerUps>,
    mut score: MessageWriter<ScoreGained>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.side_shooter;
    for pickup in collected.read() {
        let Some(power_up) = PowerUp::from_kind(&pickup.kind) else {
            continue;
        };
        if power_ups.collect(power_up, t) == Collected::Medal {
            score.write(ScoreGained(t.score.maxed_power_up));
            sfx.write(PlaySfx(Sfx::Medal));
        } else {
            sfx.write(PlaySfx(power_up.sfx()));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::Session;

    fn t() -> SideShooterTuning {
        SideShooterTuning::default()
    }

    #[test]
    fn kinds_round_trip() {
        for p in PowerUp::ALL {
            assert_eq!(PowerUp::from_kind(&p.kind()), Some(p));
        }
        assert_eq!(PowerUp::from_kind("power_up:g"), Some(PowerUp::Gun));
        assert_eq!(PowerUp::from_kind("power_up:X"), None);
        assert_eq!(PowerUp::from_kind("power_up:GG"), None);
        assert_eq!(PowerUp::from_kind("skiff"), None);
    }

    #[test]
    fn fresh_state_matches_the_design_doc() {
        let json = PowerUps::fresh(&t()).to_json();
        assert_eq!(
            json,
            serde_json::json!({"gun": 1, "drones": 0, "tail": false, "speed": 0, "ward": false, "bombs": 2})
        );
    }

    #[test]
    fn each_power_up_rises_to_its_max_then_gives_medals() {
        let t = t();
        let cases = [
            (PowerUp::Gun, 3),
            (PowerUp::Drone, 2),
            (PowerUp::Tail, 1),
            (PowerUp::Speed, 2),
            (PowerUp::Ward, 1),
            (PowerUp::Bomb, 2),
        ];
        for (power_up, upgrades) in cases {
            let mut p = PowerUps::fresh(&t);
            for _ in 0..upgrades {
                assert_eq!(p.collect(power_up, &t), Collected::Upgraded, "{power_up:?}");
            }
            assert_eq!(p.collect(power_up, &t), Collected::Medal, "{power_up:?}");
        }
        let mut p = PowerUps::fresh(&t);
        for power_up in PowerUp::ALL {
            for _ in 0..5 {
                p.collect(power_up, &t);
            }
        }
        assert_eq!(
            p,
            PowerUps {
                gun: 4,
                drones: 2,
                tail: true,
                speed: 2,
                ward: true,
                bombs: 4
            }
        );
        assert_eq!(p.hero_speed(&t), 140.0);
    }

    #[test]
    fn losing_a_life_costs_one_step() {
        let t = t();
        let mut p = PowerUps {
            gun: 4,
            drones: 2,
            tail: true,
            speed: 2,
            ward: true,
            bombs: 0,
        };
        p.lose_life(&t);
        assert_eq!(
            p,
            PowerUps {
                gun: 3,
                drones: 1,
                tail: true,
                speed: 2,
                ward: false,
                bombs: 2
            }
        );
        let mut low = PowerUps::fresh(&t);
        low.bombs = 4;
        low.lose_life(&t);
        assert_eq!(low.gun, 1, "never below gun 1");
        assert_eq!(low.drones, 0);
        assert_eq!(low.bombs, 4, "bombs above the start are kept");
    }

    #[test]
    fn power_ups_survive_a_save_and_continue() {
        let t = t();
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        let mut p = PowerUps::from_session(&session.power_ups, &t);
        assert_eq!(p, PowerUps::fresh(&t));
        p.collect(PowerUp::Gun, &t);
        p.collect(PowerUp::Tail, &t);
        p.collect(PowerUp::Bomb, &t);
        session.power_ups = p.to_json();

        // Stage 1 cleared: the Save is written, then loaded by Continue.
        session.clear_stage();
        let saved = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&saved).unwrap();
        assert_eq!(PowerUps::from_session(&loaded.power_ups, &t), p);

        // Cleared at the Cabinet Transition.
        let mut session = loaded;
        session.clear_stage();
        assert_eq!(
            PowerUps::from_session(&session.power_ups, &t),
            PowerUps::fresh(&t)
        );
    }

    #[test]
    fn unreadable_or_partial_saves_fall_back_sensibly() {
        let t = t();
        let junk = serde_json::json!("nonsense");
        assert_eq!(PowerUps::from_session(&junk, &t), PowerUps::fresh(&t));
        let partial = serde_json::json!({"gun": 3});
        let p = PowerUps::from_session(&partial, &t);
        assert_eq!(p.gun, 3);
        assert_eq!(p.bombs, 2);
    }

    #[test]
    fn capsules_appear_inside_the_play_area() {
        let t = t();
        assert_eq!(capsule_position(Vec2::new(200.0, 0.0), &t).x, HALF_W - 16.0);
        assert_eq!(
            capsule_position(Vec2::new(0.0, 200.0), &t).y,
            PLAY_TOP - 16.0
        );
        assert_eq!(
            capsule_position(Vec2::new(0.0, -200.0), &t).y,
            -HALF_H + 16.0
        );
        assert_eq!(
            capsule_position(Vec2::new(10.0, 20.0), &t),
            Vec2::new(10.0, 20.0)
        );
    }
}
