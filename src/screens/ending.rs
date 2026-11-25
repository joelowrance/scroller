//! The Ending: the "you finished" sequence after Cabinet 4's last Stage
//! (`docs/DESIGN.md`, "The Run"). The Run is over afterwards, with no second
//! lap. About 13 seconds, then it waits for the player; the spans are the
//! constants below.
//!
//! - **The summit** ([`SUMMIT`]), in Cabinet 4's colours at dawn. The Hero
//!   vaults onto the Crown's deck, runs to the beacon that beats out the
//!   Static's signal, and shoots its lamp out ([`OFF`]). The sky brightens,
//!   the stars go and the sun rises over the cloud sea while the Hero turns
//!   to watch it. "THE CROWN FALLS SILENT".
//! - **Looking back** ([`LOOK_BACK`]): "YOU FINISHED!" and one panel per
//!   Cabinet, each in its own palette, calling back to the Run: the Wren over
//!   the sea, the Wren climbing over the clouds, the Hero under the
//!   parachute, and the Hero at the dark beacon. Under each, that Cabinet's
//!   points (`Session::cabinet_scores`).
//! - **The tally**: lives left, the lives bonus (`ending_life_bonus` in
//!   `assets/tuning.ron` for each one) counting into the total, then
//!   "THE END" ([`DONE`]).
//!
//! The bonus is added to the score as the Ending opens, so skipping can't
//! lose it. After [`INPUT_DELAY`], A or Start skips to the tally; at the
//! tally it moves on as Game Over does: initials if the score makes the High
//! Score Table, otherwise the table. The music is [`Song::Daybreak`]
//! (chosen by `song_for_screen`).
//!
//! [`Song::Daybreak`]: crate::audio::Song::Daybreak

use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

use bevy::prelude::*;

use super::results::{GOLD, INPUT_DELAY, after_result};
use super::transition::Span;
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::{CABINET_COUNT, CABINETS};
use crate::display::{Backdrop, HALF_H, HALF_W};
use crate::flow::Screen;
use crate::font::{Blink, PixelText, text_at};
use crate::highscore::HighScoreTable;
use crate::input::{Button, Controls};
use crate::run::Session;
use crate::sprites::Sprites;
use crate::sprites::palette::{EXTRA, PALETTES, SHADE};
use crate::tuning::Tuning;

/// The summit, in Cabinet 4's palette.
const SUMMIT: Span = Span(0.0, 7.0);
/// The Hero vaults up onto the deck from the Crown's side.
const VAULT: Span = Span(0.0, 0.8);
/// The Hero runs to the beacon.
const RUN: Span = Span(0.8, 1.8);
/// The Hero aims up at the lamp; the shot flies in [`SHOT`].
const AIM: Span = Span(1.8, 2.9);
const SHOT: Span = Span(2.2, 2.45);
/// The lamp goes out.
const OFF: f32 = SHOT.1;
const BURST: Span = Span(OFF, OFF + 0.45);
/// The Hero turns to watch the sun.
const TURN: f32 = 3.4;
const SUNRISE: Span = Span(2.8, 6.0);
const CAPTION_1: Span = Span(3.2, SUMMIT.1);
const CAPTION_2: Span = Span(4.0, SUMMIT.1);
/// "YOU FINISHED!" and the panels, until the player moves on.
const LOOK_BACK: Span = Span(SUMMIT.1, f32::MAX);
/// When each Cabinet's panel appears.
const FIRST_PANEL: f32 = 7.4;
const PANEL_EVERY: f32 = 0.8;
const LIVES: Span = Span(10.8, f32::MAX);
/// The lives bonus counts into the total.
const COUNT: Span = Span(11.2, 12.8);
/// Everything is shown: "THE END" and "PRESS A".
const DONE: f32 = 13.2;
const AFTER: Span = Span(DONE, f32::MAX);

const _: () = assert!(VAULT.1 <= RUN.0 && RUN.1 <= AIM.0 && SHOT.0 >= AIM.0 && SHOT.1 < AIM.1);
const _: () = assert!(AIM.1 <= TURN && TURN < SUMMIT.1 && SUNRISE.1 <= SUMMIT.1);
const _: () =
    assert!(FIRST_PANEL + PANEL_EVERY * (CABINET_COUNT - 1) as f32 <= LIVES.0 && COUNT.1 <= DONE);

