//! Cabinet 1's enemies (docs/cabinets/1-side-shooter.md): spawning them
//! from the Stage timeline, running their [`Brain`]s, the fairness rules on
//! enemy fire, Formations and the Power-up drop rule, and the debug enemy
//! gallery.
//!
//! Stage files name an enemy by kind and may add params:
//!
//! ```ron
//! (at: 120.0, kind: "skiff", pos: (176.0, 40.0), params: {"size": 5, "drop": "G"}),
//! (at: 300.0, kind: "gull", pos: (176.0, 0.0), params: {"fires": true}),
//! (at: 400.0, kind: "barnacle", pos: (176.0, 100.0), params: {"ceiling": true}),
//! (at: 500.0, kind: "arc_gate", pos: (176.0, 0.0), params: {"floor_y": -100.0, "ceiling_y": 92.0}),
//! ```
//!
//! - `size`: Formation size (default: the kind's `formation_size` in tuning).
//! - `drop`: the Power-up letter the Formation drops when cleared. Without
//!   it, a Formation of more than one pays the Formation bonus instead.
//! - `fires`: Gulls only, the Stage 2 variant that shoots.
//! - `flip`: Gulls only, the wave in opposite phase, so two snakes cross.
//! - `ceiling`: Barnacles only, mounted upside down.
//! - `floor_y` / `ceiling_y`: Arc Gates only, the pylons' centres. Both are
//!   expected; a missing one logs a warning and falls back to
//!   [`ARC_DEFAULT_FLOOR_Y`] / [`ARC_DEFAULT_CEILING_Y`].
//!
//! `pos` is the Formation's lead: the tip of a Skiff V, the head of a Gull
//! snake, the centre of a Mote cloud. Hooks ignore it and come from behind,
//! either side of the Hero.

use bevy::prelude::*;
use serde::Deserialize;

use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::{ScoreGained, cabinet_is};
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::kit::{
    DespawnOffscreen, EnemyDestroyed, Health, Hero, Hitbox, Immune, KitSet, Layer, ScoreValue,
    SpawnRequested, StageRestarted, StageRun, StageTimeline, projectile, stage_live,
};
use crate::run::Session;
use crate::tuning::Tuning;

pub mod brain;
pub mod look;

use super::stages::terrain::{self, Terrain};
use crate::cabinets::shooter::formation::{Formation, Member, Reward};
use brain::{Arc, Brain, Ctx, Out};

/// How far past the right edge a Hook's path is checked against terrain.
const HOOK_LOOKAHEAD: f32 = 120.0;

/// The Power-up letters a Stage file may name in `drop`.
const POWER_UPS: [char; 6] = ['G', 'D', 'T', 'S', 'W', 'B'];

const GALLERY_STAGE: &str = include_str!("../../../../assets/stages/1-gallery.ron");

use super::CABINET;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Skiff,
    Gull,
    Bellbuoy,
    Lamplighter,
    Hook,
    Ferry,
    Barnacle,
    Clam,
    Mote,
    /// One of the two pylons of an Arc Gate.
    ArcPylon,
}

/// An enemy of Cabinet 1.
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

/// The warning chevron for a Hook; seconds left.
#[derive(Component)]
struct Chevron(f32);

/// The live part of an Arc Gate, between its two pylons.
#[derive(Component)]
struct ArcBeam {
    arc: Arc,
    pylons: [Entity; 2],
}

/// Settings a Stage file can give a spawn (see the module docs).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct SpawnParams {
    #[serde(deserialize_with = "present")]
    pub size: Option<u32>,
    #[serde(deserialize_with = "present")]
    pub drop: Option<String>,
    pub fires: bool,
    /// Gulls only: fly the wave in opposite phase, to cross another snake.
    pub flip: bool,
    pub ceiling: bool,
    #[serde(deserialize_with = "present")]
    pub floor_y: Option<f32>,
    #[serde(deserialize_with = "present")]
    pub ceiling_y: Option<f32>,
}

