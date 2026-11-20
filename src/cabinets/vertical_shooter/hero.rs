//! The Wren, nose up: the Hero's ship in Cabinet 2.
//!
//! Moves in eight directions, fires volleys up the screen while A is held,
//! launches homing Rockets at ground targets once it has them, and sets off
//! the Squall with B. The ship itself is Cabinet 1's `hero` sprite turned
//! 90°, so it is recognisably the same craft.

use std::collections::HashSet;
use std::f32::consts::FRAC_PI_2;

use bevy::prelude::*;

use super::palette;
use super::power_ups::PowerUps;
use super::tuning::{VerticalShooterTuning, vec2};
use crate::cabinets::ScoreGained;
use crate::cabinets::shooter::pattern::turn_toward;
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::input::{Button, Controls};
use crate::kit::{
    Damage, Explode, Hero, HitStop, Hitbox, Hurt, Invulnerable, Layer, Shield, ShieldAbsorbed,
    StageRestarted, Velocity, projectile,
};
use crate::sprites::{SpriteAnimation, Sprites};
use crate::tuning::Tuning;

/// The Wren nose up: Cabinet 1's 16×10 sprite turned 90°.
pub const WREN_SIZE: Vec2 = Vec2::new(10.0, 16.0);
const ENGINE_FLICKER_SECONDS: f32 = 1.0 / 15.0;
const POD_SIZE: Vec2 = Vec2::splat(8.0);
/// Which Cabinet palette the shared sprites are baked in.
const CABINET: usize = super::CABINET;

#[derive(Component)]
pub struct Wren;

/// Flying up from the bottom edge; no control until it arrives.
#[derive(Component)]
pub(super) struct Entry {
    from: Vec2,
    to: Vec2,
    elapsed: f32,
}

#[derive(Component, Default)]
pub(super) struct Gun {
    cooldown: f32,
    /// Seconds until the next pair of Rockets may launch.
    rocket_cooldown: f32,
}

/// A Pod: slot 0 flies to the Wren's left, slot 1 to its right.
#[derive(Component)]
pub(super) struct Pod(u32);

#[derive(Component)]
pub(super) struct WardGlow;

/// A homing Rocket: it hits ground targets only.
#[derive(Component)]
pub struct Rocket {
    pub target: Option<Entity>,
    /// Radians.
    pub heading: f32,
}

/// The Squall: two edges sweeping up and down from the row where B was
/// pressed. Only the moving edges touch things, as with Cabinet 1's ring.
#[derive(Component)]
pub(super) struct Squall {
    pub(super) up: f32,
    pub(super) down: f32,
    hit: HashSet<Entity>,
}

/// One of the Squall's two lines (+1 up, -1 down).
#[derive(Component)]
pub(super) struct SquallEdge(f32);

#[derive(Component)]
pub(super) struct BombIcon(u32);

/// A shadow cast on the ground by an air sprite (the Wren, a Pod, an air
/// enemy). It is a child, so it follows its owner.
#[derive(Component)]
pub struct Shadow;

/// One shot of a volley, relative to the Wren.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ShotSpec {
    pub offset: Vec2,
    pub direction: Vec2,
    pub size: Vec2,
    pub damage: u32,
}

/// The shots fired by one volley.
pub fn volley(power: &PowerUps, t: &VerticalShooterTuning) -> Vec<ShotSpec> {
    let s = &t.shot;
    let nose = Vec2::new(0.0, WREN_SIZE.y / 2.0);
    let up = |offset: Vec2, size: (f32, f32), damage: u32| ShotSpec {
        offset,
        direction: Vec2::Y,
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
        shots.push(up(nose, s.size, s.damage));
    } else {
        let half = Vec2::new(s.twin_spacing / 2.0, 0.0);
        shots.push(up(nose - half, size, damage));
        shots.push(up(nose + half, size, damage));
    }
    if power.gun >= 3 {
        for sign in [1.0, -1.0] {
            shots.push(ShotSpec {
                offset: nose,
                direction: Vec2::from_angle((90.0 + sign * s.spread_degrees).to_radians()),
                size: vec2(s.size),
                damage: s.damage,
            });
        }
    }
    for slot in 0..power.pods.min(2) {
        let pod = pod_offset(slot, t) + Vec2::new(0.0, POD_SIZE.y / 2.0);
        shots.push(up(pod, s.size, s.damage));
    }
    shots
}

