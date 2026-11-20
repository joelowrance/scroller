//! A vertical arcade menu: Up/Down to move, the selected line marked `> <`.

use bevy::prelude::*;

use crate::font::{CELL_H, PixelText, text_at};
use crate::input::{Button, Controls};

const LINE_SPACING: f32 = CELL_H as f32 + 4.0;

#[derive(Component)]
#[require(Transform, Visibility)]
pub struct Menu {
    pub items: Vec<&'static str>,
    pub selected: usize,
    /// Only the active menu responds to input.
    pub active: bool,
    pub color: Color,
}

#[derive(Component)]
struct MenuLine(usize);

impl Menu {
    pub fn new(items: Vec<&'static str>, color: Color) -> Self {
        Self {
            items,
            selected: 0,
            active: true,
            color,
        }
    }

    pub fn selected_item(&self) -> &'static str {
        self.items[self.selected]
    }

    /// Spawns the menu with its first line at `y`.
    pub fn spawn(self, commands: &mut Commands, y: f32) -> Entity {
        let lines: Vec<usize> = (0..self.items.len()).collect();
        commands
            .spawn((self, Transform::from_xyz(0.0, y, 0.0)))
            .with_children(|parent| {
                for i in lines {
                    parent.spawn((
                        MenuLine(i),
                        text_at(
                            PixelText::new("", Color::WHITE),
                            0.0,
                            -(i as f32) * LINE_SPACING,
                            1,
                        ),
                    ));
                }
            })
            .id()
    }
}

fn navigate(controls: Res<Controls>, mut menus: Query<&mut Menu>) {
    for mut menu in &mut menus {
        if !menu.active || menu.items.is_empty() {
            continue;
        }
        let len = menu.items.len();
        if controls.just_pressed(Button::Down) {
            menu.selected = (menu.selected + 1) % len;
        }
        if controls.just_pressed(Button::Up) {
            menu.selected = (menu.selected + len - 1) % len;
        }
    }
}

fn render(
    menus: Query<(&Menu, &Children), Changed<Menu>>,
    mut lines: Query<(&MenuLine, &mut PixelText)>,
) {
    for (menu, children) in &menus {
        for child in children {
            let Ok((line, mut text)) = lines.get_mut(*child) else {
                continue;
            };
            let label = menu.items[line.0];
            let selected = line.0 == menu.selected;
            text.text = if selected {
                format!("> {label} <")
            } else {
                label.to_string()
            };
            text.color = if selected {
                menu.color
            } else {
                Color::srgb(0.6, 0.6, 0.6)
            };
        }
    }
}

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (navigate, render).chain());
    }
}