/// Where an Arc Gate's pylons go when a Stage file leaves out
/// `floor_y` / `ceiling_y` (a warning is logged): just inside the play area.
pub const ARC_DEFAULT_FLOOR_Y: f32 = -100.0;
pub const ARC_DEFAULT_CEILING_Y: f32 = 92.0;

impl SpawnParams {
    /// The centres of an Arc Gate's floor and ceiling pylons.
    pub fn pylon_ys(&self) -> (f32, f32) {
        (
            self.floor_y.unwrap_or(ARC_DEFAULT_FLOOR_Y),
            self.ceiling_y.unwrap_or(ARC_DEFAULT_CEILING_Y),
        )
    }
}

/// Reads an optional param written plainly (`"size": 5`, not `Some(5)`).
fn present<'de, D, T>(d: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(d).map(Some)
}

/// The debug enemy gallery, when it is switched on: it replaces Stage 1.
pub fn gallery_timeline(tuning: &Tuning) -> Option<StageTimeline> {
    (tuning.debug_keys && tuning.side_shooter.enemy_gallery)
        .then(|| StageTimeline::parse(GALLERY_STAGE).expect("enemy gallery Stage must be valid"))
}

fn on_screen(at: Vec2) -> bool {
    at.x.abs() <= HALF_W && at.y.abs() <= HALF_H
}

/// Where each member of a spawn starts and how it behaves.
pub fn layout(
    kind: Kind,
    lead: Vec2,
    params: &SpawnParams,
    hero: Option<Vec2>,
    t: &crate::cabinets::side_shooter::tuning::EnemyTypes,
) -> Vec<(Vec2, Brain)> {
    let size = |default: u32| params.size.unwrap_or(default).max(1);
    match kind {
        Kind::Skiff => (0..size(t.skiff.formation_size))
            .map(|i| {
                // The tip first, then pairs further back, above and below.
                let k = i.div_ceil(2) as f32;
                let side = match i {
                    0 => 0.0,
                    _ if i % 2 == 1 => 1.0,
                    _ => -1.0,
                };
                let (dx, dy) = t.skiff.v_spacing;
                let offset = Vec2::new(k * dx, side * k * dy);
                let brain = brain::Skiff::new(
                    t.skiff.peel_x + offset.x,
                    i as f32 * t.skiff.peel_interval,
                    side,
                );
                (lead + offset, Brain::Skiff(brain))
            })
            .collect(),
        Kind::Gull => (0..size(t.gull.formation_size))
            .map(|i| {
                let behind = i as f32 * t.gull.speed * t.gull.spacing_seconds;
                let gull = brain::Gull::new(lead.x, lead.y, params.fires).flipped(params.flip);
                (lead + Vec2::X * behind, Brain::Gull(gull))
            })
            .collect(),
        Kind::Hook => {
            let hero_y = hero.map_or(0.0, |h| h.y);
            (0..size(t.hook.formation_size))
                .map(|i| {
                    // Alternate above and below, a little wider each pair.
                    let pair = (i / 2) as f32;
                    let side = if i % 2 == 0 { 1.0 } else { -1.0 };
                    let offset = side * t.hook.pass_offset * (1.0 + pair * t.hook.pair_spread);
                    let hook = brain::Hook::new(hero_y, offset, t);
                    (Vec2::new(brain::HOOK_WAIT_X, hook.row), Brain::Hook(hook))
                })
                .collect()
        }
        Kind::Clam => (0..size(t.clam.formation_size))
            .map(|i| {
                let (dx, dy) = t.clam.pair_spacing;
                let at = lead + Vec2::new(dx, dy) * i as f32;
                (at, Brain::Clam(brain::Clam::new(at.y)))
            })
            .collect(),
        Kind::Mote => {
            let n = size(t.mote.formation_size);
            (0..n)
                .map(|i| {
                    // An even spread over the cloud (a golden-angle spiral).
                    let r = t.mote.cloud_radius * ((i as f32 + 0.5) / n as f32).sqrt();
                    let a = i as f32 * 2.399_963;
                    (
                        lead + Vec2::from_angle(a) * r,
                        Brain::Mote(brain::Mote::new()),
                    )
                })
                .collect()
        }
        Kind::Bellbuoy => vec![(
            Vec2::new(lead.x, t.bellbuoy.wave_y),
            Brain::Bellbuoy(brain::Bellbuoy::new()),
        )],
        Kind::Lamplighter => vec![(lead, Brain::Lamplighter(brain::Lamplighter::new()))],
        Kind::Ferry => vec![(lead, Brain::Ferry(brain::Ferry::new(t)))],
        Kind::Barnacle => vec![(
            lead,
            Brain::Barnacle(brain::Barnacle::new(params.ceiling, t)),
        )],
        Kind::ArcPylon => {
            let (floor, ceiling) = params.pylon_ys();
            vec![
                (Vec2::new(lead.x, floor), Brain::Drift),
                (Vec2::new(lead.x, ceiling), Brain::Drift),
            ]
        }
    }
}

