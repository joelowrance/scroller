//! Cabinet 1: STATIC TIDE, the side-scrolling shooter
//! (docs/cabinets/1-side-shooter.md).
//!
//! The Hero ship and Power-ups are built in #7; this module holds the
//! enemies (#8) so far.

use bevy::prelude::*;

pub mod enemies;
pub mod tuning;

pub struct SideShooterPlugin;

impl Plugin for SideShooterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(enemies::EnemiesPlugin);
    }
}
