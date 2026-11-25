//! The Hero's Power-ups in Cabinet 4: their state, the capsules that carry
//! them, and collecting them (docs/cabinets/4-climber.md, "Power-ups").
//!
//! The state lives in `Session.power_ups` as JSON, as in Cabinets 1–3:
//! `{ "gun": 1, "ward": false, "jet": false, "canopy": false, "mag": false }`.
//!
//! A capsule appears for a spawn of kind `power_up:<letter>`. It falls onto
//! the ledge below, rests there bobbing, and scrolls away with the tower.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::palette;
use super::tower::{Tower, screen_y, tower_y};
use super::tuning::{ClimberTuning, vec2};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::cabinets::side_shooter::power_ups::{KIND_PREFIX, PLAY_TOP};
use crate::display::HALF_H;
use crate::flow::Screen;
use crate::font::{PixelText, text_at};
use crate::kit::{DespawnOffscreen, Hitbox, Layer, Pickup, PickupCollected, StageRun};
use crate::tuning::Tuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Gun,
    Ward,
    Jet,
    Canopy,
    Mag,
}

impl PowerUp {
    pub const ALL: [PowerUp; 5] = [
        PowerUp::Gun,
        PowerUp::Ward,
        PowerUp::Jet,
        PowerUp::Canopy,
        PowerUp::Mag,
    ];

    pub fn letter(self) -> char {
        match self {
            PowerUp::Gun => 'G',
            PowerUp::Ward => 'W',
            PowerUp::Jet => 'J',
            PowerUp::Canopy => 'C',
            PowerUp::Mag => 'M',
        }
    }

    pub fn from_letter(letter: char) -> Option<Self> {
        let letter = letter.to_ascii_uppercase();
        Self::ALL.into_iter().find(|p| p.letter() == letter)
    }

    /// Reads a spawn or pickup kind such as `power_up:C`.
    pub fn from_kind(kind: &str) -> Option<Self> {
        let mut letters = kind.strip_prefix(KIND_PREFIX)?.chars();
        let letter = letters.next()?;
        if letters.next().is_some() {
            return None;
        }
        Self::from_letter(letter)
    }

    pub fn kind(self) -> String {
        format!("{KIND_PREFIX}{}", self.letter())
    }

    /// The chirp when it is picked up (G, W and J share Cabinet 3's).
    pub fn sfx(self) -> Sfx {
        match self {
            PowerUp::Gun => Sfx::PowerUpGun,
            PowerUp::Ward => Sfx::PowerUpWard,
            PowerUp::Jet => Sfx::PowerUpJet,
            PowerUp::Canopy => Sfx::PowerUpCanopy,
            PowerUp::Mag => Sfx::PowerUpMag,
        }
    }
}

/// What a collected Power-up did.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Collected {
    Upgraded,
    /// It was already at its maximum: a medal instead.
    Medal,
}

/// Everything the Hero has picked up this Cabinet.
#[derive(Resource, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PowerUps {
    /// Gun level, 1 to `gun_max`.
    pub gun: u32,
    pub ward: bool,
    pub jet: bool,
    pub canopy: bool,
    pub mag: bool,
}

impl Default for PowerUps {
    fn default() -> Self {
        Self::fresh()
    }
}

impl PowerUps {
    /// The state at the start of the Cabinet (and of Practice).
    pub fn fresh() -> Self {
        Self {
            gun: 1,
            ward: false,
            jet: false,
            canopy: false,
            mag: false,
        }
    }

