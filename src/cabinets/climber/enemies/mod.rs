//! Cabinet 4's enemies and hazards (docs/cabinets/4-climber.md): spawning
//! them from the Stage timeline, running their [`Brain`]s in the right
//! space, the fairness rules on enemy fire, solid ledges stopping shots,
//! canisters, scrapfalls, Formations, Standoff members, the Sweeper, the
//! gust arrows and the debug enemy gallery.
//!
//! Stage files name an enemy by kind and may add params:
//!
//! ```ron
//! (at: 56.0, kind: "flit", pos: (176.0, 40.0), params: {"side": "right", "drop": "G"}),
//! (at: 1400.0, kind: "flit", pos: (-48.0, -128.0), params: {"from": "below"}),
//! (at: 304.0, kind: "limpet", pos: (8.0, 0.0), params: {"size": 2}),
//! (at: 1096.0, kind: "scrapfall", pos: (0.0, 128.0), params: {"from_x": -112.0, "to_x": -24.0, "seconds": 5.0}),
//! (at: 688.0, kind: "sweeper", pos: (64.0, 64.0), params: {"dir": "cw"}),
//! ```
//!
//! - Ledge enemies (Limpet, Picket, Pitcher) stand on the highest ledge
//!   under `pos.x` at or below the top edge and still on screen; `pos.y` is
//!   ignored.
//! - Things fixed in the face (Vents, Sweepers) are at `pos` when the camera
//!   is at the spawn's `at`, and move with the tower from there.
//! - Flyers, Scrap, Plumbs, Motes and the Winch are placed on screen.

use bevy::prelude::*;
use serde::Deserialize;

use super::palette;
use super::power_ups::{PowerUp, PowerUps};
use super::tower::{Tower, screen_y, tower_y};
use super::tuning::{ClimberTuning, EnemyRules, EnemyTypes};
use crate::audio::{PlaySfx, Sfx};
use crate::cabinets::ScoreGained;
use crate::cabinets::shooter::formation::{Formation, Member, Reward};
use crate::cabinets::side_shooter::power_ups::PLAY_TOP;
use crate::display::{HALF_H, HALF_W};
use crate::flow::Screen;
use crate::kit::{
    DespawnOffscreen, EnemyDestroyed, Flash, Health, Hero, Hitbox, Hurt, Immune, KitSet, Layer,
    ScoreValue, SpawnRequested, StageRestarted, StageRun, Velocity, projectile, stage_live,
};
use crate::run::Session;
use crate::sprites::Sprites;
use crate::tuning::Tuning;

pub mod brain;
pub mod look;

use brain::{
    Brain, Canister, Ctx, Flit, Limpet, Mote, Out, Picket, Pitcher, Plumb, Scrap, Scrapfall, Vent,
    Winch,
};

const GALLERY_STAGE: &str = include_str!("../../../../assets/stages/4-gallery.ron");

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    Flit,
    Limpet,
    Scrap,
    Picket,
    Vent,
    Plumb,
    Pitcher,
    /// A Pitcher's canister: 1 HP, shot down for no points.
    Canister,
    Mote,
    /// The Winch's hook, the only part that takes damage.
    Winch,
    /// The Winch's carriage: immune, hurts on touch.
    Carriage,
}

impl Kind {
    /// Lives on the tower (tower space); everything else is on screen.
    pub fn on_tower(self) -> bool {
        matches!(
            self,
            Kind::Limpet | Kind::Picket | Kind::Pitcher | Kind::Vent | Kind::Canister
        )
    }

    /// Stands on a ledge: its `pos` is its feet.
    pub fn stands(self) -> bool {
        matches!(self, Kind::Picket | Kind::Pitcher)
    }
}

/// An enemy of Cabinet 4.
#[derive(Component, Debug)]
pub struct Enemy {
    pub kind: Kind,
    on_screen: f32,
    clock: f32,
}

#[derive(Component, Debug)]
pub struct EnemyBrain(pub Brain);

/// An enemy's position in its brain's space: tower space for the kinds on
/// the tower, the centre on screen for the rest.
#[derive(Component, Debug, Clone, Copy)]
pub struct EnemyPos(pub Vec2);

/// A bullet fired by an enemy (counted for the bullet cap).
#[derive(Component)]
pub struct EnemyBullet;

/// Part of the running Standoff.
#[derive(Component)]
pub struct StandoffMember;

/// A Standoff timed out: it leaves the way it came.
#[derive(Component)]
pub struct Leaving;

/// Waiting below the bottom edge while its chevron blinks (a Flit from
/// below).
#[derive(Component)]
struct Waiting(f32);

/// Anything this module spawned that a rewind removes.
#[derive(Component)]
struct EnemyPart;

/// A warning chevron: seconds left to blink.
#[derive(Component)]
struct Chevron(f32);

