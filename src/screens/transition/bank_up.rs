//! The 1 → 2 Cabinet Transition: the Wren banks upward, the view turns 90°
//! to look up the shaft, then pulls back to look down on the cloud sea as
//! the Wren clears the shaft mouth (`docs/cabinets/2-vertical-shooter.md`,
//! "The 1 → 2 view change"). 5.5 seconds; the spans are the constants below:
//!
//! - [`GLIDE`]: the Wren eases from where Stage 1-2 left it to the cruise
//!   spot, the shaft's walls close in from above and below, and speed streaks
//!   fade in. "CABINET 1 CLEAR!" ([`CAPTION_1`]).
//! - [`BANK`]: the Wren pitches its nose up and settles again.
//! - [`TURN`]: the scene turns 90°. Ceiling and floor become the shaft's side
//!   walls, and the scroll runs downward.
//! - [`RECOLOUR`]: the walls and the Wren take Cabinet 2's colours.
//! - [`CLEAR`]: the Wren clears the shaft mouth. The shaft shrinks and falls
//!   away as the camera pulls up and back, the streaks fade ([`STREAKS_OUT`]),
//!   and the backdrop turns to Dusk. The Plum and Steel cloud sea fades in
//!   below ([`SKY`]) with the Wren's Dusk shadow on it.
//! - [`CAPTION_2`]: "CABINET 2 / VERTICAL SHOOTER" over the clouds, with the
//!   Wren holding at Cabinet 2's start spot, (0, −80) on screen.
//!
//! Then Cabinet 2's Stage Card. A music sting, [`Song::BankUp`], runs under it.
//!
//! The turn rotates a scene root, not the camera, so the captions stay
//! upright and nothing has to be put back afterwards.

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::color::Mix;
use bevy::prelude::*;

use super::into_cabinet_2;
use crate::audio::{Song, TransitionMusic};
use crate::cabinets::CABINETS;
use crate::cabinets::side_shooter::WrenLastSeen;
use crate::cabinets::side_shooter::hero::wren_visual;
use crate::cabinets::side_shooter::palette as one;
use crate::cabinets::vertical_shooter::palette as two;
use crate::display::{Backdrop, HALF_H, HALF_W};
use crate::flow::{CardReason, Screen};
use crate::font::{CELL_H, CELL_W, PixelText, text_at};
use crate::sprites::Sprites;

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

/// Length of the whole sequence, in seconds.
pub(super) const SECONDS: f32 = 5.5;
pub(super) const GLIDE: Span = Span(0.0, 1.0);
pub(super) const BANK: Span = Span(0.7, 2.2);
pub(super) const TURN: Span = Span(1.0, 2.6);
pub(super) const RECOLOUR: Span = Span(2.2, 3.2);
pub(super) const CLEAR: Span = Span(2.6, 3.8);
pub(super) const STREAKS_OUT: Span = Span(2.6, 3.4);
pub(super) const SKY: Span = Span(2.8, 3.8);
pub(super) const CAPTION_1: Span = Span(0.15, 2.0);
pub(super) const CAPTION_2: Span = Span(3.6, SECONDS);

// The order the beats rely on, checked when compiling: the view goes
// overhead only after the turn, and the captions never share the screen
// with the streaks or each other.
const _: () = assert!(CLEAR.0 >= TURN.1 && SKY.0 >= TURN.1);
const _: () = assert!(CAPTION_2.0 >= STREAKS_OUT.1 && CAPTION_1.1 <= CAPTION_2.0);
const _: () = assert!(CAPTION_2.1 >= SECONDS);

