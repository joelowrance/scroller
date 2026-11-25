//! The 3 → 4 Cabinet Transition: the Hero walks to the foot of the Spire,
//! jumps onto the tower's wall and starts to climb, and the view pans upward
//! into the climber (`docs/cabinets/3-run-and-gun.md`, Stage 2's end, and
//! `docs/cabinets/4-climber.md`, "Premise"). 5.5 seconds; the spans are the
//! constants below:
//!
//! - [`WALK`]: side-on, in Cabinet 3's colours. The Hero runs right along the
//!   ground to the wall that fills the right of the screen. "CABINET 3
//!   CLEAR!" ([`CAPTION_1`]).
//! - [`JUMP`]: the Hero leaps onto the wall and grabs hold.
//! - [`PAN`]: the Hero climbs and the view pans upward. The ground drops away
//!   below; the wall, the Hero and the backdrop cross-fade into Cabinet 4's
//!   colours ([`FADE`]). A landing, the first one up the tower, slides in
//!   from the top.
//! - [`HOP`]: the Hero hops onto the landing and stands. "CABINET 4 /
//!   VERTICAL CLIMBER" ([`CAPTION_2`]).
//!
//! Then Cabinet 4's Stage Card, whose Stage 1 opens on that landing. A music
//! sting, [`Song::Upswing`], runs under it.

use bevy::prelude::*;

use crate::audio::{Song, TransitionMusic};
use crate::cabinets::CABINETS;
use crate::display::{Backdrop, HALF_W};
use crate::flow::{CardReason, Screen};
use crate::font::{CELL_H, CELL_W, PixelText, text_at};
use crate::sprites::Sprites;

use super::Span;

/// Length of the whole sequence, in seconds.
pub(super) const SECONDS: f32 = 5.5;
const WALK: Span = Span(0.0, 1.3);
const JUMP: Span = Span(1.3, 1.9);
const PAN: Span = Span(1.9, 4.6);
const HOP: Span = Span(4.6, 5.0);
/// The colours cross-fade while the ground is already gone.
const FADE: Span = Span(2.4, 4.2);
const CAPTION_1: Span = Span(0.1, 1.6);
const CAPTION_2: Span = Span(4.2, SECONDS);

const _: () = assert!(WALK.1 <= JUMP.0 && JUMP.1 <= PAN.0 && PAN.1 <= HOP.0);
const _: () = assert!(HOP.1 <= SECONDS && CAPTION_1.1 <= PAN.0 && CAPTION_2.0 >= FADE.1);

/// Palette indices: Cabinet 3 and Cabinet 4.
const FROM: usize = 2;
const TO: usize = 3;

const WALL_W: f32 = 96.0;
const WALL_H: f32 = 240.0;
/// Where the wall's face is: it fills the right of the screen.
const WALL_X: f32 = HALF_W - WALL_W;
/// Top of Cabinet 3's ground, as in the 2 → 3 Transition.
const GROUND_Y: f32 = -96.0;
/// The Hero sprite is 12 x 20.
const HERO_HALF_H: f32 = 10.0;
/// Where the Hero starts, and where it stops short of the wall.
const START_X: f32 = -80.0;
const STOP_X: f32 = WALL_X - 8.0;
/// Where the Hero hangs on the wall, in x and on screen in y.
const HANG_X: f32 = WALL_X - 6.0;
const HANG_Y: f32 = -20.0;
/// Where the Hero stands on the landing at the end (its feet on the top).
const STAND: Vec2 = Vec2::new(WALL_X - 20.0, -50.0);
/// How far the view rises in all.
const RISE: f32 = 300.0;
/// The landing: a row of ledge blocks beside the wall, 8 px thick.
const LANDING_BLOCKS: i32 = 6;
const BLOCK: f32 = 8.0;
const LANDING_Y: f32 = STAND.y - HERO_HALF_H - BLOCK / 2.0;

