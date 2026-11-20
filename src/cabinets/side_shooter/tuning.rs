//! Cabinet 1's numbers, the `side_shooter` section of `assets/tuning.ron`.
//! Every key is described in `docs/cabinets/1-side-shooter.md`.
//!
//! Each struct defaults field by field, so the file can list only some keys.

use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct SideShooterTuning {
    /// Multiplier on every enemy bullet speed in Stage 2.
    pub stage2_bullet_speed_scale: f32,
    /// Debug: play the enemy gallery instead of Stage 1 (needs `debug_keys`).
    /// `?gallery` on the web URL sets it too.
    pub enemy_gallery: bool,
    pub enemy: EnemyRules,
    pub enemies: EnemyTypes,
    pub score: ScoreTuning,
}

impl Default for SideShooterTuning {
    fn default() -> Self {
        Self {
            stage2_bullet_speed_scale: 1.15,
            enemy_gallery: false,
            enemy: EnemyRules::default(),
            enemies: EnemyTypes::default(),
            score: ScoreTuning::default(),
        }
    }
}

/// Rules shared by every enemy.
#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct EnemyRules {
    pub bullet_size: (f32, f32),
    pub bullet_hitbox: (f32, f32),
    /// An enemy's hitbox is its sprite shrunk by this much on each side.
    pub hitbox_inset: f32,
    pub fire_grace_seconds: f32,
    pub min_fire_distance: f32,
    pub max_bullets: usize,
    pub no_spawn_radius: f32,
}

impl Default for EnemyRules {
    fn default() -> Self {
        Self {
            bullet_size: (5.0, 5.0),
            bullet_hitbox: (2.0, 2.0),
            hitbox_inset: 1.0,
            fire_grace_seconds: 0.5,
            min_fire_distance: 40.0,
            max_bullets: 40,
            no_spawn_radius: 48.0,
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct ScoreTuning {
    pub formation_bonus_per_enemy: u64,
}

impl Default for ScoreTuning {
    fn default() -> Self {
        Self {
            formation_bonus_per_enemy: 100,
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct EnemyTypes {
    pub skiff: Skiff,
    pub gull: Gull,
    pub bellbuoy: Bellbuoy,
    pub lamplighter: Lamplighter,
    pub hook: Hook,
    pub ferry: Ferry,
    pub barnacle: Barnacle,
    pub clam: Clam,
    pub mote: Mote,
    pub arc_gate: ArcGate,
}

/// Declares a tuning struct whose `Default` is the listed values.
macro_rules! tuning_struct {
    ($name:ident { $($field:ident: $ty:ty = $value:expr),* $(,)? }) => {
        #[derive(Clone, Debug, Deserialize)]
        #[serde(default)]
        pub struct $name {
            $(pub $field: $ty,)*
        }

        impl Default for $name {
            fn default() -> Self {
                Self { $($field: $value,)* }
            }
        }
    };
}

tuning_struct!(Skiff {
    hp: u32 = 1,
    score: u64 = 100,
    formation_size: u32 = 5,
    v_spacing: (f32, f32) = (12.0, 10.0),
    speed: f32 = 90.0,
    peel_x: f32 = 40.0,
    peel_interval: f32 = 0.15,
    peel_turn_rate: f32 = 180.0,
    exit_speed: f32 = 140.0,
    launched_speed: f32 = 120.0,
});

tuning_struct!(Gull {
    hp: u32 = 1,
    score: u64 = 100,
    formation_size: u32 = 6,
    spacing_seconds: f32 = 0.25,
    speed: f32 = 70.0,
    amplitude: f32 = 28.0,
    period: f32 = 1.6,
    shot_speed: f32 = 80.0,
    fire_x: f32 = 0.0,
});

tuning_struct!(Bellbuoy {
    hp: u32 = 10,
    score: u64 = 300,
    wave_y: f32 = -104.0,
    bob_amplitude: f32 = 2.0,
    bob_period: f32 = 1.4,
    trigger_range: f32 = 80.0,
    telegraph_seconds: f32 = 0.4,
    fan_count: u32 = 3,
    fan_degrees: f32 = 15.0,
    shot_speed: f32 = 90.0,
    cooldown: f32 = 2.5,
});

tuning_struct!(Lamplighter {
    hp: u32 = 36,
    score: u64 = 500,
    speed: f32 = 80.0,
    brake_distance: f32 = 24.0,
    stop_x: f32 = 100.0,
    hover_seconds: f32 = 3.0,
    bob_amplitude: f32 = 2.0,
    bob_period: f32 = 1.5,
    telegraph_seconds: f32 = 0.4,
    shots: u32 = 2,
    shot_speed: f32 = 100.0,
    fire_interval: f32 = 1.2,
    exit_speed: f32 = 80.0,
});

tuning_struct!(Hook {
    hp: u32 = 2,
    score: u64 = 200,
    formation_size: u32 = 2,
    warning_seconds: f32 = 0.75,
    speed: f32 = 150.0,
    pass_offset: f32 = 50.0,
    terrain_clearance: f32 = 12.0,
    turn_x: f32 = 110.0,
    turn_radius: f32 = 14.0,
    charge_speed: f32 = 170.0,
});

tuning_struct!(Ferry {
    hp: u32 = 400,
    hp_stage2: u32 = 640,
    score: u64 = 2000,
    speed: f32 = 25.0,
    hold_x: f32 = 60.0,
    hold_seconds: f32 = 15.0,
    sink_speed: f32 = 20.0,
    launch_count: u32 = 2,
    first_launch_delay: f32 = 1.0,
    launch_interval: f32 = 3.0,
    ring_count: u32 = 8,
    ring_speed: f32 = 70.0,
    first_ring_delay: f32 = 2.0,
    ring_interval: f32 = 3.0,
});

tuning_struct!(Barnacle {
    hp: u32 = 10,
    score: u64 = 400,
    turn_rate: f32 = 120.0,
    telegraph_seconds: f32 = 0.3,
    shot_speed: f32 = 110.0,
    fire_interval: f32 = 1.6,
});

tuning_struct!(Clam {
    hp: u32 = 16,
    score: u64 = 800,
    formation_size: u32 = 2,
    pair_spacing: (f32, f32) = (0.0, -48.0),
    bob_amplitude: f32 = 3.0,
    bob_period: f32 = 2.0,
    closed_seconds: f32 = 2.0,
    open_seconds: f32 = 1.0,
    fan_count: u32 = 5,
    fan_degrees: f32 = 60.0,
    shot_speed: f32 = 100.0,
});

tuning_struct!(Mote {
    hp: u32 = 1,
    score: u64 = 50,
    formation_size: u32 = 10,
    cloud_radius: f32 = 40.0,
    speed: f32 = 65.0,
    turn_rate: f32 = 90.0,
    lifetime: f32 = 8.0,
});

tuning_struct!(ArcGate {
    hp: u32 = 40,
    score: u64 = 600,
    arc_width: f32 = 4.0,
    on_seconds: f32 = 1.2,
    off_seconds: f32 = 0.8,
    telegraph_seconds: f32 = 0.3,
});

#[cfg(test)]
mod tests {
    use crate::tuning::Tuning;

    /// The committed file matches the defaults here, so neither drifts.
    #[test]
    fn tuning_file_matches_the_defaults() {
        let file = Tuning::parse(include_str!("../../../assets/tuning.ron")).unwrap();
        let ours = format!("{:?}", file.side_shooter);
        let defaults = format!("{:?}", super::SideShooterTuning::default());
        assert_eq!(ours, defaults);
    }
}
