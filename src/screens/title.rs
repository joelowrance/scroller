//! The title menu and Attract Mode, which flips to the High Score Table when
//! nobody is touching the controls.

use bevy::prelude::*;

use super::menu::Menu;
use crate::demo::{self, DemoCursor};
use crate::display::Backdrop;
use crate::flow::{Progress, Screen, continue_run, save_exists, start_new_run};
use crate::font::{Blink, PixelText, text_at};
use crate::highscore::HighScoreTable;
use crate::input::{Button, Controls};
use crate::tuning::Tuning;

const CONTINUE: &str = "CONTINUE";
const NEW_RUN: &str = "NEW RUN";
const PRACTICE: &str = "PRACTICE";
const NO: &str = "NO";
const YES: &str = "YES";

const TITLE_COLOR: Color = Color::srgb(1.0, 0.85, 0.2);
const MENU_COLOR: Color = Color::srgb(0.3, 0.9, 1.0);

#[derive(Component)]
struct TitlePanel;

/// Version and short git hash, shown small on the title screen.
const VERSION_LINE: &str = concat!("V", env!("CARGO_PKG_VERSION"), " ", env!("GIT_HASH"));

#[derive(Component)]
struct ScoresPanel;

#[derive(Component)]
struct MainMenu;

#[derive(Component)]
struct ConfirmPanel;

/// Set before entering the Title to open on the High Score Table.
#[derive(Resource)]
pub struct ShowScoresFirst;

#[derive(Resource)]
struct Attract {
    idle: Timer,
    showing_scores: bool,
}

fn spawn(
    mut commands: Commands,
    tuning: Res<Tuning>,
    scores: Res<HighScoreTable>,
    progress: Res<Progress>,
    show_scores: Option<Res<ShowScoresFirst>>,
    mut backdrop: ResMut<Backdrop>,
) {
    backdrop.0 = Color::srgb(0.02, 0.0, 0.06);
    let showing_scores = show_scores.is_some();
    commands.remove_resource::<ShowScoresFirst>();
    commands.insert_resource(Attract {
        idle: Timer::from_seconds(tuning.attract_panel_seconds.max(1.0), TimerMode::Repeating),
        showing_scores,
    });

    let visible = |show: bool| {
        if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        }
    };

    // Title panel: logo and menu.
    let mut items = Vec::new();
    if save_exists() {
        items.push(CONTINUE);
    }
    items.push(NEW_RUN);
    if progress.cleared.iter().any(|c| *c) {
        items.push(PRACTICE);
    }
    let menu = Menu {
        active: !showing_scores,
        ..Menu::new(items, MENU_COLOR)
    }
    .spawn(&mut commands, -10.0);
    commands.entity(menu).insert(MainMenu);
    commands
        .spawn((
            TitlePanel,
            Transform::default(),
            visible(!showing_scores),
            DespawnOnExit(Screen::Title),
        ))
        .add_child(menu)
        .with_children(|p| {
            p.spawn(text_at(
                PixelText::new("SCROLLER", TITLE_COLOR),
                0.0,
                70.0,
                4,
            ));
            p.spawn(text_at(
                PixelText::new("AN 80S ARCADE MASHUP", Color::WHITE),
                0.0,
                38.0,
                1,
            ));
            p.spawn(text_at(
                PixelText::new(VERSION_LINE, Color::srgb(0.5, 0.5, 0.5)),
                0.0,
                -112.0,
                1,
            ));
            p.spawn((
                text_at(PixelText::new("PRESS A", Color::WHITE), 0.0, -90.0, 1),
                Blink::every(0.5),
            ));
        });

    // Confirmation when starting a new Run would abandon the Save.
    let confirm_menu = Menu {
        active: false,
        ..Menu::new(vec![NO, YES], MENU_COLOR)
    }
    .spawn(&mut commands, -30.0);
    commands
        .spawn((
            ConfirmPanel,
            Transform::default(),
            Visibility::Hidden,
            DespawnOnExit(Screen::Title),
        ))
        .add_child(confirm_menu)
        .with_children(|p| {
            p.spawn(text_at(
                PixelText::new("ABANDON YOUR SAVED RUN?", TITLE_COLOR),
                0.0,
                10.0,
                1,
            ));
        });

    // High Score Table panel.
    commands
        .spawn((
            ScoresPanel,
            Transform::default(),
            visible(showing_scores),
            DespawnOnExit(Screen::Title),
        ))
        .with_children(|p| {
            p.spawn(text_at(
                PixelText::new("HIGH SCORES", TITLE_COLOR),
                0.0,
                90.0,
                2,
            ));
            for (i, entry) in scores.entries.iter().enumerate() {
                let line = format!("{:>2}  {}  {:>7}", i + 1, entry.initials, entry.score);
                let color = if i == 0 { TITLE_COLOR } else { Color::WHITE };
                p.spawn(text_at(
                    PixelText::new(line, color),
                    0.0,
                    60.0 - i as f32 * 14.0,
                    1,
                ));
            }
        });
}