    /// Reads `Session.power_ups`. Null (a new Cabinet) or anything unreadable
    /// gives the fresh state; what is read is clamped to the limits.
    pub fn from_session(value: &serde_json::Value, t: &ClimberTuning) -> Self {
        if value.is_null() {
            return Self::fresh();
        }
        let read: Self = serde_json::from_value(value.clone()).unwrap_or_else(|_| Self::fresh());
        Self {
            gun: read.gun.clamp(1, t.power_up.gun_max),
            ..read
        }
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).expect("PowerUps always serializes")
    }

    pub fn collect(&mut self, power_up: PowerUp, t: &ClimberTuning) -> Collected {
        let raised = match power_up {
            PowerUp::Gun if self.gun < t.power_up.gun_max => {
                self.gun += 1;
                true
            }
            PowerUp::Gun => false,
            PowerUp::Ward => !std::mem::replace(&mut self.ward, true),
            PowerUp::Jet => !std::mem::replace(&mut self.jet, true),
            PowerUp::Canopy => !std::mem::replace(&mut self.canopy, true),
            PowerUp::Mag => !std::mem::replace(&mut self.mag, true),
        };
        if raised {
            Collected::Upgraded
        } else {
            Collected::Medal
        }
    }

    /// One step of power lost with a life: gun down one and the Ward gone.
    /// Jet, Canopy and Mag boots are kept.
    pub fn lose_life(&mut self) {
        self.gun = self.gun.saturating_sub(1).max(1);
        self.ward = false;
    }

    /// How much longer a crumbling ledge holds under the Hero.
    pub fn crumble_factor(&self, t: &ClimberTuning) -> f32 {
        if self.mag {
            t.power_up.mag_crumble_factor
        } else {
            1.0
        }
    }
}

/// Where a capsule is.
#[derive(Clone, Copy, PartialEq)]
enum Rest {
    Falling,
    /// On the ledge top at this height.
    On(f32),
    /// Over no ledge: at the bottom of the play area.
    Hovering,
}

/// A capsule falling onto a ledge, then resting and bobbing. Its x is as on
/// screen; its height is in tower space.
#[derive(Component)]
pub(super) struct Capsule {
    h: f32,
    rest: Rest,
    vy: f32,
    age: f32,
}