/// A scrapfall dropping Scrap.
#[derive(Component)]
struct ScrapfallRun {
    fall: Scrapfall,
    standoff: bool,
}

/// A cable from the top edge (a Plumb) or the rail (the Winch) to its owner.
#[derive(Component)]
struct Cable {
    owner: Entity,
}

/// The Winch's carriage, on its rail above its hook.
#[derive(Component)]
struct OnRail {
    hook: Entity,
}

/// A Sweeper: an arm turning on a hub in the face (a hazard, not an enemy).
#[derive(Component, Debug)]
pub struct Sweeper {
    /// The hub in tower space.
    pub hub: Vec2,
    /// The arm's angle, radians.
    pub angle: f32,
    /// +1 counter-clockwise, -1 clockwise.
    pub turn: f32,
}

impl Sweeper {
    /// Points along the arm, from the hub to the tip, `n` of them.
    pub fn arm(&self, length: f32, n: usize) -> Vec<Vec2> {
        let d = Vec2::from_angle(self.angle);
        (1..=n)
            .map(|i| self.hub + d * length * i as f32 / n as f32)
            .collect()
    }
}

/// One box of a Sweeper's arm: `index` along it.
#[derive(Component)]
struct ArmBox {
    sweeper: Entity,
    index: usize,
}

/// How many boxes cover a Sweeper's arm.
const ARM_BOXES: usize = 6;

/// A gust zone's arrows: its index in `Tower::gusts`.
#[derive(Component)]
struct GustArrows(usize);

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
    pub side: Option<String>,
    #[serde(deserialize_with = "present")]
    pub from: Option<String>,
    #[serde(deserialize_with = "present")]
    pub from_x: Option<f32>,
    #[serde(deserialize_with = "present")]
    pub to_x: Option<f32>,
    #[serde(deserialize_with = "present")]
    pub seconds: Option<f32>,
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
    (tuning.debug_keys && tuning.climber.enemy_gallery).then_some(GALLERY_STAGE)
}

/// The gallery's file (for tests).
#[cfg(test)]
pub fn gallery_file() -> &'static str {
    GALLERY_STAGE
}

pub fn kind_for(name: &str) -> Option<Kind> {
    Some(match name {
        "flit" => Kind::Flit,
        "limpet" => Kind::Limpet,
        "scrap" => Kind::Scrap,
        "picket" => Kind::Picket,
        "vent" => Kind::Vent,
        "plumb" => Kind::Plumb,
        "pitcher" => Kind::Pitcher,
        "mote" => Kind::Mote,
        "winch" => Kind::Winch,
        _ => return None,
    })
}

/// The ledge (by index) a ledge enemy at x stands on, with the camera at
/// `camera`: the highest under x at or below the top edge, still on screen.
pub fn footing(tower: &Tower, x: f32, camera: f32) -> Option<usize> {
    tower
        .ledges
        .iter()
        .enumerate()
        .filter(|(_, l)| l.covers(x) && l.h <= camera && screen_y(l.h, camera) >= -HALF_H)
        .max_by(|a, b| a.1.h.total_cmp(&b.1.h))
        .map(|(i, _)| i)
}

/// The landing the Winch works over: the highest solid ledge under x whose
/// top is on screen.
pub fn winch_floor(tower: &Tower, x: f32, camera: f32) -> Option<f32> {
    tower
        .ledges
        .iter()
        .filter(|l| Tower::is_solid(l) && l.covers(x))
        .map(|l| screen_y(l.h, camera))
        .filter(|y| (-HALF_H..PLAY_TOP - 48.0).contains(y))
        .reduce(f32::max)
}

