// Bevy systems routinely take many parameters and nested query types.
#![allow(clippy::too_many_arguments, clippy::type_complexity)]

mod cabinets;
mod display;
mod flow;
mod font;
mod highscore;
mod input;
mod run;
mod screens;
mod storage;
mod tuning;

use bevy::prelude::*;
use bevy::winit::{UpdateMode, WinitSettings};

fn main() {
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Scroller".into(),
                    // 3× the 320×240 canvas.
                    resolution: (960, 720).into(),
                    // Web: fill the page; the canvas is letterboxed inside.
                    fit_canvas_to_parent: true,
                    canvas: Some("#game".into()),
                    ..default()
                }),
                ..default()
            })
            .set(ImagePlugin::default_nearest()),
    )
    // An arcade game keeps running at full speed even without window focus
    // (a Stage auto-pauses instead; see screens/pause.rs).
    .insert_resource(WinitSettings {
        focused_mode: UpdateMode::Continuous,
        unfocused_mode: UpdateMode::Continuous,
    })
    .add_plugins((
        tuning::TuningPlugin,
        display::DisplayPlugin,
        font::FontPlugin,
        input::InputPlugin,
        flow::FlowPlugin,
        cabinets::CabinetsPlugin,
        screens::ScreensPlugin,
    ));

    #[cfg(feature = "fps-log")]
    app.add_plugins((
        bevy::diagnostic::FrameTimeDiagnosticsPlugin::default(),
        bevy::diagnostic::LogDiagnosticsPlugin::default(),
    ));

    app.run();
}