/// Cabinet 4's palette index.
const C4: usize = 3;

/// The Hero sprite is 12 x 20.
const HERO_HALF_H: f32 = 10.0;
/// The top of the Crown's deck, and its span in x.
const DECK_TOP: f32 = -72.0;
const DECK_X: (f32, f32) = (0.0, 112.0);
const BLOCK: f32 = 8.0;
/// The Hero vaults up at the deck's left end, stops short of the beacon.
const VAULT_FROM: Vec2 = Vec2::new(8.0, -104.0);
const STAND_Y: f32 = DECK_TOP + HERO_HALF_H;
const STOP_X: f32 = 66.0;
/// The beacon (16 x 32) stands on the deck; its lamp is the top 8 rows.
const BEACON: Vec2 = Vec2::new(98.0, DECK_TOP + 16.0);
const LAMP: Vec2 = Vec2::new(BEACON.x, BEACON.y + 12.0);
/// Where the shot leaves the gun when aiming up and right.
const MUZZLE: Vec2 = Vec2::new(STOP_X + 6.0, STAND_Y + 8.0);
/// The sun (32 x 16, a half disc) rises from behind the cloud sea until
/// its base sits on the horizon.
const SUN_X: f32 = -84.0;
const SUN_Y: (f32, f32) = (-100.0, CLOUD_TOP + 8.0);
const CLOUD_Y: f32 = -92.0;
/// The cloud sea is 16 px tall; its top is the horizon the sun sits on.
const CLOUD_TOP: f32 = CLOUD_Y + 8.0;

/// The look-back panels: size, centre height and spacing.
const PANEL: Vec2 = Vec2::new(70.0, 54.0);
const PANEL_Y: f32 = 50.0;
const PANEL_STEP: f32 = 76.0;

/// Captions on the summit.
const CAPTION_COLOR: Color = Color::srgb(1.0, 0.92, 0.8);
/// The look-back backdrop: a deep night blue under the panels.
const LOOK_BACK_BACKDROP: Color = Color::srgb(0.03, 0.03, 0.08);

fn smooth(x: f32) -> f32 {
    x * x * (3.0 - 2.0 * x)
}

/// The Hero on the summit: centre, frame of `c3_hero` and whether it faces
/// left.
fn hero_pose(t: f32) -> (Vec2, usize, bool) {
    let stand = Vec2::new(STOP_X, STAND_Y);
    if t < VAULT.1 {
        let k = VAULT.at(t);
        let to = Vec2::new(VAULT_FROM.x, STAND_Y);
        // Up the Crown's side and over the deck's edge.
        let at = VAULT_FROM.lerp(to, smooth(k)) + Vec2::Y * 12.0 * (k * PI).sin();
        return (at, if k < 0.55 { 5 } else { 6 }, false);
    }
    if t < RUN.1 {
        let x = VAULT_FROM.x + (STOP_X - VAULT_FROM.x) * RUN.at(t);
        return (Vec2::new(x, STAND_Y), 1 + (t / 0.1) as usize % 4, false);
    }
    if t < AIM.1 {
        return (stand, 9, false);
    }
    (stand, 0, t >= TURN)
}

/// 0 at night, 1 once the sun is up.
fn dawn(t: f32) -> f32 {
    smooth(SUNRISE.at(t))
}

fn backdrop_at(t: f32) -> Color {
    if LOOK_BACK.contains(t) {
        return LOOK_BACK_BACKDROP;
    }
    let sky = PALETTES[C4]
        .color(SHADE)
        .mix(&PALETTES[C4].color(EXTRA), 0.3);
    CABINETS[C4].backdrop.mix(&sky, dawn(t))
}

/// How much of the lives bonus has been counted in by `t`.
fn counted(bonus: u64, t: f32) -> u64 {
    (bonus as f64 * COUNT.at(t) as f64).round() as u64
}

/// How many lives' worth of bonus has been counted in by `t`.
fn lives_counted(lives: u32, t: f32) -> u32 {
    (lives as f32 * COUNT.at(t)).floor() as u32
}

/// When Cabinet `i`'s panel appears.
fn panel_shown(i: usize) -> Span {
    Span(FIRST_PANEL + PANEL_EVERY * i as f32, f32::MAX)
}

