//! The 1 → 2 Cabinet Transition: the Wren banks upward, the view turns 90°
//! to look up the shaft, then tilts to look down on the cloud sea as the
//! Wren clears the shaft mouth (`docs/cabinets/2-vertical-shooter.md`,
//! "The 1 → 2 view change"). 5.5 seconds:
//!
//! | Seconds   | What happens |
//! |-----------|--------------|
//! | 0 – 1     | The Wren glides from where Stage 1-2 left it. The shaft's walls close in from above and below, and speed streaks fade in. "CABINET 1 CLEAR!" |
//! | 0.7 – 2.2 | The Wren pitches its nose up (the bank) and settles again. |
//! | 1 – 2.6   | The scene turns 90°: ceiling and floor become the shaft's side walls, and the scroll runs downward. |
//! | 2.6 – 3.8 | The Wren clears the shaft mouth. The walls fall away and fade out with the streaks; the view is now overhead. The cloud sea fades in below in Cabinet 2's palette, the backdrop goes from Cabinet 1's to Cabinet 2's, the Wren takes Cabinet 2's colours, and its shadow appears on the clouds. |
//! | 3.6 – 5.5 | "CABINET 2 / VERTICAL SHOOTER". From 4.6 s the Wren speeds off the top. |
//!
//! Then Cabinet 2's Stage Card. A music sting, [`Song::BankUp`], runs under it.
//!
//! The turn rotates a scene root, not the camera, so the captions stay
//! upright and nothing has to be put back afterwards. The shaft is drawn in
//! Cabinet 1's palette and the clouds in Cabinet 2's, so the palette change
//! is the shaft giving way to the sky.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::color::Mix;
use bevy::prelude::*;

use super::into_cabinet_2;
use crate::audio::{Song, TransitionMusic};
use crate::cabinets::CABINETS;
use crate::cabinets::side_shooter::WrenLastSeen;
use crate::cabinets::side_shooter::hero::wren_visual;
use crate::display::{Backdrop, HALF_H, HALF_W};
use crate::flow::{CardReason, Screen};
use crate::font::{PixelText, text_at};
use crate::sprites::Sprites;
use crate::sprites::palette::{ACCENT, EXTRA, PALETTES};

/// Length of the whole sequence, in seconds.
pub(super) const SECONDS: f32 = 5.5;
/// Where the Wren cruises (scene coordinates). After the turn this is
/// (0, −80) on screen, Cabinet 2's start position.
const CRUISE: Vec2 = Vec2::new(-80.0, 0.0);
/// The shaft's inner edges, above and below the Wren before the turn.
const WALL_Y: f32 = 92.0;
/// How far the walls start further out before closing in.
const WALL_CLOSE_IN: f32 = 48.0;
/// How far the walls fall away as the Wren clears the shaft mouth.
const WALL_FALL_AWAY: f32 = 120.0;
/// Thick enough to cover the screen corners while the scene turns.
const WALL_DEPTH: f32 = 140.0;
/// Size of one `shaft_wall` sprite.
const BLOCK: Vec2 = Vec2::new(32.0, 12.0);
const BLOCK_COUNT: usize = 16;
/// Streaks fill a square this far out from the centre, which covers the
/// screen at every angle of the turn.
const FIELD: f32 = 210.0;
const STREAK_COUNT: u32 = 72;
const CLOUD_COUNT: u32 = 18;
const BANK_DEGREES: f32 = 40.0;
/// Where the Wren's shadow ends up relative to it, on screen: down and to
/// the right, as if lit from the upper left.
const SHADOW_OFFSET: Vec2 = Vec2::new(8.0, -10.0);
/// The Wren takes Cabinet 2's colours at this moment.
const RECOLOUR_AT: f32 = 3.1;

/// 0 before `from`, 1 after `to`, linear between.
fn phase(t: f32, from: f32, to: f32) -> f32 {
    ((t - from) / (to - from)).clamp(0.0, 1.0)
}

