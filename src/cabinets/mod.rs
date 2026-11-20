//! The Cabinets and the contract between them and the rest of the game.
//!
//! While the screen is [`Screen::Playing`](crate::flow::Screen::Playing), the
//! active Cabinet owns the canvas. It spawns its Stage (tagging entities with
//! `DespawnOnExit(Screen::Playing)`), reads [`Session`](crate::run::Session)
//! to learn which Stage to build, and reports back with three messages:
//! [`StageCleared`], [`LifeLost`] and [`ScoreGained`]. The flow decides what
//! happens next (Stage Card, Game Over, Cabinet Transition, ...).

use bevy::prelude::*;

mod placeholder;

pub const CABINET_COUNT: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scroll {
    Sideways,
    Upward,
}

pub struct CabinetInfo {
    pub genre: &'static str,
    pub scroll: Scroll,
    /// Background colour of the Cabinet's palette.
    pub backdrop: Color,
    /// Highlight colour of the Cabinet's palette.
    pub accent: Color,
}

/// The Cabinets in Run order.
pub const CABINETS: [CabinetInfo; CABINET_COUNT] = [
    CabinetInfo {
        genre: "SIDE-SCROLLING SHOOTER",
        scroll: Scroll::Sideways,
        backdrop: Color::srgb(0.02, 0.02, 0.10),
        accent: Color::srgb(0.30, 0.85, 1.00),
    },
    CabinetInfo {
        genre: "VERTICAL SHOOTER",
        scroll: Scroll::Upward,
        backdrop: Color::srgb(0.08, 0.00, 0.10),
        accent: Color::srgb(1.00, 0.40, 0.80),
    },
    CabinetInfo {
        genre: "RUN-AND-GUN PLATFORMER",
        scroll: Scroll::Sideways,
        backdrop: Color::srgb(0.10, 0.05, 0.00),
        accent: Color::srgb(1.00, 0.75, 0.20),
    },
    CabinetInfo {
        genre: "VERTICAL CLIMBER",
        scroll: Scroll::Upward,
        backdrop: Color::srgb(0.00, 0.08, 0.03),
        accent: Color::srgb(0.40, 1.00, 0.50),
    },
];

/// The Hero reached the end of the Stage.
#[derive(Message, Default)]
pub struct StageCleared;

/// The Hero lost a life. If lives remain, the Cabinet puts the Hero back at a
/// Restart Point itself; otherwise the flow moves to Game Over.
#[derive(Message, Default)]
pub struct LifeLost;

/// Points the Hero earned.
#[derive(Message)]
pub struct ScoreGained(pub u64);

pub struct CabinetsPlugin;

impl Plugin for CabinetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<StageCleared>()
            .add_message::<LifeLost>()
            .add_message::<ScoreGained>()
            .add_plugins(placeholder::PlaceholderCabinetPlugin);
    }
}
