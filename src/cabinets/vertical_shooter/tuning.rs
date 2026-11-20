//! Cabinet 2's numbers, the `vertical_shooter` section of `assets/tuning.ron`.
//! Every key is described in `docs/cabinets/2-vertical-shooter.md`.
//!
//! Each struct defaults field by field, so the file can list only some keys.

use serde::Deserialize;

pub use crate::cabinets::side_shooter::tuning::vec2;

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

tuning_struct!(VerticalShooterTuning {
    /// Multiplier on every enemy bullet speed in Stage 2.
    stage2_bullet_speed_scale: f32 = 1.15,
    /// Debug: play the enemy gallery instead of the Stage (needs
    /// `debug_keys`). `?gallery` on the web URL sets it too.
    enemy_gallery: bool = false,
    look: LookTuning = LookTuning::default(),
    hero: HeroTuning = HeroTuning::default(),
    shot: ShotTuning = ShotTuning::default(),
    rocket: RocketTuning = RocketTuning::default(),
    bomb: BombTuning = BombTuning::default(),
    power_up: PowerUpTuning = PowerUpTuning::default(),
    enemy: EnemyRules = EnemyRules::default(),
    enemies: EnemyTypes = EnemyTypes::default(),
    score: ScoreTuning = ScoreTuning::default(),
});

tuning_struct!(LookTuning {
    /// Where air sprites' shadows fall, relative to the sprite.
    shadow_offset: (f32, f32) = (6.0, -6.0),
});

tuning_struct!(HeroTuning {
    hitbox: (f32, f32) = (4.0, 4.0),
    speed: f32 = 100.0,
    speed_step: f32 = 20.0,
    edge_margin: f32 = 4.0,
    start: (f32, f32) = (0.0, -80.0),
    entry_from_y: f32 = -136.0,
    entry_seconds: f32 = 1.0,
    respawn_invulnerable_seconds: f32 = 2.0,
    hit_stop_seconds: f32 = 0.15,
});

tuning_struct!(ShotTuning {
    fire_interval: f32 = 0.1,
    speed: f32 = 360.0,
    damage: u32 = 1,
    size: (f32, f32) = (2.0, 8.0),
    twin_spacing: f32 = 6.0,
    spread_degrees: f32 = 15.0,
    lance_size: (f32, f32) = (3.0, 12.0),
    lance_damage: u32 = 2,
    max_on_screen: usize = 96,
});

tuning_struct!(RocketTuning {
    count: u32 = 2,
    interval: f32 = 0.5,
    launch_offset: (f32, f32) = (5.0, 0.0),
    size: (f32, f32) = (3.0, 6.0),
    speed: f32 = 200.0,
    turn_rate: f32 = 270.0,
    damage: u32 = 3,
});

tuning_struct!(BombTuning {
    starting: u32 = 2,
    max: u32 = 4,
    band_speed: f32 = 420.0,
    damage: u32 = 12,
    invulnerable_seconds: f32 = 1.0,
});

tuning_struct!(PowerUpTuning {
    size: (f32, f32) = (10.0, 10.0),
    pickup_box: (f32, f32) = (14.0, 14.0),
    edge_inset: f32 = 16.0,
    drift_speed: f32 = 30.0,
    bob_amplitude: f32 = 3.0,
    bob_period: f32 = 1.2,
    gun_max: u32 = 4,
    pod_max: u32 = 2,
    pod_offset: f32 = 14.0,
    speed_max: u32 = 2,
    ward_invulnerable_seconds: f32 = 1.0,
});

tuning_struct!(
    /// Rules shared by every enemy.
    EnemyRules {
        bullet_size: (f32, f32) = (5.0, 5.0),
        bullet_hitbox: (f32, f32) = (2.0, 2.0),
        /// An enemy's hitbox is its sprite shrunk by this much on each side.
        hitbox_inset: f32 = 1.0,
        fire_grace_seconds: f32 = 0.5,
        /// No bullet appears closer than this to the Hero.
        min_fire_distance: f32 = 40.0,
        max_bullets: usize = 40,
        no_spawn_radius: f32 = 48.0,
        bullet_lifetime: f32 = 12.0,
    }
);

tuning_struct!(EnemyTypes {
    wisp: Wisp = Wisp::default(),
    swift: Swift = Swift::default(),
    bellows: Bellows = Bellows::default(),
    loom: Loom = Loom::default(),
    crawler: Crawler = Crawler::default(),
    silo: Silo = Silo::default(),
    riser: Riser = Riser::default(),
    flak: Flak = Flak::default(),
    mote: Mote = Mote::default(),
    tether: Tether = Tether::default(),
});

tuning_struct!(Wisp {
    hp: u32 = 1,
    score: u64 = 100,
    formation_size: u32 = 6,
    spacing_seconds: f32 = 0.3,
    speed: f32 = 80.0,
    turn_y: f32 = 20.0,
    turn_rate: f32 = 150.0,
    exit_speed: f32 = 110.0,
    launched_speed: f32 = 100.0,
});

