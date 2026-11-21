//! Cabinet 2's enemies (docs/cabinets/2-vertical-shooter.md): spawning them
//! from the Stage timeline, running their [`Brain`]s, the fairness rules on
//! enemy fire, the air and ground layers, Formations and the Power-up drop
//! rule, and the debug enemy gallery.
//!
//! Stage files name an enemy by kind and may add params:
//!
//! ```ron
//! (at: 120.0, kind: "wisp", pos: (-40.0, 124.0), params: {"size": 6, "drop": "G"}),
//! (at: 300.0, kind: "bellows", pos: (0.0, 60.0), params: {"side": "left"}),
//! (at: 400.0, kind: "crawler", pos: (-120.0, 128.0), params: {"heading": 0.0}),
//! (at: 500.0, kind: "tether", pos: (0.0, 128.0), params: {"left_x": -148.0, "right_x": 148.0}),
//! ```
//!
//! - `size`: Formation size (default: the kind's `formation_size` in tuning).
//! - `drop`: the Power-up letter the Formation drops when cleared. Without
//!   it, a Formation of more than one pays the Formation bonus instead.
//! - `side`: Bellows only, the edge it enters from (default: the side of
//!   the screen `pos.x` is on). `pos.y` is its row.
//! - `heading`: Crawlers only, the road direction in degrees (0 = right).
//! - `left_x` / `right_x`: Tethers only, the balloons' x. Both are required:
//!   a Tether without them is skipped with a warning.
//!
//! `pos` is the Formation's lead: the head of a Wisp column, the front of a
//! Crawler convoy, the centre of a Mote cloud or of a Loom. Swifts ignore it
//! and come from below, either side of the Hero. Ground targets are placed
//! where they first appear and then scroll down with the ground.

use bevy::prelude::*;
use serde::Deserialize;

use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::cabinets::shooter::formation::{Formation, Member, Reward};
use crate::display::HALF_H;
use crate::flow::{Screen, gameplay_running};
use crate::kit::{
    DespawnOffscreen, EnemyDestroyed, Health, Hero, Hitbox, Immune, KitSet, Layer, Lifetime,
    ScoreValue, SpawnRequested, StageRestarted, StageRun, StageTimeline, projectile,
};
use crate::run::Session;
use crate::sprites::Sprites;
use crate::tuning::Tuning;

use super::power_ups::PowerUp;
use super::tuning::{EnemyTypes, VerticalShooterTuning};

pub mod brain;
pub mod look;

use brain::{
    Bellows, Brain, Crawler, Ctx, Flak, LoomCore, LoomShuttle, Mote, Out, RiserLauncher, RiserPod,
    Silo, Swift, Wisp, on_screen, tether_pulse,
};

const GALLERY_STAGE: &str = include_str!("../../../../assets/stages/2-gallery.ron");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Wisp,
    Swift,
    Bellows,
    LoomCore,
    LoomShuttle,
    Crawler,
    Silo,
    RiserLauncher,
    RiserPod,
    Flak,
    Mote,
    TetherBalloon,
}

impl Kind {
    /// Ground targets never touch the Wren and cast no shadow. A Riser pod
    /// starts on the ground and takes off later.
    pub fn on_ground(self) -> bool {
        matches!(
            self,
            Kind::Crawler | Kind::Silo | Kind::RiserLauncher | Kind::RiserPod | Kind::Flak
        )
    }

    /// Draw depth: ground under air, the Loom's core under its shuttles.
    pub fn z(self) -> f32 {
        match self {
            k if k.on_ground() => 2.5,
            Kind::LoomCore => 3.8,
            _ => 4.0,
        }
    }
}

/// An enemy of Cabinet 2.
#[derive(Component, Debug)]
pub struct Enemy {
    pub kind: Kind,
    /// Seconds spent on screen, for the fire grace period.
    on_screen: f32,
    /// Seconds alive, for blinking.
    clock: f32,
}

#[derive(Component, Debug)]
pub struct EnemyBrain(pub Brain);

/// A bullet fired by an enemy (counted for the bullet cap).
#[derive(Component)]
pub struct EnemyBullet;

