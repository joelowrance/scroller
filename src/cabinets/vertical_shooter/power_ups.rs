//! The Wren's Power-ups in Cabinet 2: their state, the capsules that carry
//! them, and collecting them.
//!
//! The state lives in `Session.power_ups` as JSON, as in Cabinet 1:
//! `{ "gun": 1, "pods": 0, "rockets": false, "speed": 0, "ward": false, "bombs": 2 }`.
//! It is cleared at the Cabinet Transition, so Cabinet 1's state never
//! reaches this Cabinet.
//!
//! A capsule appears for a spawn of kind `power_up:<letter>` (a cleared
//! Formation asks for one, or a Stage file does directly).

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::palette;
use super::tuning::{VerticalShooterTuning, vec2};
use crate::cabinets::ScoreGained;
use crate::cabinets::side_shooter::power_ups::{KIND_PREFIX, PLAY_TOP};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::font::{PixelText, text_at};
use crate::kit::{DespawnOffscreen, Hitbox, Layer, Pickup, PickupCollected};
use crate::tuning::Tuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Gun,
    Pod,
    Rockets,
    Speed,
    Ward,
    Bomb,
}

impl PowerUp {
    pub const ALL: [PowerUp; 6] = [
        PowerUp::Gun,
        PowerUp::Pod,
        PowerUp::Rockets,
        PowerUp::Speed,
        PowerUp::Ward,
        PowerUp::Bomb,
    ];

    pub fn letter(self) -> char {
        match self {
            PowerUp::Gun => 'G',
            PowerUp::Pod => 'P',
            PowerUp::Rockets => 'R',
            PowerUp::Speed => 'S',
            PowerUp::Ward => 'W',
            PowerUp::Bomb => 'B',
        }
    }

    pub fn from_letter(letter: char) -> Option<Self> {
        let letter = letter.to_ascii_uppercase();
        Self::ALL.into_iter().find(|p| p.letter() == letter)
    }

    /// Reads a spawn or pickup kind such as `power_up:R`.
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
    pub pods: u32,
    pub rockets: bool,
    /// Speed steps.
    pub speed: u32,
    pub ward: bool,
    pub bombs: u32,
}

impl Default for PowerUps {
    fn default() -> Self {
        Self::fresh(&VerticalShooterTuning::default())
    }
}

impl PowerUps {
    /// The state at the start of the Cabinet (and of Practice).
    pub fn fresh(t: &VerticalShooterTuning) -> Self {
        Self {
            gun: 1,
            pods: 0,
            rockets: false,
            speed: 0,
            ward: false,
            bombs: t.bomb.starting,
        }
    }

    /// Reads `Session.power_ups`. Null (a new Cabinet) or anything unreadable
    /// gives the fresh state.
    pub fn from_session(value: &serde_json::Value, t: &VerticalShooterTuning) -> Self {
        if value.is_null() {
            return Self::fresh(t);
        }
        serde_json::from_value(value.clone()).unwrap_or_else(|_| Self::fresh(t))
    }

    pub fn to_json(self) -> serde_json::Value {
        serde_json::to_value(self).expect("PowerUps always serializes")
    }

    pub fn collect(&mut self, power_up: PowerUp, t: &VerticalShooterTuning) -> Collected {
        let p = &t.power_up;
        let raised = match power_up {
            PowerUp::Gun => raise(&mut self.gun, p.gun_max),
            PowerUp::Pod => raise(&mut self.pods, p.pod_max),
            PowerUp::Speed => raise(&mut self.speed, p.speed_max),
            PowerUp::Bomb => raise(&mut self.bombs, t.bomb.max),
            PowerUp::Rockets => !std::mem::replace(&mut self.rockets, true),
            PowerUp::Ward => !std::mem::replace(&mut self.ward, true),
        };
        if raised {
            Collected::Upgraded
        } else {
            Collected::Medal
        }
    }

    /// One step of power lost with a life: gun down one, the Ward and the
    /// right-hand Pod gone, bombs topped back up to the starting count.
    pub fn lose_life(&mut self, t: &VerticalShooterTuning) {
        self.gun = self.gun.saturating_sub(1).max(1);
        self.ward = false;
        self.pods = self.pods.saturating_sub(1);
        self.bombs = self.bombs.max(t.bomb.starting);
    }

