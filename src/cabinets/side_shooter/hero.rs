//! The Wren: the Hero's ship in Cabinet 1.
//!
//! Moves in eight directions, fires volleys while A is held, and sets off the
//! Undertow bomb with B. Its look comes from the `*_visual` functions, plain
//! shapes for now, to be swapped for the sprites from #4.

use std::collections::HashSet;

use bevy::prelude::*;

use super::palette;
use super::power_ups::{PLAY_TOP, PowerUps};
use super::tuning::{SideShooterTuning, vec2};
use crate::cabinets::ScoreGained;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::input::{Button, Controls};
use crate::kit::{
    Damage, Explode, Hero, HitStop, Hitbox, Hurt, Invulnerable, Layer, Shield, ShieldAbsorbed,
    StageRestarted, TerrainHitbox, projectile,
};
use crate::sprites::{SpriteAnimation, Sprites};
use crate::tuning::Tuning;

pub const WREN_SIZE: Vec2 = Vec2::new(16.0, 10.0);
/// Seconds per frame of the Wren's engine flicker (15 changes a second).
const ENGINE_FLICKER_SECONDS: f32 = 1.0 / 15.0;
const DRONE_SIZE: Vec2 = Vec2::splat(8.0);
const RING_DOTS: usize = 64;

#[derive(Component)]
pub struct Wren;

/// Flying in from the left edge; no control until it arrives.
#[derive(Component)]
pub(super) struct Entry {
    from: Vec2,
    to: Vec2,
    elapsed: f32,
}

#[derive(Component, Default)]
pub(super) struct Gun {
    cooldown: f32,
    pub(super) volleys: u32,
}

/// A drone: slot 0 flies above the Wren, slot 1 below.
#[derive(Component)]
pub(super) struct Drone(u32);

#[derive(Component)]
pub(super) struct WardGlow;

/// The Undertow: a ring of force growing out from where B was pressed. It
/// only touches what its edge passes over, so bullets fired into the area
/// it has already swept are not cancelled.
#[derive(Component)]
pub(super) struct Undertow {
    radius: f32,
    hit: HashSet<Entity>,
}

#[derive(Component)]
pub(super) struct UndertowDot(usize);

#[derive(Component)]
pub(super) struct BombIcon(u32);

/// One shot of a volley, relative to the Wren.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotSpec {
    pub offset: Vec2,
    pub direction: Vec2,
    pub size: Vec2,
    pub damage: u32,
}

/// The shots fired by one volley. `volley` counts volleys from 0; the Tail
/// fires on every `tail_every`-th one.
pub fn volley(power: &PowerUps, volley: u32, t: &SideShooterTuning) -> Vec<ShotSpec> {
    let s = &t.shot;
    let nose = Vec2::new(WREN_SIZE.x / 2.0, 0.0);
    let forward = |offset: Vec2, size: (f32, f32), damage: u32| ShotSpec {
        offset,
        direction: Vec2::X,
        size: vec2(size),
        damage,
    };
    let mut shots = Vec::new();
    let (size, damage) = if power.gun >= 4 {
        (s.lance_size, s.lance_damage)
    } else {
        (s.size, s.damage)
    };
    if power.gun <= 1 {
        shots.push(forward(nose, s.size, s.damage));
    } else {
        let half = Vec2::new(0.0, s.twin_spacing / 2.0);
        shots.push(forward(nose + half, size, damage));
        shots.push(forward(nose - half, size, damage));
    }
    if power.gun >= 3 {
        for sign in [1.0, -1.0] {
            shots.push(ShotSpec {
                offset: nose,
                direction: Vec2::from_angle((sign * s.spread_degrees).to_radians()),
                size: vec2(s.size),
                damage: s.damage,
            });
        }
    }
    for slot in 0..power.drones.min(2) {
        let drone = drone_offset(slot, t) + Vec2::new(DRONE_SIZE.x / 2.0, 0.0);
        shots.push(forward(drone, s.size, s.damage));
    }
    if power.tail && volley.is_multiple_of(s.tail_every.max(1)) {
        shots.push(ShotSpec {
            offset: -nose,
            direction: Vec2::NEG_X,
            size: vec2(s.size),
            damage: s.damage,
        });
    }
    shots
}