/// The enemy a Stage file's spawn kind names, if it is one of ours.
pub fn kind_for(name: &str) -> Option<Kind> {
    Some(match name {
        "skiff" => Kind::Skiff,
        "gull" => Kind::Gull,
        "bellbuoy" => Kind::Bellbuoy,
        "lamplighter" => Kind::Lamplighter,
        "hook" => Kind::Hook,
        "ferry" => Kind::Ferry,
        "barnacle" => Kind::Barnacle,
        "clam" => Kind::Clam,
        "mote" => Kind::Mote,
        "arc_gate" => Kind::ArcPylon,
        _ => return None,
    })
}

/// Spawns one enemy, unless it would appear on screen too close to the Hero.
fn spawn_enemy(
    commands: &mut Commands,
    tuning: &Tuning,
    stage2: bool,
    hero: Option<Vec2>,
    kind: Kind,
    at: Vec2,
    brain: Brain,
    flipped: bool,
) -> Option<Entity> {
    let rules = &tuning.side_shooter.enemy;
    let t = &tuning.side_shooter.enemies;
    if on_screen(at) && hero.is_some_and(|h| h.distance(at) < rules.no_spawn_radius) {
        return None;
    }
    let (hp, score) = match kind {
        Kind::Skiff => (t.skiff.hp, t.skiff.score),
        Kind::Gull => (t.gull.hp, t.gull.score),
        Kind::Bellbuoy => (t.bellbuoy.hp, t.bellbuoy.score),
        Kind::Lamplighter => (t.lamplighter.hp, t.lamplighter.score),
        Kind::Hook => (t.hook.hp, t.hook.score),
        Kind::Ferry if stage2 => (t.ferry.hp_stage2, t.ferry.score),
        Kind::Ferry => (t.ferry.hp, t.ferry.score),
        Kind::Barnacle => (t.barnacle.hp, t.barnacle.score),
        Kind::Clam => (t.clam.hp, t.clam.score),
        Kind::Mote => (t.mote.hp, t.mote.score),
        Kind::ArcPylon => (t.arc_gate.hp, t.arc_gate.score),
    };
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
        Transform::from_translation(at.extend(4.0)),
        Hitbox::new(hitbox.x, hitbox.y),
        Layer::Enemy,
        Health(hp.max(1)),
        ScoreValue(score),
        DespawnOffscreen::new(size.max_element() / 2.0 + 4.0),
        DespawnOnExit(Screen::Playing),
    ));
    if kind == Kind::Clam {
        entity.insert(Immune);
    }
    look::body(&mut entity, kind, flipped);
    Some(entity.id())
}