    pub fn hero_speed(&self, t: &VerticalShooterTuning) -> f32 {
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

/// A capsule drifting down and bobbing sideways.
#[derive(Component)]
pub(super) struct Capsule {
    base_x: f32,
    age: f32,
}

/// Where a capsule appears: at least `edge_inset` inside the play area.
pub fn capsule_position(at: Vec2, t: &VerticalShooterTuning) -> Vec2 {
    let inset = t.power_up.edge_inset;
    Vec2::new(
        at.x.clamp(-HALF_W + inset, HALF_W - inset),
        at.y.clamp(-HALF_H + inset, PLAY_TOP - inset),
    )
}

/// The capsule's look: an Orchid rim and a Foam letter on Dusk.
fn capsule_visual(size: Vec2, letter: char) -> impl Bundle {
    (
        Sprite::from_color(palette::ORCHID, size),
        children![
            (
                Sprite::from_color(palette::DUSK, size - 2.0),
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
    t: &VerticalShooterTuning,
) {
    let at = capsule_position(at, t);
    let size = vec2(t.power_up.size);
    // Overlapping the Hero's small hitbox with a box this size is the same
    // test as the capsule overlapping the pickup box around the Wren.
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
            base_x: at.x,
            age: 0.0,
        },
        DespawnOffscreen::new(size.y),
        DespawnOnExit(Screen::Playing),
    ));
}

pub(super) fn move_capsules(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut capsules: Query<(&mut Capsule, &mut Transform)>,
) {
    let p = &tuning.vertical_shooter.power_up;
    let dt = time.delta_secs();
    for (mut capsule, mut transform) in &mut capsules {
        capsule.age += dt;
        transform.translation.y -= p.drift_speed * dt;
        let phase = capsule.age / p.bob_period.max(0.01) * std::f32::consts::TAU;
        transform.translation.x = (capsule.base_x + p.bob_amplitude * phase.sin()).round();
    }
}

pub(super) fn collect_capsules(
    tuning: Res<Tuning>,
    mut collected: MessageReader<PickupCollected>,
    mut power_ups: ResMut<PowerUps>,
    mut score: MessageWriter<ScoreGained>,
) {
    let t = &tuning.vertical_shooter;
    for pickup in collected.read() {
        let Some(power_up) = PowerUp::from_kind(&pickup.kind) else {
            continue;
        };
        if power_ups.collect(power_up, t) == Collected::Medal {
            score.write(ScoreGained(t.score.maxed_power_up));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::Session;

    fn t() -> VerticalShooterTuning {
        VerticalShooterTuning::default()
    }

    #[test]
    fn kinds_round_trip() {
        for p in PowerUp::ALL {
            assert_eq!(PowerUp::from_kind(&p.kind()), Some(p));
        }
        assert_eq!(PowerUp::from_kind("power_up:r"), Some(PowerUp::Rockets));
        assert_eq!(PowerUp::from_kind("power_up:D"), None, "Cabinet 1's Drone");
        assert_eq!(PowerUp::from_kind("power_up:T"), None, "Cabinet 1's Tail");
        assert_eq!(PowerUp::from_kind("wisp"), None);
    }

    #[test]
    fn fresh_state_matches_the_design_doc() {
        assert_eq!(
            PowerUps::fresh(&t()).to_json(),
            serde_json::json!({"gun": 1, "pods": 0, "rockets": false, "speed": 0, "ward": false, "bombs": 2})
        );
    }

    #[test]
    fn each_power_up_rises_to_its_max_then_gives_medals() {
        let t = t();
        let cases = [
            (PowerUp::Gun, 3),
            (PowerUp::Pod, 2),
            (PowerUp::Rockets, 1),
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
        p.collect(PowerUp::Speed, &t);
        p.collect(PowerUp::Speed, &t);
        assert_eq!(p.hero_speed(&t), 140.0);
    }

    #[test]
    fn losing_a_life_costs_one_step() {
        let t = t();
        let mut p = PowerUps {
            gun: 4,
            pods: 2,
            rockets: true,
            speed: 2,
            ward: true,
            bombs: 0,
        };
        p.lose_life(&t);
        assert_eq!(
            p,
            PowerUps {
                gun: 3,
                pods: 1,
                rockets: true,
                speed: 2,
                ward: false,
                bombs: 2
            }
        );
        let mut low = PowerUps::fresh(&t);
        low.lose_life(&t);
        assert_eq!((low.gun, low.pods), (1, 0), "never below the start");
    }

    #[test]
    fn power_ups_survive_a_save_and_continue_but_not_the_transition() {
        let t = t();
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        session.cabinet = 1;
        let mut p = PowerUps::from_session(&session.power_ups, &t);
        p.collect(PowerUp::Rockets, &t);
        p.collect(PowerUp::Pod, &t);
        session.power_ups = p.to_json();

        session.clear_stage();
        let saved = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&saved).unwrap();
        assert_eq!(PowerUps::from_session(&loaded.power_ups, &t), p);

        let mut session = loaded;
        session.clear_stage();
        assert_eq!(
            PowerUps::from_session(&session.power_ups, &t),
            PowerUps::fresh(&t)
        );
    }

    #[test]
    fn cabinet_one_state_or_junk_falls_back_sensibly() {
        let t = t();
        let junk = serde_json::json!("nonsense");
        assert_eq!(PowerUps::from_session(&junk, &t), PowerUps::fresh(&t));
        // Only the shared fields of a Cabinet 1 state would be read.
        let partial = serde_json::json!({"gun": 3, "drones": 2});
        let p = PowerUps::from_session(&partial, &t);
        assert_eq!((p.gun, p.pods, p.bombs), (3, 0, 2));
    }

    #[test]
    fn capsules_appear_inside_the_play_area() {
        let t = t();
        assert_eq!(capsule_position(Vec2::new(200.0, 0.0), &t).x, HALF_W - 16.0);
        assert_eq!(
            capsule_position(Vec2::new(0.0, 200.0), &t).y,
            PLAY_TOP - 16.0
        );
    }
}