fn drone_offset(slot: u32, t: &SideShooterTuning) -> Vec2 {
    let y = t.power_up.drone_offset;
    Vec2::new(0.0, if slot == 0 { y } else { -y })
}

/// Where the Wren's centre may go: inside the play area with a margin.
pub fn clamp_to_play_area(at: Vec2, t: &SideShooterTuning) -> Vec2 {
    let half = WREN_SIZE / 2.0 + t.hero.edge_margin;
    Vec2::new(
        at.x.clamp(-HALF_W + half.x, HALF_W - half.x),
        at.y.clamp(-HALF_H + half.y, PLAY_TOP - half.y),
    )
}

/// The Wren's look: `assets/sprites/hero.txt` in Cabinet 1's palette (Foam
/// body, Spark canopy), its two frames flickering the Ember engine.
fn wren_visual(sprites: &Sprites) -> impl Bundle {
    (
        sprites.sprite("hero", 0),
        SpriteAnimation::looping([0, 1], ENGINE_FLICKER_SECONDS),
        children![(ward_visual(), Visibility::Hidden, WardGlow)],
    )
}

/// The Ward: a Spark frame around the Wren.
fn ward_visual() -> impl Bundle {
    let (w, h) = (WREN_SIZE.x + 6.0, WREN_SIZE.y + 6.0);
    let bar = |size: Vec2, x: f32, y: f32| {
        (
            Sprite::from_color(palette::SPARK.with_alpha(0.8), size),
            Transform::from_xyz(x, y, 0.0),
        )
    };
    (
        Transform::from_xyz(0.0, 0.0, 0.2),
        children![
            bar(Vec2::new(w, 1.0), 0.0, h / 2.0),
            bar(Vec2::new(w, 1.0), 0.0, -h / 2.0),
            bar(Vec2::new(1.0, h), w / 2.0, 0.0),
            bar(Vec2::new(1.0, h), -w / 2.0, 0.0),
        ],
    )
}

/// A drone: a small Spark pod. Swap for the #4 sprite.
fn drone_visual() -> impl Bundle {
    (
        Sprite::from_color(palette::SPARK, DRONE_SIZE),
        children![(
            Sprite::from_color(palette::FOAM, Vec2::splat(2.0)),
            Transform::from_xyz(1.0, 0.0, 0.1),
        )],
    )
}

/// One dot of the Undertow ring. Swap for the #4 sprite.
fn undertow_dot_visual() -> impl Bundle {
    Sprite::from_color(palette::FOAM, Vec2::splat(2.0))
}