/// Anything this module spawned that a rewind to a Restart Point removes.
#[derive(Component)]
struct EnemyPart;

/// The warning chevron for a Swift; seconds left.
#[derive(Component)]
struct Chevron(f32);

/// A marked flak point: the burst comes when `left` runs out.
#[derive(Component)]
pub struct Crosshair {
    pub left: f32,
}

/// A flak burst's short-lived hit zone (it hits the Hero only).
#[derive(Component)]
pub struct FlakBurst;

/// The line between two Tether balloons.
#[derive(Component)]
struct TetherLine {
    age: f32,
    balloons: [Entity; 2],
}

/// A Loom core's two shuttles; its shield drops when both are gone.
#[derive(Component)]
pub struct LoomLink {
    pub shuttles: [Entity; 2],
}

/// A shuttle docked beside its Loom core.
#[derive(Component)]
pub struct Docked {
    pub core: Entity,
    pub side: f32,
}

/// Settings a Stage file can give a spawn (see the module docs).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SpawnParams {
    #[serde(deserialize_with = "present")]
    pub size: Option<u32>,
    #[serde(deserialize_with = "present")]
    pub drop: Option<String>,
    #[serde(deserialize_with = "present")]
    pub side: Option<String>,
    pub heading: f32,
    #[serde(deserialize_with = "present")]
    pub left_x: Option<f32>,
    #[serde(deserialize_with = "present")]
    pub right_x: Option<f32>,
}

/// Reads an optional param written plainly (`"size": 5`, not `Some(5)`).
fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}

/// The debug enemy gallery, when it is switched on: it replaces the Stage.
pub fn gallery_timeline(tuning: &Tuning) -> Option<StageTimeline> {
    (tuning.debug_keys && tuning.vertical_shooter.enemy_gallery).then(|| {
        StageTimeline::parse(GALLERY_STAGE).expect("Cabinet 2 enemy gallery must be valid")
    })
}

fn cabinet_active(session: Option<Res<Session>>) -> bool {
    session.is_some_and(|s| s.cabinet == super::CABINET)
}

/// The kind a Stage file's spawn names, if it is one of ours. A Loom names
/// its core; the shuttles come with it.
pub fn kind_for(name: &str) -> Option<Kind> {
    Some(match name {
        "wisp" => Kind::Wisp,
        "swift" => Kind::Swift,
        "bellows" => Kind::Bellows,
        "loom" => Kind::LoomCore,
        "crawler" => Kind::Crawler,
        "silo" => Kind::Silo,
        "riser" => Kind::RiserLauncher,
        "flak" => Kind::Flak,
        "mote" => Kind::Mote,
        "tether" => Kind::TetherBalloon,
        _ => return None,
    })
}

