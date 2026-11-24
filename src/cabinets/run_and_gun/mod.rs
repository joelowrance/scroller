//! Cabinet 3: UNDERFIELD, the run-and-gun platformer
//! (docs/cabinets/3-run-and-gun.md). So far its Stage files and terrain
//! (#20); the gameplay comes with #19.

pub mod terrain;

#[cfg(test)]
mod stage_checks;

/// The Cabinet's index in `CABINETS`.
pub const CABINET: usize = 2;

/// Cabinet 3's two Stage files.
pub const STAGES: [&str; 2] = [
    include_str!("../../../assets/stages/3-1.ron"),
    include_str!("../../../assets/stages/3-2.ron"),
];
