//! How Cabinet 4's enemies and hazards look: #25's `c4_*` sprites in the
//! Cabinet palette, and which frame shows each state.

use bevy::prelude::*;

use super::Kind;
use super::brain::{Brain, PicketPhase, VentPhase};
use crate::cabinets::climber::palette::*;

/// The sprite of each kind.
pub fn sprite(kind: Kind) -> &'static str {
    match kind {
        Kind::Flit => "c4_flit",
        Kind::Limpet => "c4_limpet",
        Kind::Scrap => "c4_scrap",
        Kind::Picket => "c4_picket",
        Kind::Vent => "c4_vent",
        Kind::Plumb => "c4_plumb",
        Kind::Pitcher => "c4_pitcher",
        Kind::Canister => "c4_canister",
        Kind::Mote => "c2_mote",
        Kind::Winch => "c4_winch_hook",
        Kind::Carriage => "c4_winch_carriage",
    }
}

/// The sprite size of each kind (docs/cabinets/4-climber.md, the enemy
/// table).
pub fn size(kind: Kind) -> Vec2 {
    let (w, h) = match kind {
        Kind::Flit => (10.0, 8.0),
        Kind::Limpet => (10.0, 6.0),
        Kind::Scrap => (10.0, 10.0),
        Kind::Picket => (12.0, 18.0),
        Kind::Vent => (16.0, 12.0),
        Kind::Plumb => (10.0, 10.0),
        Kind::Pitcher => (14.0, 14.0),
        Kind::Canister => (8.0, 8.0),
        Kind::Mote => (4.0, 4.0),
        Kind::Winch => (16.0, 16.0),
        Kind::Carriage => (32.0, 16.0),
    };
    Vec2::new(w, h)
}

/// The frame for a brain's state now; `clock` animates walking and wings.
pub fn frame(brain: &Brain, clock: f32) -> usize {
    let step = |rate: f32| (clock * rate) as usize % 2;
    match brain {
        Brain::Flit(_) => step(8.0),
        Brain::Limpet(_) => step(4.0),
        Brain::Picket(p) => match p.phase {
            PicketPhase::Walk => step(4.0),
            PicketPhase::Flash(_) | PicketPhase::Burst(..) => 2,
            PicketPhase::Cooldown(_) => 0,
        },
        Brain::Vent(v) => match v.phase {
            VentPhase::Waiting | VentPhase::Closed => 0,
            VentPhase::Glowing => 1,
            VentPhase::Open => 2,
        },
        Brain::Plumb(_) | Brain::Winch(_) => {
            // The flash before it fires blinks.
            (brain.telegraph() && step(10.0) == 0) as usize
        }
        Brain::Pitcher(p) => p.winding as usize,
        _ => 0,
    }
}

/// An enemy bullet: a Rose dot with a Foam core.
pub fn bullet(size: Vec2) -> impl Bundle {
    (
        Sprite::from_color(ROSE, size),
        children![(
            Sprite::from_color(FOAM, Vec2::ONE),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}
