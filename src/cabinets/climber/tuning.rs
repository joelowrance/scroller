//! Cabinet 4's numbers, the `climber` section of `assets/tuning.ron`.
//! Every key is described in `docs/cabinets/4-climber.md`.
//!
//! Each struct defaults field by field, so the file can list only some keys.

use serde::Deserialize;

pub use crate::cabinets::side_shooter::tuning::vec2;
use crate::kit::platform::{JumpParams, LadderParams};

/// Declares a tuning struct whose `Default` is the listed values.
macro_rules! tuning_struct {
    ($(#[$meta:meta])* $name:ident { $($(#[$fmeta:meta])* $field:ident: $ty:ty = $value:expr),* $(,)? }) => {
        $(#[$meta])*
        #[derive(Clone, Debug, Deserialize)]
        #[serde(default)]
        pub struct $name {
            $($(#[$fmeta])* pub $field: $ty,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $value,)* }
            }
        }
    };
}

tuning_struct!(ClimberTuning {
    /// Multiplier on every enemy bullet speed in Stage 2.
    stage2_bullet_speed_scale: f32 = 1.15,
    /// Debug (needs `debug_keys`): the Hero follows the Stage's route by
    /// itself, firing up. `?autopilot` on the web URL sets it too.
    debug_autopilot: bool = false,
    /// Debug (needs `debug_keys`): play the enemy gallery instead of the
    /// Stage. `?gallery` on the web URL sets it too.
    enemy_gallery: bool = false,
    camera: CameraTuning = CameraTuning::default(),
    hero: HeroTuning = HeroTuning::default(),
    jump: JumpTuning = JumpTuning::default(),
    ladder: LadderTuning = LadderTuning::default(),
    shot: ShotTuning = ShotTuning::default(),
    power_up: PowerUpTuning = PowerUpTuning::default(),
    terrain: TerrainTuning = TerrainTuning::default(),
    enemy: EnemyRules = EnemyRules::default(),
    enemies: EnemyTypes = EnemyTypes::default(),
    hazards: HazardTypes = HazardTypes::default(),
    score: ScoreTuning = ScoreTuning::default(),
});

tuning_struct!(CameraTuning {
    face_half_width: f32 = 128.0,
});

tuning_struct!(HeroTuning {
    hitbox: (f32, f32) = (6.0, 14.0),
    crouch_hitbox: (f32, f32) = (6.0, 8.0),
    body: (f32, f32) = (10.0, 20.0),
    crouch_body: (f32, f32) = (10.0, 12.0),
    run_speed: f32 = 72.0,
    air_speed: f32 = 72.0,
    respawn_fall_speed: f32 = 60.0,
    fall_line: f32 = -120.0,
    exit_seconds: f32 = 1.0,
    respawn_invulnerable_seconds: f32 = 2.0,
    hit_stop_seconds: f32 = 0.15,
});

tuning_struct!(JumpTuning {
    speed: f32 = 240.0,
    gravity_up: f32 = 600.0,
    gravity_down: f32 = 900.0,
    cut: f32 = 0.4,
    max_fall: f32 = 300.0,
    coyote_seconds: f32 = 0.08,
    buffer_seconds: f32 = 0.1,
});

impl JumpTuning {
    pub fn params(&self) -> JumpParams {
        JumpParams {
            speed: self.speed,
            gravity_up: self.gravity_up,
            gravity_down: self.gravity_down,
            cut: self.cut,
            max_fall: self.max_fall,
            coyote_seconds: self.coyote_seconds,
            buffer_seconds: self.buffer_seconds,
        }
    }
}

tuning_struct!(LadderTuning {
    climb_speed: f32 = 48.0,
    grab_window: f32 = 6.0,
    jump_speed_fraction: f32 = 0.6,
});

impl LadderTuning {
    pub fn params(&self) -> LadderParams {
        LadderParams {
            climb_speed: self.climb_speed,
            grab_window: self.grab_window,
            jump_speed_fraction: self.jump_speed_fraction,
        }
    }
}

tuning_struct!(ShotTuning {
    fire_interval: f32 = 0.125,
    fire_interval_fast: f32 = 0.1,
    speed: f32 = 300.0,
    damage: u32 = 1,
    size: (f32, f32) = (6.0, 2.0),
    muzzle: (f32, f32) = (8.0, 14.0),
    crouch_muzzle: (f32, f32) = (8.0, 6.0),
    spread_degrees: f32 = 12.0,
    bolt_size: (f32, f32) = (10.0, 3.0),
    bolt_damage: u32 = 2,
    max_on_screen: usize = 64,
});

impl ShotTuning {
    /// The gun's keys, for the shared on-foot gun (`cabinets::on_foot`).
    pub fn gun(&self) -> crate::cabinets::on_foot::Gun {
        crate::cabinets::on_foot::Gun {
            fire_interval: self.fire_interval,
            fire_interval_fast: self.fire_interval_fast,
            damage: self.damage,
            size: vec2(self.size),
            spread_degrees: self.spread_degrees,
            bolt_size: vec2(self.bolt_size),
            bolt_damage: self.bolt_damage,
        }
    }
}

tuning_struct!(PowerUpTuning {
    size: (f32, f32) = (10.0, 10.0),
    bob_amplitude: f32 = 2.0,
    bob_period: f32 = 1.0,
    gun_max: u32 = 4,
    jet_speed_fraction: f32 = 0.7,
    canopy_fall_speed: f32 = 50.0,
    mag_crumble_factor: f32 = 2.0,
    ward_invulnerable_seconds: f32 = 1.0,
});

tuning_struct!(TerrainTuning {
    tile_size: f32 = 8.0,
    crumble_seconds: f32 = 0.5,
    crumble_respawn_seconds: f32 = 3.0,
    /// Stage-check constants (read by `stage_checks.rs` only).
    max_rise: f32 = 40.0,
    max_gap: f32 = 40.0,
    jump_setup_seconds: f32 = 0.3,
    drop_margin: f32 = 48.0,
    lookahead: f32 = 16.0,
    restart_clear_rise: f32 = 64.0,
});

tuning_struct!(
    /// Rules shared by every enemy.
    EnemyRules {
        bullet_size: (f32, f32) = (5.0, 5.0),
        bullet_hitbox: (f32, f32) = (2.0, 2.0),
        hitbox_inset: f32 = 1.0,
        fire_grace_seconds: f32 = 0.5,
        min_fire_distance: f32 = 40.0,
        max_bullets: usize = 32,
        no_spawn_radius: f32 = 48.0,
        bullet_lifetime: f32 = 10.0,
        below_warning_seconds: f32 = 0.75,
        chest_height: f32 = 14.0,
        ankle_height: f32 = 4.0,
    }
);

tuning_struct!(EnemyTypes {
    flit: Flit = Flit::default(),
    limpet: Limpet = Limpet::default(),
    scrap: Scrap = Scrap::default(),
    picket: Picket = Picket::default(),
    vent: Vent = Vent::default(),
    plumb: Plumb = Plumb::default(),
    pitcher: Pitcher = Pitcher::default(),
    mote: Mote = Mote::default(),
    winch: Winch = Winch::default(),
});

tuning_struct!(Flit {
    hp: u32 = 2,
    score: u64 = 100,
    formation_size: u32 = 3,
    spacing_seconds: f32 = 0.4,
    speed: f32 = 60.0,
    wave_amplitude: f32 = 12.0,
    wave_period: f32 = 1.2,
});

tuning_struct!(Limpet {
    hp: u32 = 2,
    score: u64 = 100,
    formation_size: u32 = 2,
    speed: f32 = 24.0,
});

tuning_struct!(Scrap {
    hp: u32 = 2,
    score: u64 = 50,
    warning_seconds: f32 = 0.75,
    speed: f32 = 120.0,
    fall_interval: f32 = 0.8,
    min_spacing: f32 = 24.0,
});

tuning_struct!(Picket {
    hp: u32 = 16,
    score: u64 = 500,
    speed: f32 = 20.0,
    level_window: f32 = 16.0,
    telegraph_seconds: f32 = 0.35,
    burst: u32 = 2,
    burst_interval: f32 = 0.25,
    shot_speed: f32 = 60.0,
    cooldown_seconds: f32 = 2.0,
});

tuning_struct!(Vent {
    hp: u32 = 12,
    score: u64 = 500,
    closed_seconds: f32 = 2.2,
    telegraph_seconds: f32 = 0.3,
    open_seconds: f32 = 1.4,
    fan_count: u32 = 3,
    fan_degrees: f32 = 30.0,
    shot_speed: f32 = 55.0,
});

tuning_struct!(Plumb {
    hp: u32 = 6,
    score: u64 = 300,
    drop_speed: f32 = 60.0,
    hang_seconds: f32 = 2.0,
    telegraph_seconds: f32 = 0.3,
    shot_speed: f32 = 60.0,
    reel_speed: f32 = 40.0,
});

tuning_struct!(Pitcher {
    hp: u32 = 20,
    score: u64 = 400,
    range: f32 = 120.0,
    fire_interval: f32 = 2.4,
    telegraph_seconds: f32 = 0.4,
    canister_size: (f32, f32) = (8.0, 8.0),
    launch_speed: f32 = 60.0,
    canister_gravity: f32 = 300.0,
    burst_speed: f32 = 50.0,
});

tuning_struct!(Mote {
    hp: u32 = 1,
    score: u64 = 50,
    formation_size: u32 = 8,
    cloud_radius: f32 = 32.0,
    speed: f32 = 55.0,
    turn_rate: f32 = 90.0,
    lifetime: f32 = 8.0,
});

tuning_struct!(Winch {
    hp: u32 = 120,
    score: u64 = 5000,
    rail_y: f32 = 80.0,
    speed: f32 = 40.0,
    telegraph_seconds: f32 = 0.6,
    drop_speed: f32 = 200.0,
    burst_speed: f32 = 60.0,
    rest_seconds: f32 = 1.2,
    reel_speed: f32 = 80.0,
});

tuning_struct!(HazardTypes {
    gust: Gust = Gust::default(),
    sweeper: Sweeper = Sweeper::default(),
});

tuning_struct!(Gust {
    blow_seconds: f32 = 2.0,
    push_speed: f32 = 36.0,
    warning_seconds: f32 = 0.8,
});

tuning_struct!(Sweeper {
    length: f32 = 48.0,
    turn_rate: f32 = 60.0,
});

tuning_struct!(ScoreTuning {
    formation_bonus_per_enemy: u64 = 100,
    maxed_power_up: u64 = 1000,
    standoff_time_bonus: u64 = 50,
    stage_clear: u64 = 3000,
    no_miss: u64 = 5000,
});

#[cfg(test)]
mod tests {
    use crate::tuning::Tuning;

    /// The committed file matches the defaults here, so neither drifts.
    #[test]
    fn tuning_file_matches_the_defaults() {
        let file = Tuning::parse(include_str!("../../../assets/tuning.ron")).unwrap();
        let ours = format!("{:?}", file.climber);
        let defaults = format!("{:?}", super::ClimberTuning::default());
        assert_eq!(ours, defaults);
    }

    /// The doc's proposed section parses to exactly these defaults too.
    #[test]
    fn the_design_doc_section_matches_the_defaults() {
        let doc = include_str!("../../../docs/cabinets/4-climber.md");
        let start = doc.find("climber: (").expect("tuning block in the doc");
        let end = start + doc[start..].find("\n```").expect("end of the block");
        let block = &doc[start + "climber: ".len()..end].trim_end();
        let parsed: super::ClimberTuning = ron::from_str(block).unwrap();
        assert_eq!(
            format!("{parsed:?}"),
            format!("{:?}", super::ClimberTuning::default())
        );
    }
}
