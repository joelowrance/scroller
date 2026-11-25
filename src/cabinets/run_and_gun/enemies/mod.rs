//! Cabinet 3's enemies and hazards (docs/cabinets/3-run-and-gun.md):
//! spawning them from the Stage timeline, running their [`Brain`]s in the
//! right space, the fairness rules on enemy fire, shots under gravity, the
//! ground stopping shots, Formations, Standoff members, and the debug enemy
//! gallery.
//!
//! Stage files name an enemy by kind and may add params:
//!
//! ```ron
//! (at: 160.0, kind: "skitter", pos: (176.0, -88.0), params: {"size": 4, "drop": "G"}),
//! (at: 900.0, kind: "turret", pos: (60.0, 40.0), params: {"mount": "ceiling"}),
//! (at: 1200.0, kind: "crusher", pos: (100.0, 0.0), params: {"offset": 0.5}),
//! (at: 1300.0, kind: "arc_floor", pos: (60.0, 0.0), params: {"length": 48.0}),
//! (at: 1400.0, kind: "conveyor", pos: (40.0, 0.0), params: {"length": 64.0, "dir": "left"}),
//! ```
//!
//! For a walker or a hazard `pos.y` only picks which floor it stands on
//! (the ground or a ledge nearest to it); flyers and Turrets use it as is.
//! A walker spawned left of the screen centre comes from behind: a chevron
//! shows first and it waits off screen until the warning is over.

use bevy::prelude::*;
use serde::Deserialize;

use super::WorldX;
use super::palette;
use super::power_ups::{PowerUp, PowerUps};
use super::terrain::{Terrain, screen_x, world_x};
use super::tuning::{EnemyRules, EnemyTypes, RunAndGunTuning};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::cabinets::shooter::formation::{Formation, Member, Reward};
use crate::cabinets::shooter::pattern::Pulse;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::kit::platform::TileQuery;
use crate::kit::{
    DespawnOffscreen, EnemyDestroyed, Flash, Health, Hero, Hitbox, Hurt, Immune, KitSet, Layer,
    ScoreValue, SpawnRequested, StageRestarted, StageRun, Velocity, projectile, stage_live,
};
use crate::run::Session;
use crate::tuning::Tuning;

pub mod brain;
pub mod look;

use brain::{
    Brain, BurrowPhase, Burrower, Ctx, Dropper, Hauler, Kite, Mote, Mount, Out, Sentry,
    SentryPhase, Shot, Skitter, Strider, Tick, Turret,
};

const GALLERY_STAGE: &str = include_str!("../../../../assets/stages/3-gallery.ron");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Skitter,
    Tick,
    Kite,
    Sentry,
    Burrower,
    Dropper,
    Hauler,
    Turret,
    Mote,
    /// The Strider's core, the only part that takes damage.
    Strider,
    /// A Hauler's plough: armour, a separate immune part.
    Plough,
}

impl Kind {
    /// Flyers live in screen space; everything else in world x.
    pub fn flies(self) -> bool {
        matches!(self, Kind::Kite | Kind::Dropper | Kind::Mote)
    }

    /// Stands on the ground or a ledge (its `pos` is its feet).
    pub fn walks(self) -> bool {
        !self.flies() && self != Kind::Turret
    }
}

/// An enemy of Cabinet 3.
#[derive(Component, Debug)]
pub struct Enemy {
    pub kind: Kind,
    on_screen: f32,
    clock: f32,
}

#[derive(Component, Debug)]
pub struct EnemyBrain(pub Brain);

/// An enemy's position in its brain's space: feet in world x for walkers and
/// Turrets, the centre on screen for flyers.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyPos(pub Vec2);

/// A bullet fired by an enemy (counted for the bullet cap).
#[derive(Component)]
pub struct EnemyBullet;

/// A shot under gravity; a canister bursts along the ground when it lands.
#[derive(Component)]
pub struct Falling {
    pub gravity: f32,
    pub burst: bool,
}

/// Part of the running Standoff.
#[derive(Component)]
pub struct StandoffMember;

/// A Standoff timed out: it leaves the way it came.
#[derive(Component)]
pub struct Leaving;

/// Waiting off the left edge while its chevron blinks.
#[derive(Component)]
struct Waiting(f32);

/// Anything this module spawned that a rewind removes.
#[derive(Component)]
struct EnemyPart;

#[derive(Component)]
struct Chevron(f32);

/// A part that follows its owner: a Hauler's plough or a Strider's frame.
#[derive(Component)]
struct Follows {
    owner: Entity,
    offset: Vec2,
}

/// A Strider's shockwave running along the ground.
#[derive(Component)]
struct Shockwave {
    dir: f32,
    travelled: f32,
    ground: f32,
}