#[derive(Resource)]
struct Ending {
    elapsed: f32,
    /// The score before the lives bonus, and the bonus.
    base: u64,
    bonus: u64,
    lives: u32,
    /// Input is ignored until this runs out.
    delay: f32,
}

/// Shown only during its span.
#[derive(Component)]
struct During(Span);

/// Something on the summit that moves.
#[derive(Component)]
enum Part {
    Hero,
    Shot,
    Beacon,
    Burst,
    Sun,
    Stars,
}

/// A Wren in a look-back panel: bobs on its own beat.
#[derive(Component)]
struct Bob {
    from: Vec3,
    phase: f32,
}

/// A number in the tally that counts up.
#[derive(Component)]
enum Count {
    Bonus,
    Total,
}

fn spawn(
    mut commands: Commands,
    sprites: Res<Sprites>,
    tuning: Res<Tuning>,
    mut session: ResMut<Session>,
    mut backdrop: ResMut<Backdrop>,
) {
    let base = session.score;
    let lives = session.lives;
    let bonus = tuning.ending_life_bonus.saturating_mul(lives as u64);
    session.score = base.saturating_add(bonus);
    commands.insert_resource(Ending {
        elapsed: 0.0,
        base,
        bonus,
        lives,
        delay: INPUT_DELAY,
    });
    backdrop.0 = backdrop_at(0.0);

    spawn_summit(&mut commands, &sprites);
    spawn_look_back(&mut commands, &sprites, &session);
    spawn_tally(&mut commands, base, bonus, lives, tuning.ending_life_bonus);
}

fn spawn_summit(commands: &mut Commands, sprites: &Sprites) {
    let mut put = |sprite: Sprite, at: Vec3, part: Option<Part>| {
        let mut entity = commands.spawn((
            sprite,
            Transform::from_translation(at),
            During(SUMMIT),
            DespawnOnExit(Screen::Ending),
        ));
        if let Some(part) = part {
            entity.insert(part);
        }
    };
    let tiles = |width: f32| {
        (0..=(2.0 * HALF_W / width) as i32).map(move |i| -HALF_W + (i as f32 + 0.5) * width)
    };

    for x in tiles(32.0) {
        put(
            sprites.sprite("c4_stars", C4),
            Vec3::new(x, HALF_H - 32.0, 0.1),
            Some(Part::Stars),
        );
        put(
            sprites.sprite("c4_dawn_band", C4),
            // Mostly behind the cloud sea: only its dithered top glows at
            // the horizon, and the sun rises clear of it.
            Vec3::new(x, CLOUD_Y - 8.0, 0.2),
            None,
        );
    }
    put(
        sprites.sprite("c4_sun", C4),
        Vec3::new(SUN_X, SUN_Y.0, 0.3),
        Some(Part::Sun),
    );
    for x in tiles(64.0) {
        put(
            sprites.sprite("c4_cloud_sea", C4),
            Vec3::new(x, CLOUD_Y, 0.4),
            None,
        );
    }
    // Below the cloud tops, the sea is solid.
    put(
        Sprite::from_color(PALETTES[C4].color(SHADE), Vec2::new(2.0 * HALF_W, 20.0)),
        Vec3::new(0.0, -HALF_H + 10.0, 0.35),
        None,
    );
    let mid = (DECK_X.0 + DECK_X.1) / 2.0;
    put(
        sprites.sprite("c4_crown", C4),
        Vec3::new(mid, DECK_TOP - BLOCK - 20.0, 1.0),
        None,
    );
    let blocks = ((DECK_X.1 - DECK_X.0) / BLOCK) as i32;
    for i in 0..blocks {
        put(
            sprites.sprite("c4_deck", C4),
            Vec3::new(
                DECK_X.0 + (i as f32 + 0.5) * BLOCK,
                DECK_TOP - BLOCK / 2.0,
                2.0,
            ),
            None,
        );
    }
    put(
        sprites.sprite("c4_beacon", C4),
        BEACON.extend(2.5),
        Some(Part::Beacon),
    );
    put(
        sprites.sprite("c3_hero", C4),
        VAULT_FROM.extend(5.0),
        Some(Part::Hero),
    );
    put(
        sprites.sprite("c3_shot", C4),
        MUZZLE.extend(4.5),
        Some(Part::Shot),
    );
    put(
        sprites.sprite("burst", C4),
        LAMP.extend(6.0),
        Some(Part::Burst),
    );

    for (text, y, span) in [
        ("THE CROWN FALLS SILENT", 64.0, CAPTION_1),
        ("THE STATIC IS STILL", 50.0, CAPTION_2),
    ] {
        commands.spawn((
            text_at(PixelText::new(text, CAPTION_COLOR), 0.0, y, 1),
            During(span),
            DespawnOnExit(Screen::Ending),
        ));
    }
}

