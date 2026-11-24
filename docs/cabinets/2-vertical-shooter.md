# Cabinet 2: SPIREWARD (vertical shooter)

The design that issues #14–#17 build. Terms are from [GLOSSARY.md](../../GLOSSARY.md);
the settled design is [DESIGN.md](../DESIGN.md). The Cabinet 1 design,
[1-side-shooter.md](1-side-shooter.md), is the model for this doc: where this
doc says "as in Cabinet 1", the Cabinet 1 rule applies unchanged. The user can
override anything here by commenting on issue #13.

**Where the numbers live.** Every number names its key.

- Keys starting `vertical_shooter.` go in `assets/tuning.ron`. The proposed
  section is at the end of this doc. Inside an enemy's own section, keys are
  written short: `wisp.speed` means `vertical_shooter.enemies.wisp.speed`.
- Scroll speeds, spawn times, spawn positions, which Power-up each Formation
  drops and Restart Points go in the Stage files (`assets/stages/2-1.ron` and
  `2-2.ron`), as in Cabinet 1.
- A Formation's size comes from its spawn entry in the Stage file. Its default
  is the enemy's `formation_size` key in `tuning.ron`.
- Sprite sizes are given here so the sprite files (#4) match. They live in the
  sprite files, not in tuning.

"SPIREWARD" is the working in-game name. DESIGN.md leaves final naming until
there is something on screen, so it may change then.

**Cabinet 1 is still being finished** (#7–#11) and hasn't been playtested
(#12). Numbers copied from Cabinet 1 are listed in
[Revisit after the Cabinet 1 playtest](#revisit-after-the-cabinet-1-playtest);
when #12's feedback changes a Cabinet 1 number, check the matching one here.

## Premise

> The Static pours up the shaft and out over a high plateau above the clouds.
> Far across it, the swarm is building a tower out of everything it has
> taken: **the Spire**. The Wren chases it over the cloud sea and into the
> sprawling outer yard of the Spire's works, until the Static's flak finally
> finds it over the yard's far edge.

- **The 1 → 2 view change (for #16).** Cabinet 1 is a side view; Cabinet 2 is
  top-down, with the ground below and the Wren's shadow on it. DESIGN.md's
  "camera rotates 90°" alone would give a side view looking up a shaft, so
  the Transition must also move from profile to overhead. For example, the
  Wren climbs the shaft, the camera tilts to look down as it clears the
  shaft mouth, and the Wren's shadow appears on the clouds below as Stage 1
  begins.
- **Stage 2 ends with the Wren shot down over the outer yard's edge,** with
  the Spire still distant. That hit starts the 2 → 3 Cabinet Transition
  (#21): the Hero ejects and parachutes down through the cloud line.
- **Then the later Cabinets:** Cabinet 3 is the fight on foot back toward
  the Spire's base, and Cabinet 4 climbs it (the 3 → 4 Transition). Seeing
  the Spire on the horizon here plants it early.

## What makes it feel good (the rules behind the numbers)

Cabinet 1's nine rules all apply: bullets you can read, enemies you can see,
every threat announced, a small hitbox, teach-test-twist, waves that breathe,
power arriving steadily, a lost life that dents rather than wipes out, and HP
from time-to-kill targets. These change or are added for the vertical genre:

1. **You can always read the bullets, in this Cabinet's colours.** Enemy
   bullets are Volt (lime), a colour nothing else uses except enemy weak
   points and flak crosshairs. They are round with a Foam core. Magenta can't
   be used, because the Cabinet's accent (`CABINETS[1].accent`) is pink.
2. **The same time to react as Cabinet 1.**
   - In Cabinet 1 a typical enemy shot, from x ≈ +100 to the Hero at x = −110
     at 100 px/s, takes about 2.1 s to arrive. Here a shot from near the top
     (y ≈ +108) to the Hero (y = −80) travels about 188 px.
   - So no enemy bullet in Stage 1 is faster than 90 px/s (2.1 s over that
     distance), and none in Stage 2 is faster than 105 px/s.
   - The rule is about bullet speeds. Enemy movement is set per enemy.
3. **Air or ground, you can tell at a glance.**
   - The main cue is the hull: air enemies have Steel hulls, and ground
     targets have Clay hulls set into the ground art.
   - Every air enemy, and the Wren, also casts a Dusk shadow 6 px below and to
     the right on the ground (`vertical_shooter.look.shadow_offset: (6.0, -6.0)`).
     Shadows are a second cue: over open sky the backdrop is also Dusk, so
     they only show over clouds and ground, which is where ground targets
     are anyway.
   - **Air enemies can hit you by touching you; ground targets can't,**
     because the Wren flies over them. Hero shots hit both.
4. **No shot from behind.** Once an enemy is below the Wren (the Hero has
   passed it), it never fires again. That covers ground targets after they
   scroll past and any air enemy that drops below the Hero.
5. **The power reset is short.** Power-ups reset at the Cabinet Transition, so
   a strong Cabinet 1 player starts here at gun 1. The first three Formations
   all drop Power-ups, so gun 2, a Pod and Rockets arrive within about 20 s.
6. **The ground and the sky take turns.** The Stages alternate *ground
   fronts* (mesas or yard blocks crowded with ground targets, where Rockets
   shine) with *air fronts* (open sky, where the gun and Pods do the work).
   Each Stage has its own beat clock built from these fronts, not Cabinet 1's.

## Screen

- 320×240, the play area y ∈ [−120, +108] under the shared HUD, as in
  Cabinet 1. **Full width, no side panels:** a wide field gives room to dodge
  sideways, which suits a vertical shooter played on a 4:3 screen.
- The Hero is clamped inside the play area with a 4 px margin
  (`vertical_shooter.hero.edge_margin: 4.0`).
- Bomb icons sit at the bottom-left of the play area, as in Cabinet 1.

## Palette

One Cabinet palette (#4), shared by both Stages. Index 0 is transparent.
The numbers are the sprite characters, in the same roles as Cabinet 1's
palette (backdrop, shade, fill, accent, bright, warm, danger, extra), so the
shared Wren sprite takes Foam, Orchid and Ember here without a second copy.

| # | Name | Hex | Used for |
|---|---|---|---|
| 1 | Dusk | `#140019` | backdrop (matches `CABINETS[1].backdrop`), shadows |
| 2 | Plum | `#3A1F4F` | cloud shading, far layer, ground shading |
| 3 | Steel | `#5A6E8C` | clouds, scaffolding, **air enemy hull fill** |
| 4 | Orchid | `#FF66CC` | accent (matches `CABINETS[1].accent`), Hero canopy, Power-up rims |
| 5 | Foam | `#F4F4F0` | Hero body, **enemy outlines**, bullet cores, Power-up letters, text |
| 6 | Ember | `#FFB43C` | Hero shots, rockets, Hero engine, explosions |
| 7 | Volt | `#C8FF3C` | **enemy bullets, enemy weak points and flak crosshairs only** |
| 8 | Clay | `#8C5A3C` | mesas, the yard floor, **ground target hull fill** |

Rules that #4 builds and #14 checks in the screenshots (the same rules as
Cabinet 1, with this Cabinet's colours):

- **Backgrounds and terrain use only Dusk, Plum, Steel and Clay.** Foam,
  Orchid, Ember and Volt never appear there.
- **Every enemy has a 1 px Foam outline** around a Steel (air) or Clay
  (ground) hull, and a Volt eye or core.
- The Wren is the only sprite with a Foam *body* and an Ember engine.
- Foam and Ember are named as in Cabinet 1 on purpose: the Hero and its shots
  look the same across both ship Cabinets. Only the canopy takes the new
  accent.

## Hero ship: the Wren, nose up

The same craft as Cabinet 1, seen from above with its nose up. The sprite is
the Cabinet 1 silhouette turned 90°, so it is recognisably the same ship.

| Property | Value | Key |
|---|---|---|
| Sprite | 10×16, Foam body, Orchid canopy, two-frame Ember engine flicker | (sprite file) |
| Hitbox vs bullets and enemies | 4×4 at the cockpit | `vertical_shooter.hero.hitbox: (4.0, 4.0)` |
| Base speed | 100 px/s, same in all 8 directions | `vertical_shooter.hero.speed: 100.0` |
| Speed per Speed step | +20 px/s | `vertical_shooter.hero.speed_step: 20.0` |
| Start / respawn position | x = 0, y = −80 | `vertical_shooter.hero.start: (0.0, -80.0)` |
| Entry | flies up from y = −136 over 1.0 s, with no control until it arrives | `vertical_shooter.hero.entry_from_y: -136.0`, `vertical_shooter.hero.entry_seconds: 1.0` |
| Invulnerability after respawn | 2.0 s, blinking (from the end of the entry) | `vertical_shooter.hero.respawn_invulnerable_seconds: 2.0` |
| Hit-stop on losing a life | 0.15 s | `vertical_shooter.hero.hit_stop_seconds: 0.15` |

**No terrain collision in this Cabinet.** The Wren flies high above the
ground, so there is no terrain hitbox and no walls. Tethers and Flak
Batteries take over the job Cabinet 1's Stage 2 walls did: squeezing the
space the player can use.

### A: fire (hold to auto-fire)

As in Cabinet 1: 10 volleys/s (`vertical_shooter.shot.fire_interval: 0.1`),
and tapping is never faster than holding.

- Hero shots are 2×8 Ember dashes (`vertical_shooter.shot.size: (2.0, 8.0)`)
  flying up at 360 px/s (`vertical_shooter.shot.speed: 360.0`), 1 damage
  each (`vertical_shooter.shot.damage: 1`). They hit air enemies and ground
  targets alike.
- **Shot cap:** 96 (`vertical_shooter.shot.max_on_screen: 96`). It is a
  safety valve, as in Cabinet 1. Full power fires 6 shots per volley (4 gun,
  2 Pods). A shot crosses the play area in about 0.6 s, so about 36 are alive
  at once.

### B: the Squall (bomb)

Two lines of force sweep out from the Wren's row, one up and one down, each
across the full width.

- The two edges move away from the Wren's row at 420 px/s
  (`vertical_shooter.bomb.band_speed: 420.0`) until they reach the top and
  bottom of the play area. The whole screen is swept in under 0.6 s.
- **Only the moving edges touch things,** as with Cabinet 1's Undertow ring
  (#45). Each edge hits whatever it passes over between one frame and the
  next, so nothing slips through on a slow frame. Bullets fired into space
  an edge has already swept are not cancelled. That keeps the Squall a burst
  rather than a shield, and it behaves like the bomb the player already
  knows.
- Every enemy bullet an edge touches is cancelled for 10 points
  (`vertical_shooter.score.bomb_cancel: 10`).
- Every enemy an edge touches, **air or ground**, takes 12 damage once
  (`vertical_shooter.bomb.damage: 12`).
  - That kills Wisps, Swifts, Motes, Crawlers and Riser pods.
  - Bellows, Riser launchers, open Silos, Flak Batteries, Loom shuttles and
    cores, and Tether balloons survive but are hurt.
  - A **closed Silo** and a **shielded Loom core** take no damage.
- 1.0 s of invulnerability from the moment B is pressed
  (`vertical_shooter.bomb.invulnerable_seconds: 1.0`).
- **Count:** start with 2 (`vertical_shooter.bomb.starting: 2`) and hold at
  most 4 (`vertical_shooter.bomb.max: 4`). After a lost life the count is
  topped up to `starting`, and bombs carry from Stage 1 to Stage 2. B does
  nothing while an edge is still moving. All as in Cabinet 1.

A pair of lines rather than Cabinet 1's ring because it matches the screen's
shape: the threats in a vertical shooter are spread in rows across the width.

## Power-ups

### How they're earned

Exactly as in Cabinet 1: Formations drop the Power-up their spawn entry names
or else pay a Formation bonus, a member that leaves alive spoils the
Formation, and the pickup rules are the same.

- **Ground Formations count.** A convoy of Crawlers is a Formation like any
  other, so shooting the ground pays.
- A Loom counts as a one-member Formation and drops its Power-up when its core
  dies. Wisps a Loom launches and pods a Riser launches belong to no
  Formation.
- A Power-up appears where the last member died, clamped 16 px inside the
  play area (`vertical_shooter.power_up.edge_inset: 16.0`). It drifts
  **down** at 30 px/s (`vertical_shooter.power_up.drift_speed: 30.0`), bobs
  3 px sideways (`vertical_shooter.power_up.bob_amplitude: 3.0`) every 1.2 s
  (`vertical_shooter.power_up.bob_period: 1.2`), and stays until it leaves
  the bottom of the screen.
- It is a 10×10 capsule (`vertical_shooter.power_up.size: (10.0, 10.0)`) with
  an Orchid rim and a Foam letter, collected within a 14×14 box around the
  Wren (`vertical_shooter.power_up.pickup_box: (14.0, 14.0)`).
- A Power-up already at its maximum gives a 1,000-point medal
  (`vertical_shooter.score.maxed_power_up: 1000`).

### The six Power-ups

| Letter | Name | Effect | Max | Key(s) |
|---|---|---|---|---|
| **G** | Gun | Raises the gun level by 1 (see below). | level 4 | `vertical_shooter.power_up.gun_max: 4` |
| **P** | Pod | Adds an 8×8 pod 14 px to the left of the Wren, and a second one 14 px to the right. A pod fires one shot straight up with every volley and **absorbs enemy bullets** that touch it. Enemy bodies pass through it. Pods can't be destroyed. | 2 | `vertical_shooter.power_up.pod_max: 2`, `vertical_shooter.power_up.pod_offset: 14.0` |
| **R** | Rockets | Every 0.5 s, launches 2 small rockets that home on the nearest ground target above the Wren. If there is none on screen, no rockets fire. Rockets hit **ground targets only**, passing under air enemies. | 1 | `vertical_shooter.rocket.*` (below) |
| **S** | Speed | +1 speed step (+20 px/s). | 2 steps (140 px/s) | `vertical_shooter.power_up.speed_max: 2` |
| **W** | Ward | Absorbs one hit from a bullet or an enemy, then gives 1.0 s of invulnerability. | 1 | `vertical_shooter.power_up.ward_invulnerable_seconds: 1.0` |
| **B** | Bomb | +1 Squall. | 4 | `vertical_shooter.bomb.max` |

What differs from Cabinet 1, and why:

- **Pods sit beside the Wren, not above and below it.** Cabinet 1's drones
  widened the shot vertically. Here the threats come from above, so the pods
  widen it sideways.
- **Rockets replace the Tail.** Little attacks from behind in this genre
  (Swifts do, but they are announced and have no bullets), while ground
  targets are new. Rockets make the air/ground split matter: G is the
  all-round gun, and R is the answer to a crowded ground front.

**Rockets:**

- 2 per launch (`vertical_shooter.rocket.count: 2`), every 0.5 s
  (`vertical_shooter.rocket.interval: 0.5`).
- They launch 5 px either side of the Wren's centre, at its wings
  (`vertical_shooter.rocket.launch_offset: (5.0, 0.0)`).
- Each is a 3×6 Ember dart (`vertical_shooter.rocket.size: (3.0, 6.0)`). It
  flies at 200 px/s (`vertical_shooter.rocket.speed: 200.0`) and turns at up
  to 270°/s (`vertical_shooter.rocket.turn_rate: 270.0`).
- 3 damage each (`vertical_shooter.rocket.damage: 3`), so 12 damage per
  second against ground targets.
- A rocket whose target dies picks the next nearest ground target above it.
  If there is none, it flies straight on and despawns off screen.

### Gun levels

| Level | Volley | Feel |
|---|---|---|
| 1 | One shot straight up. | A pea-shooter. |
| 2 | Two parallel shots, 6 px apart (`vertical_shooter.shot.twin_spacing: 6.0`). | Twice the damage. |
| 3 | Level 2, plus two shots angled ±15° from vertical (`vertical_shooter.shot.spread_degrees: 15.0`). | Covers the width ahead. |
| 4 | The parallel shots become **lances**: 3×12 (`vertical_shooter.shot.lance_size: (3.0, 12.0)`), 2 damage (`vertical_shooter.shot.lance_damage: 2`). Angled shots as level 3. | The payoff level. |

The spread is wider than Cabinet 1's ±10° because the field is wider than it
is deep.

### Damage budget

The Cabinet 1 table holds with Pods in place of Drones. "Large" here means a
target 32 px or more **wide** within about 90 px of the Wren: the angled shots
spread to ±24 px at that distance, so they and both Pods connect.

| Power | Narrow DPS | Large DPS | Ground bonus with Rockets |
|---|---|---|---|
| Gun 1 | 10 | 10 | +12 |
| Gun 2 | 20 | 20 | +12 |
| Gun 3 | 20 | 40 | +12 |
| Gun 4 | 40 | 60 | +12 |
| each Pod | +0 | +10 | |

Expected power ("perfect" means every Formation cleared and no life lost;
"typical" means one or two missed and one life lost). Cabinet 2 starts from
nothing, so the Stage 1 curve starts steep.

| When | Perfect | Typical |
|---|---|---|
| Stage 1, 0:18 (first ground front) | Gun 2 + 1 Pod | Gun 2 |
| Stage 1, 0:58 (first Bellows) | Gun 2 + 1 Pod + Rockets: narrow 20, large 30 | Gun 2: narrow 20, large 20 |
| Stage 1, 1:30 (first Loom) | Gun 3 + 1 Pod: narrow 20, large 50 | Gun 2 + 1 Pod: narrow 20, large 30 |
| Stage 1, 3:00 (second Loom) | Gun 3 + 2 Pods: narrow 20, large 60 | Gun 3 + 1 Pod: narrow 20, large 50 |
| Stage 2 | Gun 4 + 2 Pods: narrow 40, large 80 | Gun 3 + 1 Pod: narrow 20, large 50 |

Time-to-kill targets by role, and the HP that follows. The roles and their
targets are Cabinet 1's, a shared language across both ship Cabinets. The
enemies that fill them behave differently.

| Role | Enemies | Target | HP | Check |
|---|---|---|---|---|
| Popcorn | Wisp, Mote, Swift, Riser pod | under 0.3 s even at gun 1 | 1, 1, 2, 1 | Swift: 0.2 s at gun 1 |
| Small ground | Crawler, Riser launcher | about 0.5–0.8 s at typical power | 10, 16 | Crawler 0.5 s and launcher 0.8 s at narrow 20; 0.3 s / 0.5 s with Rockets |
| Gunner | Bellows | 1.5–2 s, so it gets a burst off | 36 | 1.8 s at narrow 20, 0.9 s at gun 4 |
| Timing lock | Silo | dies inside one 1.2 s opening at typical power | 16 | 0.8 s at narrow 20; 0.5 s with Rockets |
| Ground gunner | Flak Battery | about 1 s at typical power | 24 | 1.2 s at narrow 20; 0.75 s with Rockets |
| Power check | Tether balloon | about 2 s at typical power | 40 | 2 s at narrow 20; 4 s at gun 1 |
| Set piece | Loom (two phases) | about 10–12 s at perfect power, under its 20 s hold at typical power | shuttles 60 each and core 240 in Stage 1; shuttles 100 and core 400 in Stage 2 | see Loom |

### Losing a life

As in Cabinet 1: gun −1 (never below 1), the Ward is lost and so is one Pod
(the right one first), Speed, Rockets and bombs are kept (bombs topped up),
and the Stage rewinds to the latest Restart Point, replaying its Power-ups.

### Saved state

`Session.power_ups` holds this JSON shape, saved and cleared exactly as
Cabinet 1's:

```json
{ "gun": 1, "pods": 0, "rockets": false, "speed": 0, "ward": false, "bombs": 2 }
```

Practice starts from this default.

## Enemies

All are original machines of the Static. "Aimed" means aimed at the Hero's
position at the moment of firing, with no leading.

- Enemy bullets are 5×5 Volt dots with a Foam core
  (`vertical_shooter.enemy.bullet_size: (5.0, 5.0)`), with a 2×2 hitbox
  (`vertical_shooter.enemy.bullet_hitbox: (2.0, 2.0)`). They are removed after
  12 s (`vertical_shooter.enemy.bullet_lifetime: 12.0`).
- An enemy's hitbox is its sprite shrunk by 1 px on each side
  (`vertical_shooter.enemy.hitbox_inset: 1.0`).
- In Stage 2 every enemy bullet speed is multiplied by 1.15
  (`vertical_shooter.stage2_bullet_speed_scale: 1.15`). The fastest result is
  the Bellows at 103.5 px/s, under rule 2's 105 px/s.
- **Air enemies** (Wisp, Swift, Bellows, Loom, Mote, Tether, and airborne
  Riser pods) hurt the Wren on contact and cast shadows.
- **Ground targets** (Crawler, Silo, Flak Battery, Riser launcher and pods
  still climbing) are fixed to the ground and scroll down with it at the
  scroll speed. They never touch the Wren, and they draw under every air
  sprite.

General fairness rules, as in Cabinet 1 with this Cabinet's directions:

- No enemy fires in its first 0.5 s on screen
  (`vertical_shooter.enemy.fire_grace_seconds: 0.5`).
- No bullet is ever created within 40 px of the Hero
  (`vertical_shooter.enemy.min_fire_distance: 40.0`).
  - Cabinet 1 measures that from the firing enemy. Here it is measured from
    **where the bullet appears**, which is the same thing for most enemies,
    and what matters for flak bursts and Riser pods.
  - A pattern that would break it is skipped whole, not trimmed.
- No enemy fires once it is below the Hero (rule 4). This applies to every
  pattern, rings included.
- At most 40 enemy bullets at once
  (`vertical_shooter.enemy.max_bullets: 40`).
- Nothing spawns on screen within 48 px of the Hero
  (`vertical_shooter.enemy.no_spawn_radius: 48.0`).

| # | Name | Layer | Stage | Sprite | HP | Score | Default Formation |
|---|---|---|---|---|---|---|---|
| 1 | Wisp | air | 1, 2 | 8×8 | 1 | 100 | column of 6 |
| 2 | Swift | air | 1, 2 | 10×12 | 2 | 200 | pair |
| 3 | Bellows | air | 1, 2 | 20×16 | 36 | 500 | single |
| 4 | Loom | air | 1, 2 | core 32×28, shuttles 16×20 | see Loom | 500 per shuttle, 2,000 core | single; launches Wisps |
| 5 | Crawler | ground | 1, 2 | 12×12 | 10 | 300 | convoy of 3 |
| 6 | Silo | ground | 1, 2 | 16×16 | 16 | 800 | single |
| 7 | Riser | ground → air | 1, 2 | launcher 16×14, pod 6×6 | 16 / 1 | 400 launcher; pod 150 climbing, 50 airborne | single |
| 8 | Flak Battery | ground | 2 | 16×12 | 24 | 600 | single |
| 9 | Mote | air | 2 | 4×4 | 1 | 50 | cloud of 10 |
| 10 | Tether | air | 2 | balloons 10×12 | 40 per balloon | 600 per balloon | single |

HP and score keys: `<name>.hp`, `<name>.score` (with the exceptions named in
each section). Default Formation size: `<name>.formation_size` (1 when not
listed).

Of these, the Riser, the Flak Battery and the Loom's two-phase structure have
no Cabinet 1 counterpart. The others share a role or a brain with a Cabinet 1
enemy, but each behaves differently on screen.

### 1. Wisp: the falling column

A teardrop of light. A column of 6 (`wisp.formation_size: 6`), 0.3 s apart
(`wisp.spacing_seconds: 0.3`), drops in from the top edge at the spawn's x and
falls at 80 px/s (`wisp.speed: 80.0`). At y = +20 (`wisp.turn_y: 20.0`) each
Wisp curves toward the Hero's half of the screen, turning at 150°/s
(`wisp.turn_rate: 150.0`), and leaves through that side at 110 px/s
(`wisp.exit_speed: 110.0`).

**It doesn't fire.** The column is a moving wall: shoot the head as it comes,
or get out of the half it curves into. Loom shuttles also launch Wisps, which
fall straight down at 100 px/s (`wisp.launched_speed: 100.0`).

### 2. Swift: the ambush from below

A narrow blade, in pairs (`swift.formation_size: 2`).

- **Warning:** a chevron blinks on the bottom edge for 0.75 s
  (`swift.warning_seconds: 0.75`), 4 times a second
  (`swift.chevron_blink_hz: 4.0`), under the column where each Swift will
  appear.
- **Climb:** it **climbs in from behind** at 150 px/s (`swift.speed: 150.0`)
  along a column 50 px left or right of the Hero (`swift.pass_offset: 50.0`).
  One of the pair goes on each side; larger Formations go further out by half
  an offset per pair (`swift.pair_spread: 0.5`). The column is clamped to
  stay 12 px inside the play area (`swift.edge_clearance: 12.0`).
- **Turn:** at y = +80 (`swift.turn_y: 80.0`) it U-turns on a half-circle of
  radius 14 px (`swift.turn_radius: 14.0`) toward the Hero's side.
- **Dive:** it dives at 170 px/s (`swift.dive_speed: 170.0`), sliding sideways
  at up to 85 px/s (`swift.dive_steer_speed: 85.0`) onto the column the Hero
  was in when the turn ended.
- It has no bullets. It rams.

It shares its brain with Cabinet 1's Hook, with the scroll axis as a
parameter, but here it comes from below the player's line of sight.

### 3. Bellows: the gunner

A squat air barge with a vent on its belly. **It enters from a side edge**,
the one its spawn entry names (`params: {"side": "left"}` or `"right"`),
gliding in along its row at 80 px/s (`bellows.speed: 80.0`). It brakes over
its last 24 px (`bellows.brake_distance: 24.0`, never below 15% speed,
`bellows.brake_floor: 0.15`) to stop 60 px in from that edge
(`bellows.stop_inset: 60.0`).

- It hovers for 3.0 s (`bellows.hover_seconds: 3.0`), swaying 2 px
  (`bellows.sway_amplitude: 2.0`) every 1.5 s (`bellows.sway_period: 1.5`).
- Its vent glows **as soon as it stops**. After 0.4 s
  (`bellows.telegraph_seconds: 0.4`) it fires a burst of 3 aimed shots
  (`bellows.burst: 3`), 0.12 s apart (`bellows.burst_interval: 0.12`), at
  90 px/s (`bellows.shot_speed: 90.0`).
- It fires 2 bursts (`bellows.bursts: 2`) 1.4 s apart
  (`bellows.fire_interval: 1.4`), each telegraphed.
- It then climbs out of the top at 80 px/s (`bellows.exit_speed: 80.0`).

Spawn entries put a Bellows on a row between y = +40 and y = +80. It stops
about 1.0 s after appearing, so its first burst comes at about 1.4 s, just
before a typical player kills it (1.8 s).

Entering from the side, rather than from ahead as Cabinet 1's gunner does,
pulls the player's fire sideways. That's a new skill in a Cabinet where most
threats come from above. A burst of three slow shots in a line reads as one
threat to step aside from.

### 4. Loom: the two-phase carrier

A broad air platform in **three parts**: a central core with a shuttle docked
on each flank. Each part is its own enemy entity, like the Arc Gate's pylons.

**Phase 1, the shuttles.**

- The core is **shielded** (immune) while either shuttle is docked. Shots
  spark off it.
- Each shuttle is 16×20, docked 28 px either side of the core
  (`loom.shuttle_offset: 28.0`), with 60 HP in Stage 1
  (`loom.shuttle_hp: 60`) and 100 in Stage 2 (`loom.shuttle_hp_stage2: 100`).
  It is worth 500 (`loom.shuttle_score: 500`).
- **Launches:** each shuttle launches 1 Wisp, 1.0 s after the Loom appears
  (`loom.first_launch_delay: 1.0`) and then every 3.0 s
  (`loom.launch_interval: 3.0`).
- **Aimed pairs:** each shuttle fires an aimed pair of shots, 0.15 s apart
  (`loom.pair_interval: 0.15`), at 80 px/s (`loom.shot_speed: 80.0`), every
  2.5 s (`loom.fire_interval: 2.5`). Each pair is telegraphed for 0.4 s
  (`loom.telegraph_seconds: 0.4`), and the two shuttles alternate.
- A destroyed shuttle falls away to the ground as burning wreckage (scenery
  only).

**Phase 2, the core.**

- When both shuttles are gone, the core's shield drops. It takes 240 HP in
  Stage 1 (`loom.core_hp: 240`) and 400 in Stage 2
  (`loom.core_hp_stage2: 400`), and is worth 2,000 (`loom.core_score: 2000`).
- It starts **the spiral**: 3 arms (`loom.spiral_arms: 3`), one bullet per arm
  every 0.3 s (`loom.spiral_interval: 0.3`), with the arms turning at 40°/s
  (`loom.spiral_turn_rate: 40.0`) and bullets at 60 px/s
  (`loom.spiral_speed: 60.0`).
- It spirals for 2.0 s (`loom.spiral_seconds: 2.0`), then pauses 2.0 s
  (`loom.spiral_pause: 2.0`). The pause is the window to close in and shoot.

**Movement.** The Loom descends from the top at 20 px/s (`loom.speed: 20.0`)
to hold its core at y = +60 (`loom.hold_y: 60.0`), about 4 s after appearing.
It holds there for up to 20 s (`loom.hold_seconds: 20.0`), then retreats
upward at 20 px/s (`loom.retreat_speed: 20.0`), taking its Power-up with it.

**Escorts.** Each Stage places a ground front under at least one Loom, so the
set piece is fought in both layers at once. Rockets keep the ground busy
while the gun works on the shuttles.

**Time to kill.** A shuttle is a narrow target. The core is 32 px wide, a
large target, but only from within about 90 px. At y = +60 that means the
Hero has to come up to about y = −30, into the spiral's reach, to get full
damage. #17 measures the times with the Hero there:

| Stage, power | Shuttles (narrow) | Core (large) | Total |
|---|---|---|---|
| Stage 1 at 1:30, perfect (narrow 20, large 50) | 3 s + 3 s | 4.8 s | about 11 s |
| Stage 1 at 1:30, typical (20 / 30) | 3 s + 3 s | 8 s | about 14 s |
| Stage 1 at 3:00, perfect (20 / 60) | 3 s + 3 s | 4 s | about 10 s |
| Stage 2, perfect (40 / 80) | 2.5 s + 2.5 s | 5 s | about 10 s |
| Stage 2, typical (20 / 50) | 5 s + 5 s | 8 s | about 18 s |

Every case fits inside the 20 s hold. The Loom always drops its Power-up when
the core dies; one that retreats drops nothing.

### 5. Crawler: the ground convoy

A treaded turret crawling along roads on the ground.

- **Convoy:** 3 (`crawler.formation_size: 3`), 18 px apart
  (`crawler.spacing: 18.0`), moving at 12 px/s relative to the ground
  (`crawler.speed: 12.0`) in the direction the spawn entry gives
  (`params: {"heading": 0.0}`, degrees, 0 = right). On screen this adds to
  the scroll.
- **Turret:** it tracks the Hero at 120°/s (`crawler.turn_rate: 120.0`) and
  flashes for 0.3 s (`crawler.telegraph_seconds: 0.3`). It fires one shot
  **along its turret** at 80 px/s (`crawler.shot_speed: 80.0`) every 2.0 s
  (`crawler.fire_interval: 2.0`), so fast sideways movement beats it.
- Members' timers are offset by 0.4 s (`crawler.stagger: 0.4`), so a convoy
  fires a ripple, never one wall.

### 6. Silo: the timing lock

An armoured hatch in the ground.

- It is closed for 2.4 s (`silo.closed_seconds: 2.4`), immune to all damage,
  the Squall included. Shots spark off it.
- It flashes for 0.3 s (`silo.telegraph_seconds: 0.3`), then opens for 1.2 s
  (`silo.open_seconds: 1.2`) to show a Volt core.
- As it opens, it fires a ring of 6 (`silo.ring_count: 6`) at 70 px/s
  (`silo.shot_speed: 70.0`).
- **It only takes damage while open.** Its cycle starts when it scrolls on
  screen, so the first opening comes about 2.7 s in.

### 7. Riser: the ground that takes off

A launch pad on the ground that throws pods up into the air layer. It's this
Cabinet's signature enemy: a threat you can deal with while it is still
part of the ground, or have to dodge once it is airborne.

**The launcher.**

- Ground layer, 16×14, 16 HP (`riser.hp: 16`), worth 400
  (`riser.score: 400`).
- It launches a pod 1.0 s after it is fully on screen
  (`riser.first_launch_delay: 1.0`), then every 3.5 s
  (`riser.launch_interval: 3.5`), while it is above the Hero.
- Killing the launcher stops new pods. Pods already launched carry on.

**The pod, climbing.**

- For 1.0 s after launch (`riser.climb_seconds: 1.0`) the pod rises from the
  pad. It is still on the **ground layer**: it can't touch the Wren, and it
  stays over the pad as the ground scrolls.
- Its shadow appears under it and slides out from 0 to the full 6 px offset
  as it climbs. That growing shadow is the warning.
- Shot down while climbing (1 HP), it is worth 150
  (`riser.pod_score_climbing: 150`). Rockets target it too.

**The pod, airborne.**

- After the climb it becomes an **air** enemy: it can ram the Wren, and it is
  worth 50 if shot now (`riser.pod_score_airborne: 50`).
- It drifts toward the Hero's x at up to 40 px/s
  (`riser.drift_speed: 40.0`) while falling with the scroll.
- After 1.2 s airborne (`riser.fuse_seconds: 1.2`) it flashes for 0.3 s
  (`riser.telegraph_seconds: 0.3`) and bursts into a ring of 6
  (`riser.ring_count: 6`) at 70 px/s (`riser.shot_speed: 70.0`).
- If the Hero is within 40 px of it when it bursts, or it is below the Hero,
  it fizzles harmlessly instead (the fairness rules).

The lesson is to shoot pods on the way up, where they are worth three times
as much and can't hurt you, or to kill the launcher. A player who ignores a
Riser gets a sky full of slow rings.

### 8. Flak Battery: the area denier (Stage 2)

A ground gun that shoots at where the Wren *is*, not at the Wren.

- **Targeting:** every 2.4 s (`flak.fire_interval: 2.4`) it marks the Hero's
  current position with a Volt crosshair. The point is fixed in the air and
  does not scroll.
- **The burst:** 0.6 s later (`flak.warning_seconds: 0.6`) a shell bursts
  there. The burst **hits the Hero only**, if the Hero is within 10 px of the
  point, for 0.15 s (`flak.burst_radius: 10.0`, `flak.burst_seconds: 0.15`).
  It doesn't touch enemies, Power-ups or bullets.
- **The ring:** the burst also throws out a ring of 6 bullets
  (`flak.burst_count: 6`) at 60 px/s (`flak.burst_speed: 60.0`), but **only
  if the Hero is at least 40 px from the point**
  (`vertical_shooter.enemy.min_fire_distance`).
  - A player who sidesteps a little escapes the burst and gets no ring.
  - A player who has moved well away sees a ring spawn at a safe distance,
    to read and slip through.
  - No bullet ever appears point-blank.
- **Fairness:** at base speed the Hero moves 60 px in the warning time, so
  moving is always enough to escape the burst. Standing still is never safe.
  The crosshair is the telegraph, and a Battery below the Hero stops marking.
- **Stage authoring rule:** no more than 2 Batteries on screen at once
  (`vertical_shooter.enemies.flak.max_on_screen: 2`), so crosshairs never
  pile up into a trap.

### 9. Mote: the swarm (Stage 2)

The Static itself, shared with Cabinet 1: a cloud of 10 specks
(`mote.formation_size: 10`) within 40 px (`mote.cloud_radius: 40.0`).

- It appears at the **top** edge.
- Each Mote homes on the Hero at 65 px/s (`mote.speed: 65.0`), turning at
  90°/s (`mote.turn_rate: 90.0`).
- A Mote dissolves after 8 s (`mote.lifetime: 8.0`) for no points.
- Killing the whole cloud is a Formation clear.

The same brain as Cabinet 1's, with its own tuning section. Here the cloud
arrives during air fronts while Rockets are busy below, so the gun has to
choose.

### 10. Tether: the hazard (Stage 2)

Two balloons, one near each side edge, with a 4 px wide energy line between
them (`tether.line_width: 4.0`). They drift down with the scroll.

- The line is live for 1.2 s (`tether.on_seconds: 1.2`) and off for 0.8 s
  (`tether.off_seconds: 0.8`). It flickers for 0.3 s before it switches on
  (`tether.telegraph_seconds: 0.3`).
- Touching the live line is a hit, and the live line stops Hero shots.
- Live, the line is solid Foam; while warning, it is a faint Foam flicker
  (the Arc Gate's look, for the same reason: no Hero colours on a hazard).
- Destroying either balloon (40 HP, 600 points each) cuts the line for good.
  The balloons sit by the side edges, so killing one means going to an edge.
- **Placement is required:** the Stage file gives the balloons' x positions
  with `left_x` and `right_x` on the spawn entry. A Tether spawn missing
  either is skipped with a warning, so there is no hidden default (the
  lesson from #47's Arc Gate fallback).

It shares its brain with the Arc Gate, rotated. In a Cabinet with no walls,
Tethers are what squeeze the screen: a line to cross in its gap or cut at an
edge.

## Stage 1: Cloudline

- **Theme:** above the cloud sea, at first light, heading for the plateau.
  - Far layer: slow cloud banks (Plum and Steel).
  - Ground: Clay mesas pushing up through the clouds, with Static roads
    scratched across them. These are the ground fronts.
  - The Spire is a thin Steel needle on the horizon in the last half minute.
- **Scroll:** 24 px/s, rising to 28 px/s from 2:00 (Stage file).
- **Length:** about 3:20. That is 120 s at 24 px/s plus 80 s at 28 px/s,
  about 5,120 px of scroll.
- **Restart Points:** 0:00, 0:54, 1:58, 2:56, each at the start of a quiet
  stretch between fronts.

| Time | Front | Beat | What happens |
|---|---|---|---|
| 0:00 | — | calm | Restart Point. The Wren rises out of the shaft mouth (the end of #16's Transition), and the music starts. |
| 0:03 | air | teach | One Wisp column, centre → **G** (gun 2). |
| 0:09 | air | test | Two Wisp columns, left then right → **P**. |
| 0:18 | ground | teach | The first mesa slides into view: one Crawler convoy on a road, nothing else → **R**. |
| 0:30 | ground | test | A mesa field: two convoys on crossing roads and the first Silo → Formation bonus. |
| 0:48 | — | breathe | Open cloud. |
| 0:54 | — | calm | **Restart Point.** |
| 0:58 | air | teach | One Bellows glides in from the right. |
| 1:06 | air | test | Bellows from the left with a Wisp column → **G** (gun 3). |
| 1:18 | ground | teach | A lone mesa with one Riser launcher. Its pods rise one by one, and nothing else is on screen. |
| 1:30 | both | set piece | **Loom**, with a Riser pair on the mesas beneath it → **B**. |
| 1:52 | — | breathe | The cloud sea opens wide. |
| 1:58 | — | calm | **Restart Point.** Scroll rises to 28 px/s. |
| 2:02 | air | teach | Swift chevrons, then one Swift pair from below. |
| 2:12 | ground | test | Riser launchers on two mesas with a Crawler convoy between → **S**. |
| 2:26 | both | twist | Swifts from below while Riser pods rise ahead → **W**. |
| 2:40 | air | test | Wisp columns with a Bellows from each side → **P** (second pod). |
| 2:52 | — | breathe | Short. |
| 2:56 | — | calm | **Restart Point.** |
| 3:00 | both | set piece | **Loom** escorted by a Crawler convoy and Silos below → **G** (gun 4). |
| 3:20 | — | end | The clouds thin and the Spire shows on the horizon. The scroll ends: Stage clear. |

A player who clears everything leaves Stage 1 with gun 4, 2 Pods, Rockets,
Speed 1, the Ward and 3 bombs. A typical player finishes at gun 3 with one
Pod. The set pieces sit at 1:30 and 3:00, the breathers fall between fronts,
and every front is either mostly ground or mostly air, so the Stage has its
own rhythm rather than Cabinet 1's.

## Stage 2: The Outer Yard

- **Theme:** the sprawling outer works of the Static, across the plateau.
  - Ground: a Clay floor of rails, cranes and half-built segments, with Plum
    shading and Steel scaffolds. These are the ground fronts.
  - Open sky over the gaps between yard blocks forms the air fronts.
  - The Spire stays on the horizon. It gets bigger through the Stage but is
    still far off at the end.
- **What's new:**
  - Flak Batteries, Mote clouds and Tethers.
  - Bullets 15% faster.
  - Shorter breathers (about 4 s).
- **Scroll:** 26 px/s, rising to 32 px/s for the run-out (Stage file).
- **Length:** about 3:36. That is 192 s at 26 px/s plus 24 s at 32 px/s,
  about 5,760 px of scroll.
- **Restart Points:** 0:00, 0:50, 1:46, 2:40, after each set of fronts.

| Time | Front | Beat | What happens |
|---|---|---|---|
| 0:00 | — | calm | Restart Point. The yard's outer fence passes below. |
| 0:04 | ground | teach | One Flak Battery alone. |
| 0:12 | both | test | A Flak Battery with Wisp columns overhead → **G**. |
| 0:24 | ground | test | A yard block of Riser launchers and a Silo pair, where Rockets shine → **S**. |
| 0:42 | — | breathe | A crane passes underneath. |
| 0:50 | — | calm | **Restart Point.** |
| 0:54 | air | teach | First Mote cloud, in open sky. |
| 1:02 | air | teach | One Tether, alone. |
| 1:10 | air | test | A Tether with a Mote cloud behind it → **W**. |
| 1:20 | both | set piece | **Loom** (Stage 2 strength) over a pair of Flak Batteries → **B**. |
| 1:42 | — | breathe | Short. |
| 1:46 | — | calm | **Restart Point.** |
| 1:50 | both | twist | Swifts from below while a Flak Battery marks the top → **P**. |
| 2:04 | ground | test | Crawler convoys, Riser launchers and a ring of Silos → **G**. |
| 2:20 | air | test | Two Tethers 60 px apart with Wisp columns between them → Formation bonus. |
| 2:34 | — | breathe | Sparks fall from the cranes (scenery only). |
| 2:40 | — | calm | **Restart Point.** |
| 2:44 | both | test | The gauntlet: a Flak pair, Riser launchers and a Tether, then a Mote cloud → **G**. |
| 3:12 | — | run-out | Scroll rises to 32 px/s. Harmless flak bursts light the sky (scenery) as the yard's far edge approaches. No new enemies. |
| 3:36 | — | end | The scroll ends at the yard's edge, where the plateau drops away: Stage clear (bonuses paid). Then a scripted flak burst hits the Wren, starting the 2 → 3 Cabinet Transition (#21). |

**The scripted hit is not a lost life.** It plays after the Stage clear and
its bonuses, as part of the Transition. It costs no life and no score, and
the no-miss bonus has already been paid. Power-ups are cleared at the
Transition anyway.

For a perfect player, the second S and the fourth bomb are new in Stage 2.
G, P and R are maxed and become medals, and W is new only if the Ward was
used up. A player who lost lives gets the same drops as a way back up.

## Scoring

| Event | Points | Key |
|---|---|---|
| Each enemy | its score (see the enemy table) | `vertical_shooter.enemies.<name>.score` and the Loom/Riser keys |
| Formation cleared with no Power-up | 100 × members | `vertical_shooter.score.formation_bonus_per_enemy: 100` |
| Power-up picked up when already maxed | 1,000 | `vertical_shooter.score.maxed_power_up: 1000` |
| Bullet cancelled by the Squall | 10 each | `vertical_shooter.score.bomb_cancel: 10` |
| Stage clear | 3,000 | `vertical_shooter.score.stage_clear: 3000` |
| No-miss bonus | 5,000 | `vertical_shooter.score.no_miss: 5000` |

The bonuses are paid as in Cabinet 1.

**Measured totals** (#15; the test `extra_lives_land_where_the_design_doc_says`
in `vertical_shooter/balance.rs` recomputes them from the Stage files and
tuning). A perfect Stage, with bullets cancelled by the Squall left out:

| | Enemies | Formation bonus | Clear + no miss | Core total | Medals | **Perfect** | Riser pods milked | Loom Wisps milked | Most |
|---|---|---|---|---|---|---|---|---|---|
| Stage 1 | 22,100 | 3,500 | 8,000 | 33,600 | 0 | **33,600** | +2,100 | +2,000 | 37,700 |
| Stage 2 | 27,300 | 3,700 | 8,000 | 39,000 | +5,000 | **44,000** | +1,800 | +1,000 | 46,800 |

- **Medals** come from collecting every drop in order from a fresh start
  with no life or bomb lost. Stage 1's drops (G P R G B S W P G) max
  nothing. By Stage 2 the gun, Pods and Ward are full, so 5 of its 7 drops
  (G, W, P, G, G; not S or B) pay medals.
- **Milked** means leaving a Riser launcher alive while it launches (each pod
  shot while climbing), or shooting a Loom's shuttles off as late as still
  leaves time to kill its core before it retreats.
- Both Stages come in a little under the first rough targets (35–45k and
  45–55k). They were left there, because the extra lives land where they
  should: Cabinet 1's perfect core is 60,800, so a strong player starts here
  past the second extra life (70,000) only with Cabinet 1's extras. A
  perfect Stage 1 brings them to 94,400, and Stage 2 to 138,400. So the
  third extra life (120,000) comes late in Cabinet 2, during Stage 2, even
  with everything milked in Stage 1 (116,700 at most before Stage 2). A
  decent player reaches it early in Cabinet 3.

## Sound and music (for #15)

- **Stage 1 track:** about 140 BPM, a major-key lift after Cabinet 1's minor
  key: a wide, rising square lead over a pulsing bass. Open sky.
- **Stage 2 track:** about 165 BPM, tense, with a repeating low riff and
  heavier noise percussion. In the last 20 s it should build like an alarm,
  toward the hit.
- **Effects to cover:**
  - Hero and weapons: Hero shot (as in Cabinet 1), lance, rocket launch (a
    soft hiss, since it plays twice a second), the Squall.
  - Power-ups: each letter's chirp, and the maxed medal.
  - Enemy hits: spark, small and large explosions.
  - Warnings: the Swift chevron, the flak crosshair beep and its burst.
  - Enemy machinery: Riser launch thump and the rising whistle as a pod
    climbs, the Loom shield breaking and the shuttles falling away, the Loom
    spiral (a soft repeating tick), Silo hatch clank, Tether hum.
  - Ward break, lost life, the "STAGE CLEAR" sting, and the final hit
    (the Transition's first sound).

## Building it (for #14)

Cabinet 1's code is the model. Reuse it where the genre is the same, and
write new code only where Cabinet 2 differs.

- **Where it lives:** `src/cabinets/vertical_shooter/`, gated by
  `cabinet_is(1)`, laid out like `side_shooter/`: `hero.rs`, `power_ups.rs`,
  `palette.rs`, `tuning.rs` and `enemies/{mod,brain,look,formation}.rs`.
  `tuning.rs` adds one `vertical_shooter` field to `Tuning`.
- **The kit, unchanged:**
  - Stage timeline and `SpawnRequested`, Restart Point rewinds.
  - `Health`, `Hurt`, `Immune` and `EnemyDestroyed`. The Loom core's shield
    and closed Silos use `Immune`.
  - `projectile`, `Shield`, `HitStop` / `stage_live` and the Power-up capsule
    pattern (`power_up:<letter>` spawn kinds).
- **One kit addition:** a ground layer that Hero shots and rockets hit and
  nothing else touches. Ground targets use it. A Riser pod moves from the
  ground layer to the enemy layer when its climb ends.
- **Lift rather than copy** these, into the kit or a shared
  `cabinets/shooter/` module, and update Cabinet 1 to use them in the same
  change:
  - Formation tracking and the Power-up drop rule
    (`side_shooter/enemies/formation.rs`).
  - The fairness gate in `think`: grace, the bullet cap, the no-spawn radius,
    and "never from behind" with the direction as a parameter. The minimum
    distance should be measured from where each bullet appears (see the
    fairness rules above). For Cabinet 1 that is the same as today.
  - The brain helpers `fan`, `turn_toward` and `bob`.
  - The Mote brain as is, the Hook brain as the Swift, and the Arc Gate as
    the Tether, each with the scroll axis as a parameter.
- **Write new:**
  - The Wisp, Bellows, Loom (three entities and a shield), Crawler, Silo,
    Riser and Flak brains, as pure state machines with fixed-step unit tests,
    like Cabinet 1's `brain.rs`.
  - The Wren's nose-up movement, Pods, Rockets and the Squall.
  - Cabinet 2's own `PowerUps` type, with the JSON shape above, stored in
    `Session.power_ups` the same way.
- **Gallery:** an enemy gallery Stage (`assets/stages/2-gallery.ron`), shown
  with `?gallery`, as in Cabinet 1.

## Decisions made while building the gameplay (#14)

- **Where the code is:** `src/cabinets/vertical_shooter/`. The shared pieces
  moved to `src/cabinets/shooter/`, which Cabinet 1 now uses too:
  - Formations and the Power-up drop rule (`formation.rs`).
  - The pattern helpers `dir`, `fan`, `ring`, `turn_toward`, `bob` and
    `home` (`pattern.rs`).
  - The on/off `Pulse` behind both the Arc Gate and the Tether.
- **Two new kit layers:**
  - `Layer::Ground` is hit by Hero shots and rockets, and never touches the
    Hero.
  - `Layer::Rocket` hits ground targets only.
- **The Swift has its own brain, not the Hook's with an axis parameter.**
  The axis mapping took more code than the Swift's own state machine, so the
  two share the shape and the numbers, not the code.
- **The Mote uses the shared `home` step**, with its own tuning section,
  rather than Cabinet 1's `Mote` type.
- **A Riser pod is a ground target while it climbs.** When it takes off it
  switches to `Layer::Enemy` and its score drops from 150 to 50.
- **The fairness gate is one function, `may_fire`.** It checks grace, "not
  below the Hero", the minimum distance from where the bullets appear, and
  the bullet cap, all or nothing.
  - A pod that bursts too close fizzles.
  - A flak burst throws its ring only when the Hero is 40 px or more away.
  - Cabinet 1's own gate is unchanged.
- **The flak burst's hit zone is a square** 2 × `burst_radius` on a side,
  on the enemy-shot layer, for `burst_seconds`. A Pod or the Squall can stop
  it, like any bullet.
- **The Squall stops at the play area.** Each edge stops at its own
  boundary (the top of the play area, the bottom of the screen). It only
  touches what is at least partly inside the play area, so enemies waiting
  off screen (a Swift during its warning, a Loom arriving, a Bellows beside
  the edge) are spared (review of #60).
- **A Wisp curves toward the Hero's side of the Wisp itself**, not the
  Hero's half of the screen. When both are on the same half, it curves
  across the Hero's path rather than away from it, which makes the column a
  real threat.
- **A Flak Battery's first mark comes half an interval after it appears**
  (1.2 s with `flak.fire_interval: 2.4`), then every interval.
- **Rockets launch while A is held**, alongside the gun, as Cabinet 1's Tail
  fires with the gun. They don't fire on their own.
- **A Silo's cycle is closed 2.4 s, then the 0.3 s flash, then open 1.2 s**
  (3.9 s in all), so its first opening is 2.7 s after it appears.
- **Power-ups read from a Save are clamped** to their maximums.
- **Each Loom shuttle launches one Wisp per launch.** Launched Wisps belong
  to no Formation.
- **Sprites:** #15's `c2_*` sprites draw everything. Frames show the state:
  - the Bellows vent glows, and a Riser pod flashes, before they fire;
  - a Silo goes closed, then warning, then open;
  - a Loom core shows shielded, then open;
  - a Crawler's turret turns and flashes before each shot.

  Air sprites, the Wren and the Pods cast `Sprites::shadow` silhouettes.
  Swifts turn to face their path.
- **The Stage 2 bonuses and the final scripted hit are not part of this
  ticket.** The Stage-clear and no-miss bonuses wait, as Cabinet 1's do (#9).
  `Sfx::FinalHit` belongs to the 2 → 3 Transition (#21).

## Stage files and art (#15)

What #15 built, and the contract #14's code keeps with it.

- **Files:** `assets/stages/2-1.ron` (3:20) and `2-2.ron` (3:36), one line
  per beat of the tables above, each with its time at normal speed in a
  comment. The files were generated once from these tables by a throwaway
  script; edit the RON files from now on. `src/cabinets/stage_checks.rs`
  tests them against this doc: the lengths, the Restart Points, the
  Power-up order, the known enemy kinds, Tethers' `left_x`/`right_x`,
  Bellows' `side`, and no more than 2 Flak Batteries on screen.
- **Enemy kinds and params:** `wisp`, `swift`, `bellows` (`side`), `loom`,
  `crawler` (`heading`), `silo`, `riser`, `flak`, `mote`, `tether`
  (`left_x`, `right_x`), with `drop` and `size` as in Cabinet 1.
- **Where things are:** `pos` is where an enemy is when the scroll reaches
  `at`. Ground targets then move down with the ground at the scroll speed.
  They spawn just above the top edge on their mesa or yard floor, and they
  stay lined up because the scenery moves the same way.
- **Scenery** is listed in the same files as `scenery:<sprite>` entries
  (`src/kit/scenery.rs`). Each piece is placed from the scroll position every
  frame, so a rewind needs nothing special. The far cloud banks move at
  0.3–0.35 of the scroll, a mid cloud layer at 0.6, and the ground (the
  cloud sea, mesas, roads, the yard floor and its props) at 1.0. The Spire
  barely moves (0.02–0.05).
  - Stage 1: a cloud sea of Steel puffs with Plum undersides. The mesas are
    Clay plateaus with Plum slopes and a cliff face, and roads are
    scratched across them under the Crawlers.
  - Stage 2: a continuous yard floor of Clay plates in rows across the full
    width under every ground front. Rail lines run through each front, with
    scaffolds, half-built segments and cranes on the floor. Open sky shows
    over the two air fronts (0:50–1:16 and 2:18–2:30), and the yard ends at
    a Clay lip over a Plum cliff.
  - Sparks and the run-out flak bursts are scenery, so they use only Steel
    and Clay, never Ember or Volt.
- **Sprites** are `assets/sprites/c2_*.txt`, plus `enemy_bullet`, `rocket`
  and `chevron_up`, which other Cabinets can share. The Wren uses the shared
  `hero` sprite turned 90° (Foam, Orchid canopy, Ember engine come from the
  palette roles). Frames: the Bellows vent glows; the Loom core is shielded
  then open; the shuttle has a burning-wreck frame; the Silo is closed, then
  warning, then open; the Riser pad lights; the Crawler turret is separate
  so it can turn, and flashes. `Sprites::shadow` draws any sprite as a flat
  Dusk silhouette for the air-enemy and Wren shadows. Background sprites
  carry a `#! background` line, and a test holds them to Dusk, Plum, Steel
  and Clay.
- **Unattended run:** `scripts/stages-web.sh 2` plays both Stages with the
  invincible Hero and a faster clock, and fails unless each one clears in
  about its designed time. `scripts/smoke-web.sh` runs it too.

## Revisit after the Cabinet 1 playtest

These numbers were copied from Cabinet 1, or set relative to it, before
anyone has played Cabinet 1. When #12's comments change the Cabinet 1 value,
change the matching one here too, or record why not:

- Hero speed (100 px/s) and Speed step (+20), the hitbox (4×4), fire rate
  (10 volleys/s) and shot speed (360 px/s).
- Bomb counts (start 2, max 4), the bomb's 12 damage and 1.0 s
  invulnerability.
- Losing one step of power per life, and the rewind replaying Power-ups.
- The role HP and time-to-kill targets (popcorn 1–2, gunner 36, power check
  40). The Loom's totals follow the Ferry's set-piece target.
- Bullet readability: if Signal-on-Night was hard to read, check Volt on Dusk
  and Clay the same way.
- The bullet-speed caps, which are set to match Cabinet 1's time to react
  (rule 2). If Cabinet 1's bullets feel too slow or fast, move both.
- The Hook's dodge feel, since the Swift is the same brain.
- How much breathing room the 4–6 s breathers really give.

## Proposed `tuning.ron` section

For #14 to copy. The values are the ones above.

```ron
vertical_shooter: (
    stage2_bullet_speed_scale: 1.15,
    look: (shadow_offset: (6.0, -6.0)),
    hero: (
        hitbox: (4.0, 4.0), speed: 100.0, speed_step: 20.0, edge_margin: 4.0,
        start: (0.0, -80.0), entry_from_y: -136.0, entry_seconds: 1.0,
        respawn_invulnerable_seconds: 2.0, hit_stop_seconds: 0.15,
    ),
    shot: (
        fire_interval: 0.1, speed: 360.0, damage: 1, size: (2.0, 8.0),
        twin_spacing: 6.0, spread_degrees: 15.0,
        lance_size: (3.0, 12.0), lance_damage: 2, max_on_screen: 96,
    ),
    rocket: (count: 2, interval: 0.5, launch_offset: (5.0, 0.0), size: (3.0, 6.0),
             speed: 200.0, turn_rate: 270.0, damage: 3),
    bomb: (starting: 2, max: 4, band_speed: 420.0, damage: 12, invulnerable_seconds: 1.0),
    power_up: (
        size: (10.0, 10.0), pickup_box: (14.0, 14.0), edge_inset: 16.0,
        drift_speed: 30.0, bob_amplitude: 3.0, bob_period: 1.2,
        gun_max: 4, pod_max: 2, pod_offset: 14.0, speed_max: 2,
        ward_invulnerable_seconds: 1.0,
    ),
    enemy: (
        bullet_size: (5.0, 5.0), bullet_hitbox: (2.0, 2.0), hitbox_inset: 1.0,
        fire_grace_seconds: 0.5, min_fire_distance: 40.0, max_bullets: 40,
        no_spawn_radius: 48.0, bullet_lifetime: 12.0,
    ),
    enemies: (
        wisp: (hp: 1, score: 100, formation_size: 6, spacing_seconds: 0.3, speed: 80.0,
               turn_y: 20.0, turn_rate: 150.0, exit_speed: 110.0, launched_speed: 100.0),
        swift: (hp: 2, score: 200, formation_size: 2, warning_seconds: 0.75,
                chevron_blink_hz: 4.0, speed: 150.0, pass_offset: 50.0, pair_spread: 0.5,
                edge_clearance: 12.0, turn_y: 80.0, turn_radius: 14.0,
                dive_speed: 170.0, dive_steer_speed: 85.0),
        bellows: (hp: 36, score: 500, speed: 80.0, brake_distance: 24.0, brake_floor: 0.15,
                  stop_inset: 60.0, hover_seconds: 3.0, sway_amplitude: 2.0, sway_period: 1.5,
                  telegraph_seconds: 0.4, burst: 3, burst_interval: 0.12, bursts: 2,
                  fire_interval: 1.4, shot_speed: 90.0, exit_speed: 80.0),
        loom: (shuttle_hp: 60, shuttle_hp_stage2: 100, shuttle_score: 500,
               core_hp: 240, core_hp_stage2: 400, core_score: 2000,
               shuttle_offset: 28.0, speed: 20.0, hold_y: 60.0, hold_seconds: 20.0,
               retreat_speed: 20.0,
               first_launch_delay: 1.0, launch_interval: 3.0,
               telegraph_seconds: 0.4, pair_interval: 0.15, shot_speed: 80.0,
               fire_interval: 2.5,
               spiral_arms: 3, spiral_interval: 0.3, spiral_turn_rate: 40.0,
               spiral_speed: 60.0, spiral_seconds: 2.0, spiral_pause: 2.0),
        crawler: (hp: 10, score: 300, formation_size: 3, spacing: 18.0, speed: 12.0,
                  turn_rate: 120.0, telegraph_seconds: 0.3, shot_speed: 80.0,
                  fire_interval: 2.0, stagger: 0.4),
        silo: (hp: 16, score: 800, closed_seconds: 2.4, telegraph_seconds: 0.3,
               open_seconds: 1.2, ring_count: 6, shot_speed: 70.0),
        riser: (hp: 16, score: 400, first_launch_delay: 1.0, launch_interval: 3.5,
                climb_seconds: 1.0, pod_score_climbing: 150, pod_score_airborne: 50,
                drift_speed: 40.0, fuse_seconds: 1.2, telegraph_seconds: 0.3,
                ring_count: 6, shot_speed: 70.0),
        flak: (hp: 24, score: 600, fire_interval: 2.4, warning_seconds: 0.6,
               burst_radius: 10.0, burst_seconds: 0.15, burst_count: 6,
               burst_speed: 60.0, max_on_screen: 2),
        mote: (hp: 1, score: 50, formation_size: 10, cloud_radius: 40.0, speed: 65.0,
               turn_rate: 90.0, lifetime: 8.0),
        tether: (hp: 40, score: 600, line_width: 4.0, on_seconds: 1.2, off_seconds: 0.8,
                 telegraph_seconds: 0.3),
    ),
    score: (formation_bonus_per_enemy: 100, maxed_power_up: 1000, bomb_cancel: 10,
            stage_clear: 3000, no_miss: 5000),
)
```

## Decisions and reasons

- **Same Hero numbers as Cabinet 1.** The Wren is the same ship, and the
  player's hands already know it. Only the facing, the canopy colour and
  the weapons' shapes change.
- **Same role targets, different enemies and pacing.** The time-to-kill roles
  are a shared language, so power feels consistent. But the beat clock,
  Restart Points, set-piece timing and most enemy behaviour are this
  Cabinet's own, built on ground and air fronts, so a Cabinet 1 player can't
  predict the next beat.
- **Volt bullets instead of magenta.** The Cabinet accent fixed in
  `CABINETS[1]` is pink, which would blur with magenta bullets. The rule "one
  colour for danger" stays; the colour changes with the palette, as DESIGN.md
  intends.
- **Air and ground layers, told apart by hull colour and shadow.** That is
  the genre's defining idea. Ground targets that can't ram you make flying
  over a crowded ground feel bold rather than unfair.
- **The Riser crosses the layers.** A pod that is ground while it climbs and
  air once it is up makes the split something the player acts on, not just
  reads: shoot it early for more points and no risk.
- **A two-phase Loom.** The shuttles have to come off before the core opens,
  and a ground front sits under it, so the set piece is fought in both
  layers and plays differently from the Ferry.
- **Flak rings never appear point-blank.** The minimum distance is measured
  from where a bullet appears, so an area attack can't break the rule its
  firing enemy keeps.
- **No terrain collision.** A top-down sky has no walls. Tethers and Flak
  Batteries supply what Cabinet 1's Stage 2 walls did.
- **Rockets instead of the Tail, Pods beside instead of above and below.**
  Each Power-up answers this genre's threats. G, S, W and B stay identical so
  the Cabinet still feels familiar.
- **Bullet speeds matched to Cabinet 1's time to react.** The vertical play
  area is shorter, so the cap is lower to keep about 2.1 s from the top of
  the screen to the Hero.
- **A quick power ramp at the start.** Power-ups reset at the Transition, so
  the first three Formations all drop one.
- **The hit that ends Stage 2 is scripted and free,** and it happens over
  the yard's edge, far from the Spire. DESIGN.md has the ship take a hit at
  the 2 → 3 Transition, and keeping the Spire distant leaves Cabinet 3 its
  fight back to the tower.
- **No new glossary terms.** "Air enemy", "ground target" and "front" are
  Cabinet 2 ideas, defined here, like Cabinet 1's enemy names. The glossary
  keeps to words that hold across the whole Run.
