//! The Cabinets and the contract between them and the rest of the game.
//!
//! While the screen is [`Screen::Playing`](crate::flow::Screen::Playing), the
//! active Cabinet owns the canvas. It spawns its Stage (tagging entities with
//! `DespawnOnExit(Screen::Playing)`), reads [`Session`](crate::run::Session)
//! to learn which Stage to build, and reports back with three messages:
//! [`StageCleared`], [`LifeLost`] and [`ScoreGained`]. The flow decides what
//! happens next (Stage Card, Game Over, Cabinet Transition, ...).

use bevy::prelude::*;

use crate::run::Session;
use crate::sprites::palette::{ACCENT, BACKDROP, PALETTES};

mod placeholder;
pub mod side_shooter;

pub const CABINET_COUNT: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Scroll {
    Sideways,
    Upward,
}

pub struct CabinetInfo {
    pub genre: &'static str,
    pub scroll: Scroll,
    /// Background colour of the Cabinet's palette (`sprites::palette`).
    pub backdrop: Color,
    /// Highlight colour of the Cabinet's palette.
    pub accent: Color,
}

/// The Cabinets in Run order.
pub const CABINETS: [CabinetInfo; CABINET_COUNT] = [
    CabinetInfo {
        genre: "SIDE-SCROLLING SHOOTER",
        scroll: Scroll::Sideways,
        backdrop: PALETTES[0].color(BACKDROP),
        accent: PALETTES[0].color(ACCENT),
    },
    CabinetInfo {
        genre: "VERTICAL SHOOTER",
        scroll: Scroll::Upward,
        backdrop: PALETTES[1].color(BACKDROP),
        accent: PALETTES[1].color(ACCENT),
    },
    CabinetInfo {
        genre: "RUN-AND-GUN PLATFORMER",
        scroll: Scroll::Sideways,
        backdrop: PALETTES[2].color(BACKDROP),
        accent: PALETTES[2].color(ACCENT),
    },
    CabinetInfo {
        genre: "VERTICAL CLIMBER",
        scroll: Scroll::Upward,
        backdrop: PALETTES[3].color(BACKDROP),
        accent: PALETTES[3].color(ACCENT),
    },
];

/// Run condition: the Session is in the given Cabinet.
pub fn cabinet_is(index: usize) -> impl Fn(Option<Res<Session>>) -> bool + Clone {
    move |session| session.is_some_and(|s| s.cabinet == index)
}

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
            .add_plugins((
                placeholder::PlaceholderCabinetPlugin,
                side_shooter::SideShooterPlugin,
            ));
    }
}