/// One Cabinet's panel: its scene, in its palette, centred on the origin.
fn panel_scene(p: &mut ChildSpawnerCommands, sprites: &Sprites, cabinet: usize) {
    fn put(p: &mut ChildSpawnerCommands, sprite: Sprite, x: f32, y: f32, z: f32) {
        p.spawn((sprite, Transform::from_xyz(x, y, z)));
    }
    let bottom = -PANEL.y / 2.0;
    match cabinet {
        // The Wren over the night sea, past the lighthouse.
        0 => {
            put(
                p,
                sprites.sprite("lighthouse", 0),
                -24.0,
                bottom + 20.0,
                0.1,
            );
            for i in 0..4 {
                put(
                    p,
                    sprites.sprite("wave-crest", 0),
                    -24.0 + 16.0 * i as f32,
                    bottom + 3.0,
                    0.2,
                );
            }
            p.spawn((
                sprites.sprite("hero", 0),
                Transform::from_xyz(8.0, 6.0, 0.3),
                Bob {
                    from: Vec3::new(8.0, 6.0, 0.3),
                    phase: 0.0,
                },
            ));
        }
        // The Wren climbing over the clouds, nose up.
        1 => {
            put(p, sprites.sprite("c2_cloud_a", 1), 0.0, bottom + 9.0, 0.1);
            p.spawn((
                sprites.sprite("hero", 1),
                Transform::from_xyz(0.0, 8.0, 0.3).with_rotation(Quat::from_rotation_z(FRAC_PI_2)),
                Bob {
                    from: Vec3::new(0.0, 8.0, 0.3),
                    phase: 1.3,
                },
            ));
        }
        // The Hero coming down under the parachute.
        2 => {
            for i in 0..8 {
                put(
                    p,
                    sprites.sprite("c3_ground_top", 2),
                    -28.0 + 8.0 * i as f32,
                    bottom + 4.0,
                    0.1,
                );
            }
            p.spawn((
                Transform::from_xyz(0.0, 0.0, 0.3),
                Visibility::default(),
                Bob {
                    from: Vec3::new(0.0, 0.0, 0.3),
                    phase: 2.1,
                },
            ))
            .with_children(|c| {
                c.spawn((
                    sprites.sprite("parachute", 2),
                    Transform::from_xyz(0.0, 12.0, 0.0),
                ));
                c.spawn((
                    sprites.sprite("pilot", 2),
                    Transform::from_xyz(0.0, -6.0, 0.1),
                ));
            });
        }
        // The Hero beside the dark beacon, the sun up behind.
        _ => {
            put(p, sprites.sprite("c4_sun", C4), -14.0, bottom + 10.0, 0.1);
            put(
                p,
                sprites.sprite("c4_cloud_sea", C4),
                0.0,
                bottom + 8.0,
                0.2,
            );
            for i in 0..5 {
                put(
                    p,
                    sprites.sprite("c4_deck", C4),
                    -8.0 + 8.0 * i as f32,
                    bottom + 16.0,
                    0.3,
                );
            }
            put(
                p,
                sprites.frame("c4_beacon", C4, 1),
                22.0,
                bottom + 36.0,
                0.4,
            );
            put(p, sprites.sprite("c3_hero", C4), 4.0, bottom + 30.0, 0.5);
        }
    }
}

