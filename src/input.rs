//! One control scheme for every Cabinet: four directions, A, B and Start,
//! from keyboard or gamepad.
//!
//! | Button | Keys                 | Gamepad                 |
//! |--------|----------------------|-------------------------|
//! | Move   | WASD, arrow keys     | D-pad, left stick       |
//! | A      | Space, Z             | South (A / Cross)       |
//! | B      | `;`, X               | East or West            |
//! | Start  | Enter, Escape        | Start                   |

use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Button {
    Up,
    Down,
    Left,
    Right,
    A,
    B,
    Start,
}

const ALL: [Button; 7] = [
    Button::Up,
    Button::Down,
    Button::Left,
    Button::Right,
    Button::A,
    Button::B,
    Button::Start,
];

/// This frame's state of every [`Button`], merged across all devices.
#[derive(Resource, Default)]
pub struct Controls {
    held: u8,
    previous: u8,
}

impl Controls {
    pub fn held(&self, button: Button) -> bool {
        self.held & bit(button) != 0
    }

    pub fn just_pressed(&self, button: Button) -> bool {
        self.held & !self.previous & bit(button) != 0
    }

    /// Whether any button was pressed this frame.
    pub fn any_just_pressed(&self) -> bool {
        self.held & !self.previous != 0
    }

    /// Held direction as a vector (x right, y up), not normalized.
    pub fn direction(&self) -> Vec2 {
        let axis = |neg, pos| (self.held(pos) as i8 - self.held(neg) as i8) as f32;
        Vec2::new(
            axis(Button::Left, Button::Right),
            axis(Button::Down, Button::Up),
        )
    }
}

fn bit(button: Button) -> u8 {
    1 << button as u8
}

const STICK_THRESHOLD: f32 = 0.5;

fn keys_for(button: Button) -> &'static [KeyCode] {
    match button {
        Button::Up => &[KeyCode::KeyW, KeyCode::ArrowUp],
        Button::Down => &[KeyCode::KeyS, KeyCode::ArrowDown],
        Button::Left => &[KeyCode::KeyA, KeyCode::ArrowLeft],
        Button::Right => &[KeyCode::KeyD, KeyCode::ArrowRight],
        Button::A => &[KeyCode::Space, KeyCode::KeyZ],
        Button::B => &[KeyCode::Semicolon, KeyCode::KeyX],
        Button::Start => &[KeyCode::Enter, KeyCode::Escape],
    }
}

fn gamepad_held(pad: &Gamepad, button: Button) -> bool {
    let stick = pad.left_stick();
    match button {
        Button::Up => pad.pressed(GamepadButton::DPadUp) || stick.y > STICK_THRESHOLD,
        Button::Down => pad.pressed(GamepadButton::DPadDown) || stick.y < -STICK_THRESHOLD,
        Button::Left => pad.pressed(GamepadButton::DPadLeft) || stick.x < -STICK_THRESHOLD,
        Button::Right => pad.pressed(GamepadButton::DPadRight) || stick.x > STICK_THRESHOLD,
        Button::A => pad.pressed(GamepadButton::South),
        Button::B => pad.pressed(GamepadButton::East) || pad.pressed(GamepadButton::West),
        Button::Start => pad.pressed(GamepadButton::Start),
    }
}

fn read_controls(
    keyboard: Res<ButtonInput<KeyCode>>,
    gamepads: Query<&Gamepad>,
    mut controls: ResMut<Controls>,
) {
    let mut held = 0;
    for button in ALL {
        let keys = keys_for(button).iter().copied();
        // A tap that went down and up within one frame still counts as a press.
        let down = keyboard.any_pressed(keys.clone())
            || keyboard.any_just_pressed(keys)
            || gamepads.iter().any(|pad| gamepad_held(pad, button));
        if down {
            held |= bit(button);
        }
    }
    controls.previous = controls.held;
    controls.held = held;
}

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Controls>()
            .add_systems(PreUpdate, read_controls.after(bevy::input::InputSystems));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn just_pressed_only_on_the_first_frame() {
        let mut c = Controls {
            held: bit(Button::A),
            previous: 0,
        };
        assert!(c.just_pressed(Button::A));
        c.previous = c.held;
        assert!(c.held(Button::A));
        assert!(!c.just_pressed(Button::A));
    }

    #[test]
    fn opposite_directions_cancel() {
        let c = Controls {
            held: bit(Button::Left) | bit(Button::Right) | bit(Button::Up),
            previous: 0,
        };
        assert_eq!(c.direction(), Vec2::new(0.0, 1.0));
    }
}
