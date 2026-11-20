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
(#12). Numbers copied from Cabinet 1 are marked in
[Revisit after the Cabinet 1 playtest](#revisit-after-the-cabinet-1-playtest);
when #12's feedback changes a Cabinet 1 number, check the matching one here.

## Premise

> The Static pours up the shaft and out onto a high plateau above the clouds.
> There it is building a tower out of everything it has taken: **the Spire**.
> The Wren follows the swarm over the cloud line and into the construction
> yard at the Spire's foot, where the Static's flak finally finds it.

- Stage 1 starts as the Wren bursts out of the top of Cabinet 1's shaft, which
  continues the 1 → 2 Cabinet Transition (#16).
- Stage 2 ends with the Wren taking a hit at the foot of the Spire. That hit
  is the start of the 2 → 3 Cabinet Transition (#21): the Hero ejects and
  parachutes down to fight on foot (Cabinet 3).
- The Spire is the tower the Hero later climbs (the 3 → 4 Cabinet
  Transition). Showing it in the distance here plants it early.

## What makes it feel good (the rules behind the numbers)

Cabinet 1's nine rules all apply: bullets you can read, enemies you can see,
every threat announced, a small hitbox, teach-test-twist, waves that breathe,
power arriving steadily, a lost life that dents rather than wipes out, and HP
from time-to-kill targets. These change or are added for the vertical genre:

1. **You can always read the bullets, in this Cabinet's colours.** Enemy
   bullets are Volt (lime), a colour nothing else uses except enemy weak
   points. They are round with a Foam core. Magenta can't be used: the
   Cabinet's accent (`CABINETS[1].accent`) is pink.
2. **Less room to react, so slower threats.** The screen is 320 wide but the
   play area is only 228 tall, and threats now come down that short side.
   Cabinet 1 gave about 1.6 s of travel across the screen at 100 px/s. Here,
   no enemy bullet in Stage 1 is faster than 90 px/s, and none in Stage 2 is
   faster than 105 px/s. Enemies enter more slowly than their Cabinet 1
   counterparts.
3. **Air or ground, you can tell at a glance.** Every air enemy, and the
   Wren, casts a Dusk shadow 6 px below and to the right on the ground layer
   (`vertical_shooter.look.shadow_offset: (6.0, -6.0)`). Ground targets have
   no shadow. **Anything with a shadow can hit you by touching you; ground
   targets can't,** because the Wren flies over them. Hero shots hit both.
4. **No aimed shot from behind.** Once an enemy is below the Wren (the Hero
   has passed it), it never fires again. That covers ground targets after
   they scroll past and any air enemy that drops below the Hero.
5. **The power reset is short.** Power-ups reset at the Cabinet Transition, so
   a strong Cabinet 1 player starts here at gun 1. The first two Formations
   both drop Power-ups, so gun 2 and a Pod arrive within about 15 s.

## Screen

- 320×240, the play area y ∈ [−120, +108] under the shared HUD, as in
  Cabinet 1. **Full width, no side panels:** a wide field gives room to dodge
  sideways, which suits a vertical shooter played on a 4:3 screen.
- The Hero is clamped inside the play area with a 4 px margin
  (`vertical_shooter.hero.edge_margin: 4.0`).
- Bomb icons sit at the bottom-left of the play area, as in Cabinet 1.

## Palette

One Cabinet palette (#4), shared by both Stages. Index 0 is transparent.

| # | Name | Hex | Used for |
|---|---|---|---|
| 1 | Dusk | `#140019` | backdrop (matches `CABINETS[1].backdrop`), shadows |
| 2 | Plum | `#3A1F4F` | cloud shading, far layer, ground shading |
| 3 | Steel | `#5A6E8C` | clouds, scaffolding, **air enemy hull fill** |
| 4 | Clay | `#8C5A3C` | mesas, the yard floor, **ground target hull fill** |
| 5 | Orchid | `#FF66CC` | accent (matches `CABINETS[1].accent`), Hero canopy, Power-up rims |
| 6 | Foam | `#F4F4F0` | Hero body, **enemy outlines**, bullet cores, Power-up letters, text |
| 7 | Ember | `#FFB43C` | Hero shots, rockets, Hero engine, explosions |
| 8 | Volt | `#C8FF3C` | **enemy bullets, enemy weak points and flak crosshairs only** |

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
ground, so there is no terrain hitbox and no walls. The danger comes from
density instead, and Stage 2's narrow-corridor feel comes from Tethers (see
Enemies).

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

A band of force sweeps out from the Wren's row, up and down at once, across
the full width.

- Two edges move away from the Wren's row at 420 px/s
  (`vertical_shooter.bomb.band_speed: 420.0`) until they reach the top and
  bottom of the play area. The whole screen is covered in under 0.6 s.
- Every enemy bullet the band touches is cancelled for 10 points
  (`vertical_shooter.score.bomb_cancel: 10`).
- Every enemy the band touches, **air or ground**, takes 12 damage once
  (`vertical_shooter.bomb.damage: 12`). That kills Wisps, Kites, Swifts, Motes
  and Crawlers. Bellows, open Silos, Flak Batteries, Looms and Tether balloons
  survive but are hurt. A **closed Silo takes no damage**.
- 1.0 s of invulnerability from the moment B is pressed
  (`vertical_shooter.bomb.invulnerable_seconds: 1.0`).
- **Count:** start with 2 (`vertical_shooter.bomb.starting: 2`), hold at most
  4 (`vertical_shooter.bomb.max: 4`), topped up to `starting` after a lost
  life, carried from Stage 1 to Stage 2. B does nothing while a band is on
  screen. All as in Cabinet 1.

A band rather than Cabinet 1's ring because it matches the screen's shape:
the threats in a vertical shooter are spread in rows across the width.

## Power-ups

### How they're earned

Exactly as in Cabinet 1: Formations drop the Power-up their spawn entry names
or else pay a Formation bonus, a member that leaves alive spoils the
Formation, and the pickup rules are the same.

- **Ground Formations count.** A convoy of Crawlers is a Formation like any
  other, so shooting the ground pays.
- A Loom counts as a one-member Formation and always drops its Power-up.
  Wisps a Loom launches belong to no Formation.
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
  all-round gun, and R is the answer to a crowded ground.

**Rockets:**

- 2 per launch (`vertical_shooter.rocket.count: 2`), every 0.5 s
  (`vertical_shooter.rocket.interval: 0.5`).
- Each is a 3×6 Ember dart (`vertical_shooter.rocket.size: (3.0, 6.0)`)
  launched from the Wren's wings. It flies at 200 px/s
  (`vertical_shooter.rocket.speed: 200.0`) and turns at up to 270°/s
  (`vertical_shooter.rocket.turn_rate: 270.0`).
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
target 32 px or more **wide** within about 90 px.

| Power | Narrow DPS | Large DPS | Ground bonus with Rockets |
|---|---|---|---|
| Gun 1 | 10 | 10 | +12 |
| Gun 2 | 20 | 20 | +12 |
| Gun 3 | 20 | 40 | +12 |
| Gun 4 | 40 | 60 | +12 |
| each Pod | +0 | +10 | |

Expected power ("perfect" means every Formation cleared and no life lost;
"typical" means one or two missed and one life lost). Cabinet 2 starts from
nothing, so the Stage 1 curve is steeper than Cabinet 1's.

| When | Perfect | Typical |
|---|---|---|
| Stage 1, 0:40 (first Crawlers) | Gun 2 + 1 Pod | Gun 2 |
| Stage 1, 1:14 (first Bellows) | Gun 3 + 1 Pod + Rockets: narrow 20, large 50 | Gun 2 + 1 Pod: narrow 20, large 30 |
| Stage 1, 1:55 (first Loom) | Gun 3 + 1 Pod: large 50 | Gun 2 + 1 Pod: large 30 |
| Stage 1, 3:02 (finale Loom) | Gun 3 + 2 Pods: large 60 | Gun 3 + 1 Pod: large 50 |
| Stage 2 | Gun 4 + 2 Pods: narrow 40, large 80 | Gun 3 + 1 Pod: narrow 20, large 50 |

Time-to-kill targets by role, and the HP that follows:

| Role | Enemies | Target | HP | Check |
|---|---|---|---|---|
| Popcorn | Wisp, Kite, Mote, Swift | under 0.3 s even at gun 1 | 1, 1, 1, 2 | Swift: 0.2 s at gun 1 |
| Small ground | Crawler | about 0.5 s at typical power | 10 | 0.5 s at narrow 20; 0.3 s with Rockets; one Squall kills it |
| Gunner | Bellows | 1.5–2 s, so it gets a burst off | 36 | 1.8 s at narrow 20, 0.9 s at gun 4 |
| Timing lock | Silo | dies inside one 1.2 s opening at typical power | 16 | 0.8 s at narrow 20; 0.5 s with Rockets |
| Ground gunner | Flak Battery | about 1 s at typical power | 24 | 1.2 s at narrow 20; 0.75 s with Rockets |
| Power check | Tether balloon | about 2 s at typical power | 40 | 2 s at narrow 20; 4 s at gun 1 |
| Set piece | Loom | 8–10 s at perfect power | 400 in Stage 1, 640 in Stage 2 | Stage 1: 8 s perfect and 13.3 s typical at 1:55; 6.7 s / 8 s at 3:02. Stage 2: 8 s perfect, 12.8 s typical |

These are Cabinet 1's role targets and HP, kept on purpose: a player who
learned Cabinet 1's rhythm finds the same rhythm here.

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
- **Air enemies** (Wisp, Kite, Swift, Bellows, Loom, Mote, Tether) hurt the
  Wren on contact and cast shadows. **Ground targets** (Crawler, Silo, Flak
  Battery) are fixed to the ground and scroll down with it at the scroll
  speed. They never touch the Wren, and they draw under every air sprite.

General fairness rules, as in Cabinet 1 with this Cabinet's directions:

- No enemy fires in its first 0.5 s on screen
  (`vertical_shooter.enemy.fire_grace_seconds: 0.5`).
- No enemy fires within 40 px of the Hero
  (`vertical_shooter.enemy.min_fire_distance: 40.0`).
- No enemy fires once it is below the Hero (rule 4). This applies to every
  pattern, rings included.
- At most 40 enemy bullets at once
  (`vertical_shooter.enemy.max_bullets: 40`).
- Nothing spawns on screen within 48 px of the Hero
  (`vertical_shooter.enemy.no_spawn_radius: 48.0`).

| # | Name | Layer | Stage | Sprite | HP | Score | Default Formation |
|---|---|---|---|---|---|---|---|
| 1 | Wisp | air | 1, 2 | 8×8 | 1 | 100 | column of 6 |
| 2 | Kite | air | 1, 2 | 12×10 | 1 | 100 | V of 5 |
| 3 | Swift | air | 1, 2 | 10×12 | 2 | 200 | pair |
| 4 | Bellows | air | 1, 2 | 20×16 | 36 | 500 | single |
| 5 | Loom | air | 1, 2 | 56×36 | 400 / 640 | 2,000 | single; launches Wisps |
| 6 | Crawler | ground | 1, 2 | 12×12 | 10 | 300 | convoy of 3 |
| 7 | Silo | ground | 1, 2 | 16×16 | 16 | 800 | single |
| 8 | Flak Battery | ground | 2 | 16×12 | 24 | 600 | single |
| 9 | Mote | air | 2 | 4×4 | 1 | 50 | cloud of 10 |
| 10 | Tether | air | 2 | balloons 10×12 | 40 per balloon | 600 per balloon | single |

HP and score keys: `<name>.hp`, `<name>.score`. Default Formation size:
`<name>.formation_size` (1 when not listed).

### 1. Wisp: the falling column

A teardrop of light. A column of 6 (`wisp.formation_size: 6`), 0.3 s apart
(`wisp.spacing_seconds: 0.3`), drops in from the top edge at the spawn's x and
falls at 80 px/s (`wisp.speed: 80.0`). At y = +20 (`wisp.turn_y: 20.0`) each
Wisp curves toward the Hero's half of the screen, turning at 150°/s
(`wisp.turn_rate: 150.0`), and leaves through that side at 110 px/s
(`wisp.exit_speed: 110.0`). **It doesn't fire.** The column is a moving wall:
shoot the head as it comes, or get out of the half it curves into. A Loom
also launches Wisps, which fall straight down at 100 px/s
(`wisp.launched_speed: 100.0`).

### 2. Kite: the swoop

A diamond-winged glider. A V of 5 (`kite.formation_size: 5`), members 14 px
apart horizontally and 10 px vertically (`kite.v_spacing: (14.0, 10.0)`),
glides down at 70 px/s (`kite.speed: 70.0`) to y = +50 (`kite.stop_y: 50.0`),
hangs for 0.6 s (`kite.hang_seconds: 0.6`), then climbs back out of the top at
120 px/s (`kite.exit_speed: 120.0`). In Stage 1 it doesn't fire. **Stage 2
twist:** with `fires: true` on the spawn entry, each Kite flashes for 0.3 s
(`kite.telegraph_seconds: 0.3`) as its hang begins, and fires one aimed shot
at 85 px/s (`kite.shot_speed: 85.0`). The whole V fires at once: a volley of
five to weave through.

### 3. Swift: the ambush from below

A narrow blade, in pairs (`swift.formation_size: 2`). A chevron blinks on the
bottom edge for 0.75 s (`swift.warning_seconds: 0.75`), 4 times a second
(`swift.chevron_blink_hz: 4.0`), under the column where each Swift will appear.
It then **climbs in from behind** at 150 px/s (`swift.speed: 150.0`) along a
column 50 px left or right of the Hero (`swift.pass_offset: 50.0`; one of the
pair each side, larger Formations further out by half an offset per pair,
`swift.pair_spread: 0.5`). The column is clamped to stay 12 px inside the
play area (`swift.edge_clearance: 12.0`). At y = +80 (`swift.turn_y: 80.0`)
it U-turns on a half-circle of radius 14 px (`swift.turn_radius: 14.0`)
toward the Hero's side. It then dives at 170 px/s (`swift.dive_speed: 170.0`),
sliding sideways at up to 85 px/s (`swift.dive_steer_speed: 85.0`) onto the
column the Hero was in when the turn ended. It has no bullets. It rams.

It's Cabinet 1's Hook turned 90°, and the same brain can drive both. Only
the Swift's tuning differs.

### 4. Bellows: the gunner

A squat air barge with a vent on its belly. It comes down from the top at
80 px/s (`bellows.speed: 80.0`), brakes over its last 24 px
(`bellows.brake_distance: 24.0`, never below 15% speed,
`bellows.brake_floor: 0.15`) to stop at y = +70 (`bellows.stop_y: 70.0`), and
hovers for 3.0 s (`bellows.hover_seconds: 3.0`), swaying 2 px sideways
(`bellows.sway_amplitude: 2.0`) every 1.5 s (`bellows.sway_period: 1.5`).

- Its vent glows **as soon as it stops**.
- After 0.4 s (`bellows.telegraph_seconds: 0.4`) it fires a burst of 3 aimed
  shots (`bellows.burst: 3`), 0.12 s apart (`bellows.burst_interval: 0.12`),
  at 90 px/s (`bellows.shot_speed: 90.0`).
- It fires 2 bursts (`bellows.bursts: 2`) 1.4 s apart
  (`bellows.fire_interval: 1.4`), each telegraphed.
- It then leaves sideways at 80 px/s (`bellows.exit_speed: 80.0`), away from
  the Hero.

It stops about 1.0 s after appearing, so its first burst comes at about
1.4 s, just before a typical player kills it (1.8 s).

A burst of three instead of Cabinet 1's single shot because each shot is
slower. Three slow shots in a line read as one threat to step aside from.

### 5. Loom: the carrier

A broad, slow air platform with shuttles on its flanks. The whole hull takes
damage and flashes on each hit.

- **Movement:** it descends from the top at 20 px/s (`loom.speed: 20.0`) to
  hold its centre at y = +60 (`loom.hold_y: 60.0`), about 4 s after appearing.
  It holds there for up to 15 s (`loom.hold_seconds: 15.0`), then retreats
  upward at 20 px/s (`loom.retreat_speed: 20.0`), taking its Power-up with it.
- **Launches:** 2 Wisps (`loom.launch_count: 2`) from its flanks, 24 px out
  and 8 px down (`loom.launch_offset: (24.0, -8.0)`), 1.0 s after appearing
  (`loom.first_launch_delay: 1.0`), then every 3.0 s
  (`loom.launch_interval: 3.0`).
- **The spiral:** 3 arms (`loom.spiral_arms: 3`), one bullet per arm every
  0.3 s (`loom.spiral_interval: 0.3`), with the arms turning at 40°/s
  (`loom.spiral_turn_rate: 40.0`) and bullets at 60 px/s
  (`loom.spiral_speed: 60.0`).
  - It spirals for 2.0 s (`loom.spiral_seconds: 2.0`), then pauses 2.0 s
    (`loom.spiral_pause: 2.0`). The first spiral starts 2.0 s after it
    appears (`loom.first_spiral_delay: 2.0`).
  - A spiral of about 21 slow bullets is a pattern to read and drift through.
    The pause is the window to close in and shoot.
- **HP:** 400 in Stage 1 (`loom.hp: 400`) and 640 in Stage 2
  (`loom.hp_stage2: 640`).

A Loom always drops a Power-up, and one appears near the end of each Stage
half. Like the Ferry, it is a set piece.

### 6. Crawler: the ground convoy

A treaded turret crawling along roads on the ground.

- **Convoy:** 3 (`crawler.formation_size: 3`), 18 px apart
  (`crawler.spacing: 18.0`), moving at 12 px/s relative to the ground
  (`crawler.speed: 12.0`) in the direction the spawn entry gives
  (`params: {"heading": 0.0}`, degrees, 0 = right). On screen this adds to
  the scroll.
- **Turret:** it tracks the Hero at 120°/s (`crawler.turn_rate: 120.0`) and
  flashes for 0.3 s (`crawler.telegraph_seconds: 0.3`). It fires one shot
  **along its turret** at 80 px/s (`crawler.shot_speed: 80.0`) every 2.0 s
  (`crawler.fire_interval: 2.0`), so fast sideways movement beats it, as with
  the Barnacle.
- Members' timers are offset by 0.4 s (`crawler.stagger: 0.4`), so a convoy
  fires a ripple, never one wall.

### 7. Silo: the timing lock

An armoured hatch in the ground.

- It is closed for 2.4 s (`silo.closed_seconds: 2.4`), immune to all damage,
  the Squall included. Shots spark off it.
- It flashes for 0.3 s (`silo.telegraph_seconds: 0.3`), then opens for 1.2 s
  (`silo.open_seconds: 1.2`) to show a Volt core.
- As it opens, it fires a ring of 6 (`silo.ring_count: 6`) at 70 px/s
  (`silo.shot_speed: 70.0`).
- **It only takes damage while open.** Its cycle starts when it scrolls on
  screen, so the first opening comes about 2.7 s in.

### 8. Flak Battery: the area denier (Stage 2)

A ground gun that shoots at where the Wren *is*, not at the Wren.

- **Targeting:** every 2.4 s (`flak.fire_interval: 2.4`) it marks the Hero's
  current position with a Volt crosshair. The point is fixed in the air and
  does not scroll.
- **The burst:** 0.6 s later (`flak.warning_seconds: 0.6`) a shell bursts
  there.
  - Everything within 10 px of the point is hit for 0.15 s
    (`flak.burst_radius: 10.0`, `flak.burst_seconds: 0.15`).
  - The burst then throws out a ring of 6 bullets (`flak.burst_count: 6`) at
    60 px/s (`flak.burst_speed: 60.0`).
- **Fairness:** at base speed the Hero moves 60 px in the warning time, so
  moving is always enough to escape. Standing still is never safe. The
  crosshair is the telegraph, and a Battery below the Hero stops marking.
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

Its brain doesn't care about the scroll direction, so Cabinet 1's code can be
reused as is, with its own tuning section.

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
  Since the balloons sit by the walls, killing one means going to the edge.
- The Stage file places the balloons with `left_x` and `right_x` on the spawn
  entry. Both are expected. If one is missing, the game logs a warning and
  uses −148 / 148.

It's the Arc Gate turned 90°, with the same code. In a Cabinet without
terrain walls, Tethers are what squeeze the screen: a line you must cross in
its gap or cut at an edge.

## Stage 1: Cloudline

- **Theme:** above the cloud sea at the plateau's edge, at first light.
  - Far layer: slow cloud banks (Plum and Steel).
  - Mid layer: Clay mesas pushing up through the clouds, with Static roads
    scratched across them. Crawlers and Silos sit on the mesas.
  - The Spire is a thin Steel needle on the horizon in the last minute.
- **Scroll:** 24 px/s, rising to 28 px/s for the final minute (Stage file).
- **Length:** about 210 s (3:30), roughly 5,300 px of scroll.
- **Restart Points:** 0:00, 1:10, 2:20.
- **New ideas:** every Stage 1 enemy is introduced, one at a time. Ground
  targets arrive at 0:40, once the player has a gun again.

| Time | Beat | What happens |
|---|---|---|
| 0:00 | calm | Restart Point. The Wren rises out of the shaft mouth (continuing #16's Transition), and the music starts. |
| 0:04 | teach | One Wisp column, centre → **G** (gun 2). |
| 0:12 | test | Wisp columns left and right, one after the other → **P**. |
| 0:22 | teach | One Kite V. |
| 0:30 | test | A Kite V over a Wisp column → Formation bonus. |
| 0:40 | teach | First mesa: a Crawler convoy on a road, nothing else → **R**. |
| 0:52 | test | Kite V over a second convoy → **G** (gun 3). |
| 1:04 | breathe | A wide gap in the clouds. |
| 1:10 | calm | **Restart Point.** |
| 1:14 | teach | One Bellows alone. |
| 1:24 | test | A Bellows pair and a Wisp column → **S**. |
| 1:38 | teach | Two Silos on a mesa, alone. |
| 1:46 | test | Silos plus a Kite V → **W**. |
| 1:55 | set piece | **Loom** with Wisp launches → **B**. |
| 2:14 | breathe | Cloud only. |
| 2:20 | calm | **Restart Point.** Scroll rises to 28 px/s. |
| 2:24 | teach | Swift chevrons, then one Swift pair from below. |
| 2:34 | twist | Swifts from below while a Bellows holds the top → **P** (second pod). |
| 2:48 | test | Wisp columns, a Crawler convoy and a Silo together → Formation bonus. |
| 3:02 | set piece | **Loom** with Swift pairs. The finale → **G** (gun 4). |
| 3:24 | end | The clouds thin and the Spire shows on the horizon. The scroll ends: Stage clear. |

A player who clears everything leaves Stage 1 with gun 4, 2 Pods, Rockets,
Speed 1, the Ward and 3 bombs, the same shape as Cabinet 1's Stage 1. A
typical player finishes at gun 3 with one Pod.

## Stage 2: The Yard

- **Theme:** the Static's construction yard at the foot of the Spire.
  - Ground: a Clay floor of rails, cranes and half-built segments (Plum
    shading, Steel scaffolds).
  - The Spire's base grows at the top of the screen through the final
    minute.
- **What's new:**
  - Flak Batteries, Mote clouds and Tethers.
  - Kites that fire, and bullets 15% faster.
  - Shorter breathers (about 4 s).
- **Scroll:** 26 px/s, rising to 32 px/s for the run-out (Stage file).
- **Length:** about 225 s (3:45), roughly 6,000 px of scroll.
- **Restart Points:** 0:00, 1:00, 2:00, 2:55, as many as Cabinet 1's
  Stage 2, because this Stage is as dense.

| Time | Beat | What happens |
|---|---|---|
| 0:00 | calm | Restart Point. The yard's edge: rails and scaffolds. |
| 0:04 | teach | One Flak Battery alone. |
| 0:14 | test | Flak Battery plus Wisp columns → **G**. |
| 0:28 | twist | Firing Kites (Stage 2 variant) over a Crawler convoy. |
| 0:40 | teach | First Mote cloud, open sky → **S**. |
| 0:52 | breathe | A crane passes underneath. |
| 1:00 | calm | **Restart Point.** |
| 1:04 | teach | One Tether, alone. |
| 1:16 | test | A Tether over a Crawler convoy → **W**. |
| 1:30 | test | A ring of Silos and a Mote cloud → **G**. |
| 1:52 | breathe | Open yard. |
| 2:00 | calm | **Restart Point.** |
| 2:04 | twist | Swifts from below while a Flak Battery marks the top → **P**. |
| 2:16 | test | Two Tethers 60 px apart with firing Kites between them → Formation bonus. |
| 2:30 | set piece | **Loom** over a pair of Flak Batteries → **B**. |
| 2:50 | breathe | Short. Sparks fall from the cranes (scenery only). |
| 2:55 | calm | **Restart Point.** |
| 2:58 | test | The gauntlet: Flak Batteries, Silos, a Tether, then a Mote cloud → **G**. |
| 3:20 | run-out | Scroll rises to 32 px/s. The Spire's base fills the top of the screen, and harmless flak bursts light the sky (scenery). No new enemies. |
| 3:42 | end | The scroll ends: Stage clear (bonuses paid). Then a scripted flak burst hits the Wren, starting the 2 → 3 Cabinet Transition (#21). |

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
| Each enemy | its score (see the enemy table) | `vertical_shooter.enemies.<name>.score` |
| Formation cleared with no Power-up | 100 × members | `vertical_shooter.score.formation_bonus_per_enemy: 100` |
| Power-up picked up when already maxed | 1,000 | `vertical_shooter.score.maxed_power_up: 1000` |
| Bullet cancelled by the Squall | 10 each | `vertical_shooter.score.bomb_cancel: 10` |
| Stage clear | 3,000 | `vertical_shooter.score.stage_clear: 3000` |
| No-miss bonus | 5,000 | `vertical_shooter.score.no_miss: 5000` |

The bonuses are paid as in Cabinet 1.

**Rough targets** (#17 measures the real ones): about 35–45k for a perfect
Stage 1 and 45–55k for Stage 2. Ground targets are worth more than air
popcorn, so clearing the ground pays. A decent player reaches the second
extra life (70,000) during this Cabinet and the third (120,000) early in
Cabinet 3.

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
  - Enemy machinery: Silo hatch clank, Loom spiral (a soft repeating tick),
    Tether hum.
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
  - `Health`, `Hurt`, `Immune` and `EnemyDestroyed`.
  - `projectile`, `Shield`, `HitStop` / `stage_live` and the Power-up capsule
    pattern (`power_up:<letter>` spawn kinds).
- **One kit addition:** a ground layer that Hero shots and rockets hit and
  nothing else touches. Ground targets use it, so they can never ram the
  Wren.
- **Lift rather than copy** these, into the kit or a shared
  `cabinets/shooter/` module, and update Cabinet 1 to use them in the same
  change:
  - Formation tracking and the Power-up drop rule
    (`side_shooter/enemies/formation.rs`).
  - The fairness gate in `think`: grace, minimum distance, bullet cap, the
    no-spawn radius and "never from behind" with the direction as a
    parameter.
  - The brain helpers `fan`, `turn_toward` and `bob`.
  - The Mote brain as is, the Hook brain as the Swift, and the Arc Gate as
    the Tether, each with the scroll axis as a parameter.
- **Write new:**
  - Wisp, Kite, Bellows, Loom, Crawler, Silo and Flak brains, as pure state
    machines with fixed-step unit tests, like Cabinet 1's `brain.rs`.
  - The Wren's nose-up movement, Pods, Rockets and the Squall.
  - Cabinet 2's own `PowerUps` type, with the JSON shape above, stored in
    `Session.power_ups` the same way.
- **Gallery:** an enemy gallery Stage (`assets/stages/2-gallery.ron`), shown
  with `?gallery`, as in Cabinet 1.

## Revisit after the Cabinet 1 playtest

These numbers were copied from Cabinet 1, or set relative to it, before
anyone has played Cabinet 1. When #12's comments change the Cabinet 1 value,
change the matching one here too, or record why not:

- Hero speed (100 px/s) and Speed step (+20), the hitbox (4×4), fire rate
  (10 volleys/s) and shot speed (360 px/s).
- Bomb counts (start 2, max 4), the bomb's 12 damage and 1.0 s
  invulnerability.
- Losing one step of power per life, and the rewind replaying Power-ups.
- The role HP and time-to-kill targets (popcorn 1–2, gunner 36, set piece
  400 / 640).
- Bullet readability: if Signal-on-Night was hard to read, check Volt on Dusk
  and Clay the same way.
- The Hook's dodge feel, since the Swift is the same brain.
- How much breathing room the 4–6 s breathers really give.
- Scroll speeds: Cabinet 2's are set slightly slower than Cabinet 1's for the
  shorter vertical view. If Cabinet 1 feels slow, raise both.

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
    rocket: (count: 2, interval: 0.5, size: (3.0, 6.0), speed: 200.0,
             turn_rate: 270.0, damage: 3),
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
        kite: (hp: 1, score: 100, formation_size: 5, v_spacing: (14.0, 10.0), speed: 70.0,
               stop_y: 50.0, hang_seconds: 0.6, exit_speed: 120.0,
               telegraph_seconds: 0.3, shot_speed: 85.0),
        swift: (hp: 2, score: 200, formation_size: 2, warning_seconds: 0.75,
                chevron_blink_hz: 4.0, speed: 150.0, pass_offset: 50.0, pair_spread: 0.5,
                edge_clearance: 12.0, turn_y: 80.0, turn_radius: 14.0,
                dive_speed: 170.0, dive_steer_speed: 85.0),
        bellows: (hp: 36, score: 500, speed: 80.0, brake_distance: 24.0, brake_floor: 0.15,
                  stop_y: 70.0, hover_seconds: 3.0, sway_amplitude: 2.0, sway_period: 1.5,
                  telegraph_seconds: 0.4, burst: 3, burst_interval: 0.12, bursts: 2,
                  fire_interval: 1.4, shot_speed: 90.0, exit_speed: 80.0),
        loom: (hp: 400, hp_stage2: 640, score: 2000, speed: 20.0, hold_y: 60.0,
               hold_seconds: 15.0, retreat_speed: 20.0,
               launch_count: 2, launch_offset: (24.0, -8.0), first_launch_delay: 1.0,
               launch_interval: 3.0,
               spiral_arms: 3, spiral_interval: 0.3, spiral_turn_rate: 40.0,
               spiral_speed: 60.0, spiral_seconds: 2.0, spiral_pause: 2.0,
               first_spiral_delay: 2.0),
        crawler: (hp: 10, score: 300, formation_size: 3, spacing: 18.0, speed: 12.0,
                  turn_rate: 120.0, telegraph_seconds: 0.3, shot_speed: 80.0,
                  fire_interval: 2.0, stagger: 0.4),
        silo: (hp: 16, score: 800, closed_seconds: 2.4, telegraph_seconds: 0.3,
               open_seconds: 1.2, ring_count: 6, shot_speed: 70.0),
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
- **Volt bullets instead of magenta.** The Cabinet accent fixed in
  `CABINETS[1]` is pink, which would blur with magenta bullets. The rule "one
  colour for danger" stays; the colour changes with the palette, as DESIGN.md
  intends.
- **Air and ground layers, told apart by shadows.** That is the genre's
  defining idea. Ground targets that can't ram you make flying over a crowded
  ground feel bold rather than unfair, and the shadow makes the rule visible.
- **No terrain collision.** A top-down sky has no walls. Tethers and Flak
  Batteries supply what Cabinet 1's Stage 2 walls did: squeezing the space
  you can use.
- **Rockets instead of the Tail, Pods beside instead of above and below.**
  Each Power-up answers this genre's threats. G, S, W and B stay identical so
  the Cabinet still feels familiar.
- **Slower bullets and entries than Cabinet 1.** The vertical play area is
  shorter than the horizontal one, so equal speeds would give less time to
  react.
- **A quick power ramp at the start.** Power-ups reset at the Transition, so
  the first two Formations both drop one, and the player is off gun 1 almost
  at once.
- **Reuse three Cabinet 1 brains, rotated.** The Swift, Tether and Mote share
  code with the Hook, Arc Gate and Mote, which keeps #14 smaller. The Mote
  also carries the story: the Static is the same swarm.
- **The hit that ends Stage 2 is scripted and free.** DESIGN.md has the ship
  take a hit at the 2 → 3 Transition. Making it cost a life would punish a
  player for finishing.
- **No new glossary terms.** "Air enemy" and "ground target" are Cabinet 2
  ideas, defined here, like Cabinet 1's enemy names. The glossary keeps to
  words that hold across the whole Run.