fn spawn_look_back(commands: &mut Commands, sprites: &Sprites, session: &Session) {
    commands.spawn((
        text_at(PixelText::new("YOU FINISHED!", GOLD), 0.0, 100.0, 2),
        During(LOOK_BACK),
        DespawnOnExit(Screen::Ending),
    ));
    for (i, info) in CABINETS.iter().enumerate() {
        let x = (i as f32 - (CABINET_COUNT - 1) as f32 / 2.0) * PANEL_STEP;
        commands
            .spawn((
                Transform::from_xyz(x, PANEL_Y, 1.0),
                Visibility::Hidden,
                During(panel_shown(i)),
                DespawnOnExit(Screen::Ending),
            ))
            .with_children(|p| {
                p.spawn((
                    Sprite::from_color(info.accent, PANEL + Vec2::splat(2.0)),
                    Transform::from_xyz(0.0, 0.0, -0.2),
                ));
                p.spawn((
                    Sprite::from_color(info.backdrop, PANEL),
                    Transform::from_xyz(0.0, 0.0, -0.1),
                ));
                panel_scene(p, sprites, i);
                let below = -PANEL.y / 2.0;
                p.spawn(text_at(
                    PixelText::new(format!("CABINET {}", i + 1), Color::WHITE),
                    0.0,
                    below - 9.0,
                    1,
                ));
                p.spawn(text_at(
                    PixelText::new(session.cabinet_scores[i].to_string(), info.accent),
                    0.0,
                    below - 20.0,
                    1,
                ));
            });
    }
}

fn spawn_tally(commands: &mut Commands, base: u64, bonus: u64, lives: u32, per_life: u64) {
    let mut line = |text: PixelText, y: f32, scale: u32, span: Span, extra: Option<Count>| {
        let mut entity = commands.spawn((
            text_at(text, 0.0, y, scale),
            Visibility::Hidden,
            During(span),
            DespawnOnExit(Screen::Ending),
        ));
        if let Some(count) = extra {
            entity.insert(count);
        }
    };
    let accent = CABINETS[C4].accent;
    line(
        PixelText::new(format!("LIVES LEFT {lives} X {per_life}"), Color::WHITE),
        -18.0,
        1,
        LIVES,
        None,
    );
    line(
        PixelText::new(format!("BONUS {}", counted(bonus, 0.0)), accent),
        -30.0,
        1,
        LIVES,
        Some(Count::Bonus),
    );
    line(
        PixelText::new(format!("TOTAL {base}"), GOLD),
        -54.0,
        2,
        LIVES,
        Some(Count::Total),
    );
    line(
        PixelText::new("THE END", Color::WHITE),
        -82.0,
        2,
        AFTER,
        None,
    );
    // Blink drives the text's own visibility, so the span hides a parent.
    commands
        .spawn((
            Transform::default(),
            Visibility::Hidden,
            During(AFTER),
            DespawnOnExit(Screen::Ending),
        ))
        .with_children(|p| {
            p.spawn((
                text_at(PixelText::new("PRESS A", Color::WHITE), 0.0, -104.0, 1),
                Blink::every(0.5),
            ));
        });
}

fn update(
    mut commands: Commands,
    time: Res<Time>,
    controls: Res<Controls>,
    session: Res<Session>,
    scores: Res<HighScoreTable>,
    mut ending: ResMut<Ending>,
    mut backdrop: ResMut<Backdrop>,
    mut sfx: MessageWriter<PlaySfx>,
    mut next: ResMut<NextState<Screen>>,
) {
    let pressed = controls.just_pressed(Button::A) || controls.just_pressed(Button::Start);
    let ready = ending.delay <= 0.0;
    ending.delay -= time.delta_secs();
    let before = ending.elapsed;
    if ready && pressed {
        if before >= DONE {
            next.set(after_result(&mut commands, &session, &scores));
            return;
        }
        // Skip to the tally, quietly.
        ending.elapsed = DONE;
        backdrop.0 = backdrop_at(DONE);
        return;
    }
    let t = before + time.delta_secs();
    ending.elapsed = t;
    backdrop.0 = backdrop_at(t);
    if before < OFF && t >= OFF {
        sfx.write(PlaySfx(Sfx::FinalHit));
    }
    if lives_counted(ending.lives, t) > lives_counted(ending.lives, before) {
        sfx.write(PlaySfx(Sfx::Medal));
    }
}

