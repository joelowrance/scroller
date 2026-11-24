//! The Hero's Power-ups in Cabinet 3: their state, the capsules that carry
//! them, and collecting them.
//!
//! The state lives in `Session.power_ups` as JSON, as in Cabinets 1 and 2:
//! `{ "gun": 1, "flare": false, "jet": false, "ward": false, "hustle": false }`.
//!
//! A capsule appears for a spawn of kind `power_up:<letter>`. It falls to the
//! ground and rests there, bobbing, and scrolls away with the ground.

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use super::WorldX;
use super::palette;
use super::terrain::{Terrain, screen_x, world_x};
use super::tuning::{RunAndGunTuning, vec2};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::cabinets::side_shooter::power_ups::{KIND_PREFIX, PLAY_TOP};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::font::{PixelText, text_at};
use crate::kit::{DespawnOffscreen, Hitbox, Layer, Pickup, PickupCollected, StageRun};
use crate::tuning::Tuning;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUp {
    Gun,
    Flare,
    Jet,
    Ward,
    Hustle,
}

impl PowerUp {
    pub const ALL: [PowerUp; 5] = [
        PowerUp::Gun,
        PowerUp::Flare,
        PowerUp::Jet,
        PowerUp::Ward,
        PowerUp::Hustle,
    ];

    pub fn letter(self) -> char {
        match self {
            PowerUp::Gun => 'G',
            PowerUp::Flare => 'F',
            PowerUp::Jet => 'J',
            PowerUp::Ward => 'W',
            PowerUp::Hustle => 'H',
        }
    }

    pub fn from_letter(letter: char) -> Option<Self> {
        let letter = letter.to_ascii_uppercase();
        Self::ALL.into_iter().find(|p| p.letter() == letter)
    }

    /// Reads a spawn or pickup kind such as `power_up:F`.
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

    /// The chirp when it is picked up. Cabinet 3's own chirps come with its
    /// sound (#20); until then the nearest ship chirp stands in.
    pub fn sfx(self) -> Sfx {
        match self {
            PowerUp::Gun => Sfx::PowerUpGun,
            PowerUp::Flare => Sfx::PowerUpBomb,
            PowerUp::Jet => Sfx::PowerUpTail,
            PowerUp::Ward => Sfx::PowerUpWard,
            PowerUp::Hustle => Sfx::PowerUpSpeed,
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
    pub flare: bool,
    pub jet: bool,
    pub ward: bool,
    pub hustle: bool,
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
            flare: false,
            jet: false,
            ward: false,
            hustle: false,
        }
    }