fn smooth(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// How far the view has risen.
fn rise(t: f32) -> f32 {
    RISE * smooth(PAN.at(t))
}

/// 0 in Cabinet 3's colours, 1 in Cabinet 4's.
fn fade(t: f32) -> f32 {
    smooth(FADE.at(t))
}

fn backdrop_at(t: f32) -> Color {
    CABINETS[FROM].backdrop.mix(&CABINETS[TO].backdrop, fade(t))
}

/// The Hero's centre on screen and its frame of `c3_hero`.
fn hero_pose(t: f32) -> (Vec2, usize) {
    let ground = GROUND_Y + HERO_HALF_H;
    if t < WALK.1 {
        let x = START_X + (STOP_X - START_X) * WALK.at(t);
        return (Vec2::new(x, ground), 1 + (t / 0.1) as usize % 4);
    }
    if t < JUMP.1 {
        let k = JUMP.at(t);
        let x = STOP_X + (HANG_X - STOP_X) * k;
        // Up in an arc: over the top, then in against the wall.
        let y = ground + (HANG_Y - ground) * k + 14.0 * (k * std::f32::consts::PI).sin();
        return (Vec2::new(x, y), if k < 0.6 { 5 } else { 6 });
    }
    if t < PAN.1 {
        let bob = ((t - PAN.0) * 9.0).sin() * 2.0;
        return (
            Vec2::new(HANG_X, HANG_Y + bob),
            5 + ((t - PAN.0) / 0.2) as usize % 2,
        );
    }
    let k = smooth(HOP.at(t));
    let hop = 8.0 * (k * std::f32::consts::PI).sin();
    let from = Vec2::new(HANG_X, HANG_Y);
    (
        from.lerp(STAND, k) + Vec2::Y * hop,
        if k < 1.0 { 6 } else { 0 },
    )
}

#[derive(Resource)]
struct Climb {
    elapsed: f32,
}

#[derive(Component)]
enum Part {
    /// A tile of the wall, `0` or `1` in the stack, in palette `.1`.
    Wall(usize, usize),
    Ground(f32),
    /// The Hero in palette `.0`.
    Hero(usize),
    Landing(f32),
}

/// Text shown during a span, with the backdrop-coloured plate behind it.
#[derive(Component)]
struct Caption(Span);

fn spawn(
    mut commands: Commands,
    sprites: Res<Sprites>,
    mut backdrop: ResMut<Backdrop>,
    mut music: ResMut<TransitionMusic>,
) {
    backdrop.0 = backdrop_at(0.0);
    music.0 = Some(Song::Upswing);
    commands.insert_resource(Climb { elapsed: 0.0 });
    let mut part = |part: Part, sprite: Sprite, at: Vec3| {
        commands.spawn((
            part,
            sprite,
            Transform::from_translation(at),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    };
    let over = |mut sprite: Sprite| {
        sprite.color = Color::NONE;
        sprite
    };
    for cabinet in [FROM, TO] {
        let z = if cabinet == FROM { 0.0 } else { 0.1 };
        for stack in 0..2 {
            let sprite = sprites.sprite("c3_tower_wall", cabinet);
            let sprite = if cabinet == FROM {
                sprite
            } else {
                over(sprite)
            };
            part(
                Part::Wall(stack, cabinet),
                sprite,
                Vec3::new(WALL_X + WALL_W / 2.0, 0.0, 1.0 + z),
            );
        }
        let hero = sprites.frame("c3_hero", cabinet, 0);
        let hero = if cabinet == FROM { hero } else { over(hero) };
        part(Part::Hero(cabinet), hero, Vec3::new(START_X, 0.0, 5.0 + z));
    }
    let ground = sprites.frame_size("c3_ground").as_vec2().max(Vec2::ONE);
    for i in 0..=((WALL_X + HALF_W) / ground.x) as i32 {
        let x = -HALF_W + (i as f32 + 0.5) * ground.x;
        part(
            Part::Ground(GROUND_Y - ground.y / 2.0),
            sprites.sprite("c3_ground", FROM),
            Vec3::new(x, GROUND_Y - ground.y / 2.0, 2.0),
        );
    }
    for i in 0..LANDING_BLOCKS {
        let x = WALL_X - BLOCK * (i as f32 + 0.5);
        part(
            Part::Landing(LANDING_Y + RISE),
            sprites.sprite("c3_ledge", TO),
            Vec3::new(x, LANDING_Y + RISE, 3.0),
        );
    }

    let mut caption = |text: &str, color: Color, y: f32, scale: u32, span: Span| {
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
        commands.spawn((
            Sprite::from_color(backdrop_at(span.0), size),
            Transform::from_xyz(0.0, y.round(), 9.5),
            Visibility::Hidden,
            Caption(span),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    };
    caption(
        "CABINET 3 CLEAR!",
        CABINETS[FROM].accent,
        70.0,
        2,
        CAPTION_1,
    );
    caption("CABINET 4", CABINETS[TO].accent, 70.0, 2, CAPTION_2);
    caption(CABINETS[TO].genre, CABINETS[TO].accent, 50.0, 1, CAPTION_2);
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    mut state: ResMut<Climb>,
    mut backdrop: ResMut<Backdrop>,
    mut parts: Query<(&Part, &mut Transform, &mut Sprite)>,
    mut captions: Query<(&Caption, &mut Visibility)>,
    mut next: ResMut<NextState<Screen>>,
) {
    state.elapsed += time.delta_secs();
    let t = state.elapsed;
    backdrop.0 = backdrop_at(t);
    let rise = rise(t);
    let fade = fade(t);
    let (hero_at, frame) = hero_pose(t);
    for (part, mut transform, mut sprite) in &mut parts {
        match part {
            Part::Wall(stack, cabinet) => {
                let base = (-rise).rem_euclid(WALL_H) - WALL_H / 2.0;
                transform.translation.y = (base + *stack as f32 * WALL_H).round();
                if *cabinet == TO {
                    sprite.color = Color::WHITE.with_alpha(fade);
                }
            }
            Part::Ground(y) => transform.translation.y = (y - rise).round(),
            Part::Landing(y) => transform.translation.y = (y - rise).round(),
            Part::Hero(cabinet) => {
                transform.translation.x = hero_at.x.round();
                transform.translation.y = hero_at.y.round();
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = frame;
                }
                if *cabinet == TO {
                    sprite.color = Color::WHITE.with_alpha(fade);
                }
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

pub(super) struct ClimbPlugin;

impl Plugin for ClimbPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::CabinetTransition),
            spawn.run_if(super::into_cabinet_4),
        )
        .add_systems(
            Update,
            update
                .run_if(in_state(Screen::CabinetTransition))
                .run_if(resource_exists::<Climb>),
        )
        .add_systems(
            OnExit(Screen::CabinetTransition),
            |mut commands: Commands| {
                commands.remove_resource::<Climb>();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::display::HALF_H;

    #[test]
    fn the_hero_starts_on_the_ground_and_ends_standing_on_the_landing() {
        let (at, _) = hero_pose(0.0);
        assert_eq!(at, Vec2::new(START_X, GROUND_Y + HERO_HALF_H));
        let (at, frame) = hero_pose(SECONDS);
        assert!(at.distance(STAND) < 1e-3, "{at}");
        assert_eq!(frame, 0);
        // Feet on the landing's top once the view has stopped.
        assert_eq!(LANDING_Y + RISE - rise(SECONDS), LANDING_Y);
        assert_eq!(STAND.y - HERO_HALF_H, LANDING_Y + BLOCK / 2.0);
    }

    #[test]
    fn the_hero_never_jumps_between_beats() {
        let mut last = hero_pose(0.0).0;
        let mut t = 0.0;
        while t < SECONDS {
            t += 0.01;
            let at = hero_pose(t).0;
            assert!(at.distance(last) < 4.0, "{t}: {last} -> {at}");
            last = at;
        }
    }

    #[test]
    fn the_view_rises_then_rests_and_the_colours_finish_before_the_hop() {
        assert_eq!(rise(0.0), 0.0);
        assert_eq!(rise(SECONDS), RISE);
        const { assert!(FADE.0 > PAN.0 && FADE.1 <= PAN.1) };
        assert_eq!(backdrop_at(0.0), CABINETS[FROM].backdrop);
        assert_eq!(backdrop_at(SECONDS), CABINETS[TO].backdrop);
        // The ground is below the screen before the colours start to change.
        assert!(GROUND_Y - rise(FADE.0) < -HALF_H);
    }
}
