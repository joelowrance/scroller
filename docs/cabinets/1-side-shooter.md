# Cabinet 1: STATIC TIDE (side-scrolling shooter)

The design that issues #7–#11 build. Terms are from [GLOSSARY.md](../../GLOSSARY.md);
the settled design is [DESIGN.md](../DESIGN.md). Every number names the key it
lives under. Keys starting `side_shooter.` go in `assets/tuning.ron`. Scroll
speeds and spawn times go in the Stage files (`assets/stages/1-1.ron` and
`1-2.ron`), which is where the kit (#3) keeps the timeline. The user can
override anything here by commenting on issue #6.

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
   They are round, have a white core, and move slower than the Hero. Hero
   shots are long amber dashes, so the two can't be confused.
2. **Every threat is announced.** An aimed shot is preceded by a flash on the
   enemy (0.3–0.4 s). An enemy entering from behind shows a chevron on the
   left edge first. An Arc Gate flickers before it switches on. No enemy fires
   in its first 0.5 s on screen.
3. **A small hitbox and a big ship.** The Hero's 16×10 sprite has a 4×4
   hitbox at the cockpit, so near misses feel like skill.
4. **Teach, test, twist.** Each new enemy first appears alone in a calm
   moment, then mixed with known ones, then in a variation that surprises.
5. **Waves breathe.** About 20–25 s of pressure, then 4–6 s of quiet. Restart
   Points sit in the quiet moments, never in the middle of a set piece.
6. **Power arrives steadily.** Power-ups are placed by hand in the timeline, so
   a player who clears Formations gets stronger roughly every 30 s. By the end
   of Stage 2 the Wren is a firework.
7. **A lost life hurts but doesn't wipe you out.** You lose one step of power,
   not everything, so you can still recover from a bad moment.

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
| 2 | Slate | `#23305E` | far parallax, cliffs, terrain shading |
| 3 | Sea | `#2E7A8C` | water, mid parallax, enemy hulls |
| 4 | Spark | `#4DD9FF` | accent (matches `CABINETS[0].accent`), Hero canopy, Power-up rims |
| 5 | Foam | `#F4F4F0` | highlights, bullet cores, text |
| 6 | Ember | `#FFB43C` | Hero shots, Hero engine, explosions |
| 7 | Signal | `#FF3C78` | **enemy bullets and enemy weak points only** |
| 8 | Rust | `#8A3A22` | Stage 2 foundry terrain and machinery |

Background layers use only Night, Slate, Sea and Rust. Signal and Ember never
appear in backgrounds. #11 checks this in the screenshots.

## Hero ship: the Wren

| Property | Value | Key |
|---|---|---|
| Sprite | 16×10, two-frame engine flicker | (sprite file) |
| Hitbox vs bullets and enemies | 4×4 at the cockpit | `side_shooter.hero.hitbox: (4.0, 4.0)` |
| Hitbox vs terrain | 12×6, centred | `side_shooter.hero.terrain_hitbox: (12.0, 6.0)` |
| Base speed | 100 px/s, same in all 8 directions (diagonals normalised) | `side_shooter.hero.speed: 100.0` |
| Speed per Speed step | +20 px/s | `side_shooter.hero.speed_step: 20.0` |
| Start / respawn position | x = −110, y = 0, flying in from off-screen left over 1.0 s | `side_shooter.hero.entry_seconds: 1.0` |
| Invulnerability after respawn | 2.0 s, blinking | `side_shooter.hero.respawn_invulnerable_seconds: 2.0` |
| Hit-stop on losing a life | 0.15 s freeze before the explosion | `side_shooter.hero.hit_stop_seconds: 0.15` |

The terrain hitbox is bigger than the bullet hitbox on purpose. The wings
shouldn't sink into a wall, but the tiny core is what makes dodging bullets
feel fair.

### A: fire (hold to auto-fire)

- Holding A fires 10 volleys/s (`side_shooter.shot.fire_interval: 0.1`).
  Tapping is never faster than holding, so nobody has to mash.
- Hero shots are 8×2 Ember dashes moving at 360 px/s
  (`side_shooter.shot.speed: 360.0`), 1 damage each
  (`side_shooter.shot.damage: 1`). They despawn off screen.
- At most 24 Hero shots exist at once (`side_shooter.shot.max_on_screen: 24`)
  to keep the screen clean. This cap is only hit at full power.

### B: the Undertow (bomb)

A ring of force bursts out from the Wren and sweeps the screen.

- The ring grows from the Wren at 420 px/s
  (`side_shooter.bomb.ring_speed: 420.0`) until it reaches the screen edges.
- Every enemy bullet the ring touches is cancelled and turns into a 10-point
  spark (`side_shooter.score.bomb_cancel: 10`). This makes a bomb used in a
  dense moment feel like a payout instead of a loss.
- Every enemy it touches takes 8 damage once
  (`side_shooter.bomb.damage: 8`). Small enemies die. Ferries and Arc Gate
  pylons survive but are hurt.
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
  a Formation drops a Power-up where the last member died.
- The Stage file chooses which Power-up each Formation drops, so the power
  curve is designed rather than random. A Formation with no Power-up gives a
  Formation bonus instead (see Scoring).
- A Ferry always drops the Power-up its spawn entry names.
- A Power-up drifts left at 30 px/s (`side_shooter.power_up.drift_speed: 30.0`)
  with a slow bob, and stays until it scrolls off screen. It is a 10×10
  capsule with a Spark rim and a letter, so you can tell what it is before
  you commit to grabbing it.
- Picking up a Power-up that is already at its maximum gives a 1,000-point
  medal instead (`side_shooter.score.maxed_power_up: 1000`).

### The six Power-ups

| Letter | Name | Effect | Max | Key(s) |
|---|---|---|---|---|
| **G** | Gun | Raises the gun level by 1 (see below). | level 4 | `side_shooter.power_up.gun_max: 4` |
| **D** | Drone | Adds a drone that flies 18 px above the Wren, and a second one 18 px below. A drone fires a single forward shot with every volley and **absorbs enemy bullets** that touch it. Drones are immune to terrain. | 2 | `side_shooter.power_up.drone_max: 2`, `side_shooter.power_up.drone_offset: 18.0` |
| **T** | Tail | A rear shot fires backward with every second volley (5/s). Pays off against Hooks and Stage 2 threats from behind. | 1 | (on/off) |
| **S** | Speed | +1 speed step (+20 px/s). | 2 steps (140 px/s) | `side_shooter.power_up.speed_max: 2` |
| **W** | Ward | A shield that absorbs one hit from a bullet, an enemy or terrain. On absorbing a hit it flashes, pushes the Wren 8 px away from terrain, and gives 1.0 s of invulnerability. | 1 | `side_shooter.power_up.ward_invulnerable_seconds: 1.0` |
| **B** | Bomb | +1 bomb. | 4 bombs | `side_shooter.bomb.max` |

### Gun levels

| Level | Volley | Feel |
|---|---|---|
| 1 | One shot straight ahead. | A pea-shooter. You have to aim. |
| 2 | Two parallel shots, 6 px apart. | Twice the damage, a wider lane. |
| 3 | Level 2, plus two shots angled ±10°. | Covers the screen ahead. |
| 4 | The parallel shots become **lances**: 12×3 sprites, 2 damage (`side_shooter.shot.lance_damage: 2`). Angled shots as level 3. | Big enemies melt. The payoff level. |

Angle key: `side_shooter.shot.spread_degrees: 10.0`.

### Losing a life

- The gun level drops by 1 (never below 1).
- The Ward is lost, and so is one Drone (the lower one first).
- Speed, Tail and bombs are kept (bombs are topped up, see above).

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
at the Hero's position at the moment of firing, with no leading. Enemy bullets
are 5×5 Signal dots with a 2×2 hitbox. All enemy bullet speeds in Stage 2 are
multiplied by `side_shooter.stage2_bullet_speed_scale: 1.15`.

General fairness rules (from #11's checklist):

- No enemy fires during its first 0.5 s on screen
  (`side_shooter.enemy.fire_grace_seconds: 0.5`).
- No enemy fires when it is within 40 px of the Hero
  (`side_shooter.enemy.min_fire_distance: 40.0`).
- No more than 40 enemy bullets exist at once
  (`side_shooter.enemy.max_bullets: 40`). Further shots are skipped.
- Enemies never spawn on screen within 48 px of the Hero.

Every type lives under `side_shooter.enemies.<name>` with at least `hp`,
`score` and its speeds.

| # | Name | Stage | HP | Score | Formation |
|---|---|---|---|---|---|
| 1 | Skiff | 1, 2 | 1 | 100 | V of 5 |
| 2 | Gull | 1, 2 | 1 | 100 | snake of 6 |
| 3 | Bellbuoy | 1 | 3 | 300 | — |
| 4 | Lamplighter | 1, 2 | 6 | 500 | — |
| 5 | Hook | 1, 2 | 2 | 200 | pairs |
| 6 | Ferry | 1, 2 | 24 | 2,000 | carries Skiffs |
| 7 | Barnacle | 2 | 4 | 400 | — |
| 8 | Clam | 2 | 8 | 800 | pairs |
| 9 | Mote | 2 | 1 | 50 | cloud of 10 |
| 10 | Arc Gate | 2 | 10 per pylon | 600 per pylon | — |

### 1. Skiff: the rhythm keeper

A flat dart, 10×6. It flies in a V of 5 from the right at 90 px/s
(`skiff.speed: 90.0`) and holds its line to x = +40
(`skiff.peel_x: 40.0`). Then the members peel away one by one, 0.15 s apart
(`skiff.peel_interval: 0.15`), the top half curving up and the bottom half
down, leaving at 140 px/s (`skiff.exit_speed: 140.0`). **It doesn't fire.**
The threat is the timing: shoot the tip before the V opens, or chase the
halves. A Ferry also launches Skiffs, which fly straight left at 120 px/s
(`skiff.launched_speed: 120.0`).

### 2. Gull: the wave

A swept-wing glider, 12×8. A snake of 6, 0.25 s apart
(`gull.spacing_seconds: 0.25`), flies left at 70 px/s (`gull.speed: 70.0`)
on a sine wave with amplitude 28 px (`gull.amplitude: 28.0`) and period
1.6 s (`gull.period: 1.6`). In Stage 1 it doesn't fire. **Stage 2 twist:** the
spawn entry can set `fires: true`, and then each Gull drops one straight-left
shot at 80 px/s (`gull.shot_speed: 80.0`) as it crosses x = 0. That turns a
familiar snake into a curtain.

### 3. Bellbuoy: the sea mine

A round buoy, 12×12, bobbing on the wave line at the bottom of Stage 1 and
scrolling with the sea. When the Hero comes within 80 px horizontally
(`bellbuoy.trigger_range: 80.0`) it **rings**: it flashes for 0.4 s
(`bellbuoy.telegraph_seconds: 0.4`) and then fires a 3-shot fan upward and
aimed, ±15° (`bellbuoy.fan_degrees: 15.0`), at 90 px/s
(`bellbuoy.shot_speed: 90.0`). Cooldown 2.5 s (`bellbuoy.cooldown: 2.5`). It
teaches the player that hugging the bottom isn't free.

### 4. Lamplighter: the gunner

A squat craft, 20×14, with a lamp on its nose. It comes in from the right at
60 px/s (`lamplighter.speed: 60.0`), brakes to a stop at
x = +100 (`lamplighter.stop_x: 100.0`), and hovers for 3.0 s
(`lamplighter.hover_seconds: 3.0`). While hovering it lights its lamp for 0.4 s
(`lamplighter.telegraph_seconds: 0.4`) and fires one aimed shot at
100 px/s (`lamplighter.shot_speed: 100.0`), twice, 1.2 s apart
(`lamplighter.fire_interval: 1.2`). Then it leaves upward or downward, away
from the Hero's side of the screen. It's the first enemy that is worth
focusing fire on.

### 5. Hook: the ambush from behind

A hooked blade, 14×8. A chevron blinks on the left edge for 0.75 s
(`hook.warning_seconds: 0.75`) at the row where it will appear. It then
**enters from behind**, flies right at 150 px/s (`hook.speed: 150.0`) about
50 px above or below the Hero (`hook.pass_offset: 50.0`), and U-turns at
x = +110 (`hook.turn_x: 110.0`). It then charges straight left along the
Hero's current row at 170 px/s (`hook.charge_speed: 170.0`). It has no bullets.
It rams. Comes in pairs, one above and one below. It is what makes the Tail
Power-up worth having.

### 6. Ferry: the carrier

A wide barge, 48×24. The whole hull takes damage, and it flashes on each hit.
It crawls in from the right at 25 px/s (`ferry.speed: 25.0`), crosses to
x = −60, and then sinks off the bottom of the screen. Every 4.0 s
(`ferry.launch_interval: 4.0`) it launches a pair of Skiffs from its deck.
Every 3.5 s (`ferry.ring_interval: 3.5`) it fires a ring of 8 bullets at
70 px/s (`ferry.ring_speed: 70.0`). The ring is slow and evenly spaced, so it
is a pattern to thread through, not a wall. It counts as its own one-member
Formation and **always** drops a Power-up. A Ferry is a set piece. One appears
near the end of each Stage half.

### 7. Barnacle: the turret (Stage 2)

A dome cannon, 12×10, fixed to the floor or ceiling terrain and scrolling with
it. Its barrel tracks the Hero at 120°/s (`barnacle.turn_rate: 120.0`). It
flashes for 0.3 s (`barnacle.telegraph_seconds: 0.3`) and then fires one aimed
shot at 110 px/s (`barnacle.shot_speed: 110.0`) every 1.6 s
(`barnacle.fire_interval: 1.6`). **It never fires backward:** once the Hero has
passed it, it falls silent. You can deal with a Barnacle by killing it or by
getting past it.

### 8. Clam: the timing lock (Stage 2)

A hovering shell, 16×14, in pairs. It is closed for 2.0 s
(`clam.closed_seconds: 2.0`) and immune to damage then (shots spark off it).
It then opens for 1.0 s (`clam.open_seconds: 1.0`) to show a Signal core,
and fires a 5-way fan, 60° wide (`clam.fan_degrees: 60.0`), at 100 px/s
(`clam.shot_speed: 100.0`) aimed straight left. **It only takes damage while
open.** It drifts left with the scroll and bobs. Clams reward a player who
waits for the opening.

### 9. Mote: the swarm (Stage 2)

A flickering 4×4 speck. A cloud of 10 drifts in from the right. Each Mote
homes on the Hero at 65 px/s (`mote.speed: 65.0`) with a slow turn rate of
90°/s (`mote.turn_rate: 90.0`), so a sharp turn shakes it off. After 8 s
(`mote.lifetime: 8.0`) a Mote dissolves for no points. Killing the whole cloud
is a Formation clear. Motes are the Static itself: the swarm the Wren is
chasing.

### 10. Arc Gate: the hazard (Stage 2)

Two pylons, 10×16, one on the floor and one on the ceiling, with an electric
arc between them. The arc is live for 1.2 s (`arc_gate.on_seconds: 1.2`) and
off for 0.8 s (`arc_gate.off_seconds: 0.8`). It flickers for 0.3 s before it
switches on (`arc_gate.telegraph_seconds: 0.3`). Touching the live arc is a
hit. Destroying either pylon (10 HP, 600 points each) kills the arc for good.
At gun level 1 you have to time the gap. At level 3+ you can shoot through
the gate. Each Arc Gate is a check on how strong the player is.

## Stage 1: Low Tide

- **Theme:** open sea at dusk. Far layer: a sky in dithered horizontal bands
  (Night to Slate). Mid layer: distant sea stacks and a lighthouse silhouette
  (Slate). Near layer: a wave crest strip along the bottom 16 px (Sea and Foam
  flecks). The waves are scenery only, with no collision.
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
| 1:52 | set piece | **Ferry** crosses with Skiff launches → **B**. |
| 2:14 | breathe | Wreckage drifts past. |
| 2:20 | calm | **Restart Point.** Scroll rises to 36 px/s. |
| 2:24 | teach | Hook warning chevron, then one Hook pair from behind. |
| 2:34 | twist | Hooks behind while a Lamplighter holds the front → **W**. |
| 2:48 | test | Gull snakes plus Bellbuoys plus a Skiff V → **D** (second drone). |
| 3:02 | set piece | **Ferry** with Hook pairs. The finale → **G** (gun 4). |
| 3:24 | end | The sea stacks close in toward cliffs ahead (a hint of Stage 2). The scroll ends: Stage clear. |

A player who clears everything leaves Stage 1 with gun 4, 2 drones, Speed 1,
Ward, and 3 bombs. That is near full power, which Stage 2's new threats then
test. A typical player has missed a Formation or two and lost a life, so they
are around gun 3 with 1 drone.

## Stage 2: The Foundry

- **Theme:** the Static's foundry, cut into the sea cliffs. Far layer: Rust
  furnace glow behind Slate gantries. Mid layer: chains and pipes. Near layer:
  **solid terrain** on the floor and ceiling, built from 8×8 tiles (Slate with
  Rust seams).
- **What's new:**
  - **Terrain walls.** Touching terrain costs a life (or the Ward). Floor and
    ceiling heights are set per 8 px column in the Stage file. The corridor is
    never less than 72 px tall (`side_shooter.terrain.min_gap: 72.0`), checked
    against the 12×6 terrain hitbox.
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
| 2:04 | twist | Hooks from behind in a corridor (the Tail pays off) → **W**. |
| 2:16 | test | Clams set into ceiling alcoves, firing Gulls through the gaps. |
| 2:30 | set piece | **Ferry** in the great furnace hall, with Barnacles on the floor → **D**. |
| 2:50 | breathe | Short. Steam vents (scenery only). |
| 2:55 | calm | **Restart Point.** |
| 2:58 | test | The gauntlet: a 72 px corridor with an Arc Gate, Barnacles, then a Mote cloud → **B**. |
| 3:20 | run-out | Scroll rises to 40 px/s. The ceiling lifts away and the terrain ramps up into a vertical shaft full of rising Motes (no collision, scenery). |
| 3:42 | end | The scroll ends at the shaft. Stage clear, then the Cabinet Transition (the Wren banks upward). |

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
before the Stage Card, through `ScoreGained`.

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

For #7 and #8 to copy and fill in. The values are the ones above.

```ron
side_shooter: (
    stage2_bullet_speed_scale: 1.15,
    hero: (
        hitbox: (4.0, 4.0), terrain_hitbox: (12.0, 6.0),
        speed: 100.0, speed_step: 20.0, edge_margin: 4.0,
        entry_seconds: 1.0, respawn_invulnerable_seconds: 2.0, hit_stop_seconds: 0.15,
    ),
    shot: (
        fire_interval: 0.1, speed: 360.0, damage: 1, lance_damage: 2,
        spread_degrees: 10.0, max_on_screen: 24,
    ),
    bomb: (starting: 2, max: 4, ring_speed: 420.0, damage: 8, invulnerable_seconds: 1.0),
    power_up: (
        drift_speed: 30.0, gun_max: 4, drone_max: 2, drone_offset: 18.0,
        speed_max: 2, ward_invulnerable_seconds: 1.0,
    ),
    terrain: (min_gap: 72.0),
    enemy: (fire_grace_seconds: 0.5, min_fire_distance: 40.0, max_bullets: 40),
    enemies: (
        skiff: (hp: 1, score: 100, speed: 90.0, peel_x: 40.0, peel_interval: 0.15,
                exit_speed: 140.0, launched_speed: 120.0),
        gull: (hp: 1, score: 100, speed: 70.0, spacing_seconds: 0.25,
               amplitude: 28.0, period: 1.6, shot_speed: 80.0),
        bellbuoy: (hp: 3, score: 300, trigger_range: 80.0, telegraph_seconds: 0.4,
                   fan_degrees: 15.0, shot_speed: 90.0, cooldown: 2.5),
        lamplighter: (hp: 6, score: 500, speed: 60.0, stop_x: 100.0, hover_seconds: 3.0,
                      telegraph_seconds: 0.4, shot_speed: 100.0, fire_interval: 1.2),
        hook: (hp: 2, score: 200, warning_seconds: 0.75, speed: 150.0, pass_offset: 50.0,
               turn_x: 110.0, charge_speed: 170.0),
        ferry: (hp: 24, score: 2000, speed: 25.0, launch_interval: 4.0,
                ring_interval: 3.5, ring_speed: 70.0),
        barnacle: (hp: 4, score: 400, turn_rate: 120.0, telegraph_seconds: 0.3,
                   shot_speed: 110.0, fire_interval: 1.6),
        clam: (hp: 8, score: 800, closed_seconds: 2.0, open_seconds: 1.0,
               fan_degrees: 60.0, shot_speed: 100.0),
        mote: (hp: 1, score: 50, speed: 65.0, turn_rate: 90.0, lifetime: 8.0),
        arc_gate: (hp: 10, score: 600, on_seconds: 1.2, off_seconds: 0.8, telegraph_seconds: 0.3),
    ),
    score: (formation_bonus_per_enemy: 100, maxed_power_up: 1000, bomb_cancel: 10,
            stage_clear: 3000, no_miss: 5000),
)
```

## Decisions and reasons

- **Power-ups are placed by hand, not random.** A designed curve guarantees the
  steady growth that makes a shooter satisfying, and keeps a fixed-difficulty
  game predictable.
- **Lose one step, not everything.** Losing all power after one mistake is the
  genre's harshest convention, and it makes recovery from a Restart Point
  frustrating in a one-Run, no-continue design.
- **Two hitboxes.** A small one for bullets (fair dodging) and a larger one for
  terrain (the visuals match), because terrain is new in Stage 2.
- **The bomb cancels bullets into points.** It rewards using the bomb rather
  than hoarding it, and it reads clearly on screen.
- **No enemy fires backward, and every aimed shot is announced.** This keeps
  the pace fast but readable, and #11 can check it.
- **The Stage 2 ending is a shaft.** It sets up the 1 → 2 Cabinet Transition
  without needing a boss (DESIGN: no bosses).