fn smooth(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

/// The scene's turn at `t`, in radians (0 to 90° anticlockwise).
pub(super) fn turn(t: f32) -> f32 {
    smooth(phase(t, 1.0, 2.6)) * FRAC_PI_2
}

/// How far the Wren has cleared the shaft mouth: 0 inside, 1 in open sky.
fn clear_of_shaft(t: f32) -> f32 {
    smooth(phase(t, 2.6, 3.8))
}

/// The backdrop: Cabinet 1's, then Cabinet 2's once out of the shaft.
pub(super) fn backdrop_at(t: f32) -> Color {
    CABINETS[0]
        .backdrop
        .mix(&CABINETS[1].backdrop, clear_of_shaft(t))
}

/// The shaft wall's plain body behind its sprite edge: a darker Rust.
fn wall_body_color() -> Color {
    PALETTES[0].color(EXTRA).mix(&CABINETS[0].backdrop, 0.35)
}

/// The Wren's position and nose angle in scene coordinates.
pub(super) fn wren_pose(t: f32, from: Vec2) -> (Vec2, f32) {
    let glide = from.lerp(CRUISE, ease_out(phase(t, 0.0, 1.0)));
    let bank = (PI * phase(t, 0.7, 2.2)).sin();
    let exit = phase(t, 4.6, SECONDS).powi(2) * 340.0;
    let at = glide + Vec2::new(exit, 10.0 * bank);
    (at, bank * BANK_DEGREES.to_radians())
}

/// Scroll speed up the shaft, in pixels per second.
fn scroll_speed(t: f32) -> f32 {
    140.0 + 360.0 * phase(t, 4.6, SECONDS).powi(2)
}

/// A repeatable pseudo-random number in 0..1.
fn noise(i: u32, salt: u32) -> f32 {
    let mut x = i.wrapping_mul(0x9E37_79B9) ^ salt.wrapping_mul(0x85EB_CA6B);
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x & 0xFFFF) as f32 / 65535.0
}

/// Wraps `x` into `-half..half`.
fn wrap(x: f32, half: f32) -> f32 {
    (x + half).rem_euclid(half * 2.0) - half
}

#[derive(Resource)]
struct BankUp {
    elapsed: f32,
    scroll: f32,
    from: Vec2,
    recoloured: bool,
}

/// The scene root: everything in the shaft is its child, and it turns.
#[derive(Component)]
struct Shaft;

#[derive(Component)]
enum Part {
    Streak {
        layer: usize,
        x: f32,
    },
    /// One block of wall: its sprite edge, or the plain body behind it.
    Wall {
        slot: usize,
        side: f32,
        body: bool,
    },
    Wren,
    /// Screen space from here on: the overhead view below the Wren.
    Cloud {
        layer: usize,
        y: f32,
    },
    Shadow,
}

/// Text shown from `from` to `to` seconds.
#[derive(Component)]
struct Caption {
    from: f32,
    to: f32,
}