/// A hazard (not an enemy: it can't be shot).
#[derive(Component, Debug)]
pub enum Hazard {
    Crusher { offset: f32, age: f32, ground: f32 },
    ArcFloor { offset: f32, age: f32, length: f32 },
    Conveyor { length: f32, speed: f32 },
}

/// Settings a Stage file can give a spawn (see the module docs).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SpawnParams {
    #[serde(deserialize_with = "present")]
    pub size: Option<u32>,
    #[serde(deserialize_with = "present")]
    pub drop: Option<String>,
    pub standoff: bool,
    #[serde(deserialize_with = "present")]
    pub mount: Option<String>,
    pub offset: f32,
    #[serde(deserialize_with = "present")]
    pub length: Option<f32>,
    #[serde(deserialize_with = "present")]
    pub dir: Option<String>,
}

fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}

/// The debug enemy gallery, when it is switched on: it replaces the Stage.
pub fn gallery_stage(tuning: &Tuning) -> Option<&'static str> {
    (tuning.debug_keys && tuning.run_and_gun.enemy_gallery).then_some(GALLERY_STAGE)
}

/// The gallery's timeline (for tests).
#[cfg(test)]
pub fn gallery_timeline() -> crate::kit::StageTimeline {
    crate::kit::StageTimeline::parse(GALLERY_STAGE).expect("Cabinet 3 enemy gallery must be valid")
}

pub fn kind_for(name: &str) -> Option<Kind> {
    Some(match name {
        "skitter" => Kind::Skitter,
        "tick" => Kind::Tick,
        "kite" => Kind::Kite,
        "sentry" => Kind::Sentry,
        "burrower" => Kind::Burrower,
        "dropper" => Kind::Dropper,
        "hauler" => Kind::Hauler,
        "turret" => Kind::Turret,
        "mote" => Kind::Mote,
        "strider" => Kind::Strider,
        _ => return None,
    })
}

pub const HAZARD_KINDS: [&str; 3] = ["crusher", "arc_floor", "conveyor"];

/// The floor (ground or ledge top) nearest to `y` at world x `x`.
pub fn floor_near(terrain: &Terrain, x: f32, y: f32) -> Option<f32> {
    let mut floors = terrain.ledge_tops(x, x);
    floors.extend(terrain.ground_at(x));
    floors
        .into_iter()
        .min_by(|a, b| (a - y).abs().total_cmp(&(b - y).abs()))
}

/// Where each member of a spawn starts (in its brain's space) and how it
/// behaves. `lead` is on screen; `camera` converts it to world x.
pub fn layout(
    kind: Kind,
    lead: Vec2,
    params: &SpawnParams,
    hero_screen_x: f32,
    camera: f32,
    terrain: &Terrain,
    t: &EnemyTypes,
) -> Vec<(Vec2, Brain)> {
    let size = |default: u32| params.size.unwrap_or(default).max(1);
    // Walkers head for the Hero; one spawned on the left comes from behind.
    let dir = if lead.x > hero_screen_x { -1.0 } else { 1.0 };
    let wx = world_x(lead.x, camera);
    let feet = |x: f32| floor_near(terrain, x, lead.y).unwrap_or(-96.0);
    match kind {
        Kind::Skitter => (0..size(t.skitter.formation_size))
            .map(|i| {
                let x = wx - dir * t.skitter.spacing * i as f32;
                (Vec2::new(x, feet(x)), Brain::Skitter(Skitter::new(dir)))
            })
            .collect(),
        Kind::Tick => (0..size(t.tick.formation_size))
            .map(|i| {
                let x = wx - dir * 14.0 * i as f32;
                (Vec2::new(x, feet(x)), Brain::Tick(Tick::new(i, t)))
            })
            .collect(),
        Kind::Kite => (0..size(t.kite.formation_size))
            .map(|i| {
                // Each enters `spacing_seconds` after the last, from the top.
                let up = t.kite.speed * t.kite.spacing_seconds * i as f32;
                (lead + Vec2::new(0.0, up), Brain::Kite(Kite::new(dir)))
            })
            .collect(),
        Kind::Sentry => vec![(Vec2::new(wx, feet(wx)), Brain::Sentry(Sentry::default()))],
        Kind::Burrower => vec![(
            Vec2::new(wx, feet(wx)),
            Brain::Burrower(Burrower::default()),
        )],
        Kind::Dropper => vec![(lead, Brain::Dropper(Dropper::new(t)))],
        Kind::Hauler => vec![(Vec2::new(wx, feet(wx)), Brain::Hauler(Hauler::new(dir, t)))],
        Kind::Turret => {
            let mount = match params.mount.as_deref() {
                Some("ceiling") => Mount::Ceiling,
                Some("wall") => Mount::Wall,
                _ => Mount::Floor,
            };
            vec![(Vec2::new(wx, lead.y), Brain::Turret(Turret::new(mount, t)))]
        }
        Kind::Mote => {
            let n = size(t.mote.formation_size);
            (0..n)
                .map(|i| {
                    let r = t.mote.cloud_radius * ((i as f32 + 0.5) / n as f32).sqrt();
                    let a = i as f32 * 2.399_963;
                    (lead + Vec2::from_angle(a) * r, Brain::Mote(Mote::new()))
                })
                .collect()
        }
        Kind::Strider => vec![(Vec2::new(wx, feet(wx)), Brain::Strider(Strider::default()))],
        Kind::Plough => Vec::new(),
    }
}

