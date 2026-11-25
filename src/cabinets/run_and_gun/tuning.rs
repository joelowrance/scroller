//! Cabinet 3's numbers, the `run_and_gun` section of `assets/tuning.ron`.
//! Every key is described in `docs/cabinets/3-run-and-gun.md`.
//!
//! Each struct defaults field by field, so the file can list only some keys.

use serde::Deserialize;

pub use crate::cabinets::side_shooter::tuning::vec2;
use crate::kit::platform::JumpParams;

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

tuning_struct!(RunAndGunTuning {
    /// Multiplier on every enemy bullet speed in Stage 2.
    stage2_bullet_speed_scale: f32 = 1.15,
    /// Debug (needs `debug_keys`): the Hero runs, fires and jumps by itself.
    /// `?autopilot` on the web URL sets it too.
    debug_autopilot: bool = false,
    /// Debug (needs `debug_keys`): play the enemy gallery instead of the
    /// Stage. `?gallery` on the web URL sets it too.
    enemy_gallery: bool = false,
    camera: CameraTuning = CameraTuning::default(),
    hero: HeroTuning = HeroTuning::default(),
    jump: JumpTuning = JumpTuning::default(),
    shot: ShotTuning = ShotTuning::default(),
    flare: FlareTuning = FlareTuning::default(),
    power_up: PowerUpTuning = PowerUpTuning::default(),
    terrain: TerrainTuning = TerrainTuning::default(),
    enemy: EnemyRules = EnemyRules::default(),
    enemies: EnemyTypes = EnemyTypes::default(),
    hazards: HazardTypes = HazardTypes::default(),
    score: ScoreTuning = ScoreTuning::default(),
});

tuning_struct!(CameraTuning {
    push_x: f32 = -16.0,
    max_x: f32 = 64.0,
});

