//! Every screen outside the Cabinets themselves.

use bevy::prelude::*;

mod ending;
mod hud;
mod menu;
mod pause;
mod practice;
mod results;
mod stage_card;
mod title;
pub use title::ShowScoresFirst;
mod transition;

pub struct ScreensPlugin;

impl Plugin for ScreensPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            menu::MenuPlugin,
            title::TitlePlugin,
            practice::PracticePlugin,
            stage_card::StageCardPlugin,
            hud::HudPlugin,
            pause::PausePlugin,
            transition::TransitionPlugin,
            results::ResultsPlugin,
            ending::EndingPlugin,
        ));
    }
}