fn hp_and_score(kind: Kind, t: &EnemyTypes) -> (u32, u64) {
    match kind {
        Kind::Skitter => (t.skitter.hp, t.skitter.score),
        Kind::Tick => (t.tick.hp, t.tick.score),
        Kind::Kite => (t.kite.hp, t.kite.score),
        Kind::Sentry => (t.sentry.hp, t.sentry.score),
        Kind::Burrower => (t.burrower.hp, t.burrower.score),
        Kind::Dropper => (t.dropper.hp, t.dropper.score),
        Kind::Hauler => (t.hauler.hp, t.hauler.score),
        Kind::Turret => (t.turret.hp, t.turret.score),
        Kind::Mote => (t.mote.hp, t.mote.score),
        Kind::Strider => (t.strider.hp, t.strider.score),
        Kind::Plough => (1000, 0),
    }
}

/// The screen centre of an enemy at `pos` (its brain's space).
pub fn screen_centre(kind: Kind, pos: Vec2, camera: f32, t: &EnemyTypes) -> Vec2 {
    let h = look::size(kind).y;
    match kind {
        k if k.flies() => pos,
        Kind::Turret => Vec2::new(screen_x(pos.x, camera), pos.y),
        Kind::Strider => Vec2::new(screen_x(pos.x, camera), pos.y + t.strider.core_height),
        _ => Vec2::new(screen_x(pos.x, camera), pos.y + h / 2.0),
    }
}

fn spawn_enemy(
    commands: &mut Commands,
    tt: &RunAndGunTuning,
    kind: Kind,
    pos: Vec2,
    brain: Brain,
    camera: f32,
) -> Entity {
    let (hp, score) = hp_and_score(kind, &tt.enemies);
    let size = look::size(kind);
    let hitbox = (size - 2.0 * tt.enemy.hitbox_inset).max(Vec2::ONE);
    let at = screen_centre(kind, pos, camera, &tt.enemies);
    let mut e = commands.spawn((
        Enemy {
            kind,
            on_screen: 0.0,
            clock: 0.0,
        },
        EnemyBrain(brain),
        EnemyPos(pos),
        EnemyPart,
        Transform::from_translation(at.extend(4.0)),
        Hitbox::new(hitbox.x, hitbox.y),
        Layer::Enemy,
        Health(hp.max(1)),
        ScoreValue(score),
        DespawnOffscreen::new(size.max_element() / 2.0 + 24.0),
        DespawnOnExit(Screen::Playing),
    ));
    if kind == Kind::Sentry {
        e.insert(Immune);
    }
    look::body(&mut e, kind);
    e.id()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    terrain: Res<Terrain>,
    hero: Option<Single<&Transform, With<Hero>>>,
    mut requests: MessageReader<SpawnRequested>,
) {
    let tt = &tuning.run_and_gun;
    let t = &tt.enemies;
    let camera = stage.cursor.position;
    let hero = hero.map(|h| h.translation.truncate());
    for request in requests.read() {
        let params: SpawnParams = request.params().unwrap_or_default();
        if HAZARD_KINDS.contains(&request.kind.as_str()) {
            spawn_hazard(
                &mut commands,
                tt,
                &request.kind,
                request.pos,
                &params,
                camera,
                &terrain,
            );
            continue;
        }
        let Some(kind) = kind_for(&request.kind) else {
            continue;
        };
        let drop = params.drop.as_deref().and_then(|d| {
            let p = d
                .chars()
                .next()
                .filter(|_| d.len() == 1)
                .and_then(PowerUp::from_letter);
            if p.is_none() {
                warn!("unknown Power-up {d:?} for {:?}", request.kind);
            }
            p.map(PowerUp::letter)
        });
        let hero_x = hero.map_or(-120.0, |h| h.x);
        let members = layout(kind, request.pos, &params, hero_x, camera, &terrain, t);
        let behind = kind.walks() && request.pos.x < hero_x && request.pos.x < 0.0;
        let mut spawned = Vec::new();
        for (pos, brain) in members {
            let at = screen_centre(kind, pos, camera, t);
            if on_screen(at) && hero.is_some_and(|h| h.distance(at) < tt.enemy.no_spawn_radius) {
                continue;
            }
            let e = spawn_enemy(&mut commands, tt, kind, pos, brain, camera);
            if behind {
                commands
                    .entity(e)
                    .insert(Waiting(tt.enemy.behind_warning_seconds));
            }
            if params.standoff {
                commands.entity(e).insert(StandoffMember);
            }
            match kind {
                Kind::Hauler => {
                    let plough = spawn_enemy(
                        &mut commands,
                        tt,
                        Kind::Plough,
                        pos,
                        Brain::Mote(Mote::new()),
                        camera,
                    );
                    commands.entity(plough).insert((
                        Immune,
                        Follows {
                            owner: e,
                            offset: Vec2::ZERO,
                        },
                    ));
                    // The Hauler's own box is its core: the upper front.
                    let hb = look::size(Kind::Hauler) - 2.0;
                    commands.entity(e).insert(
                        Hitbox::new(hb.x, hb.y - t.hauler.plough_height)
                            .with_offset(0.0, t.hauler.plough_height / 2.0),
                    );
                }
                Kind::Strider => {
                    commands.spawn((
                        look::strider_frame(),
                        Transform::from_translation(at.extend(3.9)),
                        Hitbox::new(28.0, 44.0),
                        Layer::Hazard,
                        Follows {
                            owner: e,
                            offset: Vec2::new(0.0, 10.0),
                        },
                        EnemyPart,
                        DespawnOnExit(Screen::Playing),
                    ));
                }
                _ => {}
            }
            spawned.push(e);
        }
        if behind && !spawned.is_empty() {
            commands.spawn((
                Chevron(tt.enemy.behind_warning_seconds),
                look::chevron(),
                Transform::from_xyz(-HALF_W + 3.0, request.pos.y.max(-90.0), 30.0),
                EnemyPart,
                DespawnOnExit(Screen::Playing),
            ));
        }
        if spawned.len() > 1 || drop.is_some() {
            commands.spawn((
                Formation::new(spawned.clone(), drop),
                EnemyPart,
                DespawnOnExit(Screen::Playing),
            ));
            for member in spawned {
                commands.entity(member).insert(Member);
            }
        }
    }
}