/// Where each member of a spawn starts (in its brain's space) and how it
/// behaves. `lead` is the spawn's `pos`, on screen.
pub fn layout(
    kind: Kind,
    lead: Vec2,
    params: &SpawnParams,
    camera: f32,
    tower: &Tower,
    t: &ClimberTuning,
) -> Vec<(Vec2, Brain)> {
    let e = &t.enemies;
    let size = |default: u32| params.size.unwrap_or(default).max(1);
    match kind {
        Kind::Flit => {
            let n = size(e.flit.formation_size);
            let gap = e.flit.speed * e.flit.spacing_seconds;
            if params.from.as_deref() == Some("below") {
                (0..n)
                    .map(|i| {
                        let at = Vec2::new(lead.x, -HALF_H - 8.0 - gap * i as f32);
                        (at, Brain::Flit(Flit::from_below()))
                    })
                    .collect()
            } else {
                // From the side its entry names (or the side it is on).
                let from_left = match params.side.as_deref() {
                    Some("left") => true,
                    Some("right") => false,
                    _ => lead.x < 0.0,
                };
                let dir = if from_left { 1.0 } else { -1.0 };
                let x0 = -dir * (HALF_W + 8.0);
                (0..n)
                    .map(|i| {
                        let at = Vec2::new(x0 - dir * gap * i as f32, lead.y);
                        (at, Brain::Flit(Flit::crossing(dir)))
                    })
                    .collect()
            }
        }
        Kind::Limpet => {
            let Some(i) = footing(tower, lead.x, camera) else {
                return Vec::new();
            };
            let l = &tower.ledges[i];
            let round = Limpet::round(l.from_x, l.to_x);
            (0..size(e.limpet.formation_size))
                .map(|k| {
                    let along = round * k as f32 / size(e.limpet.formation_size) as f32;
                    (
                        Limpet::place(l.from_x, l.to_x, l.h, along),
                        Brain::Limpet(Limpet::new(i, along)),
                    )
                })
                .collect()
        }
        Kind::Picket | Kind::Pitcher => {
            let Some(i) = footing(tower, lead.x, camera) else {
                return Vec::new();
            };
            let l = &tower.ledges[i];
            let half = look::size(kind).x / 2.0;
            let x = lead.x.clamp(l.from_x + half, l.to_x - half);
            let brain = if kind == Kind::Picket {
                Brain::Picket(Picket::new(i, if x > 0.0 { -1.0 } else { 1.0 }))
            } else {
                Brain::Pitcher(Pitcher::new(e))
            };
            vec![(Vec2::new(x, l.h), brain)]
        }
        Kind::Vent => vec![(
            Vec2::new(lead.x, tower_y(lead.y, camera)),
            Brain::Vent(Vent::default()),
        )],
        Kind::Scrap => vec![(
            Vec2::new(lead.x, PLAY_TOP + 8.0),
            Brain::Scrap(Scrap::new(e)),
        )],
        Kind::Plumb => vec![(
            Vec2::new(lead.x, PLAY_TOP + 5.0),
            Brain::Plumb(Plumb::default()),
        )],
        Kind::Mote => {
            let n = size(e.mote.formation_size);
            (0..n)
                .map(|i| {
                    let r = e.mote.cloud_radius * ((i as f32 + 0.5) / n as f32).sqrt();
                    let a = i as f32 * 2.399_963;
                    (lead + Vec2::from_angle(a) * r, Brain::Mote(Mote::default()))
                })
                .collect()
        }
        Kind::Winch => {
            let floor = winch_floor(tower, lead.x, camera).unwrap_or(-HALF_H + 64.0);
            let hang = e.winch.rail_y - brain::HOOK_HANG;
            vec![(
                Vec2::new(lead.x, hang),
                Brain::Winch(Winch::new(lead.x, floor)),
            )]
        }
        Kind::Canister | Kind::Carriage => Vec::new(),
    }
}

fn hp_and_score(kind: Kind, t: &EnemyTypes) -> (u32, u64) {
    match kind {
        Kind::Flit => (t.flit.hp, t.flit.score),
        Kind::Limpet => (t.limpet.hp, t.limpet.score),
        Kind::Scrap => (t.scrap.hp, t.scrap.score),
        Kind::Picket => (t.picket.hp, t.picket.score),
        Kind::Vent => (t.vent.hp, t.vent.score),
        Kind::Plumb => (t.plumb.hp, t.plumb.score),
        Kind::Pitcher => (t.pitcher.hp, t.pitcher.score),
        Kind::Canister => (1, 0),
        Kind::Mote => (t.mote.hp, t.mote.score),
        Kind::Winch => (t.winch.hp, t.winch.score),
        Kind::Carriage => (1000, 0),
    }
}

/// The screen centre of an enemy at `pos` (its brain's space).
pub fn screen_centre(kind: Kind, pos: Vec2, camera: f32) -> Vec2 {
    if !kind.on_tower() {
        return pos;
    }
    let lift = if kind.stands() {
        look::size(kind).y / 2.0
    } else {
        0.0
    };
    Vec2::new(pos.x, screen_y(pos.y, camera) + lift)
}

/// The height (tower space) of the bottom of an enemy: its feet, for the
/// "nothing fires from below the Hero" rule.
pub fn feet_height(kind: Kind, pos: Vec2, camera: f32) -> f32 {
    let centre = screen_centre(kind, pos, camera);
    tower_y(centre.y - look::size(kind).y / 2.0, camera)
}