tuning_struct!(HeroTuning {
    hitbox: (f32, f32) = (6.0, 14.0),
    crouch_hitbox: (f32, f32) = (6.0, 8.0),
    body: (f32, f32) = (10.0, 20.0),
    crouch_body: (f32, f32) = (10.0, 12.0),
    run_speed: f32 = 72.0,
    air_speed: f32 = 72.0,
    edge_margin: f32 = 4.0,
    entry_from_y: f32 = 132.0,
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

tuning_struct!(ShotTuning {
    fire_interval: f32 = 0.125,
    fire_interval_fast: f32 = 0.083,
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

tuning_struct!(FlareTuning {
    interval: f32 = 0.8,
    size: (f32, f32) = (4.0, 4.0),
    launch: (f32, f32) = (110.0, 160.0),
    damage: u32 = 4,
    radius: f32 = 12.0,
    max_on_screen: usize = 3,
});

tuning_struct!(PowerUpTuning {
    size: (f32, f32) = (10.0, 10.0),
    bob_amplitude: f32 = 2.0,
    bob_period: f32 = 1.0,
    gun_max: u32 = 4,
    jet_speed_fraction: f32 = 0.7,
    hustle_step: f32 = 12.0,
    ward_invulnerable_seconds: f32 = 1.0,
});

tuning_struct!(TerrainTuning {
    tile_size: f32 = 8.0,
    max_gap: f32 = 40.0,
    max_step: f32 = 40.0,
    crumble_seconds: f32 = 0.5,
    crumble_respawn_seconds: f32 = 3.0,
    /// Stage-check constants (read by `stage_checks.rs` only).
    restart_pit_clearance: f32 = 48.0,
    restart_hazard_clearance: f32 = 48.0,
    pit_free_run: f32 = 64.0,
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
        behind_warning_seconds: f32 = 0.75,
        chest_height: f32 = 14.0,
        ankle_height: f32 = 4.0,
        aimed_min_distance: f32 = 64.0,
    }
);

tuning_struct!(EnemyTypes {
    skitter: Skitter = Skitter::default(),
    tick: Tick = Tick::default(),
    kite: Kite = Kite::default(),
    sentry: Sentry = Sentry::default(),
    burrower: Burrower = Burrower::default(),
    dropper: Dropper = Dropper::default(),
    hauler: Hauler = Hauler::default(),
    turret: Turret = Turret::default(),
    mote: Mote = Mote::default(),
    strider: Strider = Strider::default(),
});

tuning_struct!(Skitter {
    hp: u32 = 2,
    score: u64 = 100,
    formation_size: u32 = 4,
    spacing: f32 = 16.0,
    speed: f32 = 60.0,
    max_hop: f32 = 24.0,
    hop_height: f32 = 20.0,
});

tuning_struct!(Tick {
    hp: u32 = 6,
    score: u64 = 200,
    formation_size: u32 = 2,
    hop_height: f32 = 24.0,
    hop_length: f32 = 32.0,
    rest_seconds: f32 = 0.6,
    stagger: f32 = 0.3,
    telegraph_seconds: f32 = 0.25,
});

tuning_struct!(Kite {
    hp: u32 = 2,
    score: u64 = 150,
    formation_size: u32 = 3,
    spacing_seconds: f32 = 0.4,
    cruise_y: f32 = 80.0,
    speed: f32 = 70.0,
    dive_window: f32 = 40.0,
    telegraph_seconds: f32 = 0.3,
    dive_floor: f32 = 14.0,
    dive_speed: f32 = 130.0,
});

tuning_struct!(Sentry {
    hp: u32 = 14,
    score: u64 = 500,
    down_seconds: f32 = 1.8,
    telegraph_seconds: f32 = 0.3,
    up_seconds: f32 = 1.4,
    burst: u32 = 2,
    burst_interval: f32 = 0.25,
    shot_speed: f32 = 60.0,
});

tuning_struct!(Burrower {
    hp: u32 = 6,
    score: u64 = 300,
    wake_distance: f32 = 72.0,
    telegraph_seconds: f32 = 0.4,
    shot_speed: f32 = 60.0,
    fire_interval: f32 = 1.6,
    up_seconds: f32 = 3.2,
    down_seconds: f32 = 2.0,
});

tuning_struct!(Dropper {
    hp: u32 = 20,
    score: u64 = 400,
    cruise_y: f32 = 72.0,
    speed: f32 = 40.0,
    fire_interval: f32 = 1.5,
    telegraph_seconds: f32 = 0.4,
    canister_gravity: f32 = 600.0,
    burst_speed: f32 = 60.0,
    no_drop_window: f32 = 24.0,
    canister_size: (f32, f32) = (8.0, 8.0),
});

tuning_struct!(Hauler {
    hp: u32 = 40,
    score: u64 = 800,
    speed: f32 = 20.0,
    plough_height: f32 = 10.0,
    fire_interval: f32 = 2.4,
    telegraph_seconds: f32 = 0.4,
    fan_count: u32 = 3,
    fan_degrees: f32 = 50.0,
    shot_speed: f32 = 60.0,
    shot_gravity: f32 = 120.0,
});

tuning_struct!(Turret {
    hp: u32 = 24,
    score: u64 = 400,
    turn_rate: f32 = 90.0,
    fire_interval: f32 = 2.0,
    telegraph_seconds: f32 = 0.35,
    shot_speed: f32 = 60.0,
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

tuning_struct!(Strider {
    hp: u32 = 120,
    score: u64 = 5000,
    speed: f32 = 16.0,
    hold_x: f32 = 72.0,
    pace_range: f32 = 40.0,
    core_height: f32 = 16.0,
    cycle_seconds: f32 = 4.0,
    wave_range: f32 = 120.0,
    fan_delay: f32 = 2.2,
    stomp_telegraph_seconds: f32 = 0.6,
    wave_height: f32 = 8.0,
    wave_speed: f32 = 90.0,
    fan_count: u32 = 5,
    fan_degrees: f32 = 40.0,
    shot_speed: f32 = 60.0,
    telegraph_seconds: f32 = 0.4,
});

tuning_struct!(HazardTypes {
    crusher: Crusher = Crusher::default(),
    arc_floor: ArcFloor = ArcFloor::default(),
    conveyor: Conveyor = Conveyor::default(),
});

tuning_struct!(Crusher {
    up_seconds: f32 = 1.6,
    telegraph_seconds: f32 = 0.5,
    slam_seconds: f32 = 0.15,
    down_seconds: f32 = 0.6,
});

tuning_struct!(ArcFloor {
    on_seconds: f32 = 1.0,
    off_seconds: f32 = 1.4,
    telegraph_seconds: f32 = 0.4,
});

tuning_struct!(Conveyor { speed: f32 = 24.0 });

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
        let ours = format!("{:?}", file.run_and_gun);
        let defaults = format!("{:?}", super::RunAndGunTuning::default());
        assert_eq!(ours, defaults);
    }

    /// The doc's proposed section parses to exactly these defaults too.
    #[test]
    fn the_design_doc_section_matches_the_defaults() {
        let doc = include_str!("../../../docs/cabinets/3-run-and-gun.md");
        let start = doc.find("run_and_gun: (").expect("tuning block in the doc");
        let end = start + doc[start..].find("\n```").expect("end of the block");
        let block = &doc[start + "run_and_gun: ".len()..end].trim_end();
        let parsed: super::RunAndGunTuning = ron::from_str(block).unwrap();
        assert_eq!(
            format!("{parsed:?}"),
            format!("{:?}", super::RunAndGunTuning::default())
        );
    }
}
