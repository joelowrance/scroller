//! The 2 → 3 Cabinet Transition: flak hits the Wren, the Hero ejects, the
//! view drops through the cloud line into the Underfield, the Wren crashes in
//! the distance, and the Hero parachutes down to land on foot
//! (`docs/cabinets/2-vertical-shooter.md`, Stage 2's end, and
//! `docs/cabinets/3-run-and-gun.md`, "Premise"). 6 seconds; the spans are the
//! constants below:
//!
//! - [`GLIDE`]: overhead, in Cabinet 2's colours. The Wren eases from where
//!   Stage 2-2 left it to the eject spot over the cloud sea, and a flak
//!   crosshair closes on it ([`LOCK`]). "CABINET 2 CLEAR!" ([`CAPTION_1`]).
//! - [`HIT`]: the burst ([`Sfx::FinalHit`]), and the Wren shakes.
//! - [`EJECT`]: the Hero pops out and up, toward the camera. The Wren spins
//!   away below, trailing smoke, and is lost in the clouds ([`FALL`]).
//! - [`PLUNGE`]: the view drops through the cloud line. A cloud bank sweeps
//!   up over everything; while it covers the screen the view turns side-on
//!   and takes Cabinet 3's colours. The bank stops at the top as the cloud
//!   lid over the Underfield.
//! - [`CRASH`]: far off to the right, the Wren dives into the plain. A
//!   flash, then smoke rises from the wreck to the end.
//! - [`DESCENT`]: the Hero drifts down under the parachute and lands at
//!   [`LAND_AT`], Cabinet 3's start; the canopy folds down beside it
//!   ([`FOLD`]). "CABINET 3 / RUN-AND-GUN PLATFORMER" ([`CAPTION_2`]).
//!
//! Then Cabinet 3's Stage Card. A music sting, [`Song::BailOut`], runs
//! under it.

use std::f32::consts::{FRAC_PI_2, TAU};

use bevy::color::Mix;
use bevy::prelude::*;

use crate::audio::{PlaySfx, Sfx, Song, TransitionMusic};
use crate::cabinets::CABINETS;
use crate::cabinets::vertical_shooter::WrenLastSeen;
use crate::cabinets::vertical_shooter::palette as two;
use crate::display::{Backdrop, HALF_H, HALF_W};
use crate::flow::{CardReason, Screen};
use crate::font::{CELL_H, CELL_W, PixelText, text_at};
use crate::sprites::Sprites;
use crate::sprites::palette::{EXTRA, PALETTES, SHADE};

use super::{Span, noise};

/// Length of the whole sequence, in seconds.
pub(super) const SECONDS: f32 = 6.0;
pub(super) const GLIDE: Span = Span(0.0, 0.5);
pub(super) const LOCK: Span = Span(0.0, 0.5);
pub(super) const HIT: Span = Span(0.5, 0.9);
pub(super) const EJECT: Span = Span(0.75, 1.5);
pub(super) const FALL: Span = Span(0.75, 2.0);
pub(super) const PLUNGE: Span = Span(1.7, 3.0);
pub(super) const CRASH: Span = Span(2.9, 3.5);
/// The Wren's last dive, at the start of the crash.
const DIVE: Span = Span(CRASH.0, CRASH.0 + 0.25);
pub(super) const DESCENT: Span = Span(2.35, 4.7);
pub(super) const FOLD: Span = Span(4.7, 5.2);
pub(super) const CAPTION_1: Span = Span(0.1, 1.6);
pub(super) const CAPTION_2: Span = Span(4.2, SECONDS);

/// When the cloud bank covers the whole screen: the view swaps to side-on.
const COVERED: f32 = (PLUNGE.0 + PLUNGE.1) / 2.0;