/// Where the Wren cruises (scene coordinates). After the turn this is
/// (0, −80) on screen, Cabinet 2's start position.
const CRUISE: Vec2 = Vec2::new(-80.0, 0.0);
/// The shaft's inner edges, above and below the Wren before the turn.
const WALL_Y: f32 = 92.0;
/// How far the walls start further out before closing in.
const WALL_CLOSE_IN: f32 = 48.0;
/// How far the walls fall away as the Wren clears the shaft mouth.
const WALL_FALL_AWAY: f32 = 60.0;
/// How much the shaft shrinks as the camera pulls back from it.
const PULL_BACK: f32 = 0.35;
/// Thick enough to cover the screen corners while the scene turns.
const WALL_DEPTH: f32 = 140.0;
/// Size of one `shaft_wall` sprite.
const BLOCK: Vec2 = Vec2::new(32.0, 12.0);
const BLOCK_COUNT: usize = 16;
/// Streaks fill a square this far out from the centre, which covers the
/// screen at every angle of the turn.
const FIELD: f32 = 210.0;
const STREAK_COUNT: u32 = 72;
const SCROLL_SPEED: f32 = 140.0;
const CLOUD_COUNT: u32 = 24;
/// How fast the far and near cloud banks drift down, in pixels per second.
const CLOUD_SPEED: [f32; 2] = [18.0, 30.0];
const BANK_DEGREES: f32 = 40.0;
/// The Wren's shadow on the clouds, relative to the Wren (the design doc's
/// 6 px, down and to the right).
const SHADOW_OFFSET: Vec2 = Vec2::new(6.0, -6.0);