fn spawn(
    mut commands: Commands,
    sprites: Res<Sprites>,
    last_seen: Res<WrenLastSeen>,
    mut backdrop: ResMut<Backdrop>,
    mut music: ResMut<TransitionMusic>,
) {
    backdrop.0 = backdrop_at(0.0);
    music.0 = Some(Song::BankUp);
    let from = last_seen
        .0
        .clamp(Vec2::new(-HALF_W, -HALF_H), Vec2::new(HALF_W, HALF_H));
    commands.insert_resource(BankUp {
        elapsed: 0.0,
        scroll: 0.0,
        from,
        recoloured: false,
    });
    // Faded in by `update`, so the first frame matches the Stage's last.
    let hidden = |mut sprite: Sprite| {
        sprite.color = Color::NONE;
        sprite
    };

    commands
        .spawn((
            Shaft,
            Transform::default(),
            Visibility::default(),
            DespawnOnExit(Screen::CabinetTransition),
        ))
        .with_children(|scene| {
            for i in 0..STREAK_COUNT {
                let layer = (noise(i, 1) * 3.0) as usize % 3;
                let x = (noise(i, 2) * 2.0 - 1.0) * FIELD;
                let y = (noise(i, 3) * 2.0 - 1.0) * FIELD;
                scene.spawn((
                    Part::Streak { layer, x },
                    hidden(sprites.frame("stripe", 0, layer)),
                    Transform::from_xyz(x, y, 1.0 + layer as f32 * 0.1),
                ));
            }
            for slot in 0..BLOCK_COUNT {
                for side in [1.0, -1.0] {
                    let mut edge = sprites.sprite("shaft_wall", 0);
                    // The sprite is drawn as the floor; flip it for the ceiling.
                    edge.flip_y = side > 0.0;
                    scene.spawn((
                        Part::Wall {
                            slot,
                            side,
                            body: false,
                        },
                        edge,
                        Transform::from_xyz(0.0, 0.0, 2.1),
                    ));
                    scene.spawn((
                        Part::Wall {
                            slot,
                            side,
                            body: true,
                        },
                        Sprite::from_color(wall_body_color(), Vec2::new(BLOCK.x, WALL_DEPTH)),
                        Transform::from_xyz(0.0, 0.0, 2.0),
                    ));
                }
            }
            scene.spawn((
                Part::Wren,
                wren_visual(&sprites),
                Transform::from_translation(from.extend(5.0)),
            ));
        });

    for i in 0..CLOUD_COUNT {
        let layer = i as usize % 2;
        let x = (noise(i, 4) * 2.0 - 1.0) * (HALF_W + 16.0);
        let y = (noise(i, 5) * 2.0 - 1.0) * (HALF_H + 16.0);
        commands.spawn((
            Part::Cloud { layer, y },
            hidden(sprites.frame("transition_cloud", 1, layer)),
            Transform::from_xyz(x.round(), y.round(), 0.5 + layer as f32 * 0.1),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    }
    commands.spawn((
        Part::Shadow,
        hidden(sprites.sprite("hero", 1)),
        Transform::from_xyz(0.0, 0.0, 3.0).with_scale(Vec3::splat(0.75)),
        DespawnOnExit(Screen::CabinetTransition),
    ));

    let caption = |text: PixelText, y: f32, scale: u32, from: f32, to: f32| {
        (
            text_at(text, 0.0, y, scale),
            Visibility::Hidden,
            Caption { from, to },
            DespawnOnExit(Screen::CabinetTransition),
        )
    };
    let (accent_1, accent_2) = (PALETTES[0].color(ACCENT), PALETTES[1].color(ACCENT));
    commands.spawn(caption(
        PixelText::new("CABINET 1 CLEAR!", accent_1),
        44.0,
        2,
        0.15,
        2.0,
    ));
    commands.spawn(caption(
        PixelText::new("CABINET 2", accent_2),
        48.0,
        2,
        3.6,
        SECONDS,
    ));
    commands.spawn(caption(
        PixelText::new(CABINETS[1].genre, accent_2),
        28.0,
        1,
        3.6,
        SECONDS,
    ));
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    sprites: Res<Sprites>,
    mut state: ResMut<BankUp>,
    mut backdrop: ResMut<Backdrop>,
    mut shaft: Query<&mut Transform, (With<Shaft>, Without<Part>)>,
    mut parts: Query<(&mut Part, &mut Transform, &mut Sprite)>,
    mut captions: Query<(&Caption, &mut Visibility)>,
    mut next: ResMut<NextState<Screen>>,
) {
    let dt = time.delta_secs();
    state.elapsed += dt;
    let t = state.elapsed;
    state.scroll += scroll_speed(t) * dt;
    backdrop.0 = backdrop_at(t);
    let angle = turn(t);
    for mut transform in &mut shaft {
        transform.rotation = Quat::from_rotation_z(angle);
    }

    let clear = clear_of_shaft(t);
    let shaft_alpha = 1.0 - clear;
    let streak_alpha = phase(t, 0.0, 0.5) * (1.0 - phase(t, 2.6, 3.4));
    let sky_alpha = smooth(phase(t, 2.8, 3.8));
    let wall_y = WALL_Y
        + WALL_CLOSE_IN * (1.0 - ease_out(phase(t, 0.0, 1.0)))
        + WALL_FALL_AWAY * clear * clear;
    let period = BLOCK.x * BLOCK_COUNT as f32;
    let (wren_at, nose) = wren_pose(t, state.from);
    let wren_on_screen = Vec2::from_angle(angle).rotate(wren_at);
    let recolour = !state.recoloured && t >= RECOLOUR_AT;
    state.recoloured |= recolour;

    for (mut part, mut transform, mut sprite) in &mut parts {
        match &mut *part {
            Part::Streak { layer, x } => {
                *x = wrap(*x - scroll_speed(t) * [0.5, 1.0, 1.8][*layer] * dt, FIELD);
                transform.translation.x = x.round();
                sprite.color = Color::WHITE.with_alpha(streak_alpha);
            }
            Part::Wall { slot, side, body } => {
                let x = wrap(*slot as f32 * BLOCK.x - state.scroll, period / 2.0);
                transform.translation.x = x.round();
                let depth = if *body {
                    BLOCK.y + WALL_DEPTH / 2.0
                } else {
                    BLOCK.y / 2.0
                };
                transform.translation.y = (*side * (wall_y + depth)).round();
                let tint = if *body {
                    wall_body_color()
                } else {
                    Color::WHITE
                };
                sprite.color = tint.with_alpha(shaft_alpha);
            }
            Part::Wren => {
                transform.translation.x = wren_at.x.round();
                transform.translation.y = wren_at.y.round();
                transform.rotation = Quat::from_rotation_z(nose);
                if recolour {
                    // Cabinet 2's Wren is Cabinet 1's turned nose up, in
                    // Cabinet 2's colours. Keep the engine flicker's frame.
                    let frame = sprite.texture_atlas.as_ref().map_or(0, |a| a.index);
                    *sprite = sprites.frame("hero", 1, frame);
                }
            }
            Part::Cloud { layer, y } => {
                *y = wrap(*y - [18.0, 30.0][*layer] * dt, HALF_H + 16.0);
                transform.translation.y = y.round();
                sprite.color = Color::WHITE.with_alpha(sky_alpha);
            }
            Part::Shadow => {
                let at = wren_on_screen + SHADOW_OFFSET * clear;
                transform.translation.x = at.x.round();
                transform.translation.y = at.y.round();
                transform.rotation = Quat::from_rotation_z(angle + nose);
                sprite.color = Color::BLACK.with_alpha(0.45 * sky_alpha);
            }
        }
    }
    for (caption, mut visibility) in &mut captions {
        visibility.set_if_neq(if (caption.from..caption.to).contains(&t) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if t >= SECONDS {
        commands.insert_resource(CardReason::AfterTransition);
        next.set(Screen::StageCard);
    }
}

pub(super) struct BankUpPlugin;

impl Plugin for BankUpPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::CabinetTransition),
            spawn.run_if(into_cabinet_2),
        )
        .add_systems(
            Update,
            update
                .run_if(in_state(Screen::CabinetTransition))
                .run_if(resource_exists::<BankUp>),
        )
        .add_systems(
            OnExit(Screen::CabinetTransition),
            |mut commands: Commands| {
                commands.remove_resource::<BankUp>();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_scene_turns_a_quarter_turn_and_stays_there() {
        assert_eq!(turn(0.0), 0.0);
        assert_eq!(turn(0.99), 0.0);
        assert!((turn(2.6) - FRAC_PI_2).abs() < 1e-6);
        assert!((turn(SECONDS) - FRAC_PI_2).abs() < 1e-6);
        let mut last = 0.0;
        for i in 0..=100 {
            let a = turn(i as f32 * SECONDS / 100.0);
            assert!(a >= last, "only ever turns one way");
            last = a;
        }
    }

    #[test]
    fn it_starts_on_cabinet_1s_backdrop_and_ends_on_cabinet_2s() {
        assert_eq!(backdrop_at(0.0), CABINETS[0].backdrop);
        assert_eq!(backdrop_at(2.6), CABINETS[0].backdrop, "still in the shaft");
        assert_eq!(backdrop_at(3.8), CABINETS[1].backdrop);
        assert_eq!(backdrop_at(SECONDS), CABINETS[1].backdrop);
    }

    #[test]
    fn the_wren_starts_where_the_stage_left_it_banks_and_leaves_upward() {
        let from = Vec2::new(30.0, -50.0);
        let (at, nose) = wren_pose(0.0, from);
        assert_eq!((at, nose), (from, 0.0));
        let (at, nose) = wren_pose(1.45, from);
        assert!(nose > 30f32.to_radians(), "nose up mid-bank");
        assert!(at.y > CRUISE.y);
        // Level at Cabinet 2's start position once the view is overhead.
        let (at, nose) = wren_pose(4.0, from);
        assert!(at.distance(CRUISE) < 1e-4 && nose.abs() < 1e-4);
        let on_screen = Vec2::from_angle(turn(4.0)).rotate(at);
        assert!(
            on_screen.distance(Vec2::new(0.0, -80.0)) < 1e-3,
            "{on_screen}"
        );
        // Scene +x is screen up after the turn; by the end it is off the top.
        let (at, _) = wren_pose(SECONDS, from);
        let on_screen = Vec2::from_angle(turn(SECONDS)).rotate(at);
        assert!(on_screen.y > HALF_H + 8.0, "{on_screen}");
    }

    #[test]
    fn noise_is_spread_over_0_to_1() {
        let values: Vec<f32> = (0..200).map(|i| noise(i, 7)).collect();
        assert!(values.iter().all(|v| (0.0..=1.0).contains(v)));
        let low = values.iter().filter(|v| **v < 0.5).count();
        assert!((70..130).contains(&low), "{low}");
    }
}