// The order the beats rely on, checked when compiling: the Hero is out
// before the plunge, the side view is only revealed after the swap, and the
// captions never share the screen.
const _: () = assert!(EJECT.1 <= PLUNGE.0 && HIT.0 >= LOCK.1);
const _: () = assert!(CRASH.0 > COVERED && DESCENT.0 >= COVERED);
const _: () = assert!(CAPTION_1.1 <= PLUNGE.0 && CAPTION_2.0 >= PLUNGE.1);
const _: () = assert!(FOLD.0 >= DESCENT.1 && CAPTION_2.1 >= SECONDS);
// The Hero and canopy start out under the cloud lid.
const _: () = assert!(DESCENT_FROM.y + CANOPY_ABOVE + 8.0 < LID_Y);

/// Cabinet 3's colours by its design-doc names.
mod three {
    use super::*;
    pub const BARK: Color = PALETTES[2].color(SHADE);
    pub const IRON: Color = PALETTES[2].color(EXTRA);
}

/// Where Stage 2-2's last frame has the yard's far edge.
const YARD_EDGE_Y: f32 = 22.0;
/// The yard drops away below: the Stage's last scroll speed, speeding up.
const YARD_SPEED: f32 = 32.0;
const YARD_FALL: f32 = 140.0;

/// Where the Wren is hit (overhead, on screen).
const EJECT_AT: Vec2 = Vec2::new(0.0, -30.0);
/// How far up the screen the Hero is thrown.
const EJECT_RISE: f32 = 56.0;
/// Where the Wren falls away to, shrinking into the clouds.
const FALL_TO: Vec2 = Vec2::new(46.0, -96.0);
/// Where the Hero lands: Cabinet 3's start, standing on its ground
/// (y = −96, the design doc's ground level), with the sprite 20 px tall.
pub(super) const LAND_AT: Vec2 = Vec2::new(-100.0, -86.0);
/// Where the Hero is under the canopy when the cloud bank uncovers it.
const DESCENT_FROM: Vec2 = Vec2::new(-40.0, 56.0);
/// Top of Cabinet 3's ground.
const GROUND_Y: f32 = -96.0;
/// The cloud lid's lower edge once the bank settles.
const LID_Y: f32 = 84.0;
/// Where the Wren comes down, far off on the plain (behind the ground).
const WRECK_AT: Vec2 = Vec2::new(92.0, -92.0);
/// The Spire on the horizon: a thin Iron line under the cloud lid.
const SPIRE_X: f32 = 122.0;
/// The Hero hangs this far below the canopy's centre.
const CANOPY_ABOVE: f32 = 17.0;
const CLOUD_COUNT: u32 = 12;
/// Cabinet 2's cloud sea, far and near (#57).
const CLOUD_SPRITES: [&str; 2] = ["c2_cloud_a", "c2_cloud_b"];
const CLOUD_SPEED: [f32; 2] = [18.0, 30.0];
const BANK_SPRITE: &str = "c2_cloudbank_a";
const SHADOW_OFFSET: Vec2 = Vec2::new(6.0, -6.0);
/// Seconds between smoke puffs from the falling Wren and from the wreck.
const SMOKE_EVERY: f32 = 0.12;
const SMOKE_SECONDS: f32 = 1.2;

