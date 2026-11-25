# Cabinet 4: HIGHWIRE (vertical climber)

The design that issues #24–#27 build. Terms are from [GLOSSARY.md](../../GLOSSARY.md);
the settled design is [DESIGN.md](../DESIGN.md). The Cabinet 3 design,
[3-run-and-gun.md](3-run-and-gun.md), is the model for this doc. The Hero is
still on foot, so where this doc says "as in Cabinet 3", the Cabinet 3 rule
applies unchanged, with its key moved from `run_and_gun.` to `climber.`. The
user can override anything here by commenting on issue #23.

**Where the numbers live.** Every number names its key.

- Keys starting `climber.` go in `assets/tuning.ron`. The proposed section is
  at the end of this doc. Inside an enemy's own section, keys are written
  short: `flit.speed` means `climber.enemies.flit.speed`.
- Scroll speeds, spawn heights and positions, Power-up drops, Restart Points,
  Standoffs, ledges, ladders and gust zones go in the Stage files
  (`assets/stages/4-1.ron` and `4-2.ron`), as in Cabinets 1–3.
- A Formation's size comes from its spawn entry in the Stage file. Its default
  is the enemy's `formation_size` key in `tuning.ron`.
- Sprite sizes are given here so the sprite files match. They live in the
  sprite files, not in tuning.