/// Where each member of a spawn starts and how it behaves. Empty when the
/// spawn can't be placed (a Tether without both balloons' x).
pub fn layout(
    kind: Kind,
    lead: Vec2,
    params: &SpawnParams,
    hero: Option<Vec2>,
    t: &EnemyTypes,
) -> Vec<(Vec2, Brain)> {
    let size = |default: u32| params.size.unwrap_or(default).max(1);
    match kind {
        Kind::Wisp => {
            // A column: each member one spacing further up, so they arrive
            // `spacing_seconds` apart.
            let gap = t.wisp.speed * t.wisp.spacing_seconds;
            (0..size(t.wisp.formation_size))
                .map(|i| {
                    (
                        lead + Vec2::new(0.0, gap * i as f32),
                        Brain::Wisp(Wisp::new()),
                    )
                })
                .collect()
        }
        Kind::Swift => {
            let hero_x = hero.map_or(0.0, |h| h.x);
            (0..size(t.swift.formation_size))
                .map(|i| {
                    let side = if i % 2 == 0 { -1.0 } else { 1.0 };
                    let pair = (i / 2) as f32;
                    let offset = side * t.swift.pass_offset * (1.0 + pair * t.swift.pair_spread);
                    let brain = Swift::new(hero_x, offset, t);
                    (
                        Vec2::new(brain.column, brain::SWIFT_WAIT_Y),
                        Brain::Swift(brain),
                    )
                })
                .collect()
        }
        Kind::Bellows => {
            let side = match params.side.as_deref() {
                Some("left") => -1.0,
                Some("right") => 1.0,
                _ if lead.x < 0.0 => -1.0,
                _ => 1.0,
            };
            vec![(
                Vec2::new(Bellows::entry_x(side), lead.y),
                Brain::Bellows(Bellows::new(side)),
            )]
        }
        Kind::LoomCore => vec![(lead, Brain::LoomCore(LoomCore::new()))],
        Kind::Crawler => (0..size(t.crawler.formation_size))
            .map(|i| {
                // Followers trail behind the lead along the road.
                let back = brain_dir(params.heading) * -t.crawler.spacing * i as f32;
                (
                    lead + back,
                    Brain::Crawler(Crawler::new(params.heading, i, t)),
                )
            })
            .collect(),
        Kind::Silo => vec![(lead, Brain::Silo(Silo::default()))],
        Kind::RiserLauncher => vec![(lead, Brain::RiserLauncher(RiserLauncher::default()))],
        Kind::Flak => vec![(lead, Brain::Flak(Flak::new(t)))],
        Kind::Mote => {
            let n = size(t.mote.formation_size);
            (0..n)
                .map(|i| {
                    // An even spread over the cloud (a golden-angle spiral).
                    let r = t.mote.cloud_radius * ((i as f32 + 0.5) / n as f32).sqrt();
                    let a = i as f32 * 2.399_963;
                    (lead + Vec2::from_angle(a) * r, Brain::Mote(Mote::new()))
                })
                .collect()
        }
        Kind::TetherBalloon => match (params.left_x, params.right_x) {
            (Some(left), Some(right)) => vec![
                (Vec2::new(left.min(right), lead.y), Brain::Drift),
                (Vec2::new(left.max(right), lead.y), Brain::Drift),
            ],
            _ => Vec::new(),
        },
        Kind::LoomShuttle | Kind::RiserPod => Vec::new(),
    }
}

fn brain_dir(degrees: f32) -> Vec2 {
    Vec2::from_angle(degrees.to_radians())
}

/// HP and score for a kind (the Loom and Riser pod read their own keys).
fn hp_and_score(kind: Kind, stage2: bool, t: &EnemyTypes) -> (u32, u64) {
    match kind {
        Kind::Wisp => (t.wisp.hp, t.wisp.score),
        Kind::Swift => (t.swift.hp, t.swift.score),
        Kind::Bellows => (t.bellows.hp, t.bellows.score),
        Kind::LoomCore if stage2 => (t.loom.core_hp_stage2, t.loom.core_score),
        Kind::LoomCore => (t.loom.core_hp, t.loom.core_score),
        Kind::LoomShuttle if stage2 => (t.loom.shuttle_hp_stage2, t.loom.shuttle_score),
        Kind::LoomShuttle => (t.loom.shuttle_hp, t.loom.shuttle_score),
        Kind::Crawler => (t.crawler.hp, t.crawler.score),
        Kind::Silo => (t.silo.hp, t.silo.score),
        Kind::RiserLauncher => (t.riser.hp, t.riser.score),
        Kind::RiserPod => (1, t.riser.pod_score_climbing),
        Kind::Flak => (t.flak.hp, t.flak.score),
        Kind::Mote => (t.mote.hp, t.mote.score),
        Kind::TetherBalloon => (t.tether.hp, t.tether.score),
    }
}