/// One bomb in the stock shown at the bottom left. Swap for the #4 sprite.
fn bomb_icon_visual() -> impl Bundle {
    (
        Sprite::from_color(palette::EMBER, Vec2::splat(5.0)),
        children![(
            Sprite::from_color(palette::NIGHT, Vec2::splat(1.0)),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}

fn entry_for(t: &SideShooterTuning) -> Entry {
    let to = vec2(t.hero.start);
    Entry {
        from: Vec2::new(t.hero.entry_from_x, to.y),
        to,
        elapsed: 0.0,
    }
}

/// Invulnerable through the entry and for the respawn time after it.
fn entry_invulnerability(t: &SideShooterTuning) -> f32 {
    t.hero.entry_seconds + t.hero.respawn_invulnerable_seconds
}

pub(super) fn spawn_wren(
    commands: &mut Commands,
    sprites: &Sprites,
    t: &SideShooterTuning,
    power: &PowerUps,
) {
    let entry = entry_for(t);
    let hitbox = vec2(t.hero.hitbox);
    let terrain = vec2(t.hero.terrain_hitbox);
    commands.spawn((
        wren_visual(sprites),
        Transform::from_translation(entry.from.extend(5.0)),
        (Hero, Wren, Layer::Hero),
        Hitbox::new(hitbox.x, hitbox.y),
        TerrainHitbox(Hitbox::new(terrain.x, terrain.y)),
        Invulnerable(entry_invulnerability(t)),
        Shield {
            charges: power.ward as u32,
            invulnerable_seconds: t.power_up.ward_invulnerable_seconds,
        },
        HitStop(t.hero.hit_stop_seconds),
        Gun::default(),
        entry,
        DespawnOnExit(Screen::Playing),
    ));
    for i in 0..t.bomb.max {
        commands.spawn((
            bomb_icon_visual(),
            Transform::from_xyz(-HALF_W + 6.0 + i as f32 * 8.0, -HALF_H + 6.0, 40.0),
            BombIcon(i),
            DespawnOnExit(Screen::Playing),
        ));
    }
}

pub(super) fn fly_in(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut wren: Single<(Entity, &mut Entry, &mut Transform), With<Wren>>,
) {
    let (entity, entry, transform) = &mut *wren;
    entry.elapsed += time.delta_secs();
    let f = (entry.elapsed / tuning.side_shooter.hero.entry_seconds.max(0.01)).min(1.0);
    // Ease out: fast in from the edge, settling into place.
    let eased = 1.0 - (1.0 - f) * (1.0 - f);
    let at = entry.from.lerp(entry.to, eased);
    transform.translation.x = at.x.round();
    transform.translation.y = at.y.round();
    if f >= 1.0 {
        commands.entity(*entity).remove::<Entry>();
    }
}

pub(super) fn steer(
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    power: Res<PowerUps>,
    mut wren: Single<&mut Transform, (With<Wren>, Without<Entry>)>,
) {
    let t = &tuning.side_shooter;
    let step = controls.direction().normalize_or_zero() * power.hero_speed(t) * time.delta_secs();
    let at = clamp_to_play_area(wren.translation.truncate() + step, t);
    wren.translation.x = at.x;
    wren.translation.y = at.y;
}

pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    power: Res<PowerUps>,
    mut wren: Single<(&mut Gun, &Transform, Has<Entry>), With<Wren>>,
    shots: Query<&Layer>,
) {
    let t = &tuning.side_shooter;
    let (gun, transform, entering) = &mut *wren;
    gun.cooldown -= time.delta_secs();
    if *entering || !controls.held(Button::A) {
        gun.cooldown = gun.cooldown.max(0.0);
        return;
    }
    if gun.cooldown > 0.0 {
        return;
    }
    // Carry the time left over, so the rate is the same at every frame rate.
    // A frame long enough to owe two volleys still fires only one.
    gun.cooldown = (gun.cooldown + t.shot.fire_interval).max(0.0);
    let volley = volley(&power, gun.volleys, t);
    gun.volleys = gun.volleys.wrapping_add(1);
    // The shot cap is a safety valve: skip a whole volley, never part of one.
    let alive = shots.iter().filter(|l| **l == Layer::HeroShot).count();
    if alive + volley.len() > t.shot.max_on_screen {
        return;
    }
    let at = transform.translation.truncate();
    for shot in volley {
        let from = at + shot.offset;
        commands
            .spawn(projectile(
                Layer::HeroShot,
                from,
                shot.direction * t.shot.speed,
                shot.size,
                palette::EMBER,
                2.0,
            ))
            .insert((
                // Angled shots are drawn along their path. The hitbox stays
                // axis-aligned, which is close enough for a 10° shot.
                Transform::from_translation(from.extend(8.0))
                    .with_rotation(Quat::from_rotation_z(shot.direction.to_angle())),
                Damage(shot.damage),
            ));
    }
}

/// Keeps the drones and the Ward in step with the Power-ups.
pub(super) fn sync_equipment(
    mut commands: Commands,
    tuning: Res<Tuning>,
    power: Res<PowerUps>,
    mut wren: Single<(&Transform, &mut Shield), With<Wren>>,
    mut glow: Single<&mut Visibility, With<WardGlow>>,
    mut drones: Query<(Entity, &Drone, &mut Transform), Without<Wren>>,
) {
    let t = &tuning.side_shooter;
    let (wren_transform, shield) = &mut *wren;
    let at = wren_transform.translation.truncate();
    let mut have = [false; 2];
    for (entity, drone, mut transform) in &mut drones {
        if drone.0 >= power.drones {
            commands.entity(entity).despawn();
            continue;
        }
        have[drone.0 as usize] = true;
        let pos = at + drone_offset(drone.0, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
    for slot in 0..power.drones.min(2) {
        if !have[slot as usize] {
            let pos = at + drone_offset(slot, t);
            commands.spawn((
                drone_visual(),
                Transform::from_translation(pos.extend(5.0)),
                Drone(slot),
                Layer::ShotBlocker,
                Hitbox::new(DRONE_SIZE.x, DRONE_SIZE.y),
                DespawnOnExit(Screen::Playing),
            ));
        }
    }
    if power.is_changed() {
        shield.charges = power.ward as u32;
        shield.invulnerable_seconds = t.power_up.ward_invulnerable_seconds;
    }
    glow.set_if_neq(if power.ward {
        Visibility::Inherited
    } else {
        Visibility::Hidden
    });
}

/// Drones blink with the Wren. Runs after the kit's blink.
pub(super) fn drones_blink(
    wren: Single<&Visibility, With<Wren>>,
    mut drones: Query<&mut Visibility, (With<Drone>, Without<Wren>)>,
) {
    for mut visibility in &mut drones {
        visibility.set_if_neq(**wren);
    }
}

/// The Ward took a hit: it's gone, and a terrain hit pushes the Wren away.
pub(super) fn ward_absorbed(
    tuning: Res<Tuning>,
    mut absorbed: MessageReader<ShieldAbsorbed>,
    mut power: ResMut<PowerUps>,
    mut wren: Single<&mut Transform, With<Wren>>,
    hitters: Query<(&Layer, &Transform), Without<Wren>>,
    mut explode: MessageWriter<Explode>,
) {
    let t = &tuning.side_shooter;
    for hit in absorbed.read() {
        power.ward = false;
        let at = wren.translation.truncate();
        explode.write(Explode {
            at,
            color: palette::SPARK,
        });
        if let Some((Layer::Terrain, from)) = hit.from.and_then(|e| hitters.get(e).ok()) {
            let away = (at - from.translation.truncate()).normalize_or(Vec2::NEG_X);
            let pushed = clamp_to_play_area(at + away * t.power_up.ward_push, t);
            wren.translation.x = pushed.x;
            wren.translation.y = pushed.y;
        }
    }
}

/// After a lost life: one step of power is gone and the Wren flies in again.
pub(super) fn after_restart(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut restarted: MessageReader<StageRestarted>,
    mut power: ResMut<PowerUps>,
    mut wren: Single<(Entity, &mut Transform, &mut Invulnerable, &mut Gun), With<Wren>>,
    rings: Query<Entity, With<Undertow>>,
) {
    if restarted.read().count() == 0 {
        return;
    }
    let t = &tuning.side_shooter;
    power.lose_life(t);
    let (entity, transform, invulnerable, gun) = &mut *wren;
    let entry = entry_for(t);
    transform.translation.x = entry.from.x;
    transform.translation.y = entry.from.y;
    invulnerable.0 = entry_invulnerability(t);
    **gun = Gun::default();
    commands.entity(*entity).insert(entry);
    for ring in &rings {
        commands.entity(ring).despawn();
    }
}

pub(super) fn bomb(
    mut commands: Commands,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    mut power: ResMut<PowerUps>,
    mut wren: Single<(&Transform, &mut Invulnerable), (With<Wren>, Without<Entry>)>,
    rings: Query<(), With<Undertow>>,
) {
    let t = &tuning.side_shooter;
    if !controls.just_pressed(Button::B) || power.bombs == 0 || !rings.is_empty() {
        return;
    }
    power.bombs -= 1;
    let (transform, invulnerable) = &mut *wren;
    invulnerable.0 = invulnerable.0.max(t.bomb.invulnerable_seconds);
    let at = transform.translation.truncate();
    commands
        .spawn((
            Transform::from_translation(at.extend(25.0)),
            Visibility::default(),
            Undertow {
                radius: 0.0,
                hit: HashSet::new(),
            },
            DespawnOnExit(Screen::Playing),
        ))
        .with_children(|ring| {
            for i in 0..RING_DOTS {
                ring.spawn((undertow_dot_visual(), Transform::default(), UndertowDot(i)));
            }
        });
}

/// The distance from `centre` to the nearest point of a box.
fn distance_to_box(centre: Vec2, at: Vec2, half: Vec2) -> f32 {
    let nearest = centre.clamp(at - half, at + half);
    centre.distance(nearest)
}

/// The distance from `centre` to the farthest corner of a box.
fn farthest_on_box(centre: Vec2, at: Vec2, half: Vec2) -> f32 {
    ((at - centre).abs() + half).length()
}

/// Whether a ring growing from `inner` to `outer` this frame passes over a
/// box. Enemy bullets are slower than the ring, so the edge can't skip one.
fn ring_touches(centre: Vec2, inner: f32, outer: f32, at: Vec2, half: Vec2) -> bool {
    distance_to_box(centre, at, half) <= outer && farthest_on_box(centre, at, half) >= inner
}

pub(super) fn grow_undertow(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut rings: Query<(Entity, &mut Undertow, &Transform)>,
    mut dots: Query<(&UndertowDot, &mut Transform), Without<Undertow>>,
    targets: Query<
        (Entity, &Layer, &Transform, Option<&Hitbox>),
        (Without<Undertow>, Without<UndertowDot>),
    >,
    mut hurt: MessageWriter<Hurt>,
    mut score: MessageWriter<ScoreGained>,
    mut explode: MessageWriter<Explode>,
) {
    let t = &tuning.side_shooter;
    for (entity, mut ring, transform) in &mut rings {
        // The band covers two frames' growth, so a bullet flying in towards
        // the centre can't slip between this frame's edge and the last.
        let step = t.bomb.ring_speed * time.delta_secs();
        let inner = ring.radius - step;
        ring.radius += step;
        let centre = transform.translation.truncate();
        let mut cancelled = 0;
        for (target, layer, at, hitbox) in &targets {
            let half = hitbox.map_or(Vec2::ZERO, |h| h.size / 2.0);
            let pos = at.translation.truncate();
            if !ring_touches(centre, inner, ring.radius, pos, half) {
                continue;
            }
            match layer {
                Layer::EnemyShot => {
                    commands.entity(target).try_despawn();
                    cancelled += 1;
                    explode.write(Explode {
                        at: pos,
                        color: palette::SIGNAL,
                    });
                }
                Layer::Enemy if ring.hit.insert(target) => {
                    hurt.write(Hurt {
                        target,
                        damage: t.bomb.damage,
                    });
                }
                _ => {}
            }
        }
        if cancelled > 0 {
            score.write(ScoreGained(cancelled * t.score.bomb_cancel));
        }
        // Done once the ring has passed every corner of the screen.
        let corner = Vec2::new(HALF_W, HALF_H);
        let reach = (centre.abs() + corner).length();
        if ring.radius > reach {
            commands.entity(entity).despawn();
        }
    }
    for (dot, mut transform) in &mut dots {
        let Ok((_, ring, _)) = rings.single() else {
            continue;
        };
        let angle = dot.0 as f32 / RING_DOTS as f32 * std::f32::consts::TAU;
        let at = (Vec2::from_angle(angle) * ring.radius).round();
        transform.translation.x = at.x;
        transform.translation.y = at.y;
    }
}

pub(super) fn show_bombs(power: Res<PowerUps>, mut icons: Query<(&BombIcon, &mut Visibility)>) {
    for (icon, mut visibility) in &mut icons {
        visibility.set_if_neq(if icon.0 < power.bombs {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn power(gun: u32, drones: u32, tail: bool) -> PowerUps {
        PowerUps {
            gun,
            drones,
            tail,
            ..PowerUps::default()
        }
    }

    fn t() -> SideShooterTuning {
        SideShooterTuning::default()
    }

    #[test]
    fn gun_levels_follow_the_design_doc() {
        let t = t();
        let g1 = volley(&power(1, 0, false), 0, &t);
        assert_eq!(g1.len(), 1);
        assert_eq!(g1[0].direction, Vec2::X);

        let g2 = volley(&power(2, 0, false), 0, &t);
        assert_eq!(g2.len(), 2);
        assert_eq!((g2[0].offset.y - g2[1].offset.y).abs(), 6.0);

        let g3 = volley(&power(3, 0, false), 0, &t);
        assert_eq!(g3.len(), 4);
        let angles: Vec<_> = g3[2..]
            .iter()
            .map(|s| s.direction.to_angle().to_degrees().round())
            .collect();
        assert_eq!(angles, [10.0, -10.0]);

        let g4 = volley(&power(4, 0, false), 0, &t);
        assert_eq!(g4.len(), 4);
        assert!(
            g4[..2]
                .iter()
                .all(|s| s.size == Vec2::new(12.0, 3.0) && s.damage == 2)
        );
        assert!(
            g4[2..]
                .iter()
                .all(|s| s.size == Vec2::new(8.0, 2.0) && s.damage == 1)
        );
    }

    #[test]
    fn drones_add_a_forward_shot_each_and_the_tail_fires_every_second_volley() {
        let t = t();
        let p = power(4, 2, true);
        let counts: Vec<_> = (0..4).map(|v| volley(&p, v, &t).len()).collect();
        assert_eq!(counts, [7, 6, 7, 6], "6.5 shots per volley on average");
        let tail: Vec<_> = volley(&p, 0, &t)
            .into_iter()
            .filter(|s| s.direction == Vec2::NEG_X)
            .collect();
        assert_eq!(tail.len(), 1);
    }

    #[test]
    fn full_power_stays_well_under_the_shot_cap() {
        // A forward shot crosses the screen in under a second; at 10 volleys
        // per second that is about 65 shots alive, below the cap of 96.
        let t = t();
        let p = power(4, 2, true);
        let per_second: usize = (0..10).map(|v| volley(&p, v, &t).len()).sum();
        let lifetime = (HALF_W * 2.0) / t.shot.speed;
        assert!((per_second as f32 * lifetime) < t.shot.max_on_screen as f32);
    }

    #[test]
    fn the_wren_stays_inside_the_play_area() {
        let t = t();
        let corner = clamp_to_play_area(Vec2::new(500.0, 500.0), &t);
        assert_eq!(corner, Vec2::new(HALF_W - 12.0, PLAY_TOP - 9.0));
        let other = clamp_to_play_area(Vec2::new(-500.0, -500.0), &t);
        assert_eq!(other, Vec2::new(-HALF_W + 12.0, -HALF_H + 9.0));
    }

    #[test]
    fn ring_reaches_boxes_by_their_nearest_edge() {
        let half = Vec2::new(5.0, 5.0);
        assert_eq!(
            distance_to_box(Vec2::ZERO, Vec2::new(20.0, 0.0), half),
            15.0
        );
        assert_eq!(distance_to_box(Vec2::ZERO, Vec2::ZERO, half), 0.0);
    }
}