#[allow(clippy::too_many_arguments)]
fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    session: Res<Session>,
    hero: Option<Single<&Transform, With<Hero>>>,
    stage: Res<StageRun>,
    terrain: Option<Res<Terrain>>,
    mut requests: MessageReader<SpawnRequested>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let hero = hero.map(|h| h.translation.truncate());
    // The tightest corridor from the left edge to a little past the right:
    // where a Hook will fly.
    let corridor = terrain.filter(|t| !t.is_empty()).map(|t| {
        let left = terrain::scroll_at(-HALF_W, stage.cursor.position);
        t.corridor(left, left + 2.0 * HALF_W + HOOK_LOOKAHEAD)
    });
    let stage2 = session.stage == 1;
    let t = &tuning.side_shooter.enemies;
    for request in requests.read() {
        // Kinds this module doesn't know (Power-ups, terrain) belong to
        // other parts of the Cabinet.
        let Some(kind) = kind_for(&request.kind) else {
            continue;
        };
        let params: SpawnParams = request.params().unwrap_or_else(|e| {
            warn!("bad params for {:?}: {e}", request.kind);
            SpawnParams::default()
        });
        let drop = params.drop.as_deref().and_then(|d| {
            let letter = d
                .chars()
                .next()
                .filter(|c| d.len() == 1 && POWER_UPS.contains(c));
            if letter.is_none() {
                warn!("unknown Power-up {d:?} for {:?}", request.kind);
            }
            letter
        });
        if kind == Kind::ArcPylon && (params.floor_y.is_none() || params.ceiling_y.is_none()) {
            warn!(
                "arc_gate at x = {} has no floor_y/ceiling_y; using {ARC_DEFAULT_FLOOR_Y}/{ARC_DEFAULT_CEILING_Y}",
                request.pos.x
            );
        }
        let mut members = layout(kind, request.pos, &params, hero, t);
        if let Some((floor, ceiling)) = corridor {
            for (at, brain) in &mut members {
                if let Brain::Hook(hook) = brain {
                    *hook = hook.clone().within(floor, ceiling, t);
                    at.y = hook.row;
                }
            }
        }
        let mut spawned = Vec::new();
        for (i, (at, brain)) in members.into_iter().enumerate() {
            let flipped =
                (kind == Kind::Barnacle && params.ceiling) || (kind == Kind::ArcPylon && i == 1);
            if let Some(e) = spawn_enemy(
                &mut commands,
                &tuning,
                stage2,
                hero,
                kind,
                at,
                brain,
                flipped,
            ) {
                spawned.push(e);
                if kind == Kind::Hook {
                    sfx.write(PlaySfx(Sfx::HookWarning));
                    commands.spawn((
                        Chevron(t.hook.warning_seconds),
                        look::chevron(),
                        Transform::from_xyz(-HALF_W + 3.0, at.y, 30.0),
                        EnemyPart,
                        DespawnOnExit(Screen::Playing),
                    ));
                }
            }
        }
        if kind == Kind::ArcPylon {
            if let [low, high] = spawned[..] {
                spawn_arc(&mut commands, &tuning, request.pos.x, &params, [low, high]);
            }
            // Arc Gates are not Formations unless the Stage asks for a drop.
            if drop.is_none() {
                continue;
            }
        }
        if spawned.len() > 1 || drop.is_some() {
            commands.spawn((
                Formation::new(spawned.clone(), drop),
                EnemyPart,
                DespawnOnExit(Screen::Playing),
            ));
            for member in spawned {
                commands.entity(member).try_insert(Member);
            }
        }
    }
}

fn spawn_arc(
    commands: &mut Commands,
    tuning: &Tuning,
    x: f32,
    params: &SpawnParams,
    pylons: [Entity; 2],
) {
    let (floor, ceiling) = params.pylon_ys();
    let half_pylon = look::size(Kind::ArcPylon).y / 2.0;
    let gap = (ceiling - floor - 2.0 * half_pylon).max(1.0);
    let width = tuning.side_shooter.enemies.arc_gate.arc_width;
    commands.spawn((
        ArcBeam {
            arc: Arc::default(),
            pylons,
        },
        Sprite::from_color(look::palette::FOAM, Vec2::new(width, gap)),
        Visibility::Hidden,
        Transform::from_xyz(x, (floor + ceiling) / 2.0, 3.5),
        Hitbox::new(width, gap),
        EnemyPart,
        DespawnOffscreen::new(8.0),
        DespawnOnExit(Screen::Playing),
    ));
}