fn smooth(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

/// The scene's turn at `t`, in radians (0 to 90° anticlockwise).
pub(super) fn turn(t: f32) -> f32 {
    smooth(TURN.at(t)) * FRAC_PI_2
}

/// How far the Wren has cleared the shaft mouth: 0 inside, 1 in open sky.
fn clear_of_shaft(t: f32) -> f32 {
    smooth(CLEAR.at(t))
}

/// The backdrop: Cabinet 1's, then Cabinet 2's once out of the shaft.
pub(super) fn backdrop_at(t: f32) -> Color {
    CABINETS[0]
        .backdrop
        .mix(&CABINETS[1].backdrop, clear_of_shaft(t))
}

/// The shaft wall's plain body behind its sprite edge, in each Cabinet's
/// colours: a darker Rust, then Plum.
fn wall_body_color(t: f32) -> Color {
    let rust = one::RUST.mix(&one::NIGHT, 0.35);
    rust.mix(&two::PLUM, smooth(RECOLOUR.at(t)))
}

/// The Wren's position and nose angle in scene coordinates. It starts and
/// stops gently, so a Wren sitting still at the end of the Stage doesn't
/// jump.
pub(super) fn wren_pose(t: f32, from: Vec2) -> (Vec2, f32) {
    let glide = from.lerp(CRUISE, smooth(GLIDE.at(t)));
    let bank = (PI * BANK.at(t)).sin();
    let at = glide + Vec2::new(0.0, 10.0 * bank);
    (at, bank * BANK_DEGREES.to_radians())
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

#[derive(Clone, Copy, PartialEq)]
enum WallPiece {
    /// The `shaft_wall` sprite in Cabinet `n`'s palette.
    Edge(usize),
    /// The plain body behind it.
    Body,
}

#[derive(Component)]
enum Part {
    Streak {
        layer: usize,
        x: f32,
    },
    Wall {
        slot: usize,
        side: f32,
        piece: WallPiece,
    },
    Wren,
    /// Screen space from here on: the overhead view below the Wren.
    Cloud {
        layer: usize,
        y: f32,
    },
    Shadow,
}

/// Text shown during a span, with the backdrop-coloured plate behind it.
#[derive(Component)]
struct Caption(Span);

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
                    for cabinet in [0, 1] {
                        let mut edge = sprites.sprite("shaft_wall", cabinet);
                        // The sprite is drawn as the floor; flip it for the
                        // ceiling.
                        edge.flip_y = side > 0.0;
                        scene.spawn((
                            Part::Wall {
                                slot,
                                side,
                                piece: WallPiece::Edge(cabinet),
                            },
                            edge,
                            Transform::from_xyz(0.0, 0.0, 2.1 + cabinet as f32 * 0.01),
                        ));
                    }
                    scene.spawn((
                        Part::Wall {
                            slot,
                            side,
                            piece: WallPiece::Body,
                        },
                        Sprite::from_color(wall_body_color(0.0), Vec2::new(BLOCK.x, WALL_DEPTH)),
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

    // Two banks are placed to drift under the Wren while "CABINET 2" shows,
    // so its shadow is seen on them; the rest are scattered.
    let under_wren = |layer: usize, at: f32| {
        let below = Vec2::new(0.0, -80.0) + SHADOW_OFFSET;
        Vec2::new(below.x, below.y + CLOUD_SPEED[layer] * at)
    };
    let placed = [(0, under_wren(0, 4.3)), (1, under_wren(1, 5.1))];
    let scattered = (0..CLOUD_COUNT).map(|i| {
        let x = (noise(i, 4) * 2.0 - 1.0) * (HALF_W + 16.0);
        let y = (noise(i, 5) * 2.0 - 1.0) * (HALF_H + 16.0);
        (i as usize % 2, Vec2::new(x, y))
    });
    for (layer, at) in placed.into_iter().chain(scattered) {
        let (x, y) = (at.x, wrap(at.y, HALF_H + 16.0));
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
        Transform::from_xyz(0.0, 0.0, 3.0),
        DespawnOnExit(Screen::CabinetTransition),
    ));

    let mut caption = |text: &str, color: Color, y: f32, scale: u32, span: Span| {
        // A plate one pixel bigger than the text all round, in the backdrop
        // colour, so nothing behind can cut through the letters.
        let size = Vec2::new(
            (text.chars().count() as u32 * CELL_W * scale) as f32 + 2.0,
            (CELL_H * scale) as f32 + 2.0,
        );
        commands.spawn((
            text_at(PixelText::new(text, color), 0.0, y, scale),
            Visibility::Hidden,
            Caption(span),
            DespawnOnExit(Screen::CabinetTransition),
        ));
        // Not a child: the font rebuilds a text's children when it changes.
        commands.spawn((
            Sprite::from_color(backdrop_at(span.0), size),
            Transform::from_xyz(0.0, y.round(), 9.5),
            Visibility::Hidden,
            Caption(span),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    };
    caption("CABINET 1 CLEAR!", one::SPARK, 44.0, 2, CAPTION_1);
    caption("CABINET 2", two::ORCHID, 48.0, 2, CAPTION_2);
    caption(CABINETS[1].genre, two::ORCHID, 28.0, 1, CAPTION_2);
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
    state.scroll += SCROLL_SPEED * dt;
    backdrop.0 = backdrop_at(t);
    let angle = turn(t);
    let clear = clear_of_shaft(t);
    let scale = 1.0 - PULL_BACK * clear;
    for mut transform in &mut shaft {
        transform.rotation = Quat::from_rotation_z(angle);
        transform.scale = Vec3::splat(scale);
    }

    let shaft_alpha = 1.0 - clear;
    let streak_alpha = GLIDE.at(t * 2.0) * (1.0 - STREAKS_OUT.at(t));
    let sky_alpha = smooth(SKY.at(t));
    let recolour = smooth(RECOLOUR.at(t));
    let wall_y =
        WALL_Y + WALL_CLOSE_IN * (1.0 - ease_out(GLIDE.at(t))) + WALL_FALL_AWAY * clear * clear;
    let period = BLOCK.x * BLOCK_COUNT as f32;
    let (wren_at, nose) = wren_pose(t, state.from);
    let wren_on_screen = Vec2::from_angle(angle).rotate(wren_at);
    let swap_wren = !state.recoloured && recolour >= 0.5;
    state.recoloured |= swap_wren;

    for (mut part, mut transform, mut sprite) in &mut parts {
        match &mut *part {
            Part::Streak { layer, x } => {
                *x = wrap(*x - SCROLL_SPEED * [0.5, 1.0, 1.8][*layer] * dt, FIELD);
                transform.translation.x = x.round();
                sprite.color = Color::WHITE.with_alpha(streak_alpha);
            }
            Part::Wall { slot, side, piece } => {
                let x = wrap(*slot as f32 * BLOCK.x - state.scroll, period / 2.0);
                transform.translation.x = x.round();
                let (depth, color) = match piece {
                    WallPiece::Body => (BLOCK.y + WALL_DEPTH / 2.0, wall_body_color(t)),
                    WallPiece::Edge(0) => (BLOCK.y / 2.0, Color::WHITE.with_alpha(1.0 - recolour)),
                    WallPiece::Edge(_) => (BLOCK.y / 2.0, Color::WHITE.with_alpha(recolour)),
                };
                transform.translation.y = (*side * (wall_y + depth)).round();
                sprite.color = color.with_alpha(color.alpha() * shaft_alpha);
            }
            Part::Wren => {
                // Scene coordinates shrink with the pull-back; the Wren
                // doesn't, so it ends at (0, −80) on screen at full size.
                transform.translation.x = (wren_at.x / scale).round();
                transform.translation.y = (wren_at.y / scale).round();
                transform.rotation = Quat::from_rotation_z(nose);
                transform.scale = Vec3::splat(1.0 / scale);
                if swap_wren {
                    // Cabinet 2's Wren is Cabinet 1's turned nose up, in
                    // Cabinet 2's colours. Keep the engine flicker's frame.
                    let frame = sprite.texture_atlas.as_ref().map_or(0, |a| a.index);
                    *sprite = sprites.frame("hero", 1, frame);
                }
            }
            Part::Cloud { layer, y } => {
                *y = wrap(*y - CLOUD_SPEED[*layer] * dt, HALF_H + 16.0);
                transform.translation.y = y.round();
                sprite.color = Color::WHITE.with_alpha(sky_alpha);
            }
            Part::Shadow => {
                let at = wren_on_screen + SHADOW_OFFSET;
                transform.translation.x = at.x.round();
                transform.translation.y = at.y.round();
                transform.rotation = Quat::from_rotation_z(angle + nose);
                sprite.color = two::DUSK.with_alpha(sky_alpha);
            }
        }
    }
    for (caption, mut visibility) in &mut captions {
        visibility.set_if_neq(if caption.0.contains(t) {
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

    fn on_screen(t: f32, from: Vec2) -> Vec2 {
        let (at, _) = wren_pose(t, from);
        Vec2::from_angle(turn(t)).rotate(at)
    }

    #[test]
    fn the_scene_turns_a_quarter_turn_and_stays_there() {
        assert_eq!(turn(0.0), 0.0);
        assert_eq!(turn(TURN.0), 0.0);
        assert!((turn(TURN.1) - FRAC_PI_2).abs() < 1e-6);
        assert!((turn(SECONDS) - FRAC_PI_2).abs() < 1e-6);
        let mut last = 0.0;
        for i in 0..=100 {
            let a = turn(i as f32 * SECONDS / 100.0);
            assert!(a >= last, "only ever turns one way");
            last = a;
        }
    }

    #[test]
    fn the_view_goes_overhead_only_after_the_turn() {
        assert_eq!(backdrop_at(0.0), CABINETS[0].backdrop);
        assert_eq!(
            backdrop_at(CLEAR.0),
            CABINETS[0].backdrop,
            "still in the shaft"
        );
        assert_eq!(backdrop_at(CLEAR.1), CABINETS[1].backdrop);
        assert_eq!(backdrop_at(SECONDS), CABINETS[1].backdrop);
        assert_eq!(wall_body_color(SECONDS), two::PLUM);
    }

    #[test]
    fn the_wren_starts_where_the_stage_left_it_and_hands_over_at_cabinet_2s_start() {
        let from = Vec2::new(30.0, -50.0);
        let (at, nose) = wren_pose(0.0, from);
        assert_eq!((at, nose), (from, 0.0));
        // It eases off: the first tenth of a second covers under 5%.
        let (early, _) = wren_pose(0.1, from);
        assert!(early.distance(from) < 0.05 * from.distance(CRUISE));
        let (at, nose) = wren_pose(1.45, from);
        assert!(nose > 30f32.to_radians(), "nose up mid-bank");
        assert!(at.y > CRUISE.y);
        for t in [CLEAR.1, SECONDS] {
            let (_, nose) = wren_pose(t, from);
            assert!(nose.abs() < 1e-4);
            assert!(
                on_screen(t, from).distance(Vec2::new(0.0, -80.0)) < 1e-3,
                "{}",
                on_screen(t, from)
            );
        }
    }

    #[test]
    fn noise_is_spread_over_0_to_1() {
        let values: Vec<f32> = (0..200).map(|i| noise(i, 7)).collect();
        assert!(values.iter().all(|v| (0.0..=1.0).contains(v)));
        let low = values.iter().filter(|v| **v < 0.5).count();
        assert!((70..130).contains(&low), "{low}");
    }
}
