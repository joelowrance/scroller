//! Cabinet 4: HIGHWIRE, the vertical climber (docs/cabinets/4-climber.md).
//! So far its Stage files and terrain (#25); the gameplay comes with #24.

pub mod terrain;

#[cfg(test)]
mod stage_checks;

/// The Cabinet's index in `CABINETS`.
pub const CABINET: usize = 3;

/// Cabinet 4's two Stage files.
pub const STAGES: [&str; 2] = [
    include_str!("../../../assets/stages/4-1.ron"),
    include_str!("../../../assets/stages/4-2.ron"),
];