/// Steps every brain and fires what the fairness rules allow.
#[allow(clippy::type_complexity)]
fn think(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    session: Res<Session>,
    stage: Res<StageRun>,
    hero: Option<Single<&Transform, (With<Hero>, Without<Enemy>)>>,
    mut enemies: Query<(
        Entity,
        &mut Enemy,
        &mut EnemyBrain,
        &mut Transform,
        Has<Immune>,
    )>,
    bullets: Query<(), With<EnemyBullet>>,
) {
    let tt = &tuning.side_shooter;
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
    for (entity, mut enemy, mut brain, mut transform, immune) in &mut enemies {
        let mut at = transform.translation.truncate();
        let out: Out = brain.0.step(&mut at, &ctx, &tt.enemies);
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        enemy.clock += ctx.dt;
        if on_screen(at) {
            enemy.on_screen += ctx.dt;
        }
        if out.despawn {
            commands.entity(entity).try_despawn();
            continue;
        }
        // A Clam is immune while shut; change it only as the shell moves.
        if let Brain::Clam(clam) = &brain.0 {
            let open = clam.is_open(&tt.enemies);
            if open && immune {
                commands.entity(entity).try_remove::<Immune>();
            } else if !open && !immune {
                commands.entity(entity).try_insert(Immune);
            }
        }
        let (offset_x, offset_y) = tt.enemies.ferry.launch_offset;
        for i in 0..out.launch {
            let spread = i as f32 - (out.launch as f32 - 1.0) / 2.0;
            let from = at + Vec2::new(offset_x, spread * offset_y);
            spawn_enemy(
                &mut commands,
                &tuning,
                stage2,
                hero,
                Kind::Skiff,
                from,
                Brain::Skiff(brain::Skiff::launched()),
                false,
            );
        }
        if out.shots.is_empty() {
            continue;
        }
        // Only from on screen: nothing fires at the Hero from where it can't
        // be seen, even after its grace period.
        let may_fire = on_screen(at)
            && enemy.on_screen >= rules.fire_grace_seconds
            && hero.is_none_or(|h| h.distance(at) >= rules.min_fire_distance);
        if !may_fire {
            continue;
        }
        for velocity in out.shots {
            if live_bullets >= rules.max_bullets {
                break;
            }
            live_bullets += 1;
            let (w, h) = rules.bullet_size;
            let (hw, hh) = rules.bullet_hitbox;
            commands
                .spawn((
                    projectile(
                        Layer::EnemyShot,
                        at,
                        velocity * speed_scale,
                        Vec2::new(w, h),
                        look::BULLET,
                        rules.bullet_lifetime,
                    ),
                    EnemyBullet,
                ))
                .insert(Hitbox::new(hw, hh))
                .with_children(|c| {
                    c.spawn(look::bullet_core());
                });
        }
    }
}