tuning_struct!(Swift {
    hp: u32 = 2,
    score: u64 = 200,
    formation_size: u32 = 2,
    warning_seconds: f32 = 0.75,
    chevron_blink_hz: f32 = 4.0,
    speed: f32 = 150.0,
    pass_offset: f32 = 50.0,
    pair_spread: f32 = 0.5,
    edge_clearance: f32 = 12.0,
    turn_y: f32 = 80.0,
    turn_radius: f32 = 14.0,
    dive_speed: f32 = 170.0,
    dive_steer_speed: f32 = 85.0,
});

tuning_struct!(Bellows {
    hp: u32 = 36,
    score: u64 = 500,
    speed: f32 = 80.0,
    brake_distance: f32 = 24.0,
    brake_floor: f32 = 0.15,
    stop_inset: f32 = 60.0,
    hover_seconds: f32 = 3.0,
    sway_amplitude: f32 = 2.0,
    sway_period: f32 = 1.5,
    telegraph_seconds: f32 = 0.4,
    burst: u32 = 3,
    burst_interval: f32 = 0.12,
    bursts: u32 = 2,
    fire_interval: f32 = 1.4,
    shot_speed: f32 = 90.0,
    exit_speed: f32 = 80.0,
});

tuning_struct!(Loom {
    shuttle_hp: u32 = 60,
    shuttle_hp_stage2: u32 = 100,
    shuttle_score: u64 = 500,
    core_hp: u32 = 240,
    core_hp_stage2: u32 = 400,
    core_score: u64 = 2000,
    shuttle_offset: f32 = 28.0,
    speed: f32 = 20.0,
    hold_y: f32 = 60.0,
    hold_seconds: f32 = 20.0,
    retreat_speed: f32 = 20.0,
    first_launch_delay: f32 = 1.0,
    launch_interval: f32 = 3.0,
    telegraph_seconds: f32 = 0.4,
    pair_interval: f32 = 0.15,
    shot_speed: f32 = 80.0,
    fire_interval: f32 = 2.5,
    spiral_arms: u32 = 3,
    spiral_interval: f32 = 0.3,
    spiral_turn_rate: f32 = 40.0,
    spiral_speed: f32 = 60.0,
    spiral_seconds: f32 = 2.0,
    spiral_pause: f32 = 2.0,
});

tuning_struct!(Crawler {
    hp: u32 = 10,
    score: u64 = 300,
    formation_size: u32 = 3,
    spacing: f32 = 18.0,
    speed: f32 = 12.0,
    turn_rate: f32 = 120.0,
    telegraph_seconds: f32 = 0.3,
    shot_speed: f32 = 80.0,
    fire_interval: f32 = 2.0,
    stagger: f32 = 0.4,
});

tuning_struct!(Silo {
    hp: u32 = 16,
    score: u64 = 800,
    closed_seconds: f32 = 2.4,
    telegraph_seconds: f32 = 0.3,
    open_seconds: f32 = 1.2,
    ring_count: u32 = 6,
    shot_speed: f32 = 70.0,
});

tuning_struct!(Riser {
    hp: u32 = 16,
    score: u64 = 400,
    first_launch_delay: f32 = 1.0,
    launch_interval: f32 = 3.5,
    climb_seconds: f32 = 1.0,
    pod_score_climbing: u64 = 150,
    pod_score_airborne: u64 = 50,
    drift_speed: f32 = 40.0,
    fuse_seconds: f32 = 1.2,
    telegraph_seconds: f32 = 0.3,
    ring_count: u32 = 6,
    shot_speed: f32 = 70.0,
});

tuning_struct!(Flak {
    hp: u32 = 24,
    score: u64 = 600,
    fire_interval: f32 = 2.4,
    warning_seconds: f32 = 0.6,
    burst_radius: f32 = 10.0,
    burst_seconds: f32 = 0.15,
    burst_count: u32 = 6,
    burst_speed: f32 = 60.0,
    max_on_screen: u32 = 2,
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

tuning_struct!(Tether {
    hp: u32 = 40,
    score: u64 = 600,
    line_width: f32 = 4.0,
    on_seconds: f32 = 1.2,
    off_seconds: f32 = 0.8,
    telegraph_seconds: f32 = 0.3,
});

tuning_struct!(ScoreTuning {
    formation_bonus_per_enemy: u64 = 100,
    maxed_power_up: u64 = 1000,
    bomb_cancel: u64 = 10,
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
        let ours = format!("{:?}", file.vertical_shooter);
        let defaults = format!("{:?}", super::VerticalShooterTuning::default());
        assert_eq!(ours, defaults);
    }

    /// The doc's proposed section parses to exactly these defaults too.
    #[test]
    fn the_design_doc_section_matches_the_defaults() {
        let doc = include_str!("../../../docs/cabinets/2-vertical-shooter.md");
        let start = doc
            .find("vertical_shooter: (")
            .expect("tuning block in the doc");
        let end = start + doc[start..].find("\n```").expect("end of the block");
        let block = &doc[start + "vertical_shooter: ".len()..end].trim_end();
        let parsed: super::VerticalShooterTuning = ron::from_str(block).unwrap();
        assert_eq!(
            format!("{parsed:?}"),
            format!("{:?}", super::VerticalShooterTuning::default())
        );
    }
}
