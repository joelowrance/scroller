# Cabinet 1: STATIC TIDE (side-scrolling shooter)

The design that issues #7–#11 build. Terms are from [GLOSSARY.md](../../GLOSSARY.md);
the settled design is [DESIGN.md](../DESIGN.md). The user can override anything
here by commenting on issue #6.

**Where the numbers live.** Every number names its key.

- Keys starting `side_shooter.` go in `assets/tuning.ron`. The proposed section
  is at the end of this doc. Inside an enemy's own section, keys are written
  short: `skiff.speed` means `side_shooter.enemies.skiff.speed`.
- Scroll speeds, spawn times, spawn positions, which Power-up each Formation
  drops, terrain heights and Restart Points go in the Stage files
  (`assets/stages/1-1.ron` and `1-2.ron`). That is where the kit (#3) keeps the
  timeline.
- A Formation's size comes from its spawn entry in the Stage file. Its default
  is the enemy's `formation_size` key in `tuning.ron`.
- Sprite sizes are given here so the sprite files (#4) match. They live in the
  sprite files, not in tuning.

"STATIC TIDE" is the working in-game name. DESIGN.md leaves final naming until
there is something on screen, so it may change then.

## Premise

> A machine swarm called **the Static** is rolling in off the sea. The Hero
> takes the courier ship **Wren** out along the coast to find where it comes
> from, and follows it through a foundry carved into the cliffs to a shaft
> where the swarm pours straight up into the sky. The Wren banks up after it.

Stage 2 ends at the mouth of a vertical shaft, so the 1 → 2 Cabinet Transition
(the ship banks upward and the camera rotates) follows directly from the level
layout.

## What makes it feel good (the rules behind the numbers)

These rules matter more than any single number. When a tuning change breaks
one of them, the rule wins.

1. **You can always read the bullets.** Enemy bullets use one colour (Signal
   magenta) that nothing else in the palette uses except enemy weak points.
   They are round and have a white core. In Stage 1 no enemy bullet is faster
   than the Hero's base speed (100 px/s). In Stage 2 none is faster than the
   Hero at full Speed (140 px/s). Hero shots are long amber dashes, so the two
   can't be confused.
2. **You can always see the enemies.** Every enemy sprite has a 1 px Foam
   outline. Backgrounds and terrain never use Foam (see Palette), so an enemy
   stands out on any layer.
3. **Every threat is announced.** An aimed shot is preceded by a flash on the
   enemy (0.3–0.4 s). An enemy entering from behind shows a chevron on the
   left edge first. An Arc Gate flickers before it switches on. No enemy fires
   in its first 0.5 s on screen, and no aimed shot is ever fired backward at a
   Hero who has already passed the enemy.
4. **A small hitbox and a big ship.** The Hero's 16×10 sprite has a 4×4
   hitbox at the cockpit, so near misses feel like skill.
5. **Teach, test, twist.** Each new enemy first appears alone in a calm
   moment, then mixed with known ones, then in a variation that surprises.
6. **Waves breathe.** About 20–25 s of pressure, then 4–6 s of quiet. Restart
   Points sit in the quiet moments, never in the middle of a set piece.
7. **Power arrives steadily.** Power-ups are placed by hand in the timeline, so
   a player who clears Formations gets stronger roughly every 30 s. By the end
   of Stage 2 the Wren is a firework.
8. **A lost life hurts but doesn't wipe you out.** You lose one step of power,
   not everything, so you can still recover from a bad moment.
9. **Enemies live long enough to do their job.** Each enemy's HP comes from a
   time-to-kill target for its role at the power a player is expected to have
   when it appears (see Damage budget). Popcorn dies instantly. A gunner gets a
   shot off. A carrier gets to show its pattern.

## Screen

- 320×240. The shared HUD uses the top 12 px, so the play area is
  y ∈ [−120, +108] (origin at screen centre).
- The Hero is clamped inside the play area with a margin
  (`side_shooter.hero.edge_margin: 4.0`) px.
- The Cabinet draws the bomb count itself as small icons at the bottom-left of
  the play area. This keeps the shared HUD untouched.

## Palette

One Cabinet palette (#4), shared by both Stages. Index 0 is transparent.

| # | Name | Hex | Used for |
|---|---|---|---|
| 1 | Night | `#05051A` | backdrop (matches `CABINETS[0].backdrop`) |
| 2 | Slate | `#23305E` | far parallax, cliffs, terrain, wave strip shading |
| 3 | Sea | `#2E7A8C` | water, mid parallax, enemy hull fill |
| 4 | Spark | `#4DD9FF` | accent (matches `CABINETS[0].accent`), Hero canopy, Power-up rims |
| 5 | Foam | `#F4F4F0` | Hero body, **enemy outlines**, bullet cores, Power-up letters, text |
| 6 | Ember | `#FFB43C` | Hero shots, Hero engine, explosions |
| 7 | Signal | `#FF3C78` | **enemy bullets and enemy weak points only** |
| 8 | Rust | `#8A3A22` | Stage 2 terrain and machinery, Stage 2 enemy hull fill |

Rules that #4 builds and #11 checks in the screenshots:

- **Backgrounds and terrain use only Night, Slate, Sea and Rust.** Foam,
  Spark, Ember and Signal never appear there. The Stage 1 wave strip is Sea
  and Slate, with no Foam flecks.
- **Every enemy has a 1 px Foam outline** around a Sea or Rust hull, and a
  Signal eye or core. Because backgrounds never use Foam, an enemy reads
  against water, cliffs and terrain alike, even when its hull matches them.
- The Wren is the only sprite with a Foam *body* and an Ember engine, so it
  never reads as an enemy.

## Hero ship: the Wren

| Property | Value | Key |
|---|---|---|
| Sprite | 16×10, Foam body, Spark canopy, two-frame Ember engine flicker | (sprite file) |
| Hitbox vs bullets and enemies | 4×4 at the cockpit | `side_shooter.hero.hitbox: (4.0, 4.0)` |
| Hitbox vs terrain | 12×6, centred | `side_shooter.hero.terrain_hitbox: (12.0, 6.0)` |
| Base speed | 100 px/s, same in all 8 directions (diagonals normalised) | `side_shooter.hero.speed: 100.0` |
| Speed per Speed step | +20 px/s | `side_shooter.hero.speed_step: 20.0` |
| Start / respawn position | x = −110, y = 0 | `side_shooter.hero.start: (-110.0, 0.0)` |
| Entry | flies in from x = −176 over 1.0 s, with no control until it arrives | `side_shooter.hero.entry_from_x: -176.0`, `side_shooter.hero.entry_seconds: 1.0` |
| Invulnerability after respawn | 2.0 s, blinking (counted from the end of the entry) | `side_shooter.hero.respawn_invulnerable_seconds: 2.0` |
| Hit-stop on losing a life | 0.15 s freeze before the explosion | `side_shooter.hero.hit_stop_seconds: 0.15` |

The terrain hitbox is bigger than the bullet hitbox on purpose. The wings
shouldn't sink into a wall, but the tiny core is what makes dodging bullets
feel fair.

### A: fire (hold to auto-fire)

- Holding A fires 10 volleys/s (`side_shooter.shot.fire_interval: 0.1`).
  Tapping is never faster than holding, so nobody has to mash.
- Hero shots are 8×2 Ember dashes (`side_shooter.shot.size: (8.0, 2.0)`)
  moving at 360 px/s (`side_shooter.shot.speed: 360.0`), 1 damage each
  (`side_shooter.shot.damage: 1`). They despawn off screen.
- **Shot cap:** at most 96 Hero shots exist at once
  (`side_shooter.shot.max_on_screen: 96`). It is a safety valve and should
  never trigger in play. Full power fires 6.5 shots per volley (4 gun, 2
  drones, a Tail shot every second volley). A forward shot lives at most
  about 0.9 s (from x = −156 to the right edge), so about 59 shots are alive
  at once. If a volley *would* go over the cap, the whole volley is skipped,
  never part of it, so the pattern never looks broken.

### B: the Undertow (bomb)

A ring of force bursts out from the Wren and sweeps the screen.

- The ring grows from the Wren at 420 px/s
  (`side_shooter.bomb.ring_speed: 420.0`) until it reaches the screen edges.
- Every enemy bullet the ring touches is cancelled and turns into a 10-point
  spark (`side_shooter.score.bomb_cancel: 10`). This makes a bomb used in a
  dense moment feel like a payout instead of a loss.
- Every enemy it touches takes 12 damage once
  (`side_shooter.bomb.damage: 12`). That kills every small enemy: Skiff, Gull,
  Mote, Hook, Bellbuoy and Barnacle. Lamplighters, open Clams, Ferries and
  Arc Gate pylons survive but are hurt. A **closed Clam takes no damage**,
  because the shell blocks everything, but its bullets are still cancelled.
- The Wren is invulnerable for 1.0 s from the moment B is pressed
  (`side_shooter.bomb.invulnerable_seconds: 1.0`). This makes the bomb a panic
  button that works.
- **Count:** start with 2 (`side_shooter.bomb.starting: 2`), hold at most 4
  (`side_shooter.bomb.max: 4`). Bomb Power-ups add 1. After a lost life the
  count is topped back up to `starting` if it was below it. Bombs carry from
  Stage 1 to Stage 2 with the other Power-ups.
- B does nothing while a ring is still on screen.

## Power-ups

### How they're earned

- Enemies that enter together are a **Formation**. Destroying every member of
  a Formation drops a Power-up where the last member died. If a member leaves
  the screen alive, that Formation can no longer be cleared.
- The Stage file chooses which Power-up each Formation drops, so the power
  curve is designed rather than random. A Formation with no Power-up gives a
  Formation bonus instead (see Scoring).
- A Ferry counts as a one-member Formation and always drops the Power-up its
  spawn entry names. **Skiffs a Ferry launches are not part of any Formation.**
  They never drop anything and give no Formation bonus.
- A Power-up appears where the last member died, clamped at least 16 px
  inside the play area (`side_shooter.power_up.edge_inset: 16.0`), so one
  killed at an edge or partly off screen can still be collected.
- A Power-up drifts left at 30 px/s (`side_shooter.power_up.drift_speed: 30.0`)
  and bobs 3 px (`side_shooter.power_up.bob_amplitude: 3.0`) with a 1.2 s
  period (`side_shooter.power_up.bob_period: 1.2`). It stays until it scrolls
  off screen.
- It is a 10×10 capsule (`side_shooter.power_up.size: (10.0, 10.0)`) with a
  Spark rim and a Foam letter, so you can tell what it is before you commit to
  grabbing it. It is collected when it overlaps a 14×14 box around the Wren
  (`side_shooter.power_up.pickup_box: (14.0, 14.0)`). The box is generous on
  purpose: grabbing a Power-up should never feel fiddly.
- Picking up a Power-up that is already at its maximum gives a 1,000-point
  medal instead (`side_shooter.score.maxed_power_up: 1000`).

### The six Power-ups

| Letter | Name | Effect | Max | Key(s) |
|---|---|---|---|---|
| **G** | Gun | Raises the gun level by 1 (see below). | level 4 | `side_shooter.power_up.gun_max: 4` |
| **D** | Drone | Adds an 8×8 drone 14 px above the Wren, and a second one 14 px below. A drone fires one forward shot with every volley. It **absorbs enemy bullets** that touch it, but enemy bodies pass through it. Drones can't be destroyed and ignore terrain. | 2 | `side_shooter.power_up.drone_max: 2`, `side_shooter.power_up.drone_offset: 14.0` |
| **T** | Tail | A rear shot fires straight backward with every second volley (5/s). Pays off against Hooks and Stage 2 threats from behind. | 1 | `side_shooter.shot.tail_every: 2` |
| **S** | Speed | +1 speed step (+20 px/s). | 2 steps (140 px/s) | `side_shooter.power_up.speed_max: 2` |
| **W** | Ward | A shield that absorbs one hit from a bullet, an enemy or terrain. On absorbing a hit it flashes and gives 1.0 s of invulnerability. If the hit was terrain, it also pushes the Wren 8 px away from it. | 1 | `side_shooter.power_up.ward_invulnerable_seconds: 1.0`, `side_shooter.power_up.ward_push: 8.0` |
| **B** | Bomb | +1 bomb. | 4 bombs | `side_shooter.bomb.max` |

### Gun levels

| Level | Volley | Feel |
|---|---|---|
| 1 | One shot straight ahead. | A pea-shooter. You have to aim. |
| 2 | Two parallel shots, 6 px apart (`side_shooter.shot.twin_spacing: 6.0`). | Twice the damage, a wider lane. |
| 3 | Level 2, plus two shots angled ±10° (`side_shooter.shot.spread_degrees: 10.0`). | Covers the screen ahead. Big targets take the extra shots too. |
| 4 | The parallel shots become **lances**: 12×3 (`side_shooter.shot.lance_size: (12.0, 3.0)`), 2 damage (`side_shooter.shot.lance_damage: 2`). Angled shots as level 3. | Big enemies melt. The payoff level. |

### Damage budget

Damage per second (DPS) at 10 volleys/s. "Narrow" is a small target lined up
dead ahead: only the parallel shots hit it. "Large" is a target 32 px or more
tall within about 90 px: the angled shots and both drones hit it too.

| Power | Narrow DPS | Large DPS |
|---|---|---|
| Gun 1 | 10 | 10 |
| Gun 2 | 20 | 20 |
| Gun 3 | 20 | 40 |
| Gun 4 | 40 | 60 |
| each Drone | +0 | +10 |

Expected power when each enemy role first matters ("perfect" means every
Formation cleared and no life lost; "typical" means one or two Formations
missed and one life lost):

| When | Perfect | Typical |
|---|---|---|
| Stage 1, 1:14 (first Lamplighter) | Gun 2 + 1 Drone: narrow 20, large 30 | Gun 2: narrow 20, large 20 |
| Stage 1, 1:52 (first Ferry) | Gun 3 + 1 Drone: narrow 20, large 50 | Gun 2 + 1 Drone: narrow 20, large 30 |
| Stage 1, 3:02 (finale Ferry) | Gun 3 + 2 Drones: narrow 20, large 60 | Gun 3 + 1 Drone: narrow 20, large 50 |
| Stage 2 | Gun 4 + 2 Drones: narrow 40, large 80 | Gun 3 + 1 Drone: narrow 20, large 50 |

Time-to-kill targets by role, and the HP that follows from them:

| Role | Enemies | Target | HP | Check |
|---|---|---|---|---|
| Popcorn | Skiff, Gull, Mote, Hook | under 0.3 s even at gun 1 | 1, 1, 1, 2 | Hook: 0.2 s at gun 1 |
| Small hazard | Bellbuoy, Barnacle | about 0.5 s at typical power | 10 each | 0.5 s at narrow 20; 1.0 s at gun 1; one Undertow kills it |
| Gunner | Lamplighter | 1.5–2 s, so it gets a shot off | 36 | 1.8 s at narrow 20 (typical in both Stages), 0.9 s at gun 4 |
| Timing lock | Clam | dies inside one 1.0 s opening at typical power | 16 | 0.8 s at narrow 20; two openings at gun 1 |
| Power check | Arc Gate pylon | about 2 s at typical power; long but possible at gun 1 | 40 | 2 s at narrow 20; 4 s at gun 1 (a pylon is on screen for about 10 s) |
| Set piece | Ferry | 8–10 s at perfect power: at least two launch and ring cycles | 400 in Stage 1, 640 in Stage 2 | Stage 1: 8 s perfect and 13 s typical at 1:52, 6.7 s / 8 s at 3:02. Stage 2: 8 s perfect, 12.8 s typical |

A typical player can take up to 15 s on a Ferry: it holds position for that
long before leaving (see Ferry).

### Losing a life

- The gun level drops by 1 (never below 1).
- The Ward is lost, and so is one Drone (the lower one first).
- Speed, Tail and bombs are kept (bombs are topped up, see above).
- The Stage rewinds to the latest Restart Point (DESIGN.md). **Everything
  after that Restart Point plays again,** including its Formations and the
  Power-ups they drop, so a lost G can usually be won back within about 30 s.
  Score earned from the replayed section counts. Replaying it already cost a
  life, which is the price.

A strong player only takes a dent from one mistake. A player who keeps losing
lives slides back toward level 1, which is the natural easing that a
fixed-difficulty game can offer.

### Saved state

`Session.power_ups` holds this JSON shape. The Save records it at a Stage clear,
and it is cleared at the Cabinet Transition:

```json
{ "gun": 1, "drones": 0, "tail": false, "speed": 0, "ward": false, "bombs": 2 }
```

Practice starts from this default.

## Enemies

All are original machines of the Static. Enemies face left. "Aimed" means aimed
at the Hero's position at the moment of firing, with no leading.

- Enemy bullets are 5×5 Signal dots with a Foam core
  (`side_shooter.enemy.bullet_size: (5.0, 5.0)`), with a 2×2 hitbox
  (`side_shooter.enemy.bullet_hitbox: (2.0, 2.0)`).
- An enemy's hitbox is its sprite shrunk by 1 px on each side
  (`side_shooter.enemy.hitbox_inset: 1.0`).
- In Stage 2 every enemy bullet speed is multiplied by 1.15
  (`side_shooter.stage2_bullet_speed_scale: 1.15`). The fastest result is the
  Barnacle at 126.5 px/s, under the 140 px/s limit in rule 1.

General fairness rules (from #11's checklist):

- No enemy fires during its first 0.5 s on screen
  (`side_shooter.enemy.fire_grace_seconds: 0.5`).
- No enemy fires when it is within 40 px of the Hero
  (`side_shooter.enemy.min_fire_distance: 40.0`).
- No aimed shot is fired at a Hero who is to the right of the enemy (the Hero
  has passed it). Rings (Ferry) and fixed fans (Clam) don't aim, so this
  doesn't apply to them.
- No more than 40 enemy bullets exist at once
  (`side_shooter.enemy.max_bullets: 40`). Further shots are skipped.
- Enemies never spawn on screen within 48 px of the Hero
  (`side_shooter.enemy.no_spawn_radius: 48.0`).

| # | Name | Stage | Sprite | HP | Score | Default Formation |
|---|---|---|---|---|---|---|
| 1 | Skiff | 1, 2 | 10×6 | 1 | 100 | V of 5 |
| 2 | Gull | 1, 2 | 12×8 | 1 | 100 | snake of 6 |
| 3 | Bellbuoy | 1 | 12×12 | 10 | 300 | single |
| 4 | Lamplighter | 1, 2 | 20×14 | 36 | 500 | single |
| 5 | Hook | 1, 2 | 14×8 | 2 | 200 | pair |
| 6 | Ferry | 1, 2 | 48×32 | 400 / 640 | 2,000 | single; launches Skiffs |
| 7 | Barnacle | 2 | 12×10 | 10 | 400 | single |
| 8 | Clam | 2 | 16×14 | 16 | 800 | pair |
| 9 | Mote | 2 | 4×4 | 1 | 50 | cloud of 10 |
| 10 | Arc Gate | 2 | pylons 10×16 | 40 per pylon | 600 per pylon | single |

HP and score keys: `<name>.hp`, `<name>.score`. Default Formation size:
`<name>.formation_size` (1 when not listed).

### 1. Skiff: the rhythm keeper

A flat dart. It flies in a V of 5 (`skiff.formation_size: 5`), with members
12 px apart horizontally and 10 px vertically
(`skiff.v_spacing: (12.0, 10.0)`), from the right at 90 px/s
(`skiff.speed: 90.0`). It holds its line to x = +40 (`skiff.peel_x: 40.0`).
Then the members peel away one by one, 0.15 s apart
(`skiff.peel_interval: 0.15`), turning at 180°/s (`skiff.peel_turn_rate: 180.0`):
the top half curves up, the bottom half curves down, and they leave at
140 px/s (`skiff.exit_speed: 140.0`). **It doesn't fire.** The threat is the
timing: shoot the tip before the V opens, or chase the halves. A Ferry also
launches Skiffs, which fly straight left at 120 px/s
(`skiff.launched_speed: 120.0`).

### 2. Gull: the wave

A swept-wing glider. A snake of 6 (`gull.formation_size: 6`), 0.25 s apart
(`gull.spacing_seconds: 0.25`), flies left at 70 px/s (`gull.speed: 70.0`)
on a sine wave with amplitude 28 px (`gull.amplitude: 28.0`) and period
1.6 s (`gull.period: 1.6`). In Stage 1 it doesn't fire. **Stage 2 twist:** the
spawn entry can set `fires: true`, and then each Gull drops one straight-left
shot at 80 px/s (`gull.shot_speed: 80.0`) as it crosses x = 0
(`gull.fire_x: 0.0`). That turns a familiar snake into a curtain.

### 3. Bellbuoy: the sea mine

A round buoy bobbing on the wave line at y = −104 (`bellbuoy.wave_y: -104.0`),
2 px up and down (`bellbuoy.bob_amplitude: 2.0`) every 1.4 s
(`bellbuoy.bob_period: 1.4`), scrolling with the sea. When the Hero comes
within 80 px horizontally (`bellbuoy.trigger_range: 80.0`), and is still to
its left, it **rings**: it flashes for 0.4 s
(`bellbuoy.telegraph_seconds: 0.4`) and then fires a fan of 3
(`bellbuoy.fan_count: 3`) aimed at the Hero, ±15° (`bellbuoy.fan_degrees: 15.0`),
at 90 px/s (`bellbuoy.shot_speed: 90.0`). Cooldown 2.5 s
(`bellbuoy.cooldown: 2.5`). Once the Hero has passed it, it stays silent. It
teaches the player that hugging the bottom isn't free.

### 4. Lamplighter: the gunner

A squat craft with a lamp on its nose. It comes in from the right at 80 px/s
(`lamplighter.speed: 80.0`), brakes over its last 24 px
(`lamplighter.brake_distance: 24.0`) to stop at x = +100
(`lamplighter.stop_x: 100.0`), and hovers for 3.0 s
(`lamplighter.hover_seconds: 3.0`), bobbing 2 px (`lamplighter.bob_amplitude: 2.0`) every 1.5 s
(`lamplighter.bob_period: 1.5`).
It lights its lamp **as soon as it stops**. After 0.4 s
(`lamplighter.telegraph_seconds: 0.4`) it fires one aimed shot at
100 px/s (`lamplighter.shot_speed: 100.0`), and it fires 2 shots in all
(`lamplighter.shots: 2`), 1.2 s apart (`lamplighter.fire_interval: 1.2`).
It reaches its stop about 1.2 s after appearing, so its first shot comes at
about 1.6 s, just before a typical player kills it (1.8 s). Then it leaves
upward or downward at 80 px/s (`lamplighter.exit_speed: 80.0`), away from the
Hero's half of the screen. It's the first enemy worth focusing fire on.

### 5. Hook: the ambush from behind

A hooked blade, in pairs (`hook.formation_size: 2`). A chevron blinks on the
left edge for 0.75 s (`hook.warning_seconds: 0.75`) at the row where each Hook
will appear. It then **enters from behind** and flies right at 150 px/s
(`hook.speed: 150.0`) along a row 50 px above or below the Hero
(`hook.pass_offset: 50.0`; one of the pair above, one below; larger
Formations add pairs half an offset further out). The row is
clamped to stay at least 12 px clear of terrain and the play-area edges
(`hook.terrain_clearance: 12.0`). It U-turns at x = +110 (`hook.turn_x: 110.0`) on a half-circle of radius 14 px
(`hook.turn_radius: 14.0`) toward the Hero's side, and then charges straight left along the Hero's current row at 170 px/s
(`hook.charge_speed: 170.0`). It has no bullets. It rams. Drones don't stop
it. It is what makes the Tail Power-up worth having.

Stage authoring rule: Hooks only appear where the corridor is at least 120 px
tall (`side_shooter.terrain.hook_min_gap: 120.0`), so the pass rows have room.

### 6. Ferry: the carrier

A wide barge. The whole hull takes damage, and it flashes on each hit. It
crawls in from the right at 25 px/s (`ferry.speed: 25.0`) and stops with its
centre at x = +60 (`ferry.hold_x: 60.0`), about 5 s after appearing. It holds
there for up to 15 s (`ferry.hold_seconds: 15.0`) and then sinks off the
bottom at 20 px/s (`ferry.sink_speed: 20.0`), taking its Power-up with it.

- It launches 2 Skiffs from its deck (`ferry.launch_count: 2`) 1.0 s after
  appearing (`ferry.first_launch_delay: 1.0`), then every 3.0 s
  (`ferry.launch_interval: 3.0`).
- It fires a ring of 8 bullets (`ferry.ring_count: 8`) at 70 px/s
  (`ferry.ring_speed: 70.0`) 2.0 s after appearing
  (`ferry.first_ring_delay: 2.0`), then every 3.0 s
  (`ferry.ring_interval: 3.0`). The ring is slow and evenly spaced, so it is a
  pattern to thread through, not a wall.
- HP 400 in Stage 1 (`ferry.hp: 400`) and 640 in Stage 2
  (`ferry.hp_stage2: 640`). At perfect power it lasts about 8 s, which is
  three launches and two or three rings.

A Ferry counts as a one-member Formation and **always** drops a Power-up. A
Ferry is a set piece. One appears near the end of each Stage half.

### 7. Barnacle: the turret (Stage 2)

A dome cannon fixed to the floor or ceiling terrain and scrolling with it. Its
barrel tracks the Hero at 120°/s (`barnacle.turn_rate: 120.0`). It flashes for
0.3 s (`barnacle.telegraph_seconds: 0.3`) and then fires one aimed shot at
110 px/s (`barnacle.shot_speed: 110.0`) every 1.6 s
(`barnacle.fire_interval: 1.6`). **It never fires backward:** once the Hero has
passed it, it falls silent. You can deal with a Barnacle by killing it or by
getting past it.

### 8. Clam: the timing lock (Stage 2)

A hovering shell, in pairs (`clam.formation_size: 2`), the second 48 px
below the first (`clam.pair_spacing: (0.0, -48.0)`). It drifts left with the
scroll and bobs 3 px (`clam.bob_amplitude: 3.0`) every 2.0 s
(`clam.bob_period: 2.0`). It is closed for 2.0 s (`clam.closed_seconds: 2.0`)
and immune to all damage then, the Undertow included (shots spark off it). It
then opens for 1.0 s (`clam.open_seconds: 1.0`) to show a Signal core, and
fires a fan of 5 (`clam.fan_count: 5`), 60° wide (`clam.fan_degrees: 60.0`),
at 100 px/s (`clam.shot_speed: 100.0`) straight left. **It only takes damage
while open.** Clams reward a player who waits for the opening.

### 9. Mote: the swarm (Stage 2)

A flickering speck. A cloud of 10 (`mote.formation_size: 10`) appears within a
40 px radius (`mote.cloud_radius: 40.0`) at the right edge. Each Mote homes on
the Hero at 65 px/s (`mote.speed: 65.0`) with a slow turn rate of 90°/s
(`mote.turn_rate: 90.0`), so a sharp turn shakes it off. After 8 s
(`mote.lifetime: 8.0`) a Mote dissolves for no points, and its cloud can no
longer be cleared. Killing the whole cloud is a Formation clear. Motes are the
Static itself: the swarm the Wren is chasing.

### 10. Arc Gate: the hazard (Stage 2)

Two pylons, one on the floor and one on the ceiling, with a 4 px wide electric
arc between them (`arc_gate.arc_width: 4.0`). The arc is live for 1.2 s
(`arc_gate.on_seconds: 1.2`) and off for 0.8 s (`arc_gate.off_seconds: 0.8`).
It flickers for 0.3 s before it switches on
(`arc_gate.telegraph_seconds: 0.3`). Touching the live arc is a hit, and the live arc stops Hero shots.
Destroying either pylon (40 HP, 600 points each) kills the arc for good.
Pylons sit against the walls, so killing one means flying close to the floor
or ceiling. That's quick at high power and a long, risky job at gun 1, where
timing the gap is the better choice. Each Arc Gate is a check on how strong
the player is.

## Stage 1: Low Tide

- **Theme:** open sea at dusk. Far layer: a sky in dithered horizontal bands
  (Night to Slate). Mid layer: distant sea stacks and a lighthouse silhouette
  (Slate). Near layer: a wave crest strip along the bottom 16 px (Sea and
  Slate). The waves are scenery only, with no collision.
- **Scroll:** 30 px/s, rising to 36 px/s for the final minute (Stage file).
- **Length:** about 210 s (3:30) at normal speed.
- **Restart Points:** 0:00, 1:10, 2:20.
- **New ideas:** every enemy from 1 to 6 is introduced here, one at a time.

Pacing outline (times at normal speed; "→ X" means the Formation drops
Power-up X):

| Time | Beat | What happens |
|---|---|---|
| 0:00 | calm | Restart Point. Open water, music starts, the Wren flies in. |
| 0:05 | teach | One Skiff V, middle. |
| 0:12 | test | Skiff Vs high and low, then middle → **G** (gun 2). |
| 0:28 | teach | Gull snake, top to bottom. |
| 0:36 | twist | Two Gull snakes crossing in opposite phase → **D**. |
| 0:46 | teach | First Bellbuoys on the wave line, three in a row, nothing else. |
| 0:55 | test | Skiff V over the Bellbuoys → Formation bonus. |
| 1:04 | breathe | Quiet sea. The lighthouse passes. |
| 1:10 | calm | **Restart Point.** |
| 1:14 | teach | One Lamplighter alone. |
| 1:24 | test | Lamplighter pair plus a Gull snake → **S**. |
| 1:38 | test | Skiff Vs from alternating sides of centre, Bellbuoys below → **G** (gun 3). |
| 1:52 | set piece | **Ferry** with Skiff launches → **B**. |
| 2:14 | breathe | Wreckage drifts past. |
| 2:20 | calm | **Restart Point.** Scroll rises to 36 px/s. |
| 2:24 | teach | Hook warning chevron, then one Hook pair from behind. |
| 2:34 | twist | Hooks behind while a Lamplighter holds the front → **W**. |
| 2:48 | test | Gull snakes plus Bellbuoys plus a Skiff V → **D** (second drone). |
| 3:02 | set piece | **Ferry** with Hook pairs. The finale → **G** (gun 4). |
| 3:24 | end | The sea stacks close in toward cliffs ahead (a hint of Stage 2). The scroll ends: Stage clear. |

The Ferry beats (1:52 and 3:02) leave up to about 20 s for the Ferry to hold
and sink. Nothing else is scheduled over a Ferry except its named escorts.

A player who clears everything leaves Stage 1 with gun 4, 2 drones, Speed 1,
Ward, and 3 bombs. That is near full power, which Stage 2's new threats then
test. A typical player has missed a Formation or two and lost a life. Since
the rewind replays the lost Power-ups, they usually finish at gun 3 with one
or two drones.

## Stage 2: The Foundry

- **Theme:** the Static's foundry, cut into the sea cliffs. Far layer: Rust
  furnace glow behind Slate gantries. Mid layer: chains and pipes. Near layer:
  **solid terrain** on the floor and ceiling, built from 8×8 tiles
  (`side_shooter.terrain.tile_size: 8.0`; Slate with Rust seams).
- **What's new:**
  - **Terrain walls.** Touching terrain costs a life (or the Ward). Floor and
    ceiling heights are set per 8 px column in the Stage file. The corridor is
    never less than 72 px tall (`side_shooter.terrain.min_gap: 72.0`), checked
    against the 12×6 terrain hitbox. Where Hooks appear it is at least 120 px.
  - New enemies: Barnacles on the terrain, Clams, Mote clouds, Arc Gates.
  - Gulls that fire, Hooks more often, bullets 15% faster.
  - Shorter breathers (about 4 s instead of 6 s).
- **Scroll:** 32 px/s, dropping to 26 px/s in the narrow passages so the
  terrain stays fair, and rising to 40 px/s for the run-out (Stage file).
- **Length:** about 225 s (3:45) at normal speed.
- **Restart Points:** 0:00, 1:00, 2:00, 2:55. There are more than in Stage 1
  because terrain deaths come in clusters.

| Time | Beat | What happens |
|---|---|---|
| 0:00 | calm | Restart Point. Cliff mouth: terrain rises from the floor only. |
| 0:04 | teach | Floor Barnacles, one at a time. |
| 0:14 | test | Ceiling appears. Barnacles above and below, plus a Skiff V → **T**. |
| 0:28 | twist | Firing Gulls (Stage 2 variant) in an open hall. |
| 0:40 | teach | One Clam pair in open space. |
| 0:52 | breathe | Wide chamber, gantries in the background. |
| 1:00 | calm | **Restart Point.** |
| 1:04 | teach | First Mote cloud, open space → **S**. |
| 1:16 | test | The corridor narrows to 96 px. Barnacles plus a Lamplighter. Scroll 26 px/s. |
| 1:30 | teach | First Arc Gate, alone, in a straight corridor. |
| 1:40 | test | Two Arc Gates 80 px apart with a Mote cloud → **G**. |
| 1:52 | breathe | The corridor opens out. |
| 2:00 | calm | **Restart Point.** |
| 2:04 | twist | Hooks from behind in a 128 px corridor (the Tail pays off) → **W**. |
| 2:16 | test | Clams set into ceiling alcoves, firing Gulls through the gaps. |
| 2:30 | set piece | **Ferry** in the great furnace hall, with Barnacles on the floor → **D**. |
| 2:50 | breathe | Short. Steam vents (scenery only). |
| 2:55 | calm | **Restart Point.** |
| 2:58 | test | The gauntlet: a 72 px corridor with an Arc Gate, Barnacles, then a Mote cloud → **B**. |
| 3:20 | run-out | Scroll rises to 40 px/s. The ceiling lifts away and the terrain ramps up into a vertical shaft full of rising Motes (no collision, scenery). |
| 3:42 | end | The scroll ends at the shaft. Stage clear, then the Cabinet Transition (the Wren banks upward). |

For a perfect player, the T, the second S and the fourth bomb are new in
Stage 2. G and D are already at their maximum and become medals, and the W is
new only if the Ward was used up. A player who lost lives gets those same
drops as a way back to full power before the Transition.

## Scoring

| Event | Points | Key |
|---|---|---|
| Each enemy | its score (see the enemy table) | `side_shooter.enemies.<name>.score` |
| Formation cleared with no Power-up | 100 × members | `side_shooter.score.formation_bonus_per_enemy: 100` |
| Power-up picked up when already maxed | 1,000 | `side_shooter.score.maxed_power_up: 1000` |
| Bullet cancelled by the Undertow | 10 each | `side_shooter.score.bomb_cancel: 10` |
| Stage clear | 3,000 | `side_shooter.score.stage_clear: 3000` |
| No-miss bonus (Stage cleared without losing a life) | 5,000 | `side_shooter.score.no_miss: 5000` |

The Stage clear and no-miss bonuses are awarded at the end of the scroll,
before the Stage Card, through `ScoreGained`. Score from a section replayed
after a Restart Point counts normally.

**Rough targets** (#11 measures the real ones): about 30–40k for a perfect
Stage 1 and 40–50k for Stage 2. A decent player reaches the first extra life
(20,000) late in Stage 1 and the second (70,000) during Cabinet 2.

## Sound and music (for #10)

- Stage 1 track: driving, about 150 BPM, minor key, an arpeggiated bass under
  a bright square lead. Hopeful, heading out.
- Stage 2 track: about 160 BPM, darker, heavier noise-channel percussion,
  with the lead an octave lower. In the last 20 s it should sound like
  climbing (rising sequences).
- Effects to cover: Hero shot (a short, quiet tick, since it plays 10×/s),
  lance shot, Undertow, each Power-up letter (a rising chirp, with a different
  top note per type), maxed medal, enemy hit (spark), small and large
  explosions, Clam shell ping (shots bouncing off), Arc Gate hum, Hook warning,
  Ward break, lost life, and a "STAGE CLEAR" sting.

## Proposed `tuning.ron` section

For #7 and #8 to copy. The values are the ones above.

```ron
side_shooter: (
    stage2_bullet_speed_scale: 1.15,
    hero: (
        hitbox: (4.0, 4.0), terrain_hitbox: (12.0, 6.0),
        speed: 100.0, speed_step: 20.0, edge_margin: 4.0,
        start: (-110.0, 0.0), entry_from_x: -176.0, entry_seconds: 1.0,
        respawn_invulnerable_seconds: 2.0, hit_stop_seconds: 0.15,
    ),
    shot: (
        fire_interval: 0.1, speed: 360.0, damage: 1, size: (8.0, 2.0),
        twin_spacing: 6.0, spread_degrees: 10.0,
        lance_size: (12.0, 3.0), lance_damage: 2,
        tail_every: 2, max_on_screen: 96,
    ),
    bomb: (starting: 2, max: 4, ring_speed: 420.0, damage: 12, invulnerable_seconds: 1.0),
    power_up: (
        size: (10.0, 10.0), pickup_box: (14.0, 14.0), edge_inset: 16.0,
        drift_speed: 30.0, bob_amplitude: 3.0, bob_period: 1.2,
        gun_max: 4, drone_max: 2, drone_offset: 14.0, speed_max: 2,
        ward_invulnerable_seconds: 1.0, ward_push: 8.0,
    ),
    terrain: (tile_size: 8.0, min_gap: 72.0, hook_min_gap: 120.0),
    enemy: (
        bullet_size: (5.0, 5.0), bullet_hitbox: (2.0, 2.0), hitbox_inset: 1.0,
        fire_grace_seconds: 0.5, min_fire_distance: 40.0, max_bullets: 40,
        no_spawn_radius: 48.0,
    ),
    enemies: (
        skiff: (hp: 1, score: 100, formation_size: 5, v_spacing: (12.0, 10.0),
                speed: 90.0, peel_x: 40.0, peel_interval: 0.15, peel_turn_rate: 180.0,
                exit_speed: 140.0, launched_speed: 120.0),
        gull: (hp: 1, score: 100, formation_size: 6, spacing_seconds: 0.25,
               speed: 70.0, amplitude: 28.0, period: 1.6, shot_speed: 80.0, fire_x: 0.0),
        bellbuoy: (hp: 10, score: 300, wave_y: -104.0, bob_amplitude: 2.0, bob_period: 1.4,
                   trigger_range: 80.0, telegraph_seconds: 0.4, fan_count: 3,
                   fan_degrees: 15.0, shot_speed: 90.0, cooldown: 2.5),
        lamplighter: (hp: 36, score: 500, speed: 80.0, brake_distance: 24.0, stop_x: 100.0,
                      hover_seconds: 3.0, bob_amplitude: 2.0, bob_period: 1.5, telegraph_seconds: 0.4,
                      shots: 2, shot_speed: 100.0, fire_interval: 1.2, exit_speed: 80.0),
        hook: (hp: 2, score: 200, formation_size: 2, warning_seconds: 0.75, speed: 150.0,
               pass_offset: 50.0, terrain_clearance: 12.0, turn_x: 110.0, turn_radius: 14.0,
               charge_speed: 170.0),
        ferry: (hp: 400, hp_stage2: 640, score: 2000, speed: 25.0, hold_x: 60.0,
                hold_seconds: 15.0, sink_speed: 20.0,
                launch_count: 2, first_launch_delay: 1.0, launch_interval: 3.0,
                ring_count: 8, ring_speed: 70.0, first_ring_delay: 2.0, ring_interval: 3.0),
        barnacle: (hp: 10, score: 400, turn_rate: 120.0, telegraph_seconds: 0.3,
                   shot_speed: 110.0, fire_interval: 1.6),
        clam: (hp: 16, score: 800, formation_size: 2, pair_spacing: (0.0, -48.0),
               bob_amplitude: 3.0, bob_period: 2.0,
               closed_seconds: 2.0, open_seconds: 1.0, fan_count: 5, fan_degrees: 60.0,
               shot_speed: 100.0),
        mote: (hp: 1, score: 50, formation_size: 10, cloud_radius: 40.0, speed: 65.0,
               turn_rate: 90.0, lifetime: 8.0),
        arc_gate: (hp: 40, score: 600, arc_width: 4.0, on_seconds: 1.2, off_seconds: 0.8,
                   telegraph_seconds: 0.3),
    ),
    score: (formation_bonus_per_enemy: 100, maxed_power_up: 1000, bomb_cancel: 10,
            stage_clear: 3000, no_miss: 5000),
)
```

## Decisions and reasons

- **Power-ups are placed by hand, not random.** A designed curve guarantees the
  steady growth that makes a shooter satisfying, and keeps a fixed-difficulty
  game predictable.
- **HP comes from time-to-kill targets, not guesswork.** Each role has a target
  at the power a player is expected to have, so growing stronger is
  noticeable, and the set pieces still get to play out.
- **Lose one step, not everything.** Losing all power after one mistake is the
  genre's harshest convention, and it makes recovery from a Restart Point
  frustrating in a one-Run, no-continue design. The rewind replays the lost
  Power-ups, so the way back is short.
- **Two hitboxes.** A small one for bullets (fair dodging) and a larger one for
  terrain (the visuals match), because terrain is new in Stage 2.
- **Enemies have Foam outlines that backgrounds never use.** Same reasoning as
  the bullet colour: you can always see what can hurt you.
- **The bomb cancels bullets into points.** It rewards using the bomb rather
  than hoarding it, and it reads clearly on screen.
- **No enemy fires backward, and every aimed shot is announced.** This keeps
  the pace fast but readable, and #11 can check it.
- **The Stage 2 ending is a shaft.** It sets up the 1 → 2 Cabinet Transition
  without needing a boss (DESIGN: no bosses).