fn on_screen(at: Vec2) -> bool {
    at.x.abs() <= HALF_W && at.y.abs() <= HALF_H
}

/// The fairness rules for one pattern. Skipped whole if any fails.
#[allow(clippy::too_many_arguments)]
pub fn may_fire(
    rules: &EnemyRules,
    shots: &[(Vec2, bool)],
    on_screen_seconds: f32,
    hero_centre: Option<Vec2>,
    hero_head: f32,
    shooter: Vec2,
    live_bullets: usize,
) -> bool {
    if on_screen_seconds < rules.fire_grace_seconds
        || live_bullets + shots.len() > rules.max_bullets
    {
        return false;
    }
    let Some(hero) = hero_centre else {
        return true;
    };
    shots.iter().all(|&(from, aimed)| {
        hero.distance(from) >= rules.min_fire_distance
            // Aimed shots only from above the Hero's head or from afar.
            && (!aimed
                || shooter.y > hero_head
                || hero.distance(shooter) > rules.aimed_min_distance)
    })
}

/// Steps every brain, places enemies on screen, and fires what the fairness
/// rules allow.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn think(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    stage: Res<StageRun>,
    terrain: Res<Terrain>,
    hero: Option<Single<(&Transform, &super::hero::Runner), Without<Enemy>>>,
    mut enemies: Query<(
        Entity,
        &mut Enemy,
        &mut EnemyBrain,
        &mut EnemyPos,
        &mut Transform,
        Option<&mut Waiting>,
        Has<Leaving>,
    )>,
    bullets: Query<(), With<EnemyBullet>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let tt = &tuning.run_and_gun;
    let t = &tt.enemies;
    let rules = &tt.enemy;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    let scale = if session.stage == 1 {
        tt.stage2_bullet_speed_scale
    } else {
        1.0
    };
    let hero_feet = hero.as_ref().map(|h| Vec2::new(h.1.body.x, h.1.body.feet));
    let hero_centre = hero.as_ref().map(|h| h.0.translation.truncate());
    let hero_head = hero_feet.map_or(f32::MAX, |f| f.y + tt.hero.body.1);
    let mut live = bullets.iter().count();
    for (entity, mut enemy, mut brain, mut pos, mut transform, waiting, leaving) in &mut enemies {
        if enemy.kind == Kind::Plough {
            continue;
        }
        if let Some(mut w) = waiting {
            w.0 -= dt;
            if w.0 <= 0.0 {
                commands.entity(entity).remove::<Waiting>();
            }
            let at = screen_centre(enemy.kind, pos.0, camera, t);
            transform.translation.x = at.x.min(-HALF_W - 12.0);
            continue;
        }
        if leaving {
            // Off the way it came: toward the nearer side edge (a Strider
            // walks off to the right).
            let at = screen_centre(enemy.kind, pos.0, camera, t);
            let side = if enemy.kind == Kind::Strider || at.x > 0.0 {
                1.0
            } else {
                -1.0
            };
            pos.0.x += side * 80.0 * dt;
            let at = screen_centre(enemy.kind, pos.0, camera, t);
            transform.translation.x = at.x.round();
            transform.translation.y = at.y.round();
            continue;
        }
        let hero_here = if enemy.kind.flies() {
            hero_feet.map(|f| Vec2::new(screen_x(f.x, camera), f.y))
        } else {
            hero_feet
        };
        let ctx = Ctx {
            dt,
            camera,
            hero: hero_here,
            terrain: &terrain,
        };
        let warned = brain.0.telegraph(t);
        let out: Out = brain.0.step(&mut pos.0, &ctx, t, rules);
        // Each warning's sound, as the warning begins.
        if !warned && brain.0.telegraph(t) {
            let warning = match &brain.0 {
                Brain::Sentry(_) => Some(Sfx::SentryRise),
                Brain::Burrower(_) => Some(Sfx::BurrowerDust),
                Brain::Dropper(_) => Some(Sfx::DropperHatch),
                Brain::Strider(s) if s.cycle < t.strider.stomp_telegraph_seconds => {
                    Some(Sfx::StriderLegLift)
                }
                _ => None,
            };
            if let Some(w) = warning {
                sfx.write(PlaySfx(w));
            }
        }
        let at = screen_centre(enemy.kind, pos.0, camera, t);
        transform.translation.x = at.x.round();
        transform.translation.y = at.y.round();
        enemy.clock += dt;
        if on_screen(at) {
            enemy.on_screen += dt;
        }
        // Where its shots leave from, on screen.
        let base = if enemy.kind.walks() {
            Vec2::new(screen_x(pos.0.x, camera), pos.0.y)
        } else {
            at
        };
        if !out.shots.is_empty() {
            let pattern: Vec<_> = out
                .shots
                .iter()
                .map(|s| (base + s.offset, s.aimed))
                .collect();
            if may_fire(
                rules,
                &pattern,
                enemy.on_screen,
                hero_centre,
                hero_head,
                at,
                live,
            ) {
                live += out.shots.len();
                for shot in &out.shots {
                    fire(&mut commands, rules, t, base + shot.offset, shot, scale);
                }
            }
        }
        if out.stomp {
            sfx.write(PlaySfx(Sfx::StriderStomp));
            sfx.write(PlaySfx(Sfx::StriderShockwave));
            for dir in [-1.0, 1.0] {
                commands.spawn((
                    look::hazard(Vec2::new(6.0, t.strider.wave_height)),
                    Transform::from_xyz(at.x, pos.0.y + t.strider.wave_height / 2.0, 4.5),
                    WorldX(pos.0.x),
                    Hitbox::new(6.0, t.strider.wave_height),
                    Layer::Hazard,
                    Shockwave {
                        dir,
                        travelled: 0.0,
                        ground: pos.0.y,
                    },
                    EnemyPart,
                    DespawnOnExit(Screen::Playing),
                ));
            }
        }
        if out.despawn {
            commands.entity(entity).try_despawn();
        }
    }
}