fn spawn_enemy(
    commands: &mut Commands,
    sprites: &Sprites,
    t: &ClimberTuning,
    kind: Kind,
    pos: Vec2,
    brain: Brain,
    camera: f32,
) -> Entity {
    let (hp, score) = hp_and_score(kind, &t.enemies);
    let size = look::size(kind);
    let hitbox = (size - 2.0 * t.enemy.hitbox_inset).max(Vec2::ONE);
    let at = screen_centre(kind, pos, camera);
    let mut e = commands.spawn((
        Enemy {
            kind,
            on_screen: 0.0,
            clock: 0.0,
        },
        sprites.sprite(look::sprite(kind), super::CABINET),
        EnemyBrain(brain),
        EnemyPos(pos),
        EnemyPart,
        Transform::from_translation(at.extend(4.0)),
        Hitbox::new(hitbox.x, hitbox.y),
        Layer::Enemy,
        Health(hp.max(1)),
        ScoreValue(score),
        DespawnOnExit(Screen::Playing),
    ));
    // Things that come from off screen on purpose leave on their own.
    if !matches!(
        kind,
        Kind::Flit | Kind::Scrap | Kind::Plumb | Kind::Winch | Kind::Carriage
    ) {
        e.insert(DespawnOffscreen::new(size.max_element() / 2.0 + 24.0));
    }
    if matches!(kind, Kind::Vent | Kind::Carriage) {
        e.insert(Immune);
    }
    e.id()
}