fn capsule_visual(size: Vec2, letter: char) -> impl Bundle {
    (
        Sprite::from_color(palette::MINT, size),
        children![
            (
                Sprite::from_color(palette::ABYSS, size - 2.0),
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

/// Spawns a capsule at screen position `at` (kept inside the face).
pub(super) fn spawn_capsule(
    commands: &mut Commands,
    power_up: PowerUp,
    at: Vec2,
    camera: f32,
    t: &ClimberTuning,
) {
    let size = vec2(t.power_up.size);
    let face = t.camera.face_half_width - size.x;
    let x = at.x.clamp(-face, face);
    let y = at.y.clamp(-HALF_H + 16.0, PLAY_TOP - 16.0);
    // Collected by the Hero's body box rather than its small hitbox.
    let hitbox = size + vec2(t.hero.body) - vec2(t.hero.hitbox);
    commands.spawn((
        capsule_visual(size, power_up.letter()),
        Transform::from_xyz(x, y, 6.0),
        Pickup {
            kind: power_up.kind(),
        },
        Layer::Pickup,
        Hitbox::new(hitbox.x, hitbox.y),
        Capsule {
            h: tower_y(y, camera),
            rest: Rest::Falling,
            vy: 0.0,
            age: 0.0,
        },
        DespawnOffscreen::new(size.x),
        DespawnOnExit(Screen::Playing),
    ));
}

pub(super) fn move_capsules(
    time: Res<Time>,
    tuning: Res<Tuning>,
    tower: Res<Tower>,
    stage: Res<StageRun>,
    mut capsules: Query<(&mut Capsule, &mut Transform)>,
) {
    let t = &tuning.climber;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    let half = t.power_up.size.1 / 2.0;
    // Over no ledge, it hovers at the bottom of the play area.
    let lowest = tower_y(-HALF_H, camera) + half;
    for (mut capsule, mut transform) in &mut capsules {
        capsule.age += dt;
        let x = transform.translation.x;
        match capsule.rest {
            Rest::On(top) => {
                let phase = capsule.age / t.power_up.bob_period.max(0.01) * std::f32::consts::TAU;
                capsule.h = top + half + t.power_up.bob_amplitude * phase.sin();
            }
            Rest::Hovering => capsule.h = capsule.h.max(lowest),
            Rest::Falling => {
                capsule.vy = (capsule.vy - t.jump.gravity_down * dt).max(-t.jump.max_fall);
                let floor = tower.floor_below(x, capsule.h - half);
                capsule.h += capsule.vy * dt;
                match floor {
                    Some(top) if capsule.h - half <= top => {
                        capsule.h = top + half;
                        capsule.rest = Rest::On(top);
                    }
                    None if capsule.h <= lowest => {
                        capsule.h = lowest;
                        capsule.rest = Rest::Hovering;
                    }
                    _ => {}
                }
            }
        }
        transform.translation.y = screen_y(capsule.h, camera).round();
    }
}

pub(super) fn collect_capsules(
    tuning: Res<Tuning>,
    mut collected: MessageReader<PickupCollected>,
    mut power_ups: ResMut<PowerUps>,
    mut score: MessageWriter<ScoreGained>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.climber;
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

    fn t() -> ClimberTuning {
        ClimberTuning::default()
    }

    #[test]
    fn kinds_round_trip() {
        for p in PowerUp::ALL {
            assert_eq!(PowerUp::from_kind(&p.kind()), Some(p));
        }
        for other in ["power_up:F", "power_up:H", "power_up:P", "power_up:GG"] {
            assert_eq!(PowerUp::from_kind(other), None, "{other}");
        }
    }

    #[test]
    fn fresh_state_matches_the_design_doc() {
        assert_eq!(
            PowerUps::fresh().to_json(),
            serde_json::json!({"gun": 1, "ward": false, "jet": false, "canopy": false, "mag": false})
        );
    }

    #[test]
    fn each_power_up_rises_to_its_max_then_gives_medals() {
        let t = t();
        let cases = [
            (PowerUp::Gun, 3),
            (PowerUp::Ward, 1),
            (PowerUp::Jet, 1),
            (PowerUp::Canopy, 1),
            (PowerUp::Mag, 1),
        ];
        for (power_up, upgrades) in cases {
            let mut p = PowerUps::fresh();
            for _ in 0..upgrades {
                assert_eq!(p.collect(power_up, &t), Collected::Upgraded, "{power_up:?}");
            }
            assert_eq!(p.collect(power_up, &t), Collected::Medal, "{power_up:?}");
        }
        let mut p = PowerUps::fresh();
        assert_eq!(p.crumble_factor(&t), 1.0);
        p.collect(PowerUp::Mag, &t);
        assert_eq!(p.crumble_factor(&t), 2.0);
    }

    #[test]
    fn losing_a_life_costs_a_gun_level_and_the_ward_only() {
        let mut p = PowerUps {
            gun: 4,
            ward: true,
            jet: true,
            canopy: true,
            mag: true,
        };
        p.lose_life();
        assert_eq!(
            p,
            PowerUps {
                gun: 3,
                ward: false,
                jet: true,
                canopy: true,
                mag: true,
            }
        );
        let mut low = PowerUps::fresh();
        low.lose_life();
        assert_eq!(low.gun, 1);
    }

    #[test]
    fn power_ups_carry_from_stage_1_to_stage_2_through_a_save() {
        let t = t();
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        session.cabinet = super::super::CABINET;
        let mut p = PowerUps::from_session(&session.power_ups, &t);
        assert_eq!(p, PowerUps::fresh(), "a new Cabinet starts fresh");
        p.collect(PowerUp::Canopy, &t);
        p.collect(PowerUp::Gun, &t);
        session.power_ups = p.to_json();
        session.clear_stage();
        let saved = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&saved).unwrap();
        assert_eq!(loaded.stage, 1);
        assert_eq!(PowerUps::from_session(&loaded.power_ups, &t), p);
    }

    #[test]
    fn junk_or_wild_saves_fall_back_sensibly() {
        let t = t();
        assert_eq!(
            PowerUps::from_session(&serde_json::json!("nonsense"), &t),
            PowerUps::fresh()
        );
        let wild = PowerUps::from_session(&serde_json::json!({"gun": 9, "mag": true}), &t);
        assert_eq!((wild.gun, wild.mag, wild.ward), (4, true, false));
        // Cabinet 3's shape (a Flare, a Hustle) reads as fresh extras.
        let c3 = PowerUps::from_session(
            &serde_json::json!({"gun": 2, "flare": true, "jet": true, "ward": false, "hustle": true}),
            &t,
        );
        assert_eq!((c3.gun, c3.jet, c3.canopy), (2, true, false));
    }
}