fn pod_offset(slot: u32, t: &VerticalShooterTuning) -> Vec2 {
    let x = t.power_up.pod_offset;
    Vec2::new(if slot == 0 { -x } else { x }, 0.0)
}

/// Where the Wren's centre may go: inside the play area with a margin.
pub fn clamp_to_play_area(at: Vec2, t: &VerticalShooterTuning) -> Vec2 {
    let half = WREN_SIZE / 2.0 + t.hero.edge_margin;
    Vec2::new(
        at.x.clamp(-HALF_W + half.x, HALF_W - half.x),
        at.y.clamp(-HALF_H + half.y, PLAY_TOP - half.y),
    )
}

/// A Dusk shadow of `size`, offset onto the ground below its owner, whose
/// z is `owner_z` (so the shadow lands under the ground targets at z 2–3).
pub fn shadow(size: Vec2, owner_z: f32, t: &VerticalShooterTuning) -> impl Bundle {
    let offset = vec2(t.look.shadow_offset);
    (
        Shadow,
        Sprite::from_color(palette::DUSK.with_alpha(0.85), size),
        Transform::from_xyz(offset.x, offset.y, 1.5 - owner_z),
    )
}

/// The Wren's look: the shared `hero` sprite in Cabinet 2's palette (Foam
/// body, Orchid canopy, Ember engine), turned nose up, with its shadow.
fn wren_visual(sprites: &Sprites, t: &VerticalShooterTuning) -> impl Bundle {
    (
        Visibility::default(),
        children![
            (
                sprites.sprite("hero", CABINET),
                SpriteAnimation::looping([0, 1], ENGINE_FLICKER_SECONDS),
                Transform::from_rotation(Quat::from_rotation_z(FRAC_PI_2)),
            ),
            shadow(Vec2::new(8.0, 12.0), 5.0, t),
            (ward_visual(), Visibility::Hidden, WardGlow),
        ],
    )
}