fn fire(
    commands: &mut Commands,
    rules: &EnemyRules,
    t: &EnemyTypes,
    at: Vec2,
    shot: &Shot,
    scale: f32,
) {
    let size = if shot.burst {
        bevy::math::Vec2::new(t.dropper.canister_size.0, t.dropper.canister_size.1)
    } else {
        Vec2::new(rules.bullet_size.0, rules.bullet_size.1)
    };
    let hit = if shot.burst {
        size - 2.0
    } else {
        Vec2::new(rules.bullet_hitbox.0, rules.bullet_hitbox.1)
    };
    let mut e = commands.spawn((
        projectile(
            Layer::EnemyShot,
            at,
            shot.velocity * scale,
            size,
            palette::ICE,
            rules.bullet_lifetime,
        ),
        EnemyBullet,
    ));
    e.insert(Hitbox::new(hit.x, hit.y))
        .insert(look::bullet(size));
    if shot.gravity > 0.0 {
        e.insert(Falling {
            gravity: shot.gravity,
            burst: shot.burst,
        });
    }
}

/// Shots under gravity speed up as they fall; the ground stops every shot,
/// and a canister that lands bursts into two ankle shots along it.
#[allow(clippy::type_complexity)]
fn shots_and_ground(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    stage: Res<StageRun>,
    terrain: Res<Terrain>,
    mut shots: Query<(
        Entity,
        &Layer,
        &Transform,
        Option<&Falling>,
        Option<&mut Velocity>,
    )>,
) {
    let tt = &tuning.run_and_gun;
    let camera = stage.cursor.position;
    let dt = time.delta_secs();
    let scale = if session.stage == 1 {
        tt.stage2_bullet_speed_scale
    } else {
        1.0
    };
    for (entity, layer, transform, falling, velocity) in &mut shots {
        if !matches!(layer, Layer::EnemyShot | Layer::HeroShot) {
            continue;
        }
        if let (Some(f), Some(mut v)) = (falling, velocity) {
            v.0.y -= f.gravity * dt;
        }
        let at = transform.translation.truncate();
        let Some(top) = terrain.ground_at(world_x(at.x, camera)) else {
            continue;
        };
        if at.y > top {
            continue;
        }
        commands.entity(entity).try_despawn();
        if falling.is_some_and(|f| f.burst) {
            let shot = |dir: f32| Shot {
                offset: Vec2::ZERO,
                velocity: Vec2::new(dir * tt.enemies.dropper.burst_speed, 0.0),
                gravity: 0.0,
                burst: false,
                aimed: false,
            };
            let from = Vec2::new(at.x, top + tt.enemy.ankle_height);
            for dir in [-1.0, 1.0] {
                fire(
                    &mut commands,
                    &tt.enemy,
                    &tt.enemies,
                    from,
                    &shot(dir),
                    scale,
                );
            }
        }
    }
}