/// Spawns one enemy, unless it would appear on screen too close to the Hero.
#[allow(clippy::too_many_arguments)]
fn spawn_enemy(
    commands: &mut Commands,
    sprites: &Sprites,
    tuning: &VerticalShooterTuning,
    stage2: bool,
    hero: Option<Vec2>,
    kind: Kind,
    at: Vec2,
    brain: Brain,
) -> Option<Entity> {
    let rules = &tuning.enemy;
    if on_screen(at) && hero.is_some_and(|h| h.distance(at) < rules.no_spawn_radius) {
        return None;
    }
    let (hp, score) = hp_and_score(kind, stage2, &tuning.enemies);
    let size = look::size(kind);
    let hitbox = (size - 2.0 * rules.hitbox_inset).max(Vec2::ONE);
    let mut entity = commands.spawn((
        Enemy {
            kind,
            on_screen: 0.0,
            clock: 0.0,
        },
        EnemyBrain(brain),
        EnemyPart,
        Transform::from_translation(at.extend(kind.z())),
        Hitbox::new(hitbox.x, hitbox.y),
        if kind.on_ground() {
            Layer::Ground
        } else {
            Layer::Enemy
        },
        Health(hp.max(1)),
        ScoreValue(score),
        // Swifts wait below the screen and Bellows beside it; Looms and
        // ground targets arrive from above. All of them are safe until they
        // first appear.
        DespawnOffscreen::new(size.max_element() / 2.0 + 16.0),
        DespawnOnExit(Screen::Playing),
    ));
    if matches!(kind, Kind::Silo | Kind::LoomCore) {
        entity.insert(Immune);
    }
    look::body(&mut entity, kind, sprites, tuning);
    Some(entity.id())
}

fn read_drop(params: &SpawnParams, kind: &str) -> Option<char> {
    let d = params.drop.as_deref()?;
    let letter = d
        .chars()
        .next()
        .filter(|_| d.len() == 1)
        .and_then(PowerUp::from_letter)
        .map(PowerUp::letter);
    if letter.is_none() {
        warn!("unknown Power-up {d:?} for {kind:?}");
    }
    letter
}

fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    sprites: Res<Sprites>,
    hero: Option<Single<&Transform, With<Hero>>>,
    mut requests: MessageReader<SpawnRequested>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let hero = hero.map(|h| h.translation.truncate());
    let stage2 = session.stage == 1;
    let tt = &tuning.vertical_shooter;
    let t = &tt.enemies;
    for request in requests.read() {
        // Kinds this module doesn't know (Power-ups, scenery) belong to
        // other parts of the Cabinet.
        let Some(kind) = kind_for(&request.kind) else {
            continue;
        };
        let params: SpawnParams = request.params().unwrap_or_else(|e| {
            warn!("bad params for {:?}: {e}", request.kind);
            SpawnParams::default()
        });
        let drop = read_drop(&params, &request.kind);
        let members = layout(kind, request.pos, &params, hero, t);
        if kind == Kind::TetherBalloon && members.is_empty() {
            warn!(
                "tether at y = {} needs both left_x and right_x; skipped",
                request.pos.y
            );
            continue;
        }
        let mut spawned = Vec::new();
        for (at, brain) in members {
            if let Some(e) = spawn_enemy(&mut commands, &sprites, tt, stage2, hero, kind, at, brain)
            {
                spawned.push(e);
                if kind == Kind::Swift {
                    sfx.write(PlaySfx(Sfx::SwiftWarning));
                    commands.spawn((
                        Chevron(t.swift.warning_seconds),
                        look::chevron(&sprites),
                        Transform::from_xyz(at.x, -HALF_H + 3.0, 30.0),
                        EnemyPart,
                        DespawnOnExit(Screen::Playing),
                    ));
                }
            }
        }
        match (kind, &spawned[..]) {
            (Kind::LoomCore, &[core]) => {
                spawn_shuttles(&mut commands, &sprites, tt, stage2, core, request.pos)
            }
            (Kind::TetherBalloon, &[left, right]) => {
                spawn_line(&mut commands, tt, request.pos.y, &params, [left, right]);
                // Tethers are not Formations unless the Stage asks for a drop.
                if drop.is_none() {
                    continue;
                }
            }
            _ => {}
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

fn spawn_shuttles(
    commands: &mut Commands,
    sprites: &Sprites,
    tuning: &VerticalShooterTuning,
    stage2: bool,
    core: Entity,
    at: Vec2,
) {
    let t = &tuning.enemies;
    let mut shuttles = [Entity::PLACEHOLDER; 2];
    for (i, side) in [-1.0, 1.0].into_iter().enumerate() {
        let pos = at + Vec2::new(side * t.loom.shuttle_offset, 0.0);
        let brain = Brain::LoomShuttle(LoomShuttle::new(side, t));
        // The core was allowed to spawn, so its shuttles are too (no Hero
        // is ever near the top edge where a Loom appears).
        let e = spawn_enemy(
            commands,
            sprites,
            tuning,
            stage2,
            None,
            Kind::LoomShuttle,
            pos,
            brain,
        )
        .expect("shuttles always spawn");
        commands.entity(e).insert(Docked { core, side });
        shuttles[i] = e;
    }
    commands.entity(core).insert(LoomLink { shuttles });
}

fn spawn_line(
    commands: &mut Commands,
    tuning: &VerticalShooterTuning,
    y: f32,
    params: &SpawnParams,
    balloons: [Entity; 2],
) {
    let (Some(a), Some(b)) = (params.left_x, params.right_x) else {
        return;
    };
    let (left, right) = (a.min(b), a.max(b));
    let half_balloon = look::size(Kind::TetherBalloon).x / 2.0;
    let gap = (right - left - 2.0 * half_balloon).max(1.0);
    let width = tuning.enemies.tether.line_width;
    commands.spawn((
        TetherLine { age: 0.0, balloons },
        Sprite::from_color(Color::WHITE, Vec2::new(gap, width)),
        Visibility::Hidden,
        Transform::from_xyz((left + right) / 2.0, y, 3.9),
        Hitbox::new(gap, width),
        EnemyPart,
        DespawnOffscreen::new(8.0),
        DespawnOnExit(Screen::Playing),
    ));
}

/// The fairness rules, for one enemy's pattern this frame. A pattern that
/// breaks any of them is skipped whole, never trimmed.
pub fn may_fire(
    rules: &super::tuning::EnemyRules,
    from: Vec2,
    on_screen_seconds: f32,
    hero: Option<Vec2>,
    pattern: usize,
    live_bullets: usize,
) -> bool {
    on_screen_seconds >= rules.fire_grace_seconds
        // Never once the enemy is below the Hero.
        && hero.is_none_or(|h| from.y >= h.y)
        // Measured from where the bullets appear.
        && hero.is_none_or(|h| h.distance(from) >= rules.min_fire_distance)
        && live_bullets + pattern <= rules.max_bullets
}

/// One enemy bullet: the shared sprite, with the small bullet hitbox.
fn fire_bullet(
    commands: &mut Commands,
    sprites: &Sprites,
    rules: &super::tuning::EnemyRules,
    at: Vec2,
    velocity: Vec2,
) {
    let (w, h) = rules.bullet_size;
    let (hw, hh) = rules.bullet_hitbox;
    commands
        .spawn((
            projectile(
                Layer::EnemyShot,
                at,
                velocity,
                Vec2::new(w, h),
                Color::WHITE,
                rules.bullet_lifetime,
            ),
            EnemyBullet,
        ))
        .insert((look::bullet(sprites), Hitbox::new(hw, hh)));
}

/// Steps every brain and fires what the fairness rules allow.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn think(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    stage: Res<StageRun>,
    sprites: Res<Sprites>,
    hero: Option<Single<&Transform, (With<Hero>, Without<Enemy>)>>,
    mut enemies: Query<(
        Entity,
        &mut Enemy,
        &mut EnemyBrain,
        &mut Transform,
        &mut Layer,
        &mut ScoreValue,
        Has<Immune>,
    )>,
    bullets: Query<(), With<EnemyBullet>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let tt = &tuning.vertical_shooter;
    let t = &tt.enemies;
    let rules = &tt.enemy;
    let hero = hero.map(|h| h.translation.truncate());
    let ctx = Ctx {
        dt: time.delta_secs(),
        hero,
        scroll: stage.speed(),
    };
    let stage2 = session.stage == 1;
    let speed_scale = if stage2 {
        tt.stage2_bullet_speed_scale
    } else {
        1.0
    };
    let mut live_bullets = bullets.iter().count();
    for (entity, mut enemy, mut brain, mut transform, mut layer, mut value, immune) in &mut enemies
    {
        let mut at = transform.translation.truncate();
        let out: Out = brain.0.step(&mut at, &ctx, t);
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        enemy.clock += ctx.dt;
        if on_screen(at) {
            enemy.on_screen += ctx.dt;
        }
        match &brain.0 {
            // A Silo is immune while shut; change it only as the hatch moves.
            Brain::Silo(silo) => {
                let open = silo.open(t);
                if open && immune {
                    commands.entity(entity).try_remove::<Immune>();
                    sfx.write(PlaySfx(Sfx::SiloClank));
                } else if !open && !immune {
                    commands.entity(entity).try_insert(Immune);
                }
            }
            // A pod that has taken off is an air enemy from now on.
            Brain::RiserPod(pod) if pod.airborne(t) && *layer == Layer::Ground => {
                *layer = Layer::Enemy;
                value.0 = t.riser.pod_score_airborne;
                transform.translation.z = Kind::Wisp.z();
            }
            _ => {}
        }
        let fires = !out.shots.is_empty()
            && may_fire(
                rules,
                at,
                enemy.on_screen,
                hero,
                out.shots.len(),
                live_bullets,
            );
        if fires {
            live_bullets += out.shots.len();
            if enemy.kind == Kind::LoomCore {
                sfx.write(PlaySfx(Sfx::LoomTick));
            }
            for velocity in &out.shots {
                fire_bullet(&mut commands, &sprites, rules, at, *velocity * speed_scale);
            }
        }
        if let Some(mark) = out.mark
            && may_fire(rules, at, enemy.on_screen, hero, 0, 0)
        {
            sfx.write(PlaySfx(Sfx::FlakMark));
            commands.spawn((
                Crosshair {
                    left: t.flak.warning_seconds,
                },
                look::crosshair(&sprites),
                Transform::from_translation(mark.extend(28.0)),
                EnemyPart,
                DespawnOnExit(Screen::Playing),
            ));
        }
        for _ in 0..out.launch {
            let (launched, brain) = match enemy.kind {
                Kind::LoomShuttle => (Kind::Wisp, Brain::Wisp(Wisp::launched())),
                _ => {
                    sfx.write(PlaySfx(Sfx::RiserLaunch));
                    (Kind::RiserPod, Brain::RiserPod(RiserPod::default()))
                }
            };
            spawn_enemy(
                &mut commands,
                &sprites,
                tt,
                stage2,
                None,
                launched,
                at,
                brain,
            );
        }
        if out.despawn {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Keeps shuttles docked beside their core.
fn dock_shuttles(
    mut commands: Commands,
    tuning: Res<Tuning>,
    cores: Query<&Transform, (With<LoomLink>, Without<Docked>)>,
    mut shuttles: Query<(Entity, &Docked, &mut Transform)>,
) {
    let offset = tuning.vertical_shooter.enemies.loom.shuttle_offset;
    for (entity, docked, mut transform) in &mut shuttles {
        let Ok(core) = cores.get(docked.core) else {
            // The core left the screen: its shuttles go with it.
            commands.entity(entity).try_despawn();
            continue;
        };
        transform.translation.x = core.translation.x + docked.side * offset;
        transform.translation.y = core.translation.y;
    }
}

/// A Loom's shield drops once both its shuttles are destroyed.
fn open_looms(
    mut commands: Commands,
    mut cores: Query<(Entity, &LoomLink, &mut EnemyBrain), With<Immune>>,
    alive: Query<(), With<Enemy>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    for (entity, link, mut brain) in &mut cores {
        if link.shuttles.iter().all(|s| !alive.contains(*s)) {
            commands.entity(entity).try_remove::<Immune>();
            sfx.write(PlaySfx(Sfx::LoomShieldBreak));
            if let Brain::LoomCore(core) = &mut brain.0 {
                core.exposed = true;
            }
        }
    }
}

/// Each enemy's state frame, Swifts turned along their path, Crawler
/// turrets, and Riser pod shadows sliding out as they climb.
#[allow(clippy::type_complexity)]
fn show_parts(
    tuning: Res<Tuning>,
    mut enemies: Query<(
        &Enemy,
        &EnemyBrain,
        Has<Immune>,
        &mut Sprite,
        &mut Transform,
    )>,
    mut turrets: Query<
        (&ChildOf, &mut Transform, &mut Sprite),
        (With<look::Turret>, Without<Enemy>),
    >,
    mut shadows: Query<
        (&ChildOf, &mut Transform),
        (
            With<look::EnemyShadow>,
            Without<Enemy>,
            Without<look::Turret>,
        ),
    >,
) {
    let tt = &tuning.vertical_shooter;
    let t = &tt.enemies;
    for (enemy, brain, immune, mut sprite, mut transform) in &mut enemies {
        if let (Some(frame), Some(atlas)) = (
            look::frame(enemy.kind, &brain.0, immune, t),
            &mut sprite.texture_atlas,
        ) {
            atlas.index = frame;
        }
        if enemy.kind == Kind::Swift {
            transform.rotation = Quat::from_rotation_z(look::swift_rotation(&brain.0));
        }
    }
    for (parent, mut transform, mut sprite) in &mut turrets {
        if let Ok((_, EnemyBrain(Brain::Crawler(c)), ..)) = enemies.get(parent.parent()) {
            look::aim_turret(&mut transform, c.turret);
            if let Some(atlas) = &mut sprite.texture_atlas {
                // The turret flashes before each shot.
                atlas.index = (!c.silent && c.timer < t.crawler.telegraph_seconds) as usize;
            }
        }
    }
    let (sx, sy) = tt.look.shadow_offset;
    for (parent, mut transform) in &mut shadows {
        if let Ok((_, EnemyBrain(Brain::RiserPod(pod)), ..)) = enemies.get(parent.parent()) {
            let f = pod.climb(t);
            transform.translation.x = (sx * f).round();
            transform.translation.y = (sy * f).round();
        }
    }
}

/// Tether lines move with the ground and switch on and off.
fn run_tether_lines(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut lines: Query<(
        Entity,
        &mut TetherLine,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
        Has<Layer>,
    )>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let pulse = tether_pulse(&tuning.vertical_shooter.enemies);
    let dt = time.delta_secs();
    for (entity, mut line, mut transform, mut sprite, mut visibility, has_layer) in &mut lines {
        line.age += dt;
        transform.translation.y -= stage.speed() * dt;
        let live = pulse.live(line.age);
        let (color, vis) = look::line_look(live, pulse.telegraph(line.age), line.age);
        sprite.color = color;
        visibility.set_if_neq(vis);
        // Only a live line touches anything: it hurts the Hero and stops shots.
        if live && !has_layer {
            commands.entity(entity).try_insert(Layer::Enemy);
            sfx.write(PlaySfx(Sfx::TetherHum));
        } else if !live && has_layer {
            commands.entity(entity).try_remove::<Layer>();
        }
    }
}

fn run_chevrons(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut chevrons: Query<(Entity, &mut Chevron, &mut Visibility)>,
) {
    let toggles_per_second = 2.0 * tuning.vertical_shooter.enemies.swift.chevron_blink_hz;
    for (entity, mut chevron, mut visibility) in &mut chevrons {
        chevron.0 -= time.delta_secs();
        if chevron.0 <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let on = ((chevron.0 * toggles_per_second) as u32).is_multiple_of(2);
            visibility.set_if_neq(if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// Crosshairs count down; then the shell bursts where they were. The ring
/// only appears if the Hero is far enough from the point (the fairness
/// distance is measured from where bullets appear).
fn run_crosshairs(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    sprites: Res<Sprites>,
    hero: Option<Single<&Transform, (With<Hero>, Without<Crosshair>)>>,
    mut crosshairs: Query<(Entity, &mut Crosshair, &Transform)>,
    bullets: Query<(), With<EnemyBullet>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let tt = &tuning.vertical_shooter;
    let f = &tt.enemies.flak;
    let rules = &tt.enemy;
    let hero = hero.map(|h| h.translation.truncate());
    let scale = if session.stage == 1 {
        tt.stage2_bullet_speed_scale
    } else {
        1.0
    };
    let mut live_bullets = bullets.iter().count();
    for (entity, mut crosshair, transform) in &mut crosshairs {
        crosshair.left -= time.delta_secs();
        if crosshair.left > 0.0 {
            continue;
        }
        commands.entity(entity).despawn();
        let at = transform.translation.truncate();
        let side = f.burst_radius * 2.0;
        sfx.write(PlaySfx(Sfx::FlakBurst));
        commands.spawn((
            look::burst(f.burst_radius),
            Transform::from_translation(at.extend(27.0)),
            Hitbox::new(side, side),
            Layer::EnemyShot,
            FlakBurst,
            Lifetime(f.burst_seconds),
            EnemyPart,
            DespawnOnExit(Screen::Playing),
        ));
        let n = f.burst_count as usize;
        let far = hero.is_none_or(|h| h.distance(at) >= rules.min_fire_distance);
        if far && live_bullets + n <= rules.max_bullets {
            live_bullets += n;
            for d in crate::cabinets::shooter::pattern::ring(f.burst_count, 0.0) {
                fire_bullet(
                    &mut commands,
                    &sprites,
                    rules,
                    at,
                    d * f.burst_speed * scale,
                );
            }
        }
    }
}

/// A rewind to a Restart Point: everything of ours goes, Formations too.
fn clear_on_restart(
    mut commands: Commands,
    mut restarted: MessageReader<StageRestarted>,
    parts: Query<Entity, With<EnemyPart>>,
    bullets: Query<Entity, With<EnemyBullet>>,
) {
    if restarted.read().count() == 0 {
        return;
    }
    for entity in parts.iter().chain(&bullets) {
        commands.entity(entity).try_despawn();
    }
}

/// Counts kills and escapes for each Formation and pays out when one is
/// done. Also cuts a Tether line when a balloon is destroyed.
#[allow(clippy::too_many_arguments)]
fn track_formations(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    mut formations: Query<(Entity, &mut Formation)>,
    alive: Query<(), With<Enemy>>,
    lines: Query<(Entity, &TetherLine)>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut score: MessageWriter<ScoreGained>,
) {
    for d in destroyed.read() {
        for (_, mut formation) in &mut formations {
            if formation.has(d.entity) {
                formation.killed(d.entity, d.at);
            }
        }
        for (line, tether) in &lines {
            if tether.balloons.contains(&d.entity) {
                commands.entity(line).try_despawn();
            }
        }
    }
    for (entity, mut formation) in &mut formations {
        let escaped: Vec<_> = formation
            .pending()
            .iter()
            .copied()
            .filter(|&m| !alive.contains(m))
            .collect();
        for m in escaped {
            formation.lost(m);
        }
        if !formation.is_done() {
            continue;
        }
        match formation.reward() {
            Some(Reward::PowerUp(letter, at)) => {
                if let Some(p) = PowerUp::from_letter(letter) {
                    spawns.write(SpawnRequested::new(p.kind(), at));
                }
            }
            Some(Reward::Bonus { members }) => {
                let per = tuning.vertical_shooter.score.formation_bonus_per_enemy;
                score.write(ScoreGained(per * members as u64));
            }
            None => {}
        }
        commands.entity(entity).despawn();
    }
}

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                (
                    spawn_requested,
                    think,
                    dock_shuttles,
                    show_parts,
                    run_tether_lines,
                    run_chevrons,
                    run_crosshairs,
                )
                    .chain()
                    .after(KitSet::Timeline)
                    .before(KitSet::Move),
                (clear_on_restart, track_formations, open_looms)
                    .chain()
                    .after(KitSet::Resolve)
                    .before(KitSet::Effects),
            )
                .run_if(in_state(Screen::Playing))
                .run_if(gameplay_running)
                .run_if(resource_exists::<StageRun>)
                .run_if(crate::kit::stage_live)
                .run_if(cabinet_active)
                // Only once this Cabinet's Stage has been set up, not in the
                // frame where the Session has already moved on from Cabinet 1.
                .run_if(resource_exists::<super::power_ups::PowerUps>),
        );
    }
}

#[cfg(test)]
mod tests;