/// The Ward: an Orchid frame around the Wren.
fn ward_visual() -> impl Bundle {
    let (w, h) = (WREN_SIZE.x + 6.0, WREN_SIZE.y + 6.0);
    let bar = |size: Vec2, x: f32, y: f32| {
        (
            Sprite::from_color(palette::ORCHID.with_alpha(0.8), size),
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

/// A Pod: a small Orchid block with a Foam muzzle. Swap for a #15 sprite.
fn pod_visual(t: &VerticalShooterTuning) -> impl Bundle {
    (
        Sprite::from_color(palette::ORCHID, POD_SIZE),
        children![
            (
                Sprite::from_color(palette::FOAM, Vec2::splat(2.0)),
                Transform::from_xyz(0.0, 2.0, 0.1),
            ),
            shadow(Vec2::splat(6.0), 5.0, t),
        ],
    )
}

/// One of the Squall's lines across the screen.
fn squall_edge_visual() -> impl Bundle {
    Sprite::from_color(palette::FOAM, Vec2::new(HALF_W * 2.0, 2.0))
}

/// One bomb in the stock at the bottom left.
fn bomb_icon_visual() -> impl Bundle {
    (
        Sprite::from_color(palette::EMBER, Vec2::splat(5.0)),
        children![(
            Sprite::from_color(palette::DUSK, Vec2::splat(1.0)),
            Transform::from_xyz(0.0, 0.0, 0.1),
        )],
    )
}

fn entry_for(t: &VerticalShooterTuning) -> Entry {
    let to = vec2(t.hero.start);
    Entry {
        from: Vec2::new(to.x, t.hero.entry_from_y),
        to,
        elapsed: 0.0,
    }
}

fn entry_invulnerability(t: &VerticalShooterTuning) -> f32 {
    t.hero.entry_seconds + t.hero.respawn_invulnerable_seconds
}

pub(super) fn spawn_wren(
    commands: &mut Commands,
    sprites: &Sprites,
    t: &VerticalShooterTuning,
    power: &PowerUps,
) {
    let entry = entry_for(t);
    let hitbox = vec2(t.hero.hitbox);
    commands.spawn((
        wren_visual(sprites, t),
        Transform::from_translation(entry.from.extend(5.0)),
        (Hero, Wren, Layer::Hero),
        Hitbox::new(hitbox.x, hitbox.y),
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
    let f = (entry.elapsed / tuning.vertical_shooter.hero.entry_seconds.max(0.01)).min(1.0);
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
    let t = &tuning.vertical_shooter;
    let step = controls.direction().normalize_or_zero() * power.hero_speed(t) * time.delta_secs();
    let at = clamp_to_play_area(wren.translation.truncate() + step, t);
    wren.translation.x = at.x;
    wren.translation.y = at.y;
}

/// Whether a ground target at `target` is one the Rockets go for: on screen
/// and above `from`.
pub fn rocket_can_target(from: Vec2, target: Vec2) -> bool {
    target.y > from.y && target.x.abs() <= HALF_W && target.y.abs() <= HALF_H
}

/// The nearest ground target a Rocket at `from` should go for.
pub fn nearest_ground_target(
    from: Vec2,
    targets: impl Iterator<Item = (Entity, Vec2)>,
) -> Option<Entity> {
    targets
        .filter(|(_, at)| rocket_can_target(from, *at))
        .min_by(|a, b| from.distance(a.1).total_cmp(&from.distance(b.1)))
        .map(|(e, _)| e)
}

#[allow(clippy::too_many_arguments)]
pub(super) fn fire(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    power: Res<PowerUps>,
    mut wren: Single<(&mut Gun, &Transform, Has<Entry>), With<Wren>>,
    shots: Query<&Layer>,
    ground: Query<(Entity, &Layer, &Transform), Without<Wren>>,
) {
    let t = &tuning.vertical_shooter;
    let (gun, transform, entering) = &mut *wren;
    let dt = time.delta_secs();
    gun.cooldown -= dt;
    gun.rocket_cooldown -= dt;
    if *entering || !controls.held(Button::A) {
        gun.cooldown = gun.cooldown.max(0.0);
        gun.rocket_cooldown = gun.rocket_cooldown.max(0.0);
        return;
    }
    let at = transform.translation.truncate();

    // Rockets launch on their own clock, and only with a ground target ahead.
    if power.rockets && gun.rocket_cooldown <= 0.0 {
        let targets = ground
            .iter()
            .filter(|(_, l, _)| **l == Layer::Ground)
            .map(|(e, _, tr)| (e, tr.translation.truncate()));
        if let Some(target) = nearest_ground_target(at, targets) {
            gun.rocket_cooldown = (gun.rocket_cooldown + t.rocket.interval).max(0.0);
            launch_rockets(&mut commands, at, target, t);
        }
    }

    if gun.cooldown > 0.0 {
        return;
    }
    gun.cooldown = (gun.cooldown + t.shot.fire_interval).max(0.0);
    let volley = volley(&power, t);
    let alive = shots.iter().filter(|l| **l == Layer::HeroShot).count();
    if alive + volley.len() > t.shot.max_on_screen {
        return;
    }
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
                // Shots are drawn along their path; angled ones are drawn
                // turned, with an axis-aligned hitbox (under 1 px off).
                Transform::from_translation(from.extend(8.0))
                    .with_rotation(Quat::from_rotation_z(shot.direction.to_angle() - FRAC_PI_2)),
                Damage(shot.damage),
            ));
    }
}

fn launch_rockets(commands: &mut Commands, at: Vec2, target: Entity, t: &VerticalShooterTuning) {
    let r = &t.rocket;
    let size = vec2(r.size);
    let (ox, oy) = r.launch_offset;
    for i in 0..r.count {
        // Alternate wings: left, right, left...
        let side = if i % 2 == 0 { -1.0 } else { 1.0 };
        let from = at + Vec2::new(side * ox, oy);
        commands
            .spawn(projectile(
                Layer::Rocket,
                from,
                Vec2::Y * r.speed,
                size,
                palette::EMBER,
                4.0,
            ))
            .insert((
                Rocket {
                    target: Some(target),
                    heading: FRAC_PI_2,
                },
                Damage(r.damage),
            ));
    }
}

/// Rockets turn toward their target, picking a new one if it is gone.
pub(super) fn steer_rockets(
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut rockets: Query<(&mut Rocket, &mut Velocity, &mut Transform)>,
    ground: Query<(Entity, &Layer, &Transform), Without<Rocket>>,
) {
    let r = &tuning.vertical_shooter.rocket;
    let dt = time.delta_secs();
    let targets: Vec<(Entity, Vec2)> = ground
        .iter()
        .filter(|(_, l, _)| **l == Layer::Ground)
        .map(|(e, _, t)| (e, t.translation.truncate()))
        .collect();
    for (mut rocket, mut velocity, mut transform) in &mut rockets {
        let at = transform.translation.truncate();
        let still_there = rocket
            .target
            .and_then(|e| targets.iter().find(|(t, _)| *t == e))
            .map(|(_, p)| *p);
        let aim = match still_there {
            Some(p) => Some(p),
            None => {
                rocket.target = nearest_ground_target(at, targets.iter().copied());
                rocket
                    .target
                    .and_then(|e| targets.iter().find(|(t, _)| *t == e))
                    .map(|(_, p)| *p)
            }
        };
        if let Some(aim) = aim {
            let want = (aim - at).to_angle();
            rocket.heading = turn_toward(rocket.heading, want, r.turn_rate.to_radians() * dt);
        }
        velocity.0 = Vec2::from_angle(rocket.heading) * r.speed;
        transform.rotation = Quat::from_rotation_z(rocket.heading - FRAC_PI_2);
    }
}

/// Keeps the Pods and the Ward in step with the Power-ups.
pub(super) fn sync_equipment(
    mut commands: Commands,
    tuning: Res<Tuning>,
    power: Res<PowerUps>,
    mut wren: Single<(&Transform, &mut Shield), With<Wren>>,
    mut glow: Single<&mut Visibility, With<WardGlow>>,
    mut pods: Query<(Entity, &Pod, &mut Transform), Without<Wren>>,
) {
    let t = &tuning.vertical_shooter;
    let (wren_transform, shield) = &mut *wren;
    let at = wren_transform.translation.truncate();
    let mut have = [false; 2];
    for (entity, pod, mut transform) in &mut pods {
        // Losing a Pod takes the right-hand one (slot 1) first.
        if pod.0 >= power.pods {
            commands.entity(entity).despawn();
            continue;
        }
        have[pod.0 as usize] = true;
        let pos = at + pod_offset(pod.0, t);
        transform.translation.x = pos.x;
        transform.translation.y = pos.y;
    }
    for slot in 0..power.pods.min(2) {
        if !have[slot as usize] {
            let pos = at + pod_offset(slot, t);
            commands.spawn((
                pod_visual(t),
                Transform::from_translation(pos.extend(5.0)),
                Pod(slot),
                Layer::ShotBlocker,
                Hitbox::new(POD_SIZE.x, POD_SIZE.y),
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

/// Pods blink with the Wren. Runs after the kit's blink.
pub(super) fn pods_blink(
    wren: Single<&Visibility, With<Wren>>,
    mut pods: Query<&mut Visibility, (With<Pod>, Without<Wren>)>,
) {
    for mut visibility in &mut pods {
        visibility.set_if_neq(**wren);
    }
}

/// The Ward took a hit: it's gone.
pub(super) fn ward_absorbed(
    mut absorbed: MessageReader<ShieldAbsorbed>,
    mut power: ResMut<PowerUps>,
    wren: Single<&Transform, With<Wren>>,
    mut explode: MessageWriter<Explode>,
) {
    for _ in absorbed.read() {
        power.ward = false;
        explode.write(Explode {
            at: wren.translation.truncate(),
            color: palette::ORCHID,
        });
    }
}

/// After a lost life: one step of power is gone and the Wren flies in again.
pub(super) fn after_restart(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut restarted: MessageReader<StageRestarted>,
    mut power: ResMut<PowerUps>,
    mut wren: Single<(Entity, &mut Transform, &mut Invulnerable, &mut Gun), With<Wren>>,
    leftovers: Query<Entity, Or<(With<Squall>, With<Rocket>)>>,
) {
    if restarted.read().count() == 0 {
        return;
    }
    let t = &tuning.vertical_shooter;
    power.lose_life(t);
    let (entity, transform, invulnerable, gun) = &mut *wren;
    let entry = entry_for(t);
    transform.translation.x = entry.from.x;
    transform.translation.y = entry.from.y;
    invulnerable.0 = entry_invulnerability(t);
    **gun = Gun::default();
    commands.entity(*entity).insert(entry);
    for e in &leftovers {
        commands.entity(e).despawn();
    }
}

pub(super) fn bomb(
    mut commands: Commands,
    tuning: Res<Tuning>,
    controls: Res<Controls>,
    mut power: ResMut<PowerUps>,
    mut wren: Single<(&Transform, &mut Invulnerable), (With<Wren>, Without<Entry>)>,
    squalls: Query<(), With<Squall>>,
) {
    let t = &tuning.vertical_shooter;
    if !controls.just_pressed(Button::B) || power.bombs == 0 || !squalls.is_empty() {
        return;
    }
    power.bombs -= 1;
    let (transform, invulnerable) = &mut *wren;
    invulnerable.0 = invulnerable.0.max(t.bomb.invulnerable_seconds);
    let row = transform.translation.y;
    commands
        .spawn((
            Transform::from_xyz(0.0, 0.0, 25.0),
            Visibility::default(),
            Squall {
                up: row,
                down: row,
                hit: HashSet::new(),
            },
            DespawnOnExit(Screen::Playing),
        ))
        .with_children(|squall| {
            for dir in [1.0, -1.0] {
                squall.spawn((
                    squall_edge_visual(),
                    Transform::from_xyz(0.0, row, 0.0),
                    SquallEdge(dir),
                ));
            }
        });
}

/// Whether a box at `at` with half-size `half` reaches into the band of
/// rows between `lo` and `hi`.
pub fn band_touches(lo: f32, hi: f32, at: Vec2, half: Vec2) -> bool {
    at.y + half.y >= lo && at.y - half.y <= hi
}

#[allow(clippy::too_many_arguments)]
pub(super) fn sweep_squall(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut squalls: Query<(Entity, &mut Squall)>,
    mut edges: Query<(&SquallEdge, &mut Transform), Without<Squall>>,
    targets: Query<(Entity, &Layer, &Transform, Option<&Hitbox>), Without<SquallEdge>>,
    mut hurt: MessageWriter<Hurt>,
    mut score: MessageWriter<ScoreGained>,
    mut explode: MessageWriter<Explode>,
) {
    let t = &tuning.vertical_shooter;
    for (entity, mut squall) in &mut squalls {
        // Each edge covers two frames' travel, so a bullet flying toward it
        // can't slip between this frame's line and the last.
        let step = t.bomb.band_speed * time.delta_secs();
        let bands = [
            (squall.up - step, squall.up + step),
            (squall.down - step, squall.down + step),
        ];
        squall.up += step;
        squall.down -= step;
        let mut cancelled = 0;
        for (target, layer, at, hitbox) in &targets {
            let half = hitbox.map_or(Vec2::ZERO, |h| h.size / 2.0);
            let pos = at.translation.truncate();
            if !bands
                .iter()
                .any(|&(lo, hi)| band_touches(lo, hi, pos, half))
            {
                continue;
            }
            match layer {
                Layer::EnemyShot => {
                    commands.entity(target).try_despawn();
                    cancelled += 1;
                    explode.write(Explode {
                        at: pos,
                        color: palette::VOLT,
                    });
                }
                Layer::Enemy | Layer::Ground if squall.hit.insert(target) => {
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
        if squall.up > HALF_H && squall.down < -HALF_H {
            commands.entity(entity).despawn();
        }
    }
    for (edge, mut transform) in &mut edges {
        let Ok((_, squall)) = squalls.single() else {
            continue;
        };
        transform.translation.y = if edge.0 > 0.0 { squall.up } else { squall.down }.round();
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

    fn power(gun: u32, pods: u32) -> PowerUps {
        PowerUps {
            gun,
            pods,
            ..PowerUps::default()
        }
    }

    fn t() -> VerticalShooterTuning {
        VerticalShooterTuning::default()
    }

    fn degrees(s: &ShotSpec) -> f32 {
        s.direction.to_angle().to_degrees().round()
    }

    #[test]
    fn gun_levels_follow_the_design_doc() {
        let t = t();
        let g1 = volley(&power(1, 0), &t);
        assert_eq!(g1.len(), 1);
        assert_eq!(g1[0].direction, Vec2::Y);

        let g2 = volley(&power(2, 0), &t);
        assert_eq!(g2.len(), 2);
        assert_eq!((g2[0].offset.x - g2[1].offset.x).abs(), 6.0);
        assert!(g2.iter().all(|s| s.direction == Vec2::Y));

        let g3 = volley(&power(3, 0), &t);
        assert_eq!(g3.len(), 4);
        let angles: Vec<_> = g3[2..].iter().map(degrees).collect();
        assert_eq!(angles, [105.0, 75.0], "±15° from straight up");

        let g4 = volley(&power(4, 0), &t);
        assert!(
            g4[..2]
                .iter()
                .all(|s| s.size == Vec2::new(3.0, 12.0) && s.damage == 2)
        );
        assert!(g4[2..].iter().all(|s| s.damage == 1));
    }

    #[test]
    fn pods_fire_straight_up_beside_the_wren() {
        let t = t();
        let v = volley(&power(4, 2), &t);
        assert_eq!(v.len(), 6, "full power: 4 gun shots and 2 Pods");
        let pods: Vec<_> = v[4..].iter().map(|s| s.offset.x).collect();
        assert_eq!(pods, [-14.0, 14.0]);
        assert!(v[4..].iter().all(|s| s.direction == Vec2::Y));
    }

    #[test]
    fn full_power_stays_well_under_the_shot_cap() {
        let t = t();
        let per_second = volley(&power(4, 2), &t).len() as f32 / t.shot.fire_interval;
        let lifetime = (HALF_H * 2.0) / t.shot.speed;
        assert!(per_second * lifetime < t.shot.max_on_screen as f32);
    }

    #[test]
    fn the_wren_stays_inside_the_play_area() {
        let t = t();
        assert_eq!(
            clamp_to_play_area(Vec2::new(500.0, 500.0), &t),
            Vec2::new(HALF_W - 9.0, PLAY_TOP - 12.0)
        );
        assert_eq!(
            clamp_to_play_area(Vec2::new(-500.0, -500.0), &t),
            Vec2::new(-HALF_W + 9.0, -HALF_H + 12.0)
        );
    }

    #[test]
    fn rockets_pick_the_nearest_ground_target_above() {
        let e = |n| Entity::from_raw_u32(n).unwrap();
        let from = Vec2::new(0.0, -80.0);
        let targets = [
            (e(1), Vec2::new(0.0, -100.0)), // below: never
            (e(2), Vec2::new(60.0, 20.0)),
            (e(3), Vec2::new(-10.0, 0.0)),
            (e(4), Vec2::new(0.0, 300.0)), // off screen
        ];
        assert_eq!(nearest_ground_target(from, targets.into_iter()), Some(e(3)));
        assert_eq!(
            nearest_ground_target(from, targets[..1].iter().copied()),
            None
        );
    }

    #[test]
    fn the_squall_band_reaches_boxes_by_their_edges() {
        let half = Vec2::splat(4.0);
        assert!(band_touches(10.0, 12.0, Vec2::new(0.0, 15.0), half));
        assert!(!band_touches(10.0, 12.0, Vec2::new(0.0, 17.0), half));
        assert!(band_touches(10.0, 12.0, Vec2::new(0.0, 7.0), half));
    }
}
