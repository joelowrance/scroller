//! Cabinet Transitions.
//!
//! Each of the three will become its own unique sequence (ship banks upward,
//! Hero ejects and parachutes, Hero starts climbing the tower). Until those
//! are built this placeholder plays: the outgoing Cabinet's colour bars wipe
//! across into the incoming Cabinet's palette.

use bevy::prelude::*;

use crate::cabinets::CABINETS;
use crate::display::{Backdrop, HALF_H, HALF_W, HEIGHT};
use crate::flow::{CardReason, Screen};
use crate::font::{PixelText, text_at};
use crate::run::Session;
use crate::tuning::Tuning;

const BAR_COUNT: usize = 12;

#[derive(Resource)]
struct TransitionClock(Timer);

#[derive(Component)]
struct Bar(usize);

fn spawn(
    mut commands: Commands,
    session: Res<Session>,
    tuning: Res<Tuning>,
    mut backdrop: ResMut<Backdrop>,
) {
    // `session.cabinet` is already the incoming Cabinet.
    let to = session.cabinet;
    let from = to - 1;
    backdrop.0 = CABINETS[from].backdrop;
    commands.insert_resource(TransitionClock(Timer::from_seconds(
        tuning.transition_seconds.max(0.5),
        TimerMode::Once,
    )));

    let bar_h = HEIGHT as f32 / BAR_COUNT as f32;
    for i in 0..BAR_COUNT {
        commands.spawn((
            Sprite::from_color(CABINETS[to].backdrop, Vec2::new(HALF_W * 2.0, bar_h)),
            Transform::from_xyz(HALF_W * 2.0, -HALF_H + bar_h * (i as f32 + 0.5), 1.0),
            Bar(i),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    }
    commands.spawn((
        text_at(
            PixelText::new(
                format!("CABINET {} CLEAR!", from + 1),
                CABINETS[from].accent,
            ),
            0.0,
            10.0,
            2,
        ),
        DespawnOnExit(Screen::CabinetTransition),
    ));
    commands.spawn((
        text_at(
            PixelText::new(format!("NEXT: {}", CABINETS[to].genre), CABINETS[to].accent),
            0.0,
            -15.0,
            1,
        ),
        DespawnOnExit(Screen::CabinetTransition),
    ));
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut clock: ResMut<TransitionClock>,
    mut bars: Query<(&Bar, &mut Transform)>,
    mut next: ResMut<NextState<Screen>>,
) {
    clock.0.tick(time.delta());
    let t = clock.0.fraction();
    for (bar, mut transform) in &mut bars {
        // Staggered wipe: each bar starts a little after the one below it.
        let start = bar.0 as f32 / BAR_COUNT as f32 * 0.4;
        let local = ((t - start) / 0.5).clamp(0.0, 1.0);
        let eased = 1.0 - (1.0 - local).powi(3);
        transform.translation.x = (HALF_W * 2.0 * (1.0 - eased)).round();
    }
    if clock.0.is_finished() {
        commands.insert_resource(CardReason::AfterTransition);
        next.set(Screen::StageCard);
    }
}

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::CabinetTransition), spawn)
            .add_systems(Update, update.run_if(in_state(Screen::CabinetTransition)));
    }
}