fn update(
    mut commands: Commands,
    time: Res<Time<Real>>,
    controls: Res<Controls>,
    tuning: Res<Tuning>,
    mut attract: ResMut<Attract>,
    mut cursor: ResMut<DemoCursor>,
    mut next: ResMut<NextState<Screen>>,
    mut main_menu: Single<&mut Menu, (With<MainMenu>, Without<ConfirmPanel>)>,
    mut confirm_menu: Single<&mut Menu, Without<MainMenu>>,
    mut panels: ParamSet<(
        Single<&mut Visibility, With<TitlePanel>>,
        Single<&mut Visibility, With<ScoresPanel>>,
        Single<&mut Visibility, With<ConfirmPanel>>,
    )>,
) {
    let confirming = confirm_menu.active;

    // Attract Mode: flip panels when idle; any input returns to the title.
    if controls.any_just_pressed() {
        attract.idle.reset();
        if attract.showing_scores {
            attract.showing_scores = false;
            main_menu.active = true;
            **panels.p0() = Visibility::Inherited;
            **panels.p1() = Visibility::Hidden;
            return;
        }
    } else if !confirming && attract.idle.tick(time.delta()).just_finished() {
        if !attract.showing_scores {
            // Title, then a few seconds of gameplay, then the High Score Table.
            demo::start(&mut commands, &mut cursor, &tuning, &mut next);
            return;
        }
        attract.showing_scores = false;
        main_menu.active = true;
        **panels.p0() = Visibility::Inherited;
        **panels.p1() = Visibility::Hidden;
        return;
    }

    if confirming {
        let cancel = controls.just_pressed(Button::B)
            || (controls.just_pressed(Button::A) && confirm_menu.selected_item() == NO);
        if controls.just_pressed(Button::A) && confirm_menu.selected_item() == YES {
            start_new_run(&mut commands, &tuning, &mut next);
        } else if cancel {
            confirm_menu.active = false;
            main_menu.active = true;
            **panels.p0() = Visibility::Inherited;
            **panels.p2() = Visibility::Hidden;
        }
        return;
    }

    if !controls.just_pressed(Button::A) && !controls.just_pressed(Button::Start) {
        return;
    }
    match main_menu.selected_item() {
        CONTINUE => {
            if !continue_run(&mut commands, &mut next) {
                // The Save was unreadable; rebuild the menu without it.
                next.set(Screen::Title);
            }
        }
        NEW_RUN if save_exists() => {
            main_menu.active = false;
            confirm_menu.active = true;
            confirm_menu.selected = 0;
            **panels.p0() = Visibility::Hidden;
            **panels.p2() = Visibility::Inherited;
        }
        NEW_RUN => start_new_run(&mut commands, &tuning, &mut next),
        PRACTICE => next.set(Screen::PracticeSelect),
        _ => {}
    }
}

pub struct TitlePlugin;

impl Plugin for TitlePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Title), spawn)
            .add_systems(Update, update.run_if(in_state(Screen::Title)));
    }
}