/// Parts follow their owner (a plough, a Strider's frame); shockwaves run
/// along the ground until they fade or the ground changes height.
#[allow(clippy::type_complexity)]
fn parts_and_waves(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    terrain: Res<Terrain>,
    owners: Query<(&Transform, &EnemyBrain), (With<Enemy>, Without<Follows>)>,
    mut parts: Query<(Entity, &Follows, &mut Transform), Without<Shockwave>>,
    mut waves: Query<
        (Entity, &mut Shockwave, &mut WorldX, &mut Transform),
        (Without<Follows>, Without<Enemy>),
    >,
) {
    let t = &tuning.run_and_gun.enemies;
    for (entity, follows, mut transform) in &mut parts {
        let Ok((owner, brain)) = owners.get(follows.owner) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let offset = match &brain.0 {
            // The plough covers the lower front.
            Brain::Hauler(h) => Vec2::new(h.dir_x() * 9.0, -7.0),
            _ => follows.offset,
        };
        transform.translation.x = owner.translation.x + offset.x;
        transform.translation.y = owner.translation.y + offset.y;
    }
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    for (entity, mut wave, mut wx, mut transform) in &mut waves {
        let step = t.strider.wave_speed * dt;
        wave.travelled += step;
        wx.0 += wave.dir * step;
        let ground_same = terrain
            .ground_at(wx.0)
            .is_some_and(|g| (g - wave.ground).abs() < 0.5);
        if wave.travelled >= t.strider.wave_range || !ground_same {
            commands.entity(entity).try_despawn();
            continue;
        }
        transform.translation.x = screen_x(wx.0, camera).round();
    }
}

/// Eyes flash before an attack; a Sentry is short and immune while down; a
/// buried Burrower's box sits under the ground.
#[allow(clippy::type_complexity)]
fn show_parts(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut enemies: Query<(Entity, &Enemy, &EnemyBrain, &mut Hitbox, Has<Immune>)>,
    mut eyes: Query<(&ChildOf, &mut Sprite), With<look::Eye>>,
) {
    let tt = &tuning.run_and_gun;
    let t = &tt.enemies;
    for (parent, mut sprite) in &mut eyes {
        if let Ok((_, enemy, brain, ..)) = enemies.get(parent.parent()) {
            sprite.color = look::eye_color(brain.0.telegraph(t), enemy.clock);
        }
    }
    let inset = 2.0 * tt.enemy.hitbox_inset;
    for (entity, enemy, brain, mut hitbox, immune) in &mut enemies {
        match &brain.0 {
            Brain::Sentry(s) => {
                let up = s.phase(t) == SentryPhase::Up;
                let h = if up { 18.0 } else { 8.0 };
                hitbox.size.y = h - inset;
                hitbox.offset.y = (h - 18.0) / 2.0;
                if up && immune {
                    commands.entity(entity).try_remove::<Immune>();
                } else if !up && !immune {
                    commands.entity(entity).try_insert(Immune);
                }
            }
            Brain::Burrower(b) => {
                // Buried: under the surface, out of a shot's reach.
                let risen = b.risen() || matches!(b.phase, BurrowPhase::Down(_));
                hitbox.offset.y = if risen { 0.0 } else { -14.0 };
            }
            _ => {}
        }
        let _ = enemy;
    }
}