fn show(
    ending: Res<Ending>,
    time: Res<Time>,
    mut shown: Query<(&During, &mut Visibility)>,
    mut parts: Query<(&Part, &mut Transform, &mut Sprite)>,
    mut bobs: Query<(&Bob, &mut Transform, Option<&mut Sprite>), Without<Part>>,
    mut counts: Query<(&Count, &mut PixelText)>,
) {
    let t = ending.elapsed;
    for (during, mut visibility) in &mut shown {
        visibility.set_if_neq(if during.0.contains(t) {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    let (hero_at, hero_frame, facing_left) = hero_pose(t);
    for (part, mut transform, mut sprite) in &mut parts {
        let mut frame = None;
        match part {
            Part::Hero => {
                transform.translation.x = hero_at.x.round();
                transform.translation.y = hero_at.y.round();
                sprite.flip_x = facing_left;
                frame = Some(hero_frame);
            }
            Part::Shot => {
                let at = MUZZLE.lerp(LAMP, SHOT.at(t));
                transform.translation.x = at.x.round();
                transform.translation.y = at.y.round();
                transform.rotation = Quat::from_rotation_z(FRAC_PI_4);
                sprite.color = Color::WHITE.with_alpha(if SHOT.contains(t) { 1.0 } else { 0.0 });
            }
            Part::Beacon => {
                // Blinks while lit, dark once shot out.
                let lit = t < OFF && (t / 0.35) as u32 % 3 != 2;
                frame = Some(if lit { 0 } else { 1 });
            }
            Part::Burst => {
                let k = BURST.at(t);
                sprite.color = Color::WHITE.with_alpha(if BURST.contains(t) { 1.0 } else { 0.0 });
                frame = Some(((k * 3.0) as usize).min(2));
            }
            Part::Sun => {
                transform.translation.y = (SUN_Y.0 + (SUN_Y.1 - SUN_Y.0) * dawn(t)).round();
            }
            Part::Stars => sprite.color = Color::WHITE.with_alpha(1.0 - dawn(t)),
        }
        if let (Some(frame), Some(atlas)) = (frame, sprite.texture_atlas.as_mut()) {
            atlas.index = frame;
        }
    }
    let now = time.elapsed_secs();
    for (bob, mut transform, sprite) in &mut bobs {
        let wave = ((now * 2.5 + bob.phase).sin() * 2.0).round();
        let sway = ((now * 1.2 + bob.phase).sin() * 2.0).round();
        transform.translation = bob.from + Vec3::new(sway, wave, 0.0);
        // The Wren's engine flickers.
        if let Some(atlas) = sprite.and_then(|s| s.into_inner().texture_atlas.as_mut()) {
            atlas.index = (now / 0.08) as usize % 2;
        }
    }
    let bonus = counted(ending.bonus, t);
    for (count, mut text) in &mut counts {
        let shown = match count {
            Count::Bonus => format!("BONUS {bonus}"),
            Count::Total => format!("TOTAL {}", ending.base + bonus),
        };
        if text.text != shown {
            text.text = shown;
        }
    }
}

pub(super) struct EndingPlugin;

impl Plugin for EndingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(Screen::Ending), spawn)
            .add_systems(
                Update,
                (update, show)
                    .chain()
                    .run_if(in_state(Screen::Ending))
                    .run_if(resource_exists::<Ending>),
            )
            .add_systems(OnExit(Screen::Ending), |mut commands: Commands| {
                commands.remove_resource::<Ending>();
            });
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use bevy::state::app::StatesPlugin;
    use bevy::time::TimeUpdateStrategy;

    use super::*;

    #[test]
    fn the_hero_vaults_up_runs_to_the_beacon_and_turns_to_the_sun() {
        let (at, _, left) = hero_pose(0.0);
        assert_eq!(at, VAULT_FROM);
        assert!(!left);
        assert_eq!(hero_pose(AIM.0 + 0.1).1, 9, "aiming up and right");
        let (at, frame, left) = hero_pose(SUMMIT.1);
        assert_eq!(at, Vec2::new(STOP_X, STAND_Y));
        assert_eq!(frame, 0);
        assert!(left);
        // Feet on the deck, short of the beacon.
        assert_eq!(STAND_Y - HERO_HALF_H, DECK_TOP);
        const { assert!(STOP_X + 6.0 < BEACON.x - 8.0) };
    }

    #[test]
    fn the_hero_never_jumps_between_beats() {
        let mut last = hero_pose(0.0).0;
        let mut t = 0.0;
        while t < SUMMIT.1 {
            t += 0.01;
            let at = hero_pose(t).0;
            assert!(at.distance(last) < 4.0, "{t}: {last} -> {at}");
            last = at;
        }
    }

    #[test]
    fn the_sky_brightens_then_the_look_back_has_its_own_backdrop() {
        assert_eq!(backdrop_at(0.0), CABINETS[C4].backdrop);
        assert_ne!(backdrop_at(SUNRISE.1), CABINETS[C4].backdrop);
        assert_eq!(backdrop_at(LOOK_BACK.0), LOOK_BACK_BACKDROP);
    }

    #[test]
    fn the_bonus_counts_in_one_life_at_a_time() {
        assert_eq!(counted(30_000, COUNT.0), 0);
        assert_eq!(counted(30_000, COUNT.1), 30_000);
        assert_eq!(counted(30_000, DONE), 30_000);
        let steps: Vec<u32> = (0..=40)
            .map(|i| lives_counted(3, COUNT.0 + (COUNT.1 - COUNT.0) * i as f32 / 40.0))
            .collect();
        assert_eq!(steps.first(), Some(&0));
        assert_eq!(steps.last(), Some(&3));
        assert!(steps.windows(2).all(|w| w[1] - w[0] <= 1));
    }

    #[test]
    fn panels_fit_on_screen() {
        let half = (CABINET_COUNT - 1) as f32 / 2.0 * PANEL_STEP + PANEL.x / 2.0 + 1.0;
        assert!(half <= HALF_W);
        const { assert!(PANEL_STEP > PANEL.x + 2.0) };
    }

    fn app(score: u64, lives: u32) -> App {
        let tuning = Tuning::default();
        let mut session = Session::new_run(&tuning);
        session.cabinet = CABINET_COUNT - 1;
        session.score = score;
        session.lives = lives;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, StatesPlugin))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_millis(
                100,
            )))
            .insert_resource(tuning)
            .insert_resource(session)
            .insert_resource(Backdrop(Color::BLACK))
            .insert_resource(HighScoreTable::default())
            .init_resource::<Controls>()
            .init_resource::<Sprites>()
            .add_message::<PlaySfx>()
            .init_state::<Screen>()
            .add_plugins(EndingPlugin);
        app.world_mut()
            .resource_mut::<NextState<Screen>>()
            .set(Screen::Ending);
        app.update();
        app
    }

    fn screen(app: &App) -> Screen {
        *app.world().resource::<State<Screen>>().get()
    }

    fn press_a(app: &mut App) {
        app.insert_resource(Controls::pressing(&[Button::A]));
        app.update();
        app.insert_resource(Controls::default());
        app.update();
    }

    #[test]
    fn the_lives_bonus_is_paid_as_the_ending_opens() {
        let app = app(5_000, 3);
        let per_life = Tuning::default().ending_life_bonus;
        assert_eq!(
            app.world().resource::<Session>().score,
            5_000 + 3 * per_life
        );
    }

    #[test]
    fn it_waits_at_the_tally_then_moves_on_like_game_over() {
        let mut app = app(5_000, 2);
        for _ in 0..200 {
            app.update();
        }
        assert_eq!(screen(&app), Screen::Ending, "waits for the player");
        let session = app.world().resource::<Session>().clone();
        let expected = if app
            .world()
            .resource::<HighScoreTable>()
            .qualifies(session.score)
        {
            Screen::NameEntry
        } else {
            Screen::Title
        };
        press_a(&mut app);
        assert_eq!(screen(&app), expected);
        app.update();
        let mut sprites = app.world_mut().query::<&Sprite>();
        assert_eq!(sprites.iter(app.world()).count(), 0, "all cleaned up");
    }

    #[test]
    fn a_skips_to_the_tally_but_not_straight_away() {
        let mut app = app(5_000, 1);
        press_a(&mut app);
        assert!(
            app.world().resource::<Ending>().elapsed < DONE,
            "too early to skip"
        );
        for _ in 0..12 {
            app.update();
        }
        press_a(&mut app);
        assert!(app.world().resource::<Ending>().elapsed >= DONE);
        assert_eq!(
            screen(&app),
            Screen::Ending,
            "skipping shows the tally first"
        );
    }
}