/// Warning flashes on cores, and Clam cores that show only while open.
fn show_cores(
    tuning: Res<Tuning>,
    enemies: Query<(&Enemy, &EnemyBrain)>,
    mut cores: Query<(&ChildOf, &mut Sprite, &mut Visibility), With<look::Core>>,
) {
    let t = &tuning.side_shooter.enemies;
    for (parent, mut sprite, mut visibility) in &mut cores {
        let Ok((enemy, brain)) = enemies.get(parent.parent()) else {
            continue;
        };
        sprite.color = look::core_color(brain.0.telegraph(t), enemy.clock);
        if let (Kind::Clam, Brain::Clam(clam)) = (enemy.kind, &brain.0) {
            visibility.set_if_neq(if clam.is_open(t) {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
    }
}

/// Arcs move with their pylons and switch on and off.
fn run_arcs(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut sfx: MessageWriter<PlaySfx>,
    mut arcs: Query<(
        Entity,
        &mut ArcBeam,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
        Has<Layer>,
    )>,
) {
    let t = &tuning.side_shooter.enemies;
    let dt = time.delta_secs();
    for (entity, mut beam, mut transform, mut sprite, mut visibility, has_layer) in &mut arcs {
        beam.arc.age += dt;
        transform.translation.x -= stage.speed() * dt;
        let live = beam.arc.live(t);
        let warning = beam.arc.telegraph(t);
        let (color, vis) = look::arc_look(live, warning, beam.arc.age);
        sprite.color = color;
        visibility.set_if_neq(vis);
        // Only a live arc touches anything: it hurts the Hero and stops shots.
        if live && !has_layer {
            commands.entity(entity).try_insert(Layer::Enemy);
            // Hum only where the player can see the gate.
            if transform.translation.x < HALF_W {
                sfx.write(PlaySfx(Sfx::ArcHum));
            }
        } else if !live && has_layer {
            commands.entity(entity).try_remove::<Layer>();
        }
    }
}

/// Points each Barnacle's barrel the way its brain is aiming.
fn aim_barrels(
    enemies: Query<&EnemyBrain>,
    mut barrels: Query<(&ChildOf, &mut Transform), With<look::Barrel>>,
) {
    for (parent, mut transform) in &mut barrels {
        if let Ok(EnemyBrain(Brain::Barnacle(b))) = enemies.get(parent.parent()) {
            look::aim_barrel(&mut transform, b.barrel);
        }
    }
}

fn run_chevrons(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    mut chevrons: Query<(Entity, &mut Chevron, &mut Visibility)>,
) {
    for (entity, mut chevron, mut visibility) in &mut chevrons {
        chevron.0 -= time.delta_secs();
        if chevron.0 <= 0.0 {
            commands.entity(entity).try_despawn();
        } else {
            let toggles_per_second = 2.0 * tuning.side_shooter.enemies.hook.chevron_blink_hz;
            let on = ((chevron.0 * toggles_per_second) as u32).is_multiple_of(2);
            visibility.set_if_neq(if on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
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
/// done. Also switches off an Arc Gate when a pylon is destroyed.
fn track_formations(
    mut commands: Commands,
    tuning: Res<Tuning>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    mut formations: Query<(Entity, &mut Formation)>,
    alive: Query<(), With<Enemy>>,
    arcs: Query<(Entity, &ArcBeam)>,
    mut spawns: MessageWriter<SpawnRequested>,
    mut score: MessageWriter<ScoreGained>,
) {
    for d in destroyed.read() {
        for (_, mut formation) in &mut formations {
            if formation.has(d.entity) {
                formation.killed(d.entity, d.at);
            }
        }
        for (arc, beam) in &arcs {
            if beam.pylons.contains(&d.entity) {
                commands.entity(arc).try_despawn();
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
                spawns.write(SpawnRequested::new(format!("power_up:{letter}"), at));
            }
            Some(Reward::Bonus { members }) => {
                let per = tuning.side_shooter.score.formation_bonus_per_enemy;
                score.write(ScoreGained(per * members as u64));
            }
            None => {}
        }
        commands.entity(entity).try_despawn();
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
                    show_cores,
                    aim_barrels,
                    run_arcs,
                    run_chevrons,
                )
                    .chain()
                    .after(KitSet::Timeline)
                    .before(KitSet::Move),
                (clear_on_restart, track_formations)
                    .chain()
                    .after(KitSet::Resolve)
                    .before(KitSet::Effects),
            )
                .run_if(cabinet_is(CABINET))
                .run_if(stage_live),
        );
    }
}

#[cfg(test)]
mod tests;