    /// Reads `Session.power_ups`. Null (a new Cabinet) or anything unreadable
    /// gives the fresh state; what is read is clamped to the limits.
    pub fn from_session(value: &serde_json::Value, t: &RunAndGunTuning) -> Self {
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

    pub fn collect(&mut self, power_up: PowerUp, t: &RunAndGunTuning) -> Collected {
        let raised = match power_up {
            PowerUp::Gun if self.gun < t.power_up.gun_max => {
                self.gun += 1;
                true
            }
            PowerUp::Gun => false,
            PowerUp::Flare => !std::mem::replace(&mut self.flare, true),
            PowerUp::Jet => !std::mem::replace(&mut self.jet, true),
            PowerUp::Ward => !std::mem::replace(&mut self.ward, true),
            PowerUp::Hustle => !std::mem::replace(&mut self.hustle, true),
        };
        if raised {
            Collected::Upgraded
        } else {
            Collected::Medal
        }
    }

    /// One step of power lost with a life: gun down one and the Ward gone.
    /// Flare, Jet and Hustle are kept.
    pub fn lose_life(&mut self) {
        self.gun = self.gun.saturating_sub(1).max(1);
        self.ward = false;
    }

    pub fn run_speed(&self, t: &RunAndGunTuning) -> f32 {
        t.hero.run_speed
            + if self.hustle {
                t.power_up.hustle_step
            } else {
                0.0
            }
    }

    pub fn air_speed(&self, t: &RunAndGunTuning) -> f32 {
        t.hero.air_speed
            + if self.hustle {
                t.power_up.hustle_step
            } else {
                0.0
            }
    }
}

/// A capsule falling to the ground, then resting and bobbing.
#[derive(Component)]
pub(super) struct Capsule {
    /// Where it rests: the ground (or, over a pit, the ground's height).
    rest_y: Option<f32>,
    vy: f32,
    age: f32,
}

fn capsule_visual(size: Vec2, letter: char) -> impl Bundle {
    (
        Sprite::from_color(palette::SPROUT, size),
        children![
            (
                Sprite::from_color(palette::UMBER, size - 2.0),
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

/// Spawns a capsule at screen position `at` (kept inside the screen).
pub(super) fn spawn_capsule(
    commands: &mut Commands,
    power_up: PowerUp,
    at: Vec2,
    camera: f32,
    t: &RunAndGunTuning,
) {
    let size = vec2(t.power_up.size);
    let x = at.x.clamp(-HALF_W + 16.0, HALF_W - 16.0);
    let y = at.y.clamp(-HALF_H + 16.0, PLAY_TOP - 16.0);
    // Collected by the Hero's body box rather than its small hitbox.
    let hitbox = size + vec2(t.hero.body) - vec2(t.hero.hitbox);
    commands.spawn((
        capsule_visual(size, power_up.letter()),
        Transform::from_xyz(x, y, 6.0),
        WorldX(world_x(x, camera)),
        Pickup {
            kind: power_up.kind(),
        },
        Layer::Pickup,
        Hitbox::new(hitbox.x, hitbox.y),
        Capsule {
            rest_y: None,
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
    terrain: Res<Terrain>,
    stage: Res<StageRun>,
    mut capsules: Query<(&mut Capsule, &WorldX, &mut Transform)>,
) {
    let t = &tuning.run_and_gun;
    let dt = time.delta_secs();
    let half = t.power_up.size.1 / 2.0;
    for (mut capsule, wx, mut transform) in &mut capsules {
        capsule.age += dt;
        transform.translation.x = screen_x(wx.0, stage.cursor.position).round();
        if capsule.rest_y.is_none() {
            // Over a pit, it hovers at the height of the nearest ground.
            let floor = terrain
                .ground_at(wx.0)
                .or_else(|| terrain.next_ground(wx.0, -1.0).map(|g| g.1))
                .unwrap_or(-HALF_H)
                + half;
            capsule.vy = (capsule.vy - t.jump.gravity_down * dt).max(-t.jump.max_fall);
            transform.translation.y += capsule.vy * dt;
            if transform.translation.y <= floor {
                transform.translation.y = floor;
                capsule.rest_y = Some(floor);
            }
            continue;
        }
        let phase = capsule.age / t.power_up.bob_period.max(0.01) * std::f32::consts::TAU;
        transform.translation.y =
            (capsule.rest_y.unwrap_or(0.0) + t.power_up.bob_amplitude * phase.sin()).round();
    }
}

pub(super) fn collect_capsules(
    tuning: Res<Tuning>,
    mut collected: MessageReader<PickupCollected>,
    mut power_ups: ResMut<PowerUps>,
    mut score: MessageWriter<ScoreGained>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.run_and_gun;
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

    fn t() -> RunAndGunTuning {
        RunAndGunTuning::default()
    }

    #[test]
    fn kinds_round_trip_and_other_cabinets_letters_are_not_ours() {
        for p in PowerUp::ALL {
            assert_eq!(PowerUp::from_kind(&p.kind()), Some(p));
        }
        for other in ["power_up:P", "power_up:B", "power_up:S", "power_up:D"] {
            assert_eq!(PowerUp::from_kind(other), None, "{other}");
        }
    }

    #[test]
    fn fresh_state_matches_the_design_doc() {
        assert_eq!(
            PowerUps::fresh().to_json(),
            serde_json::json!({"gun": 1, "flare": false, "jet": false, "ward": false, "hustle": false})
        );
    }

    #[test]
    fn each_power_up_rises_to_its_max_then_gives_medals() {
        let t = t();
        let cases = [
            (PowerUp::Gun, 3),
            (PowerUp::Flare, 1),
            (PowerUp::Jet, 1),
            (PowerUp::Ward, 1),
            (PowerUp::Hustle, 1),
        ];
        for (power_up, upgrades) in cases {
            let mut p = PowerUps::fresh();
            for _ in 0..upgrades {
                assert_eq!(p.collect(power_up, &t), Collected::Upgraded, "{power_up:?}");
            }
            assert_eq!(p.collect(power_up, &t), Collected::Medal, "{power_up:?}");
        }
        let mut p = PowerUps::fresh();
        p.collect(PowerUp::Hustle, &t);
        assert_eq!(p.run_speed(&t), 84.0);
    }

    #[test]
    fn losing_a_life_costs_a_gun_level_and_the_ward_only() {
        let mut p = PowerUps {
            gun: 4,
            flare: true,
            jet: true,
            ward: true,
            hustle: true,
        };
        p.lose_life();
        assert_eq!(
            p,
            PowerUps {
                gun: 3,
                flare: true,
                jet: true,
                ward: false,
                hustle: true
            }
        );
        let mut low = PowerUps::fresh();
        low.lose_life();
        assert_eq!(low.gun, 1);
    }

    #[test]
    fn power_ups_survive_a_save_and_continue_but_not_the_transition() {
        let t = t();
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        session.cabinet = 2;
        let mut p = PowerUps::from_session(&session.power_ups, &t);
        p.collect(PowerUp::Flare, &t);
        p.collect(PowerUp::Gun, &t);
        session.power_ups = p.to_json();
        session.clear_stage();
        let saved = serde_json::to_string(&session).unwrap();
        let loaded: Session = serde_json::from_str(&saved).unwrap();
        assert_eq!(PowerUps::from_session(&loaded.power_ups, &t), p);
        let mut session = loaded;
        session.clear_stage();
        assert_eq!(
            PowerUps::from_session(&session.power_ups, &t),
            PowerUps::fresh()
        );
    }

    #[test]
    fn junk_or_wild_saves_fall_back_sensibly() {
        let t = t();
        assert_eq!(
            PowerUps::from_session(&serde_json::json!("nonsense"), &t),
            PowerUps::fresh()
        );
        let wild = PowerUps::from_session(&serde_json::json!({"gun": 9, "jet": true}), &t);
        assert_eq!((wild.gun, wild.jet), (4, true));
    }
}