fn run_chevrons(
    mut commands: Commands,
    time: Res<Time>,
    mut chevrons: Query<(Entity, &mut Chevron, &mut Visibility)>,
) {
    for (entity, mut chevron, mut visibility) in &mut chevrons {
        chevron.0 -= time.delta_secs();
        if chevron.0 <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let on = ((chevron.0 * 8.0) as u32).is_multiple_of(2);
            visibility.set_if_neq(if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

fn spawn_hazard(
    commands: &mut Commands,
    tt: &RunAndGunTuning,
    kind: &str,
    pos: Vec2,
    params: &SpawnParams,
    camera: f32,
    terrain: &Terrain,
) {
    let wx = world_x(pos.x, camera);
    let ground = terrain.ground_at(wx).unwrap_or(-96.0);
    let (hazard, size) = match kind {
        "crusher" => (
            Hazard::Crusher {
                offset: params.offset,
                age: 0.0,
                ground,
            },
            Vec2::new(24.0, 32.0),
        ),
        "arc_floor" => {
            let length = params.length.unwrap_or(48.0);
            (
                Hazard::ArcFloor {
                    offset: params.offset,
                    age: 0.0,
                    length,
                },
                Vec2::new(length, 3.0),
            )
        }
        _ => {
            let length = params.length.unwrap_or(64.0);
            let dir = match params.dir.as_deref() {
                Some("right") => 1.0,
                _ => -1.0,
            };
            (
                Hazard::Conveyor {
                    length,
                    speed: dir * tt.hazards.conveyor.speed,
                },
                Vec2::new(length, 3.0),
            )
        }
    };
    commands.spawn((
        look::hazard(size),
        Transform::from_xyz(screen_x(wx, camera), ground + size.y / 2.0, 3.0),
        WorldX(wx),
        hazard,
        EnemyPart,
        DespawnOnExit(Screen::Playing),
    ));
}

/// Crushers cycle and slam (killing enemies under them, for no points); arc
/// floors switch on and off; conveyors carry the Hero.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn run_hazards(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut hazards: Query<(
        Entity,
        &mut Hazard,
        &WorldX,
        &mut Transform,
        &mut Sprite,
        Has<Layer>,
    )>,
    mut hero: Option<Single<&mut super::hero::Runner>>,
    victims: Query<(Entity, &Transform, &Hitbox), (With<Enemy>, Without<Hazard>)>,
    mut sfx: MessageWriter<PlaySfx>,
    mut hum: Local<f32>,
) {
    let h = &tuning.run_and_gun.hazards;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    // A conveyor on screen hums about once a second.
    *hum -= dt;
    for (entity, mut hazard, wx, mut transform, mut sprite, has_layer) in &mut hazards {
        transform.translation.x = screen_x(wx.0, camera).round();
        let mut live = false;
        match &mut *hazard {
            Hazard::Crusher {
                offset,
                age,
                ground,
            } => {
                *age += dt;
                let c = &h.crusher;
                let cycle = (c.up_seconds + c.telegraph_seconds + c.slam_seconds + c.down_seconds)
                    .max(0.01);
                let p = (*age + *offset).rem_euclid(cycle);
                let raise = 40.0;
                let up_until = c.up_seconds + c.telegraph_seconds;
                let lift = if p < up_until {
                    raise
                } else if p < up_until + c.slam_seconds {
                    raise * (1.0 - (p - up_until) / c.slam_seconds.max(0.01))
                } else {
                    0.0
                };
                live = p >= up_until;
                if live && !has_layer {
                    sfx.write(PlaySfx(Sfx::CrusherSlam));
                }
                transform.translation.y = *ground + 16.0 + lift;
                let warn = p >= c.up_seconds && p < up_until;
                sprite.color = if warn && ((*age * 16.0) as u32).is_multiple_of(2) {
                    palette::ICE
                } else {
                    palette::IRON
                };
                if live {
                    let box_ = Hitbox::new(22.0, 30.0).aabb(transform.translation.truncate());
                    for (victim, vt, vh) in &victims {
                        if vh.aabb(vt.translation.truncate()).overlaps(&box_) {
                            commands.entity(victim).try_despawn();
                        }
                    }
                }
            }
            Hazard::ArcFloor {
                offset,
                age,
                length,
            } => {
                *age += dt;
                let a = &h.arc_floor;
                let pulse = Pulse {
                    on_seconds: a.on_seconds,
                    off_seconds: a.off_seconds,
                    telegraph_seconds: a.telegraph_seconds,
                };
                let at = *age + *offset;
                if pulse.telegraph(at) && !pulse.telegraph(at - dt) {
                    sfx.write(PlaySfx(Sfx::ArcFloorCrackle));
                }
                live = pulse.live(at);
                sprite.color = if live {
                    palette::ICE
                } else if pulse.telegraph(at) && ((at * 20.0) as u32).is_multiple_of(2) {
                    palette::ICE.with_alpha(0.5)
                } else {
                    palette::IRON
                };
                let _ = length;
            }
            Hazard::Conveyor { length, speed } => {
                if *hum <= 0.0 && transform.translation.x.abs() <= HALF_W {
                    *hum = 1.0;
                    sfx.write(PlaySfx(Sfx::ConveyorHum));
                }
                if let Some(runner) = hero.as_mut() {
                    let b = &mut runner.body;
                    let top = transform.translation.y + 1.5;
                    if b.grounded
                        && (b.feet - top).abs() <= 2.0
                        && (b.x - wx.0).abs() <= *length / 2.0
                    {
                        b.x += *speed * dt;
                    }
                }
            }
        }
        let size = match &*hazard {
            Hazard::Crusher { .. } => Vec2::new(22.0, 30.0),
            Hazard::ArcFloor { length, .. } => Vec2::new(*length, 4.0),
            Hazard::Conveyor { .. } => Vec2::ZERO,
        };
        if live && !has_layer {
            commands
                .entity(entity)
                .try_insert((Layer::Hazard, Hitbox::new(size.x, size.y)));
        } else if !live && has_layer {
            commands.entity(entity).try_remove::<Layer>();
        }
    }
}

/// A rewind: everything of ours goes, Formations and hazards too (the
/// timeline spawns them again).
fn clear_on_restart(
    mut commands: Commands,
    mut restarted: MessageReader<StageRestarted>,
    parts: Query<Entity, Or<(With<EnemyPart>, With<EnemyBullet>)>>,
) {
    if restarted.read().count() == 0 {
        return;
    }
    for entity in &parts {
        commands.entity(entity).try_despawn();
    }
}

/// Counts kills and escapes for each Formation and pays out when one is done.
fn track_formations(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    mut formations: Query<(Entity, &mut Formation)>,
    alive: Query<(), With<Enemy>>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut score: MessageWriter<ScoreGained>,
) {
    for d in destroyed.read() {
        for (_, mut f) in &mut formations {
            if f.has(d.entity) {
                f.killed(d.entity, d.at);
            }
        }
    }
    for (entity, mut f) in &mut formations {
        let escaped: Vec<_> = f
            .pending()
            .iter()
            .copied()
            .filter(|m| !alive.contains(*m))
            .collect();
        for m in escaped {
            f.lost(m);
        }
        if !f.is_done() {
            continue;
        }
        match f.reward() {
            Some(Reward::PowerUp(letter, at)) => {
                if let Some(p) = PowerUp::from_letter(letter) {
                    spawns.write(SpawnRequested::new(p.kind(), at));
                }
            }
            Some(Reward::Bonus { members }) => {
                let per = tuning.run_and_gun.score.formation_bonus_per_enemy;
                score.write(ScoreGained(per * members as u64));
            }
            None => {}
        }
        commands.entity(entity).despawn();
    }
}

/// Hits, explosions and shots sparking off armour.
fn enemy_sounds(
    mut hurts: MessageReader<Hurt>,
    mut flashes: MessageReader<Flash>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    armour: Query<(), (With<Enemy>, With<Immune>)>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    if hurts.read().any(|h| armour.contains(h.target)) {
        sfx.write(PlaySfx(Sfx::ShellPing));
    }
    if flashes.read().count() > 0 {
        sfx.write(PlaySfx(Sfx::EnemyHit));
    }
    for d in destroyed.read() {
        sfx.write(PlaySfx(if d.score >= 800 {
            Sfx::ExplosionLarge
        } else {
            Sfx::ExplosionSmall
        }));
    }
}

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<PlaySfx>().add_systems(
            Update,
            (
                (
                    spawn_requested,
                    think,
                    shots_and_ground,
                    parts_and_waves,
                    show_parts,
                    run_hazards,
                    run_chevrons,
                )
                    .chain()
                    .after(KitSet::Timeline)
                    .before(KitSet::Move),
                (clear_on_restart, track_formations, enemy_sounds)
                    .chain()
                    .after(KitSet::Resolve)
                    .before(KitSet::Effects),
            )
                .run_if(crate::cabinets::cabinet_is(super::CABINET))
                .run_if(resource_exists::<PowerUps>)
                .run_if(resource_exists::<Terrain>)
                .run_if(stage_live),
        );
    }
}

#[cfg(test)]
mod tests;