#[allow(clippy::too_many_arguments)]
pub(super) fn spawn_requested(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    tower: Res<Tower>,
    sprites: Res<Sprites>,
    hero: Option<Single<&Transform, With<Hero>>>,
    mut requests: MessageReader<SpawnRequested>,
) {
    let t = &tuning.climber;
    let camera = stage.cursor.position;
    let hero = hero.map(|h| h.translation.truncate());
    for request in requests.read() {
        let params: SpawnParams = request.params().unwrap_or_default();
        match request.kind.as_str() {
            "sweeper" => {
                spawn_sweeper(&mut commands, &sprites, t, request.pos, &params, camera);
                continue;
            }
            "scrapfall" => {
                let (from, to) = (params.from_x.unwrap_or(-96.0), params.to_x.unwrap_or(96.0));
                let seed = (request.pos.x.abs() + from.abs() * 7.0 + camera * 13.0) as u32;
                commands.spawn((
                    ScrapfallRun {
                        fall: Scrapfall::new(from, to, params.seconds.unwrap_or(4.0), seed),
                        standoff: params.standoff,
                    },
                    EnemyPart,
                    DespawnOnExit(Screen::Playing),
                ));
                continue;
            }
            _ => {}
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
        let members = layout(kind, request.pos, &params, camera, &tower, t);
        if members.is_empty() {
            warn!("{} at {}: no ledge to stand on", request.kind, request.pos);
        }
        let below = kind == Kind::Flit && params.from.as_deref() == Some("below");
        let mut spawned = Vec::new();
        for (pos, brain) in members {
            let at = screen_centre(kind, pos, camera);
            if on_screen(at) && hero.is_some_and(|h| h.distance(at) < t.enemy.no_spawn_radius) {
                continue;
            }
            let e = spawn_enemy(&mut commands, &sprites, t, kind, pos, brain, camera);
            if below {
                commands
                    .entity(e)
                    .insert(Waiting(t.enemy.below_warning_seconds));
            }
            if params.standoff {
                commands.entity(e).insert(StandoffMember);
            }
            match kind {
                Kind::Winch => {
                    let carriage = spawn_enemy(
                        &mut commands,
                        &sprites,
                        t,
                        Kind::Carriage,
                        Vec2::new(pos.x, t.enemies.winch.rail_y),
                        Brain::Mote(Mote::default()),
                        camera,
                    );
                    commands.entity(carriage).insert(OnRail { hook: e });
                    spawn_cable(&mut commands, &sprites, "c4_winch_cable", e);
                    // The hook's box is its core.
                    commands.entity(e).insert(Hitbox::new(10.0, 10.0));
                }
                Kind::Plumb => spawn_cable(&mut commands, &sprites, "c4_plumb_cable", e),
                _ => {}
            }
            spawned.push(e);
        }
        if below && !spawned.is_empty() {
            commands.spawn((
                Chevron(t.enemy.below_warning_seconds),
                sprites.sprite("c4_chevron_up", super::CABINET),
                Transform::from_xyz(request.pos.x, -HALF_H + 3.0, 30.0),
                EnemyPart,
                DespawnOnExit(Screen::Playing),
            ));
        }
        if kind == Kind::Scrap && !spawned.is_empty() {
            scrap_chevron(&mut commands, &sprites, t, request.pos.x);
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

fn scrap_chevron(commands: &mut Commands, sprites: &Sprites, t: &ClimberTuning, x: f32) {
    commands.spawn((
        Chevron(t.enemies.scrap.warning_seconds),
        sprites.sprite("c4_chevron_down", super::CABINET),
        Transform::from_xyz(x, PLAY_TOP - 3.0, 30.0),
        EnemyPart,
        DespawnOnExit(Screen::Playing),
    ));
}

fn spawn_cable(commands: &mut Commands, sprites: &Sprites, sprite: &str, owner: Entity) {
    let mut s = sprites.sprite(sprite, super::CABINET);
    s.image_mode = SpriteImageMode::Tiled {
        tile_x: false,
        tile_y: true,
        stretch_value: 1.0,
    };
    s.custom_size = Some(Vec2::new(2.0, 8.0));
    commands.spawn((
        s,
        Transform::from_xyz(0.0, HALF_H, 3.9),
        Cable { owner },
        EnemyPart,
        DespawnOnExit(Screen::Playing),
    ));
}

fn on_screen(at: Vec2) -> bool {
    at.x.abs() <= HALF_W && at.y.abs() <= HALF_H
}

/// The fairness rules for one pattern. Skipped whole if any fails.
pub fn may_fire(
    rules: &EnemyRules,
    shots: &[Vec2],
    on_screen_seconds: f32,
    hero_centre: Option<Vec2>,
    shooter_feet: f32,
    hero_feet: Option<f32>,
    live_bullets: usize,
) -> bool {
    if on_screen_seconds < rules.fire_grace_seconds
        || live_bullets + shots.len() > rules.max_bullets
    {
        return false;
    }
    // Nothing fires from below the Hero: it has been passed.
    if hero_feet.is_some_and(|h| shooter_feet < h - 0.5) {
        return false;
    }
    let Some(hero) = hero_centre else {
        return true;
    };
    shots
        .iter()
        .all(|&from| hero.distance(from) >= rules.min_fire_distance)
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
    tower: Res<Tower>,
    sprites: Res<Sprites>,
    hero: Option<Single<(&Transform, &super::hero::Climber), Without<Enemy>>>,
    mut enemies: Query<(
        Entity,
        &mut Enemy,
        &mut EnemyBrain,
        &mut EnemyPos,
        &mut Transform,
        &mut Sprite,
        Option<&mut Waiting>,
        Has<Leaving>,
        Has<StandoffMember>,
    )>,
    bullets: Query<(), With<EnemyBullet>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.climber;
    let e = &t.enemies;
    let rules = &t.enemy;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    let scale = bullet_scale(&session, t);
    let hero_feet = hero.as_ref().map(|h| Vec2::new(h.1.body.x, h.1.body.feet));
    let hero_centre = hero.as_ref().map(|h| h.0.translation.truncate());
    let mut live = bullets.iter().count();
    for (
        entity,
        mut enemy,
        mut brain,
        mut pos,
        mut transform,
        mut sprite,
        waiting,
        leaving,
        member,
    ) in &mut enemies
    {
        if enemy.kind == Kind::Carriage {
            continue;
        }
        if let Some(mut w) = waiting {
            w.0 -= dt;
            if w.0 <= 0.0 {
                commands.entity(entity).remove::<Waiting>();
            }
            transform.translation.y = -HALF_H - 16.0;
            continue;
        }
        if leaving {
            // Off the way it came: toward the nearer side edge, or up for
            // things that hang from above.
            let at = screen_centre(enemy.kind, pos.0, camera);
            if matches!(enemy.kind, Kind::Winch | Kind::Plumb) {
                pos.0.y += 80.0 * dt;
            } else {
                pos.0.x += at.x.signum() * 80.0 * dt;
            }
            let at = screen_centre(enemy.kind, pos.0, camera);
            transform.translation.x = at.x.round();
            transform.translation.y = at.y.round();
            if at.y > HALF_H + 40.0 || at.x.abs() > HALF_W + 40.0 {
                commands.entity(entity).try_despawn();
            }
            continue;
        }
        let ctx = Ctx {
            dt,
            camera,
            hero: hero_feet,
            tower: &tower,
        };
        let out: Out = brain.0.step(&mut pos.0, &ctx, e, rules);
        if out.warned {
            let warning = match &brain.0 {
                Brain::Picket(_) => Some(Sfx::PicketFlash),
                Brain::Vent(_) => Some(Sfx::VentGlow),
                Brain::Plumb(_) => Some(Sfx::PlumbFlash),
                Brain::Pitcher(_) => Some(Sfx::PitcherWindUp),
                Brain::Winch(_) => Some(Sfx::WinchShake),
                _ => None,
            };
            if let Some(w) = warning {
                sfx.write(PlaySfx(w));
            }
        }
        if enemy.clock == 0.0 && enemy.kind == Kind::Plumb {
            sfx.write(PlaySfx(Sfx::PlumbCable));
        }
        let at = screen_centre(enemy.kind, pos.0, camera);
        transform.translation.x = at.x.round();
        transform.translation.y = at.y.round();
        enemy.clock += dt;
        if on_screen(at) {
            enemy.on_screen += dt;
        }
        if let Some(atlas) = sprite.texture_atlas.as_mut() {
            atlas.index = look::frame(&brain.0, enemy.clock);
        }
        if let Brain::Picket(p) = &brain.0 {
            sprite.flip_x = p.dir < 0.0;
        }
        // Where its shots leave from, on screen.
        let base = match enemy.kind {
            k if k.stands() => Vec2::new(at.x, screen_y(pos.0.y, camera)),
            _ => at,
        };
        if !out.shots.is_empty() {
            let pattern: Vec<_> = out.shots.iter().map(|s| base + s.offset).collect();
            let feet = feet_height(enemy.kind, pos.0, camera);
            if may_fire(
                rules,
                &pattern,
                enemy.on_screen,
                hero_centre,
                feet,
                hero_feet.map(|h| h.y),
                live,
            ) {
                live += out.shots.len();
                if matches!(brain.0, Brain::Vent(_)) {
                    sfx.write(PlaySfx(Sfx::VentPuff));
                }
                for shot in &out.shots {
                    fire(
                        &mut commands,
                        rules,
                        base + shot.offset,
                        shot.velocity * scale,
                    );
                }
            }
        }
        if let Some((offset, velocity)) = out.canister {
            let feet = feet_height(enemy.kind, pos.0, camera);
            let allowed = hero_feet.is_none_or(|h| feet >= h.y - 0.5)
                && enemy.on_screen >= rules.fire_grace_seconds;
            if allowed {
                let c = spawn_enemy(
                    &mut commands,
                    &sprites,
                    t,
                    Kind::Canister,
                    pos.0 + offset,
                    Brain::Canister(Canister {
                        vel: velocity,
                        gravity: e.pitcher.canister_gravity,
                    }),
                    camera,
                );
                if member {
                    commands.entity(c).insert(StandoffMember);
                }
            }
        }
        if out.shattered {
            commands.entity(entity).try_despawn();
            if enemy.kind == Kind::Canister {
                burst(&mut commands, t, at, scale);
            } else {
                sfx.write(PlaySfx(Sfx::ScrapShatter));
            }
        } else if out.despawn {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Enemy bullet speeds are 15% faster in Stage 2.
fn bullet_scale(session: &Session, t: &ClimberTuning) -> f32 {
    if session.stage == 1 {
        t.stage2_bullet_speed_scale
    } else {
        1.0
    }
}

/// A canister bursts on a ledge: two shots along it, one each way, at the
/// ankle.
fn burst(commands: &mut Commands, t: &ClimberTuning, at: Vec2, scale: f32) {
    let floor = at.y - look::size(Kind::Canister).y / 2.0;
    for side in [-1.0, 1.0] {
        fire(
            commands,
            &t.enemy,
            Vec2::new(at.x, floor + t.enemy.ankle_height),
            Vec2::new(side * t.enemies.pitcher.burst_speed * scale, 0.0),
        );
    }
}

fn fire(commands: &mut Commands, rules: &EnemyRules, at: Vec2, velocity: Vec2) {
    let size = Vec2::new(rules.bullet_size.0, rules.bullet_size.1);
    let hit = Vec2::new(rules.bullet_hitbox.0, rules.bullet_hitbox.1);
    commands
        .spawn((
            projectile(
                Layer::EnemyShot,
                at,
                velocity,
                size,
                palette::ROSE,
                rules.bullet_lifetime,
            ),
            EnemyBullet,
        ))
        .insert((Hitbox::new(hit.x, hit.y), look::bullet(size)));
}

/// Solid ledges and landings stop every shot; thin ledges don't.
fn shots_and_ledges(
    mut commands: Commands,
    stage: Res<StageRun>,
    tower: Res<Tower>,
    shots: Query<(Entity, &Layer, &Transform), With<Velocity>>,
) {
    let camera = stage.cursor.position;
    for (entity, layer, transform) in &shots {
        if !matches!(layer, Layer::EnemyShot | Layer::HeroShot) {
            continue;
        }
        let at = transform.translation.truncate();
        if tower.in_solid(at.x, tower_y(at.y, camera)) {
            commands.entity(entity).try_despawn();
        }
    }
}

/// Scrapfalls drop their pieces; cables follow their owners; the Winch's
/// carriage rides its rail.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn parts(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    sprites: Res<Sprites>,
    mut falls: Query<(Entity, &mut ScrapfallRun)>,
    owners: Query<(&Transform, &EnemyBrain), (Without<Cable>, Without<OnRail>)>,
    mut cables: Query<(Entity, &Cable, &mut Transform, &mut Sprite), Without<OnRail>>,
    mut carriages: Query<(Entity, &OnRail, &mut Transform), Without<Cable>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let t = &tuning.climber;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    for (entity, mut run) in &mut falls {
        let (pieces, done) = run.fall.step(dt, &t.enemies);
        for x in pieces {
            let e = spawn_enemy(
                &mut commands,
                &sprites,
                t,
                Kind::Scrap,
                Vec2::new(x, PLAY_TOP + 8.0),
                Brain::Scrap(Scrap::new(&t.enemies)),
                camera,
            );
            if run.standoff {
                commands.entity(e).insert(StandoffMember);
            }
            scrap_chevron(&mut commands, &sprites, t, x);
            sfx.write(PlaySfx(Sfx::ScrapChevron));
        }
        if done {
            commands.entity(entity).try_despawn();
        }
    }
    for (entity, cable, mut transform, mut sprite) in &mut cables {
        let Ok((owner, brain)) = owners.get(cable.owner) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let bottom = owner.translation.y;
        let top = match &brain.0 {
            Brain::Winch(_) => t.enemies.winch.rail_y,
            _ => HALF_H,
        };
        let len = (top - bottom).max(0.0);
        transform.translation.x = owner.translation.x;
        transform.translation.y = (bottom + top) / 2.0;
        sprite.custom_size = Some(Vec2::new(2.0, len.max(1.0)));
    }
    for (entity, rail, mut transform) in &mut carriages {
        let Ok((_, brain)) = owners.get(rail.hook) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        if let Brain::Winch(w) = &brain.0 {
            transform.translation.x = w.carriage_x.round();
            transform.translation.y = t.enemies.winch.rail_y;
        }
    }
}

/// Damage only gets through an open Vent.
fn vents_open(
    mut commands: Commands,
    vents: Query<(Entity, &EnemyBrain, Has<Immune>), With<Enemy>>,
) {
    for (entity, brain, immune) in &vents {
        if let Brain::Vent(v) = &brain.0 {
            if v.open() && immune {
                commands.entity(entity).try_remove::<Immune>();
            } else if !v.open() && !immune {
                commands.entity(entity).try_insert(Immune);
            }
        }
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

fn spawn_sweeper(
    commands: &mut Commands,
    sprites: &Sprites,
    t: &ClimberTuning,
    pos: Vec2,
    params: &SpawnParams,
    camera: f32,
) {
    let turn = if params.dir.as_deref() == Some("cw") {
        -1.0
    } else {
        1.0
    };
    let hub = Vec2::new(pos.x, tower_y(pos.y, camera));
    let length = t.hazards.sweeper.length;
    let sweeper = commands
        .spawn((
            Sweeper {
                hub,
                angle: 0.0,
                turn,
            },
            sprites.sprite("c4_sweeper_hub", super::CABINET),
            Transform::from_xyz(pos.x, pos.y, 3.0),
            EnemyPart,
            DespawnOnExit(Screen::Playing),
        ))
        .with_children(|c| {
            c.spawn((
                sprites.sprite("c4_sweeper_arm", super::CABINET),
                Transform::from_xyz(length / 2.0, 0.0, -0.1),
                SweeperArm,
            ));
        })
        .id();
    for index in 0..ARM_BOXES {
        commands.spawn((
            ArmBox { sweeper, index },
            Transform::from_xyz(pos.x, pos.y, 3.0),
            Hitbox::new(5.0, 5.0),
            Layer::Hazard,
            EnemyPart,
            DespawnOnExit(Screen::Playing),
        ));
    }
}

/// The arm sprite, a child of the hub.
#[derive(Component)]
struct SweeperArm;

/// Sweepers turn at a steady rate; their arm boxes follow; they hum.
#[allow(clippy::type_complexity)]
fn run_sweepers(
    mut commands: Commands,
    time: Res<Time>,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    mut sweepers: Query<(Entity, &mut Sweeper, &mut Transform, &Children), Without<ArmBox>>,
    mut arms: Query<&mut Transform, (With<SweeperArm>, Without<Sweeper>, Without<ArmBox>)>,
    mut boxes: Query<(Entity, &ArmBox, &mut Transform), (Without<Sweeper>, Without<SweeperArm>)>,
    mut sfx: MessageWriter<PlaySfx>,
    mut hum: Local<f32>,
) {
    let s = &tuning.climber.hazards.sweeper;
    let dt = time.delta_secs();
    let camera = stage.cursor.position;
    *hum -= dt;
    let mut any_on_screen = false;
    for (entity, mut sweeper, mut transform, children) in &mut sweepers {
        sweeper.angle += sweeper.turn * s.turn_rate.to_radians() * dt;
        let y = screen_y(sweeper.hub.y, camera);
        transform.translation.x = sweeper.hub.x;
        transform.translation.y = y.round();
        any_on_screen |= y.abs() < HALF_H;
        if y < -HALF_H - s.length - 16.0 {
            commands.entity(entity).try_despawn();
        }
        for child in children.iter() {
            if let Ok(mut arm) = arms.get_mut(child) {
                let d = Vec2::from_angle(sweeper.angle) * s.length / 2.0;
                arm.translation.x = d.x;
                arm.translation.y = d.y;
                arm.rotation = Quat::from_rotation_z(sweeper.angle);
            }
        }
    }
    for (entity, arm, mut transform) in &mut boxes {
        let Ok((_, sweeper, _, _)) = sweepers.get(arm.sweeper) else {
            commands.entity(entity).try_despawn();
            continue;
        };
        let points = sweeper.arm(s.length, ARM_BOXES);
        let p = points[arm.index];
        transform.translation.x = p.x;
        transform.translation.y = screen_y(p.y, camera);
    }
    if any_on_screen && *hum <= 0.0 {
        *hum = 1.0;
        sfx.write(PlaySfx(Sfx::SweeperHum));
    }
}

/// One arrow column per gust zone, at the edge the wind blows from,
/// blinking while it warns; and the warning and whistle sounds.
#[allow(clippy::type_complexity)]
fn gust_arrows(
    mut commands: Commands,
    tuning: Res<Tuning>,
    stage: Res<StageRun>,
    tower: Res<Tower>,
    sprites: Res<Sprites>,
    time: Res<Time>,
    mut arrows: Query<(&GustArrows, &mut Transform, &mut Visibility)>,
    mut last: Local<Vec<super::tower::GustPhase>>,
    mut sfx: MessageWriter<PlaySfx>,
) {
    let h = &tuning.climber.hazards.gust;
    let camera = stage.cursor.position;
    if arrows.is_empty() && !tower.gusts.is_empty() {
        for (i, g) in tower.gusts.iter().enumerate() {
            let mut s = sprites.sprite("c4_gust_arrow", super::CABINET);
            s.flip_x = g.dir == "left";
            commands.spawn((
                s,
                GustArrows(i),
                Transform::from_xyz(0.0, 0.0, 30.0),
                Visibility::Hidden,
                DespawnOnExit(Screen::Playing),
            ));
        }
        last.clear();
        return;
    }
    last.resize(tower.gusts.len(), super::tower::GustPhase::Calm);
    for (arrow, mut transform, mut visibility) in &mut arrows {
        let g = &tower.gusts[arrow.0];
        let phase = tower.gust_phase(g, h);
        let (lo, hi) = (screen_y(g.from_h, camera), screen_y(g.to_h, camera));
        let shown = hi > -HALF_H && lo < PLAY_TOP;
        if shown && phase != last[arrow.0] {
            match phase {
                super::tower::GustPhase::Warning => {
                    sfx.write(PlaySfx(Sfx::GustArrow));
                }
                super::tower::GustPhase::Blowing => {
                    sfx.write(PlaySfx(Sfx::GustWhistle));
                }
                super::tower::GustPhase::Calm => {}
            }
        }
        last[arrow.0] = phase;
        // Upwind: wind blowing left comes from the right edge.
        let x = if g.dir == "left" {
            tower.face - 6.0
        } else {
            -tower.face + 6.0
        };
        let y = ((lo.max(-HALF_H) + hi.min(PLAY_TOP)) / 2.0).round();
        transform.translation.x = x;
        transform.translation.y = y;
        let blink = ((time.elapsed_secs() * 8.0) as u32).is_multiple_of(2);
        let on = shown
            && match phase {
                super::tower::GustPhase::Warning => blink,
                super::tower::GustPhase::Blowing => true,
                super::tower::GustPhase::Calm => false,
            };
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
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
                let per = tuning.climber.score.formation_bonus_per_enemy;
                score.write(ScoreGained(per * members as u64));
            }
            None => {}
        }
        commands.entity(entity).despawn();
    }
}

/// Hits, explosions, shots sparking off armour, and the Winch's machinery.
#[allow(clippy::too_many_arguments)]
fn enemy_sounds(
    mut hurts: MessageReader<Hurt>,
    mut flashes: MessageReader<Flash>,
    mut destroyed: MessageReader<EnemyDestroyed>,
    armour: Query<(), (With<Enemy>, With<Immune>)>,
    winches: Query<&EnemyBrain>,
    mut last: Local<Option<brain::WinchPhase>>,
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
    let phase = winches.iter().find_map(|b| match &b.0 {
        Brain::Winch(w) => Some(w.phase),
        _ => None,
    });
    let key = phase.map(|p| std::mem::discriminant(&p));
    if key != last.map(|l| std::mem::discriminant(&l)) {
        match phase {
            Some(brain::WinchPhase::Drop) => {
                sfx.write(PlaySfx(Sfx::WinchDrop));
            }
            Some(brain::WinchPhase::Reel) => {
                sfx.write(PlaySfx(Sfx::WinchReel));
            }
            _ => {}
        }
    }
    *last = phase;
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
                    shots_and_ledges,
                    parts,
                    vents_open,
                    run_sweepers,
                    gust_arrows,
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
                .run_if(resource_exists::<Tower>)
                .run_if(stage_live),
        );
    }
}

#[cfg(test)]
mod tests;