fn smooth(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

fn ease_out(x: f32) -> f32 {
    1.0 - (1.0 - x).powi(3)
}

/// True once the view is side-on, in Cabinet 3's colours.
pub(super) fn side_on(t: f32) -> bool {
    t >= COVERED
}

/// The backdrop: Cabinet 2's, then Cabinet 3's from the swap.
pub(super) fn backdrop_at(t: f32) -> Color {
    if side_on(t) {
        CABINETS[2].backdrop
    } else {
        CABINETS[1].backdrop
    }
}

/// The cloud bank's lower edge, in screen y: below the screen, sweeping up
/// over it, then stopping as the cloud lid.
pub(super) fn bank_bottom(t: f32) -> f32 {
    let from = -HALF_H - BANK_HEIGHT;
    from + (LID_Y - from) * smooth(PLUNGE.at(t))
}

/// The bank's height: when its middle passes the screen's middle it covers
/// all of it.
const BANK_HEIGHT: f32 = 2.0 * HALF_H + 2.0 * (HALF_H + LID_Y) / 2.0;

/// The cloud bank's body colour: Cabinet 2's Steel cloud, then the Bark
/// underside of Cabinet 3's cloud lid.
fn bank_color(t: f32) -> Color {
    two::STEEL.mix(
        &three::BARK,
        smooth(Span(COVERED - 0.3, COVERED + 0.3).at(t)),
    )
}

/// The Wren overhead: where it is, its turn and its size.
pub(super) fn wren_pose(t: f32, from: Vec2) -> (Vec2, f32, f32) {
    let glide = from.lerp(EJECT_AT, smooth(GLIDE.at(t)));
    let fall = FALL.at(t);
    let at = glide.lerp(FALL_TO, smooth(fall));
    let shake = if HIT.contains(t) {
        let k = 1.0 - HIT.at(t);
        Vec2::new((t * 97.0).sin(), (t * 71.0).cos()) * 3.0 * k
    } else {
        Vec2::ZERO
    };
    let spin = fall * fall * 1.5 * TAU;
    (at + shake, spin, 1.0 - 0.75 * fall)
}

/// The Hero after ejecting, overhead: thrown up the screen and toward the
/// camera, so it grows.
fn pilot_overhead(t: f32) -> (Vec2, f32) {
    let e = ease_out(EJECT.at(t));
    (EJECT_AT + Vec2::Y * EJECT_RISE * e, 1.0 + 0.6 * e)
}

/// The Hero side-on: drifting down under the canopy, swaying less as it
/// nears the ground. Returns the Hero's centre and the sway angle.
pub(super) fn pilot_side(t: f32) -> (Vec2, f32) {
    let d = DESCENT.at(t);
    let x = DESCENT_FROM.x + (LAND_AT.x - DESCENT_FROM.x) * smooth(d);
    // Fast at first, then the canopy catches and it settles gently.
    let y = DESCENT_FROM.y + (LAND_AT.y - DESCENT_FROM.y) * (ease_out(d) + smooth(d)) / 2.0;
    let sway = (t * 4.0).sin() * 9f32.to_radians() * (1.0 - d);
    (Vec2::new(x, y), sway)
}

#[derive(Resource)]
struct Eject {
    elapsed: f32,
    from: Vec2,
    hit: bool,
    swapped: bool,
    smoke_due: f32,
}

#[derive(Component)]
enum Part {
    /// Stage 2-2's last sight of the yard, dropping away below; `y` is
    /// where it was at the Stage's end.
    Yard {
        y: f32,
    },
    /// Overhead, until the swap.
    Cloud {
        layer: usize,
        y: f32,
    },
    Wren,
    Shadow,
    Crosshair,
    Burst,
    PilotTop,
    /// The bank and its edges; `edge` is the sprite's offset from the
    /// bank's lower edge.
    Bank {
        edge: f32,
        plain: bool,
    },
    /// Side-on, from the swap.
    Ground,
    Spire,
    /// The Wren, a dark speck far off, diving into the plain.
    Diver,
    WreckFlash,
    Canopy,
    Pilot,
    Smoke {
        age: f32,
        drift: Vec2,
    },
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
    music.0 = Some(Song::BailOut);
    let from = last_seen
        .0
        .clamp(Vec2::new(-HALF_W, -HALF_H), Vec2::new(HALF_W, HALF_H));
    commands.insert_resource(Eject {
        elapsed: 0.0,
        from,
        hit: false,
        swapped: false,
        smoke_due: FALL.0,
    });
    let hidden = |mut sprite: Sprite| {
        sprite.color = Color::NONE;
        sprite
    };
    let mut part = |part: Part, sprite: Sprite, at: Vec3| {
        commands.spawn((
            part,
            sprite,
            Transform::from_translation(at),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    };

    // The yard as Stage 2-2 leaves it (assets/stages/2-2.ron, its end):
    // the far edge at y = 22 with floor blocks below.
    for x in [-128.0, -64.0, 0.0, 64.0, 128.0] {
        part(
            Part::Yard { y: YARD_EDGE_Y },
            sprites.sprite("c2_yard_edge", 1),
            Vec3::new(x, YARD_EDGE_Y, 0.8),
        );
    }
    for y in [-12.0, -60.0, -108.0] {
        for x in [-96.0, 32.0, 160.0] {
            part(
                Part::Yard { y },
                sprites.sprite("c2_yard_c", 1),
                Vec3::new(x, y, 0.7),
            );
        }
    }

    // Overhead.
    for i in 0..CLOUD_COUNT {
        let x = (noise(i, 4) * 2.0 - 1.0) * (HALF_W + 16.0);
        let y = (noise(i, 5) * 2.0 - 1.0) * (HALF_H + 16.0);
        let layer = i as usize % 2;
        part(
            Part::Cloud { layer, y },
            hidden(sprites.sprite(CLOUD_SPRITES[layer], 1)),
            Vec3::new(x.round(), y.round(), 0.5 + layer as f32 * 0.1),
        );
    }
    part(
        Part::Shadow,
        hidden(sprites.shadow("hero", 0, two::DUSK)),
        Vec3::new(0.0, 0.0, 1.0),
    );
    part(Part::Wren, sprites.sprite("hero", 1), from.extend(3.0));
    part(
        Part::Crosshair,
        hidden(sprites.sprite("c2_crosshair", 1)),
        from.extend(4.0),
    );
    part(
        Part::Burst,
        hidden(sprites.frame("burst", 1, 0)),
        EJECT_AT.extend(4.5),
    );
    part(
        Part::PilotTop,
        hidden(sprites.sprite("pilot_top", 1)),
        EJECT_AT.extend(5.0),
    );

    // The cloud bank: a plain body with a row of cloud edges above it in
    // Cabinet 2's palette and below it in Cabinet 3's. Above everything
    // until it settles as the lid.
    let bank_size = sprites.frame_size(BANK_SPRITE).as_vec2().max(Vec2::ONE);
    part(
        Part::Bank {
            edge: BANK_HEIGHT / 2.0,
            plain: true,
        },
        Sprite::from_color(bank_color(0.0), Vec2::new(2.0 * HALF_W, BANK_HEIGHT)),
        Vec3::new(0.0, 0.0, 8.0),
    );
    let across = (2.0 * HALF_W / bank_size.x).ceil() as i32 + 1;
    for i in 0..across {
        let x = -HALF_W + (i as f32 + 0.25) * bank_size.x;
        for (cabinet, edge, flip) in [
            (1, BANK_HEIGHT + bank_size.y / 2.0 - 1.0, false),
            (2, -bank_size.y / 2.0 + 1.0, true),
        ] {
            let mut sprite = sprites.sprite(BANK_SPRITE, cabinet);
            sprite.flip_y = flip;
            part(
                Part::Bank { edge, plain: false },
                sprite,
                Vec3::new(x.round(), 0.0, 8.1),
            );
        }
    }

    // Side-on, shown from the swap.
    let ground = sprites.frame_size("c3_ground").as_vec2().max(Vec2::ONE);
    for i in 0..=(2.0 * HALF_W / ground.x) as i32 {
        part(
            Part::Ground,
            hidden(sprites.sprite("c3_ground", 2)),
            Vec3::new(
                -HALF_W + (i as f32 + 0.5) * ground.x,
                GROUND_Y - ground.y / 2.0,
                2.0,
            ),
        );
    }
    part(
        Part::Spire,
        hidden(Sprite::from_color(
            three::IRON,
            Vec2::new(3.0, LID_Y - GROUND_Y + 8.0),
        )),
        Vec3::new(SPIRE_X, (LID_Y + GROUND_Y) / 2.0, 0.3),
    );
    let mut diver = sprites.shadow("hero", 0, three::BARK);
    diver.color = Color::NONE;
    part(Part::Diver, diver, WRECK_AT.extend(1.5));
    part(
        Part::WreckFlash,
        hidden(sprites.frame("burst", 2, 0)),
        WRECK_AT.extend(1.5),
    );
    part(
        Part::Canopy,
        hidden(sprites.sprite("parachute", 2)),
        DESCENT_FROM.extend(5.0),
    );
    part(
        Part::Pilot,
        hidden(sprites.sprite("pilot", 2)),
        DESCENT_FROM.extend(5.1),
    );

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
        // Not a child: the font rebuilds a text's children when it changes.
        commands.spawn((
            Sprite::from_color(backdrop_at(span.0), size),
            Transform::from_xyz(0.0, y.round(), 9.5),
            Visibility::Hidden,
            Caption(span),
            DespawnOnExit(Screen::CabinetTransition),
        ));
    };
    caption("CABINET 2 CLEAR!", CABINETS[1].accent, 44.0, 2, CAPTION_1);
    caption("CABINET 3", CABINETS[2].accent, 48.0, 2, CAPTION_2);
    caption(CABINETS[2].genre, CABINETS[2].accent, 28.0, 1, CAPTION_2);
}

fn set_at(transform: &mut Transform, at: Vec2) {
    transform.translation.x = at.x.round();
    transform.translation.y = at.y.round();
}

#[allow(clippy::too_many_arguments)]
fn update(
    mut commands: Commands,
    time: Res<Time>,
    sprites: Res<Sprites>,
    mut state: ResMut<Eject>,
    mut backdrop: ResMut<Backdrop>,
    mut parts: Query<(Entity, &mut Part, &mut Transform, &mut Sprite)>,
    mut captions: Query<(&Caption, &mut Visibility)>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
) {
    let dt = time.delta_secs();
    state.elapsed += dt;
    let t = state.elapsed;
    backdrop.0 = backdrop_at(t);
    if !state.hit && t >= HIT.0 {
        state.hit = true;
        sfx.write(PlaySfx(Sfx::FinalHit));
    }
    let side = side_on(t);
    state.swapped |= side;

    // Smoke: behind the falling Wren overhead, then from the wreck.
    let (wren_at, spin, wren_scale) = wren_pose(t, state.from);
    let smoke_from = if FALL.contains(t) && !side {
        Some((wren_at, 1, 0.8 * wren_scale))
    } else if t >= CRASH.1 - 0.2 {
        Some((WRECK_AT + Vec2::Y * 4.0, 2, 1.6))
    } else {
        None
    };
    while t >= state.smoke_due {
        state.smoke_due += SMOKE_EVERY;
        if let Some((at, cabinet, scale)) = smoke_from {
            let i = (state.smoke_due * 100.0) as u32;
            let drift = if cabinet == 1 {
                Vec2::new(noise(i, 8) * 8.0 - 4.0, 12.0)
            } else {
                Vec2::new(4.0 + noise(i, 8) * 6.0, 16.0)
            };
            let mut sprite = sprites.frame("c2_flak_puff", cabinet, 0);
            sprite.color = Color::WHITE.with_alpha(0.9);
            commands.spawn((
                Part::Smoke { age: 0.0, drift },
                sprite,
                Transform::from_translation(at.extend(if cabinet == 1 { 2.5 } else { 1.4 }))
                    .with_scale(Vec3::splat(scale)),
                DespawnOnExit(Screen::CabinetTransition),
            ));
        }
    }

    let fade_in = smooth(GLIDE.at(t));
    let bank_bottom = bank_bottom(t);
    let (pilot_at, sway) = pilot_side(t);
    let fold = smooth(FOLD.at(t));
    // Untinted at `alpha` while `on`, else invisible.
    let shown = |on: bool, alpha: f32| Color::WHITE.with_alpha(alpha * f32::from(on));

    for (entity, mut part, mut transform, mut sprite) in &mut parts {
        match &mut *part {
            Part::Yard { y } => {
                let dropped = YARD_SPEED * t + YARD_FALL * t * t;
                transform.translation.y = (*y - dropped).round();
                sprite.color = shown(!side && *y - dropped > -HALF_H - 80.0, 1.0);
            }
            Part::Cloud { layer, y } => {
                *y = wrap(*y - CLOUD_SPEED[*layer] * dt, HALF_H + 16.0);
                transform.translation.y = y.round();
                sprite.color = shown(!side, fade_in);
            }
            Part::Wren => {
                set_at(&mut transform, wren_at);
                transform.rotation = Quat::from_rotation_z(FRAC_PI_2 + spin);
                transform.scale = Vec3::splat(wren_scale);
                let gone = smooth(Span(FALL.1 - 0.4, FALL.1).at(t));
                sprite.color = shown(!side, 1.0 - gone);
            }
            Part::Shadow => {
                // The shadow falls on clouds far below, so it shrinks less.
                set_at(&mut transform, wren_at + SHADOW_OFFSET * wren_scale);
                transform.rotation = Quat::from_rotation_z(FRAC_PI_2 + spin);
                transform.scale = Vec3::splat(0.5 + 0.5 * wren_scale);
                let gone = FALL.at(t);
                sprite.color = two::DUSK.with_alpha(fade_in * (1.0 - gone) * f32::from(!side));
            }
            Part::Crosshair => {
                let lock = LOCK.at(t);
                set_at(&mut transform, wren_at);
                transform.scale = Vec3::splat(2.5 - 1.5 * ease_out(lock));
                // Blinks faster as it closes, gone at the hit.
                let blink = ((t * (8.0 + 16.0 * lock)).sin() > -0.3) as u8 as f32;
                sprite.color = Color::WHITE.with_alpha(blink * f32::from(t < HIT.0));
            }
            Part::Burst => {
                let k = HIT.at(t);
                let frame = ((k * 3.0) as usize).min(2);
                set_at(&mut transform, EJECT_AT);
                transform.scale = Vec3::splat(2.0 + k);
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = frame;
                }
                sprite.color = Color::WHITE.with_alpha(f32::from(HIT.contains(t)));
            }
            Part::PilotTop => {
                let (at, scale) = pilot_overhead(t);
                set_at(&mut transform, at);
                transform.scale = Vec3::splat(scale);
                transform.rotation = Quat::from_rotation_z(EJECT.at(t) * 0.6);
                sprite.color = shown(t >= EJECT.0 && !side, 1.0);
            }
            Part::Bank { edge, plain } => {
                transform.translation.y = (bank_bottom + *edge).round();
                if *plain {
                    sprite.color = bank_color(t);
                }
                // Once settled it is the lid: behind everything else.
                let z = if t >= PLUNGE.1 { 0.4 } else { 8.0 };
                transform.translation.z = z + if *plain { 0.0 } else { 0.01 };
            }
            Part::Ground => sprite.color = Color::WHITE.with_alpha(f32::from(side)),
            Part::Spire => sprite.color = three::IRON.with_alpha(f32::from(side)),
            Part::Diver => {
                let dive = DIVE.at(t);
                let at = Vec2::new(WRECK_AT.x + 40.0, LID_Y - 10.0).lerp(WRECK_AT, dive);
                set_at(&mut transform, at);
                transform.scale = Vec3::splat(0.4);
                transform.rotation = Quat::from_rotation_z(-2.3);
                sprite.color = three::BARK.with_alpha(f32::from(DIVE.contains(t)));
            }
            Part::WreckFlash => {
                // A flash as it hits, then a flicker of fire to the end.
                let (frame, scale) = if t < DIVE.1 {
                    (0, 0.0)
                } else if t < CRASH.1 {
                    ((Span(DIVE.1, CRASH.1).at(t) * 3.0).min(2.0) as usize, 3.0)
                } else {
                    (1 + (t * 10.0) as usize % 2, 1.0)
                };
                transform.scale = Vec3::splat(scale);
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = frame;
                }
                sprite.color = Color::WHITE.with_alpha(f32::from(t >= DIVE.1));
            }
            Part::Canopy => {
                // Folds down to lie on the ground behind the Hero.
                let hang = pilot_at + Vec2::from_angle(sway).rotate(Vec2::Y * CANOPY_ABOVE);
                let laid = Vec2::new(LAND_AT.x + 18.0, GROUND_Y + 3.0);
                set_at(&mut transform, hang.lerp(laid, fold));
                transform.rotation = Quat::from_rotation_z(sway * (1.0 - fold) - 0.5 * fold);
                transform.scale = Vec3::new(1.0 - 0.3 * fold, 1.0 - 0.7 * fold, 1.0);
                sprite.color = shown(side, 1.0);
            }
            Part::Pilot => {
                set_at(&mut transform, pilot_at);
                transform.rotation = Quat::from_rotation_z(sway);
                sprite.color = shown(side, 1.0);
            }
            Part::Smoke { age, drift } => {
                *age += dt;
                let k = *age / SMOKE_SECONDS;
                if k >= 1.0 {
                    commands.entity(entity).despawn();
                    continue;
                }
                transform.translation += (*drift * dt).extend(0.0);
                let frame = ((k * 4.0) as usize).min(3);
                if let Some(atlas) = &mut sprite.texture_atlas {
                    atlas.index = frame;
                }
                sprite.color = Color::WHITE.with_alpha(0.9 * (1.0 - k));
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

/// Wraps `x` into `-half..half`.
fn wrap(x: f32, half: f32) -> f32 {
    (x + half).rem_euclid(half * 2.0) - half
}

pub(super) struct EjectPlugin;

impl Plugin for EjectPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(Screen::CabinetTransition),
            spawn.run_if(super::into_cabinet_3),
        )
        .add_systems(
            Update,
            update
                .run_if(in_state(Screen::CabinetTransition))
                .run_if(resource_exists::<Eject>),
        )
        .add_systems(
            OnExit(Screen::CabinetTransition),
            |mut commands: Commands| {
                commands.remove_resource::<Eject>();
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_view_swaps_while_the_cloud_bank_covers_the_screen() {
        assert_eq!(backdrop_at(0.0), CABINETS[1].backdrop);
        assert_eq!(backdrop_at(SECONDS), CABINETS[2].backdrop);
        // Covered: the bank's top is above the screen and its bottom below.
        for t in [COVERED - 0.05, COVERED, COVERED + 0.05] {
            let bottom = bank_bottom(t);
            assert!(bottom < -HALF_H, "{t}: bottom {bottom}");
            assert!(
                bottom + BANK_HEIGHT > HALF_H,
                "{t}: top {}",
                bottom + BANK_HEIGHT
            );
        }
        // Before the plunge it is off screen below; after it, it is the lid.
        assert!(bank_bottom(0.0) + BANK_HEIGHT <= -HALF_H);
        assert_eq!(bank_bottom(SECONDS), LID_Y);
        assert_eq!(bank_color(SECONDS), three::BARK);
        assert_eq!(bank_color(0.0), two::STEEL);
    }

    #[test]
    fn the_wren_starts_where_the_stage_left_it() {
        let from = Vec2::new(40.0, -60.0);
        let (at, spin, scale) = wren_pose(0.0, from);
        assert_eq!((at, spin, scale), (from, 0.0, 1.0));
        let (early, _, _) = wren_pose(0.05, from);
        assert!(early.distance(from) < 0.05 * from.distance(EJECT_AT));
        let (at, _, _) = wren_pose(HIT.0 - 0.001, from);
        assert!(at.distance(EJECT_AT) < 0.01, "at the eject spot when hit");
    }

    #[test]
    fn the_hero_lands_at_cabinet_3s_start_and_stands_still() {
        let (at, _) = pilot_side(DESCENT.0);
        assert_eq!(at, DESCENT_FROM);
        let mut last = f32::INFINITY;
        for i in 0..=50 {
            let (at, _) = pilot_side(DESCENT.0 + i as f32 * (DESCENT.1 - DESCENT.0) / 50.0);
            assert!(at.y <= last, "only ever goes down");
            last = at.y;
        }
        for t in [DESCENT.1, SECONDS] {
            let (at, sway) = pilot_side(t);
            assert!(at.distance(LAND_AT) < 1e-3, "{at}");
            assert!(sway.abs() < 1e-4);
        }
        // Feet on the ground: the sprite is 20 px tall.
        assert_eq!(LAND_AT.y - 10.0, GROUND_Y);
    }
}