"HIGHWIRE" is the working in-game name. DESIGN.md leaves final naming until
there is something on screen (#34), so it may change then.

**What the Cabinet 1 playtest taught (#12, #65).** The user found Cabinet 1's
Stages "a little long" and "a little easy", and its firing "a little much",
while the scrolling was "great". This doc takes both lessons from the start:

- **Stages of 2:45 and 2:55,** inside the 2.5–3 minutes Cabinet 1 moved to.
  DESIGN.md still says 3–4 minutes; #67 asks the user whether every Cabinet
  should follow. This Cabinet is unbuilt, so it is the cheapest to plan
  short. If the answer is no, add a beat to each Stage.
- **A weaker gun than Cabinet 3's:** gun 2 fires 10 volleys/s, not 12, so
  typical power is about 17% lower (see [Damage budget](#damage-budget)).

Cabinets 2 and 3 haven't been played yet (#17 and #22). Numbers that follow
them are listed in [Revisit after the playtests](#revisit-after-the-playtests).

## Premise

> The Spire is the Static's mast: everything it has taken, stacked into one
> tower, with the swarm's signal beating out from **the Crown** at its top.
> The Hero climbs it through the night, up the outside of the works, and
> reaches the top at dawn.

- **Stage 1 starts at the tower's foot,** where Cabinet 3's Stage 2 ended.
  The 3 → 4 Transition (#26) shows the Hero starting to climb, with the view
  panning upward. Stage 1 opens on the first landing a little way up, with
  the Underfield's floor just below.
- **Stage 2 ends at the summit,** on the Crown's deck, as the sun rises over
  the cloud sea. The Ending (#28) starts from there. What the Hero does at
  the Crown (for example, switch the beacon off so the Static falls still)
  is #28's to decide. This doc only sets the stage for it.
- Night turns to dawn across the Cabinet: Stage 1 is night, and Stage 2's
  sky brightens as the scroll rises (scenery only; the palette is the same).

## What makes it feel good (the rules behind the numbers)

Cabinet 3's rules carry over: bullets you can read, enemies you can see,
every threat announced, a small hitbox, teach-test-twist, waves that breathe,
power arriving steadily, a lost life that dents rather than wipes out, HP
from time-to-kill targets, an exact jump, and the chest and ankle shot
heights. These change or are added for a climber:

1. **The screen rises on its own, and the bottom edge is the drop.**
   - Unlike Cabinet 3, the camera rises at the Stage's scroll speed whatever
     the Hero does. It never goes down.
   - If the Hero's feet go below the bottom of the play area, it is a lost
     life (`climber.hero.fall_line: -120.0`).
   - This is the climber's pressure, and the issue's starting direction. In
     Cabinet 3 a pit was a place; here the drop follows you.
2. **The climb is always faster than the rise.**
   - The scroll is slow: 14 px/s in Stage 1 and 16 px/s in Stage 2.
   - The Hero climbs ladders at 48 px/s and gains up to 40 px per jump, so
     a player on the route pulls ahead of the bottom edge.
   - Every Stage file carries a **route**, and a Stage check proves that,
     following it with the base moves, the Hero always stays at least 48 px
     above the bottom edge (see [Terrain](#terrain)).
3. **No unseen drops.** Every ledge the route needs is on screen before
   the Hero needs to jump to it, and no jump on the route goes blind. Up is
   ahead, so the Hero is kept low on screen to see it (rule 4).
4. **The Hero is low, the threats are high.**
   - Most threats come from above: falling Scrap, Pitchers on ledges,
     Plumbs dropping on cables and Vents in the tower face.
   - So Up is the main aim in this Cabinet. Up and the Up diagonals are
     where gun 3's spread pays.
5. **The same time to react as Cabinets 1–3.**
   - Straight shots are capped at 60 px/s in Stage 1 and 70 px/s in Stage 2,
     as in Cabinet 3: across a ledge (about 116 px) that is 1.9 s.
   - Shots from above usually travel further than shots along a ledge
     (about 150 px from the upper third to the Hero), so the same caps give
     more time there.
   - Things that fall under gravity are exempt, as in Cabinet 3. They are
     never aimed at the Hero and always telegraphed, and the cap applies to
     their launch speed and to any bullets they burst into.
6. **Nothing knocks the Hero off the route without a warning.** Gusts are
   announced with arrows at the edge they blow from, and walking against a
   gust always holds position. Crumbling ledges shake before they fall.
7. **Ladders are safe to read, not safe to stay on.** On a ladder the Hero
   can only fire straight up and can't dodge sideways quickly, so ladders sit
   where the threats come from above, never where a chest-height shooter
   covers them.

## Screen and camera

- 320×240. The play area is y ∈ [−120, +108] under the shared HUD, as in
  every Cabinet.
- **The tower face** fills x ∈ [−128, +128] (`climber.camera.face_half_width:
  128.0`). Beyond it on both sides is open sky, with the cloud sea far below.
  The face's edges are walls: the Hero can't step off the side of the tower.
- **The camera** is the kit's scroll position, as in Cabinet 2, but upward on
  a tower. It rises at the Stage file's scroll speed and stops only at a
  Standoff.
- **The lid.** The Hero's head can't go above the top of the play area: it
  stops there as if under a ceiling. A fast climber waits at the top for the
  scroll. That is rare on the route, since the route keeps the Hero in the
  lower half.
- **The Stage clears** when the camera reaches the Stage's `length`. The Hero
  then climbs off the top of the screen with no control (1.0 s,
  `climber.hero.exit_seconds: 1.0`) before the Stage Card.

The Stage length in seconds is fixed: the scroll plus the Standoffs' full
timers. Clearing a Standoff early shortens it a little, as in Cabinet 3.
"Time" in the tables below is that fixed time, with every Standoff running to
its timer.

## Palette

One Cabinet palette, shared by both Stages, in the same roles as the other
Cabinets. Foam and Ember are the same hex values as in Cabinets 1–3.

| # | Name | Hex | Used for |
|---|---|---|---|
| 1 | Abyss | `#0B0A24` | backdrop (matches `CABINETS[3].backdrop`), the night sky |
| 2 | Indigo | `#2B2A5C` | far clouds, tower shading, ledge undersides |
| 3 | Girder | `#5E6A85` | the tower face, ledges, ladders, **enemy hull fill** |
| 4 | Mint | `#66FFC2` | accent (matches `CABINETS[3].accent`), Hero visor, Power-up rims |
| 5 | Foam | `#F4F4F0` | Hero suit, **enemy outlines**, bullet cores, Power-up letters, text |
| 6 | Ember | `#FFB43C` | Hero shots, explosions, the Hero's canopy |
| 7 | Rose | `#FF3C8C` | **enemy bullets, enemy weak points and warning marks only** |
| 8 | Peach | `#C8946E` | the dawn in Stage 2: cloud tops, the sky's lower band, lit girders |

Rules that #24 and #25 check in the screenshots:

- **Backgrounds and terrain use only Abyss, Indigo, Girder and Peach** (the
  `#! background` mark). Foam, Mint, Ember and Rose never appear there.
- Peach is muted on purpose, so a dawn sky never reads as Rose bullets.
- **Every enemy has a 1 px Foam outline** around a Girder hull, with a Rose
  eye or core.
- The Hero is the only sprite with a Foam body and a Mint visor.
- Hazards (the Sweeper) are not enemies. They are Girder with a Rose warning
  light and no Foam outline, as in Cabinet 3. Gust arrows and the Scrap
  chevron are warning marks, so they are Rose.

## The Hero, climbing

The same Hero and the same sprite as Cabinet 3 (`c3_hero`), recoloured by the
palette roles, with new frames for climbing (2-frame ladder climb, ladder
hang) and the canopy.

**The same body as Cabinet 3.** These rows use Cabinet 3's values under
`climber.` keys, so either Cabinet can be retuned alone:

- hitboxes `climber.hero.hitbox: (6.0, 14.0)` and `crouch_hitbox: (6.0, 8.0)`;
- terrain boxes `climber.hero.body: (10.0, 20.0)` and `crouch_body: (10.0, 12.0)`;
- run speed `climber.hero.run_speed: 72.0` and air control
  `climber.hero.air_speed: 72.0`;
- the whole jump: `climber.jump.*`, with 48 px high and 52 px across;
- `climber.hero.respawn_invulnerable_seconds: 2.0` and
  `climber.hero.hit_stop_seconds: 0.15`;
- crouching, Down + A through thin ledges, facing and aim: all as in
  Cabinet 3.

What's new:

| Property | Value | Key |
|---|---|---|
| Respawn | The Hero drops in under a canopy from the top edge, at the Restart Point's x, onto its landing. It falls at 60 px/s with no control until it lands. | `climber.hero.respawn_fall_speed: 60.0` |
| Fall line | Feet below y = −120 is a lost life. | `climber.hero.fall_line: -120.0` |

The canopy drop is a callback to the 2 → 3 Transition, and it gives the
player a moment to read the landing.

### Ladders (Up and Down)

| Property | Value | Key |
|---|---|---|
| Climb speed | 48 px/s up or down | `climber.ladder.climb_speed: 48.0` |
| Grab | Up or Down while the Hero's centre is within 6 px of the ladder's centre. The Hero snaps onto the ladder. | `climber.ladder.grab_window: 6.0` |
| Jump off | A with left or right: a jump at 60% of the take-off speed, that way. A with no direction lets go and falls. | `climber.ladder.jump_speed_fraction: 0.6` |
| At the top | Climbing past the ladder's top puts the Hero on the ledge there. | — |
| Fire | **Straight up only** while on a ladder, with the full gun. | — |

- A ladder is 8 px wide (one tile) and runs between two ledge heights. It
  passes through thin ledges.
- Grabbing a ladder in the air works too: Up while overlapping catches it.
  That is how a missed jump can still be saved.
- On a ladder the Hero is a normal target: the same hitbox, no crouch.

### B: fire

As in Cabinet 3 (`climber.shot.*`): 6×2 Ember dashes at 300 px/s, 8
directions in the air, and the muzzle at chest height. Holding Up aims
straight up, which is this Cabinet's main aim.

**One change from Cabinet 3, after the Cabinet 1 playtest (#65):** gun 1
fires 8 volleys/s (`climber.shot.fire_interval: 0.125`), as in Cabinet 3, but
gun 2 and up fire 10 volleys/s (`climber.shot.fire_interval_fast: 0.1`), not
12. Cabinet 1's firing felt "a little much" at 10 volleys/s with a wide
spread, and this gun reaches the same rate with a narrower one.

## Power-ups

### How they're earned

As in Cabinet 3: Formations and Standoffs drop the Power-up their spawn
entry names, or a Formation pays a bonus. A Power-up appears where the last
member died, falls onto the ledge below (or hovers at the bottom of the play
area if there is none), and is collected by touching it. It looks and bobs as
in Cabinet 3 (`climber.power_up.size`, `bob_amplitude`, `bob_period`). It
scrolls off the bottom with the tower if left. A maxed Power-up gives a
1,000-point medal (`climber.score.maxed_power_up: 1000`).

They belong to this Cabinet and carry from Stage 1 to Stage 2 (DESIGN.md).
This is the last Cabinet, so they end with the Run.

### The five Power-ups

| Letter | Name | Effect | Max | Key(s) |
|---|---|---|---|---|
| **G** | Gun | Raises the gun level by 1. Levels as in Cabinet 3. | level 4 | `climber.power_up.gun_max: 4` |
| **W** | Ward | Absorbs one hit, then 1.0 s of invulnerability. It does **not** save a fall below the bottom edge. | 1 | `climber.power_up.ward_invulnerable_seconds: 1.0` |
| **J** | Jet | A second jump in mid-air at 70% of the take-off speed. Recharged on landing or grabbing a ladder. | 1 | `climber.power_up.jet_speed_fraction: 0.7` |
| **C** | Canopy | Holding A while falling opens a canopy: fall speed is capped at 50 px/s, with full air control. With Jet, the first A press in the air is the Jet, and the canopy opens only once the Jet is spent. | 1 | `climber.power_up.canopy_fall_speed: 50.0` |
| **M** | Mag boots | Gusts don't move the Hero while standing on a ledge, and crumbling ledges take twice as long to fall. | 1 | `climber.power_up.mag_crumble_factor: 2.0` |

Why these five:

- **G and W are the Run's constants,** as in every Cabinet.
- **Jet comes back from Cabinet 3** because it fits a climb even better.
  Every move on the route still uses only the base jump.
- **Canopy is the climber's safety net,** and a callback to the parachute.
  It turns a missed jump into a slow drift onto a lower ledge, as long as
  the bottom edge hasn't caught up. It only opens while falling, so it never
  changes the jump's rise.
- **Mag boots answer Stage 2's hazards:** gusts and crumbling ledges.
- **No Flare and no Hustle.** Hustle can't speed up a forced scroll, and the
  threats here are above, where the gun already aims. The Flare's job (area
  damage) goes to gun 3's spread pointed up.

### Damage budget

| Power | Narrow DPS | Large DPS | Cabinet 3, for comparison |
|---|---|---|---|
| Gun 1 | 8 | 8 | 8 / 8 |
| Gun 2 | 10 | 10 | 12 / 12 |
| Gun 3 | 10 | 30 | 12 / 36 |
| Gun 4 | 20 | 40 | 24 / 48 |

Gun 2 is a 25% step over gun 1, gun 3 triples damage on large targets, and
gun 4 doubles narrow damage, so every G is still worth taking.

**"Large" here is measured aiming up:** a target 24 px or more **wide**
within about 50 px above the Hero, where the ±12° shots spread to ±10.5 px
around the centre shot and all three connect. Further up, the angled shots
spread wider and miss narrow targets, but they still hit things to either
side: in a climber, gun 3 is how you cover a ledge above you.

Expected power ("perfect" means every Formation cleared and no life lost;
"typical" means one or two missed and one life lost). This Cabinet starts
from nothing again, so the first drop comes in the first 10 s.

| When | Perfect | Typical |
|---|---|---|
| Stage 1, 0:46 (first Picket) | Gun 2 + Canopy: 10 | Gun 2: 10 |
| Stage 1, 1:12 (first Standoff) | Gun 3 + Canopy: narrow 10, large 30 | Gun 2: 10 |
| Stage 2, 0:00 | Gun 4 + Canopy, Jet, Ward: narrow 20, large 40 | Gun 3: narrow 10, large 30 |
| Stage 2, 2:13 (the Winch) | Gun 4 + all five | Gun 3 + Mag boots |

Time-to-kill targets by role, as in Cabinets 1–3:

| Role | Enemies | Target | HP | Check |
|---|---|---|---|---|
| Popcorn | Flit, Limpet, Scrap, Mote | under 0.3 s even at gun 1 | 2, 2, 2, 1 | 0.25 s at gun 1 |
| Small | Plumb | about 0.5 s at typical power | 6 | 0.6 s at narrow 10 |
| Timing lock | Vent | dies inside one 1.4 s opening at typical power | 12 | 1.2 s at narrow 10; two openings at gun 1 |
| Gunner | Picket, Pitcher | 1.5–2 s, so it gets a burst off | 16, 20 | 1.6 s and 2.0 s at narrow 10 |
| Set piece | Winch | about 12 s at typical, inside its 22 s Standoff | 120 (hook core) | 12 s at narrow 10, 6 s at narrow 20 |

### Losing a life

As in Cabinet 3: gun −1 (never below 1) and the Ward is lost. Jet, Canopy
and Mag boots are kept. The camera rewinds to the latest Restart Point,
which replays its Power-ups, and the Hero drops in under the canopy.

A fall below the bottom edge plays the hit-stop at the edge, with no
explosion, as Cabinet 3's pits do.

### Saved state

`Session.power_ups` holds this JSON shape, saved and cleared as in Cabinets
1–3:

```json
{ "gun": 1, "ward": false, "jet": false, "canopy": false, "mag": false }
```

Practice starts from this default.

## Terrain

The tower face is built from ledges and ladders on the 8 px tile grid
(`climber.terrain.tile_size: 8.0`). Heights are scroll positions: a piece at
height `h` reaches the top edge of the screen when the camera reaches `h`,
as spawns do in every Cabinet.

```ron
ledges: [
    (h: 0.0, from_x: -128.0, to_x: 128.0, kind: "landing"),  // full width, solid
    (h: 40.0, from_x: -128.0, to_x: -64.0, kind: "thin"),
    (h: 72.0, from_x: -24.0, to_x: 40.0, kind: "crumble"),
],
ladders: [(x: 96.0, from_h: 0.0, to_h: 80.0)],
gusts: [(from_h: 900.0, to_h: 1200.0, dir: "left", period: 6.0, offset: 0.0)],
route: [(x: -100.0, h: 0.0), (x: -96.0, h: 40.0), (x: 96.0, h: 80.0)],
```

- **Ledges** are 8 px thick.
  - A `landing` is solid and full width. Landings are where Restart Points
    and Standoffs go.
  - A `thin` ledge is one-way: jump up through it, land on it from above,
    and drop through with Down + A.
  - A `solid` ledge blocks from below too; the Hero bumps its head on it.
  - A `crumble` ledge works as in Cabinet 3: it falls 0.5 s after the Hero
    lands on it (`climber.terrain.crumble_seconds: 0.5`) and is back 3.0 s
    later (`climber.terrain.crumble_respawn_seconds: 3.0`), and at a rewind.
    Stage 2 only.
- **The route** is a list of footholds that a careful player follows. It is
  only for checks and the autopilot. Nothing in the game shows it.

**The fairness checks** (`stage_checks.rs`), using the jump keys and the
base moves only:

- Each step of the route is one base move: a walk along a ledge, a jump of
  at most 40 px up (`climber.terrain.max_rise: 40.0`, 83% of the jump) and
  40 px across (`climber.terrain.max_gap: 40.0`), a drop to a lower ledge,
  or a ladder.
- **Keeping ahead.** The checks time the route at base speed (ladders at
  48 px/s, a jump at its airtime, walks at 72 px/s, plus 0.3 s per jump to
  line up, `climber.terrain.jump_setup_seconds: 0.3`). At every point the
  Hero's feet stay at least 48 px above the bottom edge
  (`climber.terrain.drop_margin: 48.0`).
- Every route foothold is on screen, at least 16 px below the top edge
  (`climber.terrain.lookahead: 16.0`), before the Hero has to jump to it
  (rule 3).
- A Restart Point stands on a landing, and no gust or Sweeper reaches the
  first 64 px of climb above it (`climber.terrain.restart_clear_rise: 64.0`).
- No ladder is in line with a chest-height shooter's ledge (rule 7).

The clearances and timings above are **Stage-check constants**: only
`stage_checks.rs` reads them. They are keyed under `terrain` so the doc, the
checks and the tuning file agree, as in Cabinet 3.

## Standoffs

As in Cabinet 3 (`kind: "standoff"`, `max_seconds`, `drop`, `waves`). In a
climber the scroll stops at a **landing**, so the bottom edge stops chasing
and the Hero fights on a full-width floor. Waves come in from both sides and
from above. Clearing one early pays 50 points per whole second left
(`climber.score.standoff_time_bonus: 50`). No Restart Point is inside one.

## Enemies

All are original machines of the Static. "Aimed" means aimed at the Hero's
position at the moment of firing, with no leading.

- Enemy bullets are 5×5 Rose dots with a Foam core
  (`climber.enemy.bullet_size: (5.0, 5.0)`, hitbox `(2.0, 2.0)`), removed
  after 10 s (`climber.enemy.bullet_lifetime: 10.0`) or when they hit a
  solid ledge or a landing. Thin ledges don't stop them.
- An enemy's hitbox is its sprite shrunk by 1 px on each side
  (`climber.enemy.hitbox_inset: 1.0`). Touching an enemy is a hit; there is
  no stomping, as in Cabinet 3.
- Chest height and ankle height are 14 and 4 px above the shooter's feet
  (`climber.enemy.chest_height: 14.0`, `ankle_height: 4.0`), as in Cabinet 3.
- In Stage 2 every enemy bullet speed is multiplied by 1.15
  (`climber.stage2_bullet_speed_scale: 1.15`). The fastest straight shot
  becomes 69 px/s, under rule 5's 70.

General fairness rules, as in Cabinet 3:

- No enemy fires in its first 0.5 s on screen
  (`climber.enemy.fire_grace_seconds: 0.5`).
- No bullet is created within 40 px of the Hero
  (`climber.enemy.min_fire_distance: 40.0`), measured where it appears. A
  pattern that would break it is skipped whole.
- At most 32 enemy bullets at once (`climber.enemy.max_bullets: 32`).
- Nothing spawns on screen within 48 px of the Hero
  (`climber.enemy.no_spawn_radius: 48.0`).
- **Nothing fires from below the Hero.** An enemy whose feet are lower than
  the Hero's never fires: it has been passed. That is this Cabinet's "no shot
  from behind".
- An enemy that enters from the bottom edge shows a chevron there first
  (`climber.enemy.below_warning_seconds: 0.75`). Only Flits do.

| # | Name | Kind | Stage | Sprite | HP | Score | Default Formation |
|---|---|---|---|---|---|---|---|
| 1 | Flit | flyer | 1, 2 | 10×8 | 2 | 100 | trio |
| 2 | Limpet | ledge crawler | 1, 2 | 10×6 | 2 | 100 | pair |
| 3 | Scrap | falling | 1, 2 | 10×10 | 2 | 50 | single |
| 4 | Picket | walker | 1, 2 | 12×18 | 16 | 500 | single |
| 5 | Vent | fixed | 1, 2 | 16×12 | 12 | 500 | single |
| 6 | Plumb | on a cable | 1, 2 | 10×10 | 6 | 300 | single |
| 7 | Pitcher | walker | 1, 2 | 14×14 | 20 | 400 | single |
| 8 | Mote | flyer | 2 | 4×4 | 1 | 50 | cloud of 8 |
| 9 | Winch | set piece | 2 | carriage 32×16, hook 16×16 | 120 (hook core) | 5,000 | single |

HP and score keys: `<name>.hp`, `<name>.score`. Default Formation size:
`<name>.formation_size` (1 when not listed).

The Mote is the Static itself, shared with Cabinets 1–3. Every other enemy is
new to this Cabinet.

### 1. Flit: the crossing flyer

A small winged sliver. Trios (`flit.formation_size: 3`), 0.4 s apart
(`flit.spacing_seconds: 0.4`).

- Each enters from the side edge its spawn entry names (`side`) at the
  spawn's height, and crosses the screen at 60 px/s (`flit.speed: 60.0`)
  in a sine wave 12 px high (`flit.wave_amplitude: 12.0`) every 1.2 s
  (`flit.wave_period: 1.2`).
- A trio with `"from": "below"` instead rises from the bottom edge at the
  spawn's x, after the chevron, and curves out through the nearer side.
- It doesn't fire. It rams. Crossing flyers at chest height are crouched
  under or shot; rising ones are the reason to look down sometimes.

### 2. Limpet: the ledge crawler

A shell-backed crawler stuck to a ledge. Pairs (`limpet.formation_size: 2`),
starting at opposite ends of their ledge.

- It crawls **all the way round its ledge**: along the top, round the end,
  along the underside, and back, at 24 px/s (`limpet.speed: 24.0`).
- On the top it is in the way of a landing. On the underside it is in the
  way of a jump up through a thin ledge.
- It doesn't fire. The lesson: look at a ledge before you jump to it.

### 3. Scrap: the falling junk

Loose plating that the Static sheds from higher up.

- A Rose chevron blinks at the top edge above where it will fall, for 0.75 s
  (`scrap.warning_seconds: 0.75`).
- It then falls straight down at 120 px/s (`scrap.speed: 120.0`), through
  thin ledges, and shatters on a solid ledge or a landing.
- It can be shot (2 HP, 50 points), so an Up-aiming player clears the way.
- Scrap is spawned singly or in a `scrapfall` entry: one piece every 0.8 s
  (`scrap.fall_interval: 0.8`) at random x within the entry's span, for its
  `seconds`. It never lands within 24 px of where the previous piece did
  (`scrap.min_spacing: 24.0`), so there is always a gap.

### 4. Picket: the ledge guard (chest shots)

An upright gun post that paces its ledge.

- It walks along its ledge at 20 px/s (`picket.speed: 20.0`) and turns at
  the ends.
- When the Hero's feet are within 16 px of its ledge's height
  (`picket.level_window: 16.0`) on the side it faces, it stops, flashes for
  0.35 s (`picket.telegraph_seconds: 0.35`) and fires 2 shots at **chest
  height** (`picket.burst: 2`), 0.25 s apart (`picket.burst_interval: 0.25`),
  at 60 px/s (`picket.shot_speed: 60.0`). It then waits 2.0 s
  (`picket.cooldown_seconds: 2.0`).
- It only fires along its own level, so the answer is to crouch under the
  burst or to arrive from below and shoot up first.

### 5. Vent: the timing lock

A grille in the tower face.

- It is closed for 2.2 s (`vent.closed_seconds: 2.2`), `Immune`.
- It glows for 0.3 s (`vent.telegraph_seconds: 0.3`), then opens for 1.4 s
  (`vent.open_seconds: 1.4`), puffing out a fan of 3 bullets **downward**,
  ±30° from straight down (`vent.fan_count: 3`, `vent.fan_degrees: 30.0`),
  at 55 px/s (`vent.shot_speed: 55.0`).
- It only takes damage while open. Its cycle starts when it is fully on
  screen.
- It sits in the face between ledges, above the route. Aim up or diagonal up
  while it is open.

### 6. Plumb: the drop on a line

A weighted eye on a cable from above.

- It drops from the top edge at its spawn's x at 60 px/s
  (`plumb.drop_speed: 60.0`), its cable trailing behind, and stops when it is
  level with the Hero's chest.
- It hangs for 2.0 s (`plumb.hang_seconds: 2.0`). After 0.3 s of flashing
  (`plumb.telegraph_seconds: 0.3`) it fires one shot left and one right at
  **chest height** at 60 px/s (`plumb.shot_speed: 60.0`), skipping a side
  where the Hero is within 40 px.
- It then reels back up at 40 px/s (`plumb.reel_speed: 40.0`) and leaves.
- Its spawn x is always at least 48 px from the route's line at that height
  (a Stage check), so it never drops onto the Hero.

### 7. Pitcher: the thrower above

A squat machine on a ledge above the Hero, with one long arm.

- It throws only while the Hero is below it and within 120 px across
  (`pitcher.range: 120.0`).
- Every 2.4 s (`pitcher.fire_interval: 2.4`) it winds up for 0.4 s
  (`pitcher.telegraph_seconds: 0.4`) and lobs a canister (8×8,
  `pitcher.canister_size: (8.0, 8.0)`) in an arc that lands where the Hero
  stood. It is launched at up to 60 px/s (`pitcher.launch_speed: 60.0`)
  under 300 px/s² of gravity (`pitcher.canister_gravity: 300.0`).
- On landing it bursts into 2 bullets along that ledge, one each way, at
  **ankle height** at 50 px/s (`pitcher.burst_speed: 50.0`).
- Thrown canisters fall through thin ledges and burst on the first solid
  ledge or landing. Shooting a canister (1 HP) bursts it harmlessly in the
  air.

### 8. Mote: the swarm (Stage 2)

The Static itself, shared with Cabinets 1–3, with its own section here, the
same values as Cabinet 3's: a cloud of 8 (`mote.formation_size: 8`) within
32 px (`mote.cloud_radius: 32.0`), homing at 55 px/s (`mote.speed: 55.0`),
turning at 90°/s (`mote.turn_rate: 90.0`), and dissolving after 8 s
(`mote.lifetime: 8.0`). It appears at the top edge. Near the Crown, the
Static is thick.

### 9. Winch: the set piece (Stage 2)

A crane carriage on a rail across the top of a landing, with a heavy hook on
a cable. It is not a boss (DESIGN.md): it is a Standoff's main wave, and the
timer runs under it, as Cabinet 3's Strider does.

- **Parts:** the carriage (Girder, `Immune`, hurts on touch) runs on a rail
  at y = +80 (`winch.rail_y: 80.0`). The hook is 16×16 with a Rose core: the
  only part that takes damage, 120 HP (`winch.hp: 120`).
- **Its cycle:**
  1. The carriage slides to above the Hero at 40 px/s (`winch.speed: 40.0`),
     for up to 1.4 s.
  2. The cable shakes and the core glows for 0.6 s
     (`winch.telegraph_seconds: 0.6`).
  3. The hook drops at 200 px/s (`winch.drop_speed: 200.0`) to the landing.
     Being under it is a hit. Its landing throws 2 bullets along the floor,
     one each way, at **ankle height** at 60 px/s (`winch.burst_speed:
     60.0`). Step aside from the drop, then jump the ankle shots.
  4. It rests on the floor for 1.2 s (`winch.rest_seconds: 1.2`), its core
     at chest height: the window for straight shots.
  5. It reels up at 80 px/s (`winch.reel_speed: 80.0`). While hanging it can
     still be shot by aiming up.
- **One answer at a time.** The drop and its ankle shots need different
  answers, in order: step aside first, then jump. The burst follows the
  fairness rule, so it never fires at a Hero within 40 px of the hook. Its
  shots therefore take at least 0.67 s to arrive after the drop, about a
  full jump's airtime. Nothing else in the Winch's pattern overlaps.
- A full cycle takes about 5–6 s, depending on how far the carriage slides
  and how high the hook hangs. The steps follow each other, so they never
  overlap.
- **Time to kill:** the hook core is 16 px wide, a narrow target. That is
  10 DPS at gun 2–3 (12 s) and 20 at gun 4 (6 s), inside the 22 s timer. At
  the timer, the Winch reels up and its carriage runs off the rail: no
  points, no drop.

### Hazards

None of these can be destroyed.

| Hazard | Stage | What it does | Keys |
|---|---|---|---|
| **The bottom edge** | 1, 2 | Falling below it is a lost life (rule 1). | `climber.hero.fall_line` |
| **Gust** | 2 | In a gust zone, wind blows sideways for 2.0 s (`gust.blow_seconds: 2.0`) every `period` seconds, pushing the Hero at 36 px/s (`gust.push_speed: 36.0`) on ledges, ladders and in the air. Rose arrows blink at the upwind edge for 0.8 s first (`gust.warning_seconds: 0.8`). Walking against it still moves the Hero forward at 36 px/s. Mag boots stop the push on a ledge. | `climber.hazards.gust.*` |
| **Crumbling ledge** | 2 | As in Cabinet 3 (see Terrain). Mag boots double its 0.5 s. | `climber.terrain.crumble_*` |
| **Sweeper** | 2 | An arm 48 px long (`sweeper.length: 48.0`) on a hub in the tower face, turning at 60°/s (`sweeper.turn_rate: 60.0`), in the direction its spawn entry gives. Touching the arm is a hit. A Rose light at its tip. It turns at a steady rate, so its timing can be read from the first look. | `climber.hazards.sweeper.*` |

Gust zones come from the Stage file (`gusts`, with `dir`, `period` and
`offset`). An authoring rule, checked by Stage checks: no gust zone covers a
crumbling ledge narrower than 32 px, so a gust and a crumble never combine
into a forced fall.

## Stage 1: The Footings

- **Theme:** the lower Spire at night.
  - Far layer: the cloud sea below and stars above (Indigo on Abyss). The
    Underfield's lights shrink as the Stage climbs.
  - The tower face: Girder plating with Indigo seams, catwalks for ledges,
    and service ladders.
- **Scroll:** 14 px/s throughout (Stage file).
- **Length:** 1,974 px of climb. That is 141 s of rising plus two Standoffs
  of 12 s each: **2:45**.
- **Restart Points:** 0:00 (height 0), 0:42 (588), 1:25 (1,016, just after
  the first Standoff), 2:06 (1,592).
- **Power-ups, in order:** G C G J W G.

| Time | Height | Beat | What happens |
|---|---|---|---|
| 0:00 | 0 | calm | Restart Point. The first landing. The music starts. |
| 0:04 | 56 | teach | A Flit trio crosses from the right → **G** (gun 2). |
| 0:13 | 176 | teach | The first ladder and thin ledges, with a Limpet pair crawling a ledge above. |
| 0:19 | 272 | test | Limpets on two ledges and a Flit trio from the left → Formation bonus. |
| 0:28 | 392 | teach | The first Scrap, one piece, then a Flit trio → **C**. |
| 0:38 | 532 | breathe | A short ladder run with nothing on it. |
| 0:42 | 588 | calm | **Restart Point.** A landing. |
| 0:43 | 600 | teach | One Picket on a catwalk. The crouch lesson, again. |
| 0:46 | 648 | test | Pickets on two levels, with a ladder between them that neither covers → **G** (gun 3). |
| 1:03 | 880 | teach | One Vent in the face above the route. Aim up. |
| 1:12 | 1,008 | Standoff | **The First Landing** (12 s): Flit trios from both sides, a Plumb from above, then a Limpet pair. Clear: Formation bonus and the time bonus. |
| 1:24 | 1,008 | — | The scroll resumes. |
| 1:25 | 1,016 | calm | **Restart Point.** |
| 1:27 | 1,048 | teach | One Plumb, then a scrapfall around a second → **J**. |
| 1:37 | 1,184 | teach | One Pitcher on a ledge above, with Limpets below it → **W**. |
| 1:47 | 1,336 | twist | Vents in a column over a ladder run, with Flits rising from below (chevrons) → Formation bonus. |
| 2:02 | 1,540 | breathe | A landing. |
| 2:06 | 1,592 | calm | **Restart Point.** |
| 2:06 | 1,600 | test | A Pitcher pair and a Picket on the ledge between them. |
| 2:22 | 1,820 | Standoff | **The Lamp Deck** (12 s): Flits, Plumbs and a scrapfall around a wide landing under the tower's warning lamps → **G** (gun 4). |
| 2:35 | 1,832 | run-out | The scroll resumes. A clear ladder run, no enemies. The first grey of dawn shows at the horizon. |
| 2:45 | 1,974 | end | The scroll ends at a landing: Stage clear (bonuses paid). |

A perfect player leaves Stage 1 with gun 4, Canopy, Jet and the Ward. A
typical player finishes at gun 3 with one or two of the others.

## Stage 2: The Crown

- **Theme:** the upper Spire at dawn.
  - The sky brightens as the scroll rises: Peach cloud tops and a Peach band
    low in the sky, growing through the Stage. Stars fade (fewer star
    sprites higher up).
  - The face narrows into open girders, with gaps to the sky behind, cable
    runs, and the Crown's beacon light flashing above in the last minute.
- **What's new:** gusts, crumbling ledges, Sweepers, Motes and the Winch.
  Bullets 15% faster. Breathers of about 4 s.
- **Scroll:** 16 px/s throughout (Stage file).
- **Length:** 2,256 px of climb. That is 141 s of rising plus two Standoffs
  of 12 s and 22 s: **2:55**.
- **Restart Points:** 0:00 (0), 0:42 (672), 1:17 (1,040, just after the
  first Standoff), 1:58 (1,696).
- **Power-ups, in order:** M G W C J W G. For a perfect player most are
  medals; M is new.

| Time | Height | Beat | What happens |
|---|---|---|---|
| 0:00 | 0 | calm | Restart Point. A landing above the night's last cloud layer. |
| 0:04 | 64 | teach | A Flit trio → **M**. |
| 0:06 | 96 | teach | The first gust zone, nothing else. |
| 0:20 | 328 | test | Crumbling ledges (wide ones, no gusts) with a Limpet pair → **G**. |
| 0:31 | 496 | teach | A Mote cloud over a landing → **W**. |
| 0:38 | 608 | breathe | Short. |
| 0:42 | 672 | calm | **Restart Point.** |
| 0:43 | 688 | teach | One Sweeper, alone, beside a ladder. |
| 0:48 | 776 | test | A Sweeper with Pickets on the ledges either side → Formation bonus. |
| 1:04 | 1,024 | Standoff | **The Wind Deck** (12 s): gusts over the landing (from 64 px above it, so they catch a jump), Flit trios from both sides, then two Plumbs. Clear: Formation bonus and the time bonus. |
| 1:16 | 1,024 | — | The scroll resumes. |
| 1:17 | 1,040 | calm | **Restart Point.** |
| 1:20 | 1,088 | test | Pitchers over a run of crumbling ledges → **C**. |
| 1:32 | 1,272 | twist | A gust zone during a scrapfall, with Vents in the face → **J**. |
| 1:42 | 1,440 | test | A Mote cloud with Plumbs dropping. |
| 1:54 | 1,632 | breathe | A landing. The Crown's beacon comes into view. |
| 1:58 | 1,696 | calm | **Restart Point.** |
| 1:58 | 1,704 | test | The gauntlet: a Sweeper, a Picket pair, a Pitcher and a gust zone → **W**. |
| 2:09 | 1,872 | breathe | Short, on a ladder run. |
| 2:13 | 1,936 | Standoff | **The Winch** (22 s, at exactly 20 s of scroll left): the Winch on the Crown's lower deck, with a Flit trio from each side at 7 s and 14 s → **G**. |
| 2:35 | 1,936 | run-out | The scroll resumes. A last ladder up (20 s, 320 px), no enemies. The sun clears the cloud sea. |
| 2:55 | 2,256 | end | The scroll ends on the Crown's deck: Stage clear (bonuses paid). The Ending (#28) follows. |

## Scoring

| Event | Points | Key |
|---|---|---|
| Each enemy | its score (see the enemy table) | `climber.enemies.<name>.score` |
| Formation cleared with no Power-up | 100 × members | `climber.score.formation_bonus_per_enemy: 100` |
| Power-up picked up when already maxed | 1,000 | `climber.score.maxed_power_up: 1000` |
| Standoff cleared early | 50 per whole second left | `climber.score.standoff_time_bonus: 50` |
| Stage clear | 3,000 | `climber.score.stage_clear: 3000` |
| No-miss bonus | 5,000 | `climber.score.no_miss: 5000` |

The bonuses are paid as in Cabinet 1.

**Rough targets** (#25 measures the real ones): about 20–28k for a perfect
Stage 1 and 28–36k for Stage 2, smaller than the other Cabinets' because the
Stages are shorter. The Run's final score then lands around 230–280k for a
strong player, which #28's score breakdown and the High Score
Table's seeded entries should be checked against.

**Measured from the Stage files (#25),** counting every enemy, the
Formation bonuses, the Stage clear and the no-miss bonus, but no Standoff
time bonus: about **22.3k** for a perfect Stage 1 and **30.4k** for Stage 2
(6,000 of it maxed medals, and the Winch's 5,000). Killing the Winch at
gun 4 adds about 800 more in time bonus. Both are inside the targets.

## Sound and music (for #25)

- **Stage 1 track:** about 112 BPM, minor key, night: a steady climbing
  arpeggio over a low drone, with a high, sparse lead. The slowest track in
  the Run, for the slowest scroll.
- **Stage 2 track:** about 144 BPM, starting in the minor and moving to the
  relative major as the dawn comes in. In the last 20 s it builds to a held
  chord that hands over to the Ending's music (#28).
- **Effects to cover:**
  - Hero: ladder rungs (soft, every frame of the climb), jump, Jet puff,
    canopy open and its flutter, land, shot, bolt.
  - Power-ups: each letter's chirp, and the maxed medal.
  - Enemy hits: spark (also for `Immune` parts), small and large explosions.
  - Warnings: the Scrap chevron, the Flit below-chevron, the gust arrows,
    the Picket and Plumb flashes, the Vent glow, the Pitcher's wind-up, the
    Winch's cable shake.
  - Machinery: Scrap shattering, the Vent's puff, the Plumb's cable, the
    Sweeper's hum, the gust's whistle, crumbling ledges, the Winch's drop
    and reel.
  - The Standoff's lock-in and release stings, as in Cabinet 3.
  - The fall below the edge (a falling whistle, then the life-lost sound),
    Ward break, lost life, and the "STAGE CLEAR" sting.

## Building it (for #24)

Cabinet 3's code is the model. Reuse it where the Hero is the same, and write
new code only where Cabinet 4 differs.

- **Where it lives:** `src/cabinets/climber/`, gated by `cabinet_is(3)`,
  laid out like `run_and_gun/`. `tuning.rs` adds one `climber` field to
  `Tuning`.
- **From Cabinet 3, shared rather than copied:**
  - The platform bodies (`kit/platform.rs`, from #19): gravity, ledges,
    one-way ledges, crumbling, and the jump numbers. **Add ladders there**:
    a body can be on a ladder, with climb movement and the grab window.
  - The Hero's run, jump, crouch and aim. Move them from `run_and_gun/` into
    a shared on-foot module (for example `cabinets/on_foot/`), with the keys
    passed in, so the two Cabinets can be tuned apart. Update Cabinet 3 to
    use it in the same change.
  - The Standoff runner, the gun levels and the Ward, the same way.
  - The Mote brain.
- **The kit, unchanged:** a forced vertical scroll is the kit's default
  timeline (Cabinet 2 already scrolls upward); Cabinet 3's "driven scroll"
  resource is simply not used. Restart Point rewinds, `Health`, `Hurt`,
  `Immune`, `EnemyDestroyed`, `Shield`, `HitStop` / `stage_live`, Formation
  tracking and the fairness gate (with "behind" meaning "below").
- **Write new:** the ledge-and-ladder terrain format and its drawing, the
  fall line, the canopy respawn, Canopy and Mag boots, gusts, the Sweeper,
  and the eight new brains (Mote is shared), as pure state machines with
  fixed-step unit tests. Cabinet 4's own `PowerUps` type, with the JSON
  shape above.
- **Where things are:** enemies on ledges and fixed in the face live in
  tower space (height + screen y), recomputed each frame from the camera, as
  Cabinet 3's walkers are. Flyers, Plumbs and falling Scrap are in screen
  space.
- **Gallery:** `assets/stages/4-gallery.ron`, shown with `?gallery`, with a
  landing, a ladder, each ledge kind, a gust zone and a Sweeper.
- **The unattended run (for #25).** As in Cabinet 3, with `debug_keys` on,
  `?autopilot` plays the Stage with the invincible Hero, but here it
  **follows the Stage file's `route`** with the base moves and the real
  physics, firing up.
  - A fall below the edge by an invincible Hero puts it back on the route's
    next foothold and counts a **rescue**. The Stage test fails if there are
    any. That makes the run a second check of rule 2, beside the timed
    route check.
  - `scripts/stages-web.sh 4` plays both Stages. The clear comes at 2:45 and
    2:55, a little sooner if the autopilot clears a Standoff early.

### Gameplay decisions (#24)

Where this doc left a choice open, #24 made these calls. Where any of them
change a rule above, the rule above now reads with this note.

- **Where the code is.** `src/cabinets/climber/`, as planned. Ladders,
  solid ledges (stood on, never dropped through, and bumping a head from
  below) and the climb live in the kit's `platform.rs`, behind `TileQuery`
  methods that default to none, so Cabinet 3 is unchanged. The gun levels,
  the aim, the muzzle and the Standoff clock moved to a shared
  `cabinets/on_foot/` module, and Cabinet 3 uses it too. Each Cabinet keeps
  its own tuning section.
- **A gust doesn't move the Hero on a ladder.** A sideways push on a
  one-tile ladder could only tear the Hero off it, which breaks rule 6 and
  makes a ladder in a gust zone a trap. Gusts push on ledges (unless Mag
  boots) and in the air, as the table says.
- **The canopy drop falls through thin ledges** onto the Restart Point's
  landing. The scroll is held from the rewind until it lands, and the 2 s
  of respawn invulnerability start on landing.
- **A Stage starts with the Hero standing** on its first landing at the
  Restart Point (the 3 → 4 Transition has just shown the climb begin), so
  the canopy drop only follows a lost life and the pace time is unchanged.
- **Landings and solid ledges stop the Hero's shots too**, as Cabinet 3's
  ground does; thin and crumbling ledges stop nothing. Crumbling ledges are
  one-way, like thin ones.
- **"Below the Hero"** (the no-fire rule) compares the bottom of the enemy's
  sprite with the Hero's feet. A Pitcher's canister is a 1 HP enemy worth
  no points: shooting it bursts it harmlessly, and touching it is a hit.
- **Limpets crawl a continuous loop** 3 px out from the ledge on every side
  (the ledge grown by half the Limpet's height), so they never jump at a
  corner. A pair starts half a loop apart.
- **A rising Flit** goes straight up for 64 px, then turns out through the
  nearer side at 90°/s.
- **The Winch's hook** hangs 24 px under the rail and drops to the highest
  landing on screen below it.
- **A scrapfall** picks its x with a fixed seed, so it falls the same way
  every time the Stage is played or rewound.
- **A gust zone's cycle** is 0.8 s of arrows, 2.0 s of wind, then calm until
  its `period` is up. Its `offset` delays the first warning.
- **The 1 s climb off the top at a Stage clear** (`hero.exit_seconds`) is
  not played yet: the flow moves on at `StageCleared` at once, as it does
  for Cabinet 3. The key stays for when it is.
- **The autopilot** follows the route with the base moves, including the
  stops part-way up a ladder. It waits under the lid until there is room to
  land a jump, never stops on a crumbling ledge it would have to wait on,
  and walks back against a gust that pushes it off its foothold. With it,
  both Stages clear with no rescues; Stage 1 clears about 3 s before its
  pace time because the autopilot finishes a Standoff early.

## Stage files and art (for #25)

- **Files:** `assets/stages/4-1.ron` (2:45) and `4-2.ron` (2:55), one line per
  beat of the tables above, each with its time in a comment, plus `ledges`,
  `ladders`, `gusts` and `route`.
- **Enemy kinds and params:** `flit` (`side`, or `"from": "below"`),
  `limpet`, `scrap`, `scrapfall` (`from_x`, `to_x`, `seconds`), `picket`,
  `vent`, `plumb`, `pitcher`, `mote`, `winch`. Hazards: `sweeper` (`dir`:
  `"cw"` or `"ccw"`). And `standoff` as in Cabinet 3. `drop` and `size` work
  as in Cabinet 1.
- **Where things are:** for an enemy on a ledge, `pos.y` is ignored. It
  stands on the ledge under `pos.x` at that height, and Stage checks test
  that there is one.
- **`stage_checks.rs`** tests the lengths, the Restart Points and their
  landings, the Power-up order, the known kinds and their params, the route
  (each step a base move, the drop margin, the lookahead), the Plumb and
  ladder placement rules, gusts against narrow crumbling ledges, and no
  Restart Point inside a Standoff.
- **Scenery** goes in the same files as `scenery:<sprite>` entries. The far
  layer (stars, cloud sea, the Underfield's lights) at 0.05–0.1 of the
  scroll; girders and cable runs behind the face at 0.5; the face itself at
  1.0.
- **Sprites** are `assets/sprites/c4_*.txt`. The Hero reuses `c3_hero` with
  added frames (climb, hang, canopy). Frames: the Vent closed, glowing and
  open; the Picket walking and firing; the Pitcher wind-up; the Plumb with
  its cable as a repeating tile; the Winch's carriage, hook and cable;
  crumbling ledges with a shaking frame; gust arrows. Face tiles, ledges,
  landings and ladders carry the `#! background` mark.

### Stage file decisions (#25)

- **Heights.** A ledge at `h` has its standing surface on the screen's top
  edge when the camera reaches `h` (screen y = 120 − (camera − h)), so
  everything a beat brings enters from the top at the beat's height. The
  first landing is at h = −184, low on the screen at the start.
- **Landings sit 48–64 px above the bottom edge** wherever the Hero
  stands on them with the scroll running: Restart Points at y = −64 to −68,
  Standoff decks at y = −56 or higher. The Restart Point after a Standoff
  moved a few pixels (1,022 → 1,016 in Stage 1, 1,596 → 1,592) to keep the
  Hero's feet 48 px clear of the edge when it respawns.
- **A Restart Point's `hero`** is where the Hero stands on its landing (feet
  on the surface, centre 10 px up), and it is a foothold of the route.
  The checks time the route from there as if the scroll runs once the
  Hero has landed, so #24 should hold the scroll during the canopy drop.
- **Ladders end at a landing through a hatch.** Landings are solid and
  full width, so a ladder is the only way through one. No jump on the route
  goes under a landing close enough to hit its head (apex 68 px).
- **Rule 7 is strict:** no part of a ladder is within 16 px of a Picket's
  ledge, including a ladder that starts there.
- **Ledge enemies** stand on the highest ledge under their `pos.x` at or
  below their `at` and still on screen; outside Standoffs each is spawned
  at its own ledge's height, so it enters with it.
- **Things fixed in the face** (Vents, Sweepers) are placed like scenery:
  at `pos` when the camera is at `at`.
- **The Wind Deck's gusts** start 64 px above its deck, not on it, because
  the Restart Point just after the Standoff stands on that deck and no gust
  may reach the first 64 px above a Restart Point.
- **The Winch's deck** is at h = 1,816, so it is at y = 0 while the scroll
  holds at 1,936 (exactly 20 s left, for the run-out music). That puts it
  80 px under the rail. It can't go lower: the gauntlet's last jump, from the
  Picket ledge at 1,736, needs 76 px of headroom under a solid landing, and
  Rule 7 rules out a ladder through that ledge. If the hook crowds the Hero,
  #24 can raise `winch.rail_y` to about 100.
- **Beats moved a few seconds** where the geometry needed it. Content may
  not enter before a Restart Point that the Hero meets after it, or it
  wouldn't replay on a rewind, and a Picket's ledge must clear the ladders.
  The tables above give the times the Stage files have.
- **Art.** The night face is Indigo plating with Abyss seams and Girder
  rivets, so that the Girder catwalks, ladders and enemy hulls read against
  it. The Crown's deck and Stage 2's girders have Peach dawn edges. The
  canopy reuses the parachute sprite from the 2 → 3 Transition rather than a
  new one. The Hero's climb (two frames, seen from behind) and hang frames
  are frames 10–12 of `c3_hero`.
- **The far layer in Stage 2** is fixed (depth 0) and fades by `at`/`until`
  windows: thinner stars, then the Peach dawn band in three rising rows,
  then the sun. The face above Stage 2's last landing is open sky, with the
  Crown and its beacon on top.

## Revisit after the playtests

Cabinet 1's playtest (#12) is already acted on here (Stage length and gun
2's fire rate). The other numbers follow Cabinets 2 and 3, which haven't been
played yet. When #17 or #22 changes the matching number there, check it here
too, or record why not:

- Everything copied from Cabinet 3's Hero: the jump, run speed, hitboxes,
  and the gun's levels. If Cabinet 3's gun also proves too strong, the 10
  volleys/s here may need to come down further.
- The Stage lengths, once #67 settles whether every Cabinet runs 2.5–3
  minutes.
- The bullet caps (60 and 70 px/s) and the reaction-time reasoning.
- Losing one gun level per life, and the rewind replaying Power-ups.
- The role HP and time-to-kill targets, and the 22 s set-piece Standoff.

And, for this Cabinet's own playtest (#27): whether 14 and 16 px/s feel like
pressure or like waiting, how much slack the 48 px drop margin really gives,
and whether gusts feel fair.

## Proposed `tuning.ron` section

For #24 to copy. The values are the ones above. `hero`, `jump` and `shot`
hold Cabinet 3's values under their own keys.

```ron
climber: (
    stage2_bullet_speed_scale: 1.15,
    debug_autopilot: false,
    camera: (face_half_width: 128.0),
    hero: (
        hitbox: (6.0, 14.0), crouch_hitbox: (6.0, 8.0),
        body: (10.0, 20.0), crouch_body: (10.0, 12.0),
        run_speed: 72.0, air_speed: 72.0,
        respawn_fall_speed: 60.0, fall_line: -120.0, exit_seconds: 1.0,
        respawn_invulnerable_seconds: 2.0, hit_stop_seconds: 0.15,
    ),
    jump: (
        speed: 240.0, gravity_up: 600.0, gravity_down: 900.0, cut: 0.4,
        max_fall: 300.0, coyote_seconds: 0.08, buffer_seconds: 0.1,
    ),
    ladder: (climb_speed: 48.0, grab_window: 6.0, jump_speed_fraction: 0.6),
    shot: (
        fire_interval: 0.125, fire_interval_fast: 0.1, speed: 300.0, damage: 1,
        size: (6.0, 2.0), muzzle: (8.0, 14.0), crouch_muzzle: (8.0, 6.0),
        spread_degrees: 12.0, bolt_size: (10.0, 3.0), bolt_damage: 2, max_on_screen: 64,
    ),
    power_up: (
        size: (10.0, 10.0), bob_amplitude: 2.0, bob_period: 1.0,
        gun_max: 4, jet_speed_fraction: 0.7, canopy_fall_speed: 50.0,
        mag_crumble_factor: 2.0, ward_invulnerable_seconds: 1.0,
    ),
    terrain: (
        tile_size: 8.0, crumble_seconds: 0.5, crumble_respawn_seconds: 3.0,
        // Stage-check constants (read by stage_checks.rs only):
        max_rise: 40.0, max_gap: 40.0, jump_setup_seconds: 0.3, drop_margin: 48.0,
        lookahead: 16.0, restart_clear_rise: 64.0,
    ),
    enemy: (
        bullet_size: (5.0, 5.0), bullet_hitbox: (2.0, 2.0), hitbox_inset: 1.0,
        fire_grace_seconds: 0.5, min_fire_distance: 40.0, max_bullets: 32,
        no_spawn_radius: 48.0, bullet_lifetime: 10.0, below_warning_seconds: 0.75,
        chest_height: 14.0, ankle_height: 4.0,
    ),
    enemies: (
        flit: (hp: 2, score: 100, formation_size: 3, spacing_seconds: 0.4, speed: 60.0,
               wave_amplitude: 12.0, wave_period: 1.2),
        limpet: (hp: 2, score: 100, formation_size: 2, speed: 24.0),
        scrap: (hp: 2, score: 50, warning_seconds: 0.75, speed: 120.0,
                fall_interval: 0.8, min_spacing: 24.0),
        picket: (hp: 16, score: 500, speed: 20.0, level_window: 16.0,
                 telegraph_seconds: 0.35, burst: 2, burst_interval: 0.25,
                 shot_speed: 60.0, cooldown_seconds: 2.0),
        vent: (hp: 12, score: 500, closed_seconds: 2.2, telegraph_seconds: 0.3,
               open_seconds: 1.4, fan_count: 3, fan_degrees: 30.0, shot_speed: 55.0),
        plumb: (hp: 6, score: 300, drop_speed: 60.0, hang_seconds: 2.0,
                telegraph_seconds: 0.3, shot_speed: 60.0, reel_speed: 40.0),
        pitcher: (hp: 20, score: 400, range: 120.0, fire_interval: 2.4,
                  telegraph_seconds: 0.4, canister_size: (8.0, 8.0), launch_speed: 60.0,
                  canister_gravity: 300.0, burst_speed: 50.0),
        mote: (hp: 1, score: 50, formation_size: 8, cloud_radius: 32.0, speed: 55.0,
               turn_rate: 90.0, lifetime: 8.0),
        winch: (hp: 120, score: 5000, rail_y: 80.0, speed: 40.0,
                telegraph_seconds: 0.6, drop_speed: 200.0, burst_speed: 60.0,
                rest_seconds: 1.2, reel_speed: 80.0),
    ),
    hazards: (
        gust: (blow_seconds: 2.0, push_speed: 36.0, warning_seconds: 0.8),
        sweeper: (length: 48.0, turn_rate: 60.0),
    ),
    score: (formation_bonus_per_enemy: 100, maxed_power_up: 1000,
            standoff_time_bonus: 50, stage_clear: 3000, no_miss: 5000),
)
```

## Decisions and reasons

- **A forced rise, unlike Cabinet 3's pushed camera.** The issue's direction
  ("falling below the bottom edge costs a life") makes the drop the genre's
  pressure, and it gives the last Cabinet a different feel from the one
  before. Cabinet 3 avoided a forced scroll because of pits. Here the
  fairness comes from a slow rise and a route that is proven to keep ahead
  of it.
- **A route in every Stage file.** It is the one thing a forced climb needs
  that a side-scroller doesn't: proof that a real path stays ahead of the
  edge. The checks time it, and the autopilot plays it.
- **The same Hero, the same jump, the same gun, under their own keys.** The
  player's hands already know them from Cabinet 3, and either Cabinet can
  still be tuned alone.
- **Ladders fire straight up only,** which is where this Cabinet's threats
  are, and ladders are kept out of chest-height shooters' lines (rule 7).
- **Standoffs on landings,** as in Cabinet 3. There are no bosses, the
  scroll is paused, and the bottom edge stops chasing.
- **Five Power-ups: G, W, J, Canopy, Mag boots.** The Canopy calls back to
  the parachute. Mag boots answer Stage 2's gusts and crumbling ledges.
  There is no Flare or Hustle, because neither helps against a forced scroll
  or threats above.
- **Rose bullets, Mint accent, Peach dawn.** Ember stays the Hero's colour.
  Rose reads well on the night's blues, and Peach is muted so the dawn never
  looks like bullets. The placeholder palette in `src/sprites/palette.rs` is
  updated to this table in the same change, and its doc test now covers
  Cabinet 4.
- **Shorter Stages and a weaker gun, from the Cabinet 1 playtest.** Cabinet 1
  was cut to under 3 minutes and its firepower lowered (#65); this Cabinet
  starts there rather than waiting for its own playtest. The user rules on
  the length for every Cabinet in #67.
- **The summit sets up the Ending; it doesn't play it.** #28 decides what
  happens at the Crown.
- **No new glossary terms.** "Route", "landing", "fall line" and "Standoff"
  (Cabinet 3's) are Cabinet ideas defined in their docs. The glossary keeps
  to words that hold across the whole Run.
