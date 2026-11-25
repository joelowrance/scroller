//! Cabinet Transitions.
//!
//! Each of the three is its own unique sequence:
//!
//! - 1 → 2, the Wren banks upward and the view turns 90°: [`bank_up`].
//! - 2 → 3, the Hero ejects and parachutes down to land on foot: [`eject`].
//! - 3 → 4, the Hero reaches the tower and climbs; the view pans up: [`climb`].

use bevy::prelude::*;

use crate::run::Session;

mod bank_up;
mod climb;
mod eject;

/// Whether the Transition starting now is the 1 → 2 one.
/// `session.cabinet` is already the incoming Cabinet.
fn into_cabinet_2(session: Res<Session>) -> bool {
    session.cabinet == 1
}

/// Whether the Transition starting now is the 2 → 3 one.
fn into_cabinet_3(session: Res<Session>) -> bool {
    session.cabinet == 2
}

/// Whether the Transition starting now is the 3 → 4 one.
fn into_cabinet_4(session: Res<Session>) -> bool {
    session.cabinet == 3
}

/// A stretch of the sequence, from `.0` to `.1` seconds.
#[derive(Clone, Copy, Debug)]
pub(super) struct Span(f32, f32);

impl Span {
    /// 0 before the span, 1 after it, linear within.
    fn at(self, t: f32) -> f32 {
        ((t - self.0) / (self.1 - self.0)).clamp(0.0, 1.0)
    }

    fn contains(self, t: f32) -> bool {
        (self.0..self.1).contains(&t)
    }
}

/// A repeatable pseudo-random number in 0..1.
fn noise(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

pub struct TransitionPlugin;

impl Plugin for TransitionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            bank_up::BankUpPlugin,
            eject::EjectPlugin,
            climb::ClimbPlugin,
        ));
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::audio::PlaySfx;
    use crate::audio::{Song, TransitionMusic};
    use crate::cabinets::CABINETS;
    use crate::cabinets::side_shooter::WrenLastSeen;
    use crate::display::Backdrop;
    use crate::flow::{CardReason, Screen};
    use crate::tuning::Tuning;

    /// A Transition into Cabinet `to`, run until it hands over.
    fn run_transition(to: usize) -> App {
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        session.cabinet = to;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .insert_resource(tuning)
            .insert_resource(session)
            .insert_resource(Backdrop(Color::BLACK))
            .init_resource::<TransitionMusic>()
            .init_resource::<WrenLastSeen>()
            .init_resource::<crate::cabinets::vertical_shooter::WrenLastSeen>()
            .add_message::<PlaySfx>()
            .init_resource::<crate::sprites::Sprites>()
            .init_state::<Screen>()
            .add_plugins(TransitionPlugin);
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::CabinetTransition);
        app.update();
        app
    }

    fn screen(app: &App) -> Screen {
        *app.world().resource::<State<Screen>>().get()
    }

    #[test]
    fn one_to_two_banks_up_then_shows_cabinet_2s_stage_card() {
        let mut app = run_transition(1);
        assert_eq!(
            app.world().resource::<TransitionMusic>().0,
            Some(Song::BankUp)
        );
        let mut frames = 0;
        while screen(&app) == Screen::CabinetTransition {
            app.update();
            frames += 1;
            assert!(frames < 65, "still in the Transition after 6.5 s");
        }
        assert!(frames >= 54, "about 5.5 seconds, took {frames} frames");
        assert_eq!(screen(&app), Screen::StageCard);
        assert_eq!(
            *app.world().resource::<CardReason>(),
            CardReason::AfterTransition
        );
        assert_eq!(app.world().resource::<Backdrop>().0, CABINETS[1].backdrop);
        let mut sprites = app.world_mut().query::<&Sprite>();
        assert_eq!(sprites.iter(app.world()).count(), 0, "all cleaned up");
    }

    #[test]
    fn two_to_three_ejects_then_shows_cabinet_3s_stage_card() {
        let mut app = run_transition(2);
        assert_eq!(
            app.world().resource::<TransitionMusic>().0,
            Some(Song::BailOut)
        );
        let mut frames = 0;
        while screen(&app) == Screen::CabinetTransition {
            app.update();
            frames += 1;
            assert!(frames < 70, "still in the Transition after 7 s");
        }
        assert!(frames >= 59, "about 6 seconds, took {frames} frames");
        assert_eq!(screen(&app), Screen::StageCard);
        assert_eq!(
            *app.world().resource::<CardReason>(),
            CardReason::AfterTransition
        );
        assert_eq!(app.world().resource::<Backdrop>().0, CABINETS[2].backdrop);
        let mut sprites = app.world_mut().query::<&Sprite>();
        assert_eq!(sprites.iter(app.world()).count(), 0, "all cleaned up");
    }

    #[test]
    fn three_to_four_climbs_then_shows_cabinet_4s_stage_card() {
        let mut app = run_transition(3);
        assert_eq!(
            app.world().resource::<TransitionMusic>().0,
            Some(Song::Upswing)
        );
        let mut frames = 0;
        while screen(&app) == Screen::CabinetTransition {
            app.update();
            frames += 1;
            assert!(frames < 65, "still in the Transition after 6.5 s");
        }
        assert!(frames >= 54, "about 5.5 seconds, took {frames} frames");
        assert_eq!(screen(&app), Screen::StageCard);
        assert_eq!(
            *app.world().resource::<CardReason>(),
            CardReason::AfterTransition
        );
        assert_eq!(app.world().resource::<Backdrop>().0, CABINETS[3].backdrop);
        let mut sprites = app.world_mut().query::<&Sprite>();
        assert_eq!(sprites.iter(app.world()).count(), 0, "all cleaned up");
    }
}
