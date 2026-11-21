# Cabinet 3: UNDERFIELD (run-and-gun platformer)

The design that issues #19–#22 build. Terms are from [GLOSSARY.md](../../GLOSSARY.md);
the settled design is [DESIGN.md](../DESIGN.md). The Cabinet 1 and 2 designs,
[1-side-shooter.md](1-side-shooter.md) and
[2-vertical-shooter.md](2-vertical-shooter.md), set the standard. Where this
doc says "as in Cabinet 1", the Cabinet 1 rule applies unchanged. The user
can override anything here by commenting on issue #18.

**Where the numbers live.** Every number names its key.

- Keys starting `run_and_gun.` go in `assets/tuning.ron`. The proposed
  section is at the end of this doc. Inside an enemy's own section, keys are
  written short: `skitter.speed` means `run_and_gun.enemies.skitter.speed`.
- Scroll speeds, spawn times, spawn positions, Power-up drops, Restart Points,
  Standoffs and the ground, pits and ledges go in the Stage files
  (`assets/stages/3-1.ron` and `3-2.ron`), as in Cabinets 1 and 2.
- A Formation's size comes from its spawn entry in the Stage file. Its default
  is the enemy's `formation_size` key in `tuning.ron`.
- Sprite sizes are given here so the sprite files match. They live in the
  sprite files, not in tuning.

"UNDERFIELD" is the working in-game name. DESIGN.md leaves final naming until
there is something on screen (#34), so it may change then.

**Nobody has playtested Cabinets 1 or 2 yet** (#12 and #17 have no feedback
so far). Numbers that follow theirs are listed in
[Revisit after the playtests](#revisit-after-the-playtests).

## Premise

> The Wren comes down over the edge of the plateau, below the cloud line,
> into the Underfield: the dim spoil plain where the Static dumps what it
> can't use. The Hero lands by the burning wreck and sets off on foot across
> the plain, toward the one thing visible from anywhere down here, the root
> of the Spire. Stage 2 ends at the foot of the tower.

- **Stage 1 starts at the crash site.** The 2 → 3 Transition (#21) ends with
  the Hero landing under a parachute. Stage 1 opens with the Hero on the
  ground in front of the Wren's wreck, which burns in the mid layer for the
  first 20 s.
- **Stage 2 ends at the foot of the Spire.** The ground stops at the tower's
  base, a wall that fills the right side of the screen. The 3 → 4 Transition
  (the Hero starts climbing and the view pans upward) starts from there.
- The Spire is in the far layer of both Stages. It is a thin Iron line on the
  horizon in Stage 1 and fills more and more of the sky through Stage 2.

## What makes it feel good (the rules behind the numbers)

Cabinet 1's nine rules carry over: bullets you can read, enemies you can see,
every threat announced, a small hitbox, teach-test-twist, waves that breathe,
power arriving steadily, a lost life that dents rather than wipes out, and HP
from time-to-kill targets. These change or are added for a platformer:

1. **The jump is the main tool, so it must be exact.**
   - The jump is the same height every time you hold A, and shorter when you
     let go early.
   - It forgives small timing slips: you can still jump just after running
     off a ledge, and a press just before landing still counts.
   - Every pit and ledge in both Stages can be cleared with the **base** jump
     and no Power-ups, with a margin. A test checks every gap against the
     jump physics (see [Terrain](#terrain)).
2. **You can always read the bullets, in this Cabinet's colours.** Enemy
   bullets are Ice (cyan) with a Foam core. Nothing else uses Ice except
   enemy weak points and warning marks. Hero shots stay Ember dashes, as in
   both ship Cabinets.
3. **The same time to react as Cabinet 1.**
   - The Hero mostly stands around x = −40 and enemies fire from about
     x = +100, so a typical shot travels about 140 px.
   - So no enemy bullet in Stage 1 is faster than 70 px/s (2.0 s). In Stage 2
     the cap is 80 px/s.
   - On foot the Hero dodges by jumping and crouching, which commits you for
     longer than steering a ship. That is why these caps are lower than
     Cabinet 2's.
4. **High and low are different threats.** Shots come at one of two
   heights: chest height (crouch under it) and ankle height (jump over it).
   Each shooter always uses the same height, and its sprite shows which:
   high shooters have their Ice eye at the top and low shooters at the bottom.
5. **The screen only moves forward, and only when the Hero pushes it.** The
   player sets the pace up to the Stage's scroll speed. Nothing pushes the
   Hero from behind, so a fall is never the camera's fault.
6. **Enemies from behind are announced.** Most enemies enter from the right.
   One that enters from the left always shows a chevron on the left edge
   first (`run_and_gun.enemy.behind_warning_seconds: 0.75`), as in Cabinet 1.
7. **The Standoffs are the set pieces.** Twice per Stage, at a Standoff the
   scroll stops and waves come from both sides. The Cabinet has no bosses
   (DESIGN.md), so these are its big moments. Clearing a Standoff early pays
   a time bonus, so skill is rewarded with points and with pace.

## Screen and camera

- 320×240. The play area is y ∈ [−120, +108] under the shared HUD, as in
  Cabinets 1 and 2.
- **Ground level** on flat stretches is y = −96, a 24 px (3-tile) strip of
  ground under the Hero's feet. Terrain heights are given in tiles above the
  bottom of the play area.
- **The camera.** The kit's scroll position *is* the camera, and it only
  increases.
  - When the Hero is right of the push line, x = −16 on screen
    (`run_and_gun.camera.push_x: -16.0`), the camera moves forward to bring
    the Hero back to it. It never moves faster than the Stage file's scroll
    speed at that position.
  - The Hero runs faster than the scroll, so a player who keeps running gets
    ahead of the camera. The Hero can't go further right than x = +64
    (`run_and_gun.camera.max_x: 64.0`). That always leaves 96 px of view
    ahead, so nothing appears on top of the Hero.
  - The left edge of the screen is a wall (`run_and_gun.hero.edge_margin:
    4.0`). The Hero can't go back to where the camera has been.
  - While a Standoff is on, the camera doesn't move.
- **The Stage clears** when the camera reaches the Stage's `length`, as in
  every Cabinet. The Hero then walks off to the right with no control (1.0 s,
  `run_and_gun.hero.exit_seconds: 1.0`) before the Stage Card.

The Stage's length in seconds is therefore set by the scroll speeds plus the
Standoffs, as in Cabinets 1 and 2. A player who stops to fight takes a little
longer. "Pace time" in the tables below is the time when the camera moves at
full speed and each Standoff runs its full length.

## Palette

One Cabinet palette (#4), shared by both Stages, in the same roles as the
other Cabinets. Foam and Ember are the same hex values as in Cabinets 1 and
2, so the Hero and its shots look the same across the Run.

| # | Name | Hex | Used for |
|---|---|---|---|
| 1 | Umber | `#1A0D00` | backdrop (matches `CABINETS[2].backdrop`) |
| 2 | Bark | `#4A2A10` | far layer, ground shading, pit depths |
| 3 | Ochre | `#8C5A2B` | ground, rubble, **walker enemy hull fill** |
| 4 | Sprout | `#9CE65A` | accent (matches `CABINETS[2].accent`), Hero visor, Power-up rims |
| 5 | Foam | `#F4F4F0` | Hero suit, **enemy outlines**, bullet cores, Power-up letters, text |
| 6 | Ember | `#FFB43C` | Hero shots, flares, explosions, the wreck's fire |
| 7 | Ice | `#66D9FF` | **enemy bullets, enemy weak points and warning marks only** |
| 8 | Iron | `#6B6F7A` | the Spire, Stage 2 machinery, **flying enemy hull fill** |

Rules that #19 and #20 check in the screenshots:

- **Backgrounds and terrain use only Umber, Bark, Ochre and Iron** (the
  `#! background` mark). Foam, Sprout, Ember and Ice never appear there. The
  wreck's fire is scenery, so it is drawn in Ochre and Bark; the Ember
  flicker is a separate foreground effect, like an explosion.
- **Every enemy has a 1 px Foam outline** around an Ochre (walker) or Iron
  (flyer) hull, with an Ice eye or core.
- The Hero is the only sprite with a Foam body and a Sprout visor.
- Hazards (crushers, arc floors) are not enemies. They are Iron with an Ice
  warning light, and they have no Foam outline. That marks them as part of
  the place, so the player doesn't try to shoot them.

## The Hero on foot

The pilot of the Wren, in a Foam flight suit with a Sprout visor. The same
person as Cabinets 1–2, who is now on foot.

| Property | Value | Key |
|---|---|---|
| Sprite | 12×20 standing, 12×12 crouching. Frames: stand, 4-frame run, jump up, fall, crouch, aim up, aim diagonal | (sprite file) |
| Hitbox vs bullets and enemies | 6×14, centred on the body; 6×8 when crouching | `run_and_gun.hero.hitbox: (6.0, 14.0)`, `run_and_gun.hero.crouch_hitbox: (6.0, 8.0)` |
| Box vs terrain | 10×20; 10×12 when crouching | `run_and_gun.hero.body: (10.0, 20.0)`, `run_and_gun.hero.crouch_body: (10.0, 12.0)` |
| Run speed | 72 px/s, reached at once (no acceleration on the ground) | `run_and_gun.hero.run_speed: 72.0` |
| Air control | full: the same 72 px/s, so a jump can be steered | `run_and_gun.hero.air_speed: 72.0` |
| Start / respawn | at the Restart Point's x, dropping in from above the top edge onto the ground below it, with no control until it lands | `run_and_gun.hero.entry_from_y: 132.0` |
| Invulnerability after respawn | 2.0 s, blinking (counted from landing) | `run_and_gun.hero.respawn_invulnerable_seconds: 2.0` |
| Hit-stop on losing a life | 0.15 s | `run_and_gun.hero.hit_stop_seconds: 0.15` |

The bullet hitbox is the torso, a bit over half the sprite. A platformer
Hero is a bigger target than a ship, so the hitbox is relatively larger than
the Wren's 4×4. It still leaves the head, feet and arms as near-miss room.

### A: jump

| Property | Value | Key |
|---|---|---|
| Take-off speed | 240 px/s up | `run_and_gun.jump.speed: 240.0` |
| Gravity while rising with A held | 600 px/s² | `run_and_gun.jump.gravity_up: 600.0` |
| Gravity when falling | 900 px/s² | `run_and_gun.jump.gravity_down: 900.0` |
| Releasing A while rising | upward speed is cut to 40% | `run_and_gun.jump.cut: 0.4` |
| Top falling speed | 300 px/s | `run_and_gun.jump.max_fall: 300.0` |
| Coyote time (jump after leaving a ledge) | 0.08 s | `run_and_gun.jump.coyote_seconds: 0.08` |
| Jump buffer (press before landing) | 0.10 s | `run_and_gun.jump.buffer_seconds: 0.1` |

What those numbers give (the Terrain test computes them from the keys):

- **Full jump:** 48 px high (6 tiles), 0.40 s up and 0.33 s down, so 0.73 s
  in the air and **52 px across** at run speed.
- **Tap jump:** about 20 px high.
- Falling is faster than rising, so the jump feels snappy, not floaty.
- A jump starts only when A is **pressed**. Holding A doesn't jump again on
  landing, so a held button never surprises the player.

**Down + A on a thin ledge** drops through it. On solid ground it does
nothing.

### B: fire (hold to auto-fire)

- **Aim:** shots go the way the Hero faces. Holding Up aims straight up.
  Up with left or right aims 45° up that way. In the air, Down aims straight
  down, and Down with a direction aims 45° down. **Down on the ground
  crouches** instead.
- **Facing** follows the last left/right input. Firing doesn't lock it, so
  the Hero can run away while shooting backward only by turning. That keeps
  the genre's "commit to a side" feel.
- **Crouching:** hold Down on the ground. The Hero can't walk while
  crouching, and fires forward at knee height (6 px above the ground). Chest
  shots pass overhead.
- Hero shots are 6×2 Ember dashes (`run_and_gun.shot.size: (6.0, 2.0)`)
  at 300 px/s (`run_and_gun.shot.speed: 300.0`), 1 damage each
  (`run_and_gun.shot.damage: 1`). They come from the muzzle, 8 px in front
  of the Hero at chest height (`run_and_gun.shot.muzzle: (8.0, 4.0)`), and
  despawn off screen or when they hit solid terrain.
- **Shot cap:** 64 (`run_and_gun.shot.max_on_screen: 64`). As in Cabinet 1,
  a volley that would go over the cap is skipped whole.

**No bomb.** On foot, B fires and there is no second weapon button
(DESIGN.md's controls table). The Flare Power-up gives this Cabinet its area
damage instead.

## Power-ups

### How they're earned

As in Cabinet 1: Formations drop the Power-up their spawn entry names, or else
pay a Formation bonus. A member that leaves alive spoils the Formation.

- A **Standoff** names its own drop. It comes when the Standoff is cleared
  (every enemy in it destroyed), not when it times out.
- A Power-up appears where the last member died. It falls to the ground and
  rests there (it never falls into a pit: over a pit it hovers at the
  ground's height). It is collected by touching it. It scrolls away with the
  ground if left.
- It is a 10×10 capsule (`run_and_gun.power_up.size: (10.0, 10.0)`) with a
  Sprout rim and a Foam letter, bobbing 2 px (`run_and_gun.power_up.bob_amplitude:
  2.0`) every 1.0 s (`run_and_gun.power_up.bob_period: 1.0`). It is
  collected by the Hero's terrain box.
- A Power-up already at its maximum gives a 1,000-point medal
  (`run_and_gun.score.maxed_power_up: 1000`).
- They belong to this Cabinet, carry from Stage 1 to Stage 2, and are
  cleared at the 3 → 4 Cabinet Transition (DESIGN.md).

### The five Power-ups

| Letter | Name | Effect | Max | Key(s) |
|---|---|---|---|---|
| **G** | Gun | Raises the gun level by 1 (see below). | level 4 | `run_and_gun.power_up.gun_max: 4` |
| **F** | Flare | While B is held, the Hero also lobs a flare every 0.8 s in an arc ahead. It bursts on the ground, a wall or an enemy, for 4 damage to every enemy within 12 px. | 1 | `run_and_gun.flare.*` (below) |
| **J** | Jet | A second jump in mid-air, 70% of the take-off speed. Recharged on landing. | 1 | `run_and_gun.power_up.jet_speed_fraction: 0.7` |
| **W** | Ward | Absorbs one hit from a bullet or an enemy, then gives 1.0 s of invulnerability. Does **not** save a fall into a pit. | 1 | `run_and_gun.power_up.ward_invulnerable_seconds: 1.0` |
| **H** | Hustle | Run speed +12 px/s. | 1 (84 px/s) | `run_and_gun.power_up.hustle_step: 12.0` |

Why these five:

- **G and W are the Run's constants.** They work as in the ship Cabinets, so
  the player knows them on sight.
- **Flare is the bomb's replacement.** It is the area damage for things
  behind low cover, in a dip, or bunched together. It is how a Flare player
  clears Burrowers and Turrets fast.
- **Jet is the platformer reward.** It makes jumps easier without being
  needed: rule 1 holds every gap to the base jump.
- **Hustle replaces Speed.** It doesn't move the camera any faster, so it
  doesn't shorten the Stage. It lets the Hero get out of trouble and reach a
  Power-up before it scrolls away. Jumps grow by about 17% with it, which is only
  easier.
- **No bomb letter**, because there is no bomb.

**Flare:**

- One flare every 0.8 s while B is held (`run_and_gun.flare.interval: 0.8`).
- It is a 4×4 Ember spark (`run_and_gun.flare.size: (4.0, 4.0)`), thrown
  forward at 110 px/s and up at 160 px/s (`run_and_gun.flare.launch:
  (110.0, 160.0)`) under the falling gravity, so it lands about 45 px ahead
  on flat ground. Aimed up, it is thrown straight up.
- It bursts on terrain or an enemy, for 4 damage (`run_and_gun.flare.damage:
  4`) to every enemy within 12 px (`run_and_gun.flare.radius: 12.0`).
- It ignores the shot cap; at most 3 exist at once
  (`run_and_gun.flare.max_on_screen: 3`).

### Gun levels

| Level | Volley | Feel |
|---|---|---|
| 1 | One shot, 8 volleys/s (`run_and_gun.shot.fire_interval: 0.125`). | A sidearm. |
| 2 | One shot, 12 volleys/s (`run_and_gun.shot.fire_interval_fast: 0.083`). | 50% more damage. |
| 3 | Level 2, plus two shots angled ±12° from the aim (`run_and_gun.shot.spread_degrees: 12.0`). | Covers ledges above and below. |
| 4 | The centre shot becomes a **bolt**: 10×3 (`run_and_gun.shot.bolt_size: (10.0, 3.0)`), 2 damage (`run_and_gun.shot.bolt_damage: 2`), and it passes through the first enemy it hits. Angled shots as level 3. | The payoff level. |

### Damage budget

"Large" here means a target 24 px or more tall within about 100 px of the
Hero, where the ±12° shots spread to ±21 px and all three connect.

| Power | Narrow DPS | Large DPS | With Flare (bunched or dug-in targets) |
|---|---|---|---|
| Gun 1 | 8 | 8 | +5 |
| Gun 2 | 12 | 12 | +5 |
| Gun 3 | 12 | 36 | +5 |
| Gun 4 | 24 | 48 | +5 |

Expected power ("perfect" means every Formation cleared and no life lost;
"typical" means one or two missed and one life lost). The Cabinet starts
from nothing, as Cabinet 2 did, so the first drop comes in the first 10 s.

| When (pace time) | Perfect | Typical |
|---|---|---|
| Stage 1, 0:56 (first Sentry) | Gun 2 + Flare | Gun 2 |
| Stage 1, 1:30 (first Standoff) | Gun 3 + Flare: narrow 12, large 36 | Gun 2: narrow 12, large 12 |
| Stage 1, 2:44 (first Hauler) | Gun 3 + Flare + Jet + Ward | Gun 3 |
| Stage 2, 0:00 | Gun 4 + all: narrow 24, large 48 | Gun 3: narrow 12, large 36 |
| Stage 2, 3:00 (the Strider) | Gun 4 + all | Gun 3 + Flare |

Time-to-kill targets by role, and the HP that follows. The roles are the
shared language from Cabinets 1 and 2.

| Role | Enemies | Target | HP | Check |
|---|---|---|---|---|
| Popcorn | Skitter, Kite, Mote | under 0.3 s even at gun 1 | 2, 2, 1 | 0.25 s at gun 1 |
| Small | Tick, Burrower | about 0.5 s at typical power | 6, 6 | 0.5 s at narrow 12 |
| Timing lock | Sentry | dies inside one 1.4 s opening at typical power | 14 | 1.2 s at narrow 12; needs two openings at gun 1 |
| Gunner | Dropper, Turret | 1.5–2 s, so it gets a burst off | 20, 24 | Dropper 1.7 s and Turret 2.0 s at narrow 12 |
| Power check | Hauler | about 1.5 s at typical, 5 s at gun 2 | 54 | 1.5 s at large 36, 4.5 s at gun 2 |
| Set piece | Strider | about 6–10 s at typical, inside its Standoff | 300 (core) | 6.3 s at large 48, 8.3 s at large 36 |

### Losing a life

As in Cabinet 1: gun −1 (never below 1) and the Ward is lost. Flare, Jet and
Hustle are kept. The Stage rewinds to the latest Restart Point and replays its
Power-ups.

**Falling into a pit** costs a life like any hit: when the Hero's feet go
below the bottom of the play area (y < −120), the hit-stop plays at the
pit's lip, with no explosion. The Ward does not save a fall.

### Saved state

`Session.power_ups` holds this JSON shape, saved and cleared as in Cabinets
1 and 2:

```json
{ "gun": 1, "flare": false, "jet": false, "ward": false, "hustle": false }
```

Practice starts from this default.

## Terrain

Stage 2 of Cabinet 1 had a floor and a ceiling. Here the ground *is* the
level, so the Stage files describe it directly. Everything is on the 8 px
tile grid (`run_and_gun.terrain.tile_size: 8.0`), and x is a scroll position
as for spawns: a piece at `s` reaches the right edge of the screen when the
camera reaches `s`.

```ron
ground: [
    (from: -320.0, to: 600.0, top: 3),   // solid from the bottom up to tile row 3 (y = -96)
    (from: 632.0, to: 1200.0, top: 3),   // 600..632 is a 32 px pit
    (from: 1200.0, to: 1400.0, top: 6),  // a 24 px step up
],
ledges: [
    (from: 900.0, to: 948.0, row: 7, kind: "thin"),     // one-way: land on it from above
    (from: 1500.0, to: 1532.0, row: 7, kind: "crumble"),
],
```

- **`ground`:** solid columns from the bottom of the play area up to `top`
  tile rows (row 0 is y = −120). Anything not covered by a `ground` entry is
  a pit. Ground blocks bullets: Hero and enemy shots alike.
- **`ledges`:** 8 px thick, standing on nothing. A `thin` ledge is one-way:
  the Hero lands on it from above and jumps up through it. A `crumble` ledge
  is solid and one-way the same way, but it shakes for 0.5 s after the Hero
  first lands (`run_and_gun.terrain.crumble_seconds: 0.5`), falls away, and
  comes back 3.0 s later (`run_and_gun.terrain.crumble_respawn_seconds: 3.0`).
  It also comes back at a rewind.
- **The fairness test** (`stage_checks.rs`) walks every pit, step and ledge
  and checks each against the jump keys, with base run speed and no Jet:
  - a pit between two grounds at the same height is at most 40 px
    (`run_and_gun.terrain.max_gap: 40.0`), 77% of the 52 px full jump;
  - a step up is at most 40 px (5 rows), 83% of the jump height
    (`run_and_gun.terrain.max_step: 40.0`);
  - any gap with a rise must fit the jump arc with 8 px to spare, at both the
    take-off and the landing;
  - a Restart Point stands on solid ground at least 48 px from any pit edge;
  - no pit in the first 64 px after a Restart Point or a Standoff's edges.
- **Hazards** are spawn entries, not terrain (see the enemy list): crushers,
  arc floors and conveyors.

## Standoffs

At a Standoff, the scroll stops and the Hero has to fight where they stand.

- A Standoff is a spawn entry (`kind: "standoff"`) at a scroll position, with
  a list of waves. Each wave is a spawn entry of its own with a delay in
  seconds since the Standoff began:

  ```ron
  (at: 3600.0, kind: "standoff", pos: (0.0, 0.0), params: {
      "max_seconds": 14.0,
      "drop": "G",
      "waves": [
          {"after": 0.5, "kind": "skitter", "pos": (176.0, -88.0), "params": {"size": 4}},
          {"after": 3.0, "kind": "skitter", "pos": (-176.0, -88.0), "params": {"size": 3}},
      ],
  }),
  ```

- The camera stops when it reaches `at`. The waves come in on their own
  clock. Waves from the left get the behind chevron (rule 6).
- **It ends** when every wave has spawned and every enemy from it is
  destroyed, or after `max_seconds`, whichever is first. Then the scroll
  resumes. Enemies still alive at the time-out leave the way they came.
- **Clearing it early** pays 50 points per whole second left
  (`run_and_gun.score.standoff_time_bonus: 50`), and its `drop`. A
  Standoff that times out pays neither.
- Restart Points are never inside a Standoff. A lost life during one rewinds
  to before it, and it replays from the start.

## Enemies

All are original machines of the Static. "Aimed" means aimed at the Hero's
position at the moment of firing, with no leading. Walkers have Ochre hulls
and stand on the ground or a ledge; flyers have Iron hulls.

- Enemy bullets are 5×5 Ice dots with a Foam core
  (`run_and_gun.enemy.bullet_size: (5.0, 5.0)`), with a 2×2 hitbox
  (`run_and_gun.enemy.bullet_hitbox: (2.0, 2.0)`). They are removed after
  10 s (`run_and_gun.enemy.bullet_lifetime: 10.0`) or when they hit ground.
- An enemy's hitbox is its sprite shrunk by 1 px on each side
  (`run_and_gun.enemy.hitbox_inset: 1.0`).
- **Touching an enemy** costs a life. The Hero doesn't stomp: in this
  Cabinet, the gun is the answer.
- In Stage 2 every enemy bullet speed is multiplied by 1.14
  (`run_and_gun.stage2_bullet_speed_scale: 1.14`). The fastest results are
  the Sentry, Turret and Strider at 79.8 px/s, under rule 3's 80 px/s.
- Walkers never walk into a pit or off a ledge's end. They turn at the edge,
  except the Skitter, which is built to jump small gaps (see below).

General fairness rules, as in Cabinet 1 with this Cabinet's directions:

- No enemy fires in its first 0.5 s on screen
  (`run_and_gun.enemy.fire_grace_seconds: 0.5`).
- No bullet is ever created within 40 px of the Hero
  (`run_and_gun.enemy.min_fire_distance: 40.0`), measured from where it
  appears, as in Cabinet 2. A pattern that would break it is skipped whole.
- At most 32 enemy bullets at once (`run_and_gun.enemy.max_bullets: 32`).
- Nothing spawns on screen within 48 px of the Hero
  (`run_and_gun.enemy.no_spawn_radius: 48.0`).
- Every shot is either chest height (14 px above the shooter's feet), ankle
  height (4 px), or aimed. Aimed shots only come from enemies above the
  Hero's head or more than 64 px away, so they can be read in time.

| # | Name | Kind | Stage | Sprite | HP | Score | Default Formation |
|---|---|---|---|---|---|---|---|
| 1 | Skitter | walker | 1, 2 | 12×8 | 2 | 100 | line of 4 |
| 2 | Tick | walker | 1, 2 | 10×10 | 6 | 200 | pair |
| 3 | Kite | flyer | 1, 2 | 14×8 | 2 | 150 | trio |
| 4 | Sentry | walker | 1, 2 | 14×18 (up), 14×8 (down) | 14 | 500 | single |
| 5 | Burrower | walker | 1, 2 | 12×12 | 6 | 300 | single |
| 6 | Dropper | flyer | 1, 2 | 24×12 | 20 | 400 | single |
| 7 | Hauler | walker | 1, 2 | 24×24 | 54 | 800 | single |
| 8 | Turret | fixed | 2 | 12×12 | 24 | 400 | single |
| 9 | Mote | flyer | 2 | 4×4 | 1 | 50 | cloud of 8 |
| 10 | Strider | walker | 2 | 32×48 | 300 (core) | 5,000 | single |

HP and score keys: `<name>.hp`, `<name>.score`. Default Formation size:
`<name>.formation_size` (1 when not listed).

The Mote is the Static itself, shared with Cabinets 1 and 2. Every other
enemy is new to this Cabinet.

### 1. Skitter: the runner

A low, many-legged scuttler. A line of 4 (`skitter.formation_size: 4`), 16 px
apart (`skitter.spacing: 16.0`), runs along the ground toward the Hero at
60 px/s (`skitter.speed: 60.0`).

- It **hops gaps** up to 24 px wide (`skitter.max_hop: 24.0`) with a small
  20 px jump, and turns back at anything wider.
- It doesn't fire. It rams. A line from the right is the first lesson: hold
  B and stand your ground. Lines from behind (with the chevron) teach turning
  around.
- It runs off the far edge of the screen and is gone. A member that leaves
  spoils the Formation.

### 2. Tick: the hopper

A round shell on springy legs. Pairs (`tick.formation_size: 2`).

- It hops toward the Hero in arcs 24 px high (`tick.hop_height: 24.0`) and
  32 px long (`tick.hop_length: 32.0`), resting 0.6 s between hops
  (`tick.rest_seconds: 0.6`). The pair is offset by half a rest
  (`tick.stagger: 0.3`).
- It **squats for 0.25 s before each hop** (`tick.telegraph_seconds: 0.25`),
  so its landing spot can be read.
- It hops onto and off ledges, and never into a pit.
- It doesn't fire. It is the lesson that enemies come at different heights:
  shoot it at rest or aim up as it comes over.

### 3. Kite: the swooper

A flat flying wedge. Trios (`kite.formation_size: 3`), 0.4 s apart
(`kite.spacing_seconds: 0.4`).

- Each enters from the top at the spawn's x and glides along y = +80
  (`kite.cruise_y: 80.0`) at 70 px/s (`kite.speed: 70.0`).
- When it is above the Hero, give or take 40 px (`kite.dive_window: 40.0`),
  its Ice eye flashes for 0.3 s (`kite.telegraph_seconds: 0.3`). It then
  dives in a curve down until its underside is 14 px above the ground (`kite.dive_floor: 14.0`)
  at 130 px/s (`kite.dive_speed: 130.0`). It pulls up past the Hero's
  position and leaves through the far edge.
- The dive floor is above a crouching Hero, so crouching under a dive is a
  valid answer, as is shooting up at it.
- It doesn't fire.

### 4. Sentry: the timing lock (chest shots)

A gun post behind a sloped shield, planted on the ground or a ledge.

- **Down:** for 1.8 s (`sentry.down_seconds: 1.8`) it is crouched behind its
  shield, 14×8. It is `Immune`, and shots spark off it.
- **Rising:** its Ice eye lights and it rises for 0.3 s
  (`sentry.telegraph_seconds: 0.3`).
- **Up:** for 1.4 s (`sentry.up_seconds: 1.4`) it stands 14×18 and can be
  hurt. It fires 2 shots (`sentry.burst: 2`), 0.25 s apart
  (`sentry.burst_interval: 0.25`), at **chest height**, toward the Hero's side
  at 70 px/s (`sentry.shot_speed: 70.0`).
- A crouching Hero is under the shots. A Hero beside it, closer than
  `min_fire_distance`, gets no shots at all.
- Its cycle starts when it is fully on screen, so its first opening comes
  2.1 s in.

The lesson is crouch under its burst and shoot it while it is up. At gun 1
it takes two openings, which is the push to grab G.

### 5. Burrower: the ground that bites (ankle shots)

An Ice eye in loose ground. It can be seen as a slight bump with an eye
before it moves.

- It waits buried (`Immune`) until the Hero is within 72 px
  (`burrower.wake_distance: 72.0`).
- Then dust spurts from the spot for 0.4 s (`burrower.telegraph_seconds:
  0.4`), and it rises to 12×12, out of the ground.
- Risen, it fires one shot at **ankle height** along the ground, toward the
  Hero, at 65 px/s (`burrower.shot_speed: 65.0`), every 1.6 s
  (`burrower.fire_interval: 1.6`).
- After 3.2 s risen (`burrower.up_seconds: 3.2`) it sinks again for 2.0 s
  (`burrower.down_seconds: 2.0`), then repeats.
- **A Flare burst reaches it while it is buried**, so a Flare player can dig
  it out before it wakes.

Ankle shots are jumped. With the Sentry, this is the "high and low" pair
(rule 4).

### 6. Dropper: the gunner overhead

A wide, slow flying barge with a hatch underneath.

- It enters from the right edge at y = +72 (`dropper.cruise_y: 72.0`) and
  crosses to the left at 40 px/s (`dropper.speed: 40.0`). It takes 8 s to
  cross, so a typical player has time to kill it.
- Every 1.5 s (`dropper.fire_interval: 1.5`), its hatch glows Ice for 0.4 s
  (`dropper.telegraph_seconds: 0.4`). It then drops a canister that falls
  under 600 px/s² of gravity (`dropper.canister_gravity: 600.0`).
- The canister is an enemy bullet (8×8). When it hits ground it bursts into
  2 bullets along the ground, one each way, at ankle height at 60 px/s
  (`dropper.burst_speed: 60.0`).
- It doesn't drop while it is within 24 px of the Hero's x
  (`dropper.no_drop_window: 24.0`). It drops ahead or behind, so the bursts
  are what you jump.

It is this Cabinet's aim-up target: Up or Up-diagonal shots are the answer,
and getting under it is safe.

### 7. Hauler: the power check

A broad, slow armoured crawler with a plough on its front.

- It walks toward the Hero at 20 px/s (`hauler.speed: 20.0`) and turns at
  edges.
- **Its plough is armour.** Shots that hit the front 8 px of its facing side
  spark off (`hauler.plough_depth: 8.0`). That part is `Immune`, but its
  Ice core on top is not. Angled shots from gun 3 and flares on top hit
  the core, as do shots from a ledge above or a jump.
  - In practice: gun 2 has to jump and shoot down (4.5 s). Gun 3 hits it
    straight on with its angled shots (1.5 s).
- Every 2.4 s (`hauler.fire_interval: 2.4`) it flashes for 0.4 s
  (`hauler.telegraph_seconds: 0.4`) and lobs 3 bullets in a fan up and
  forward (`hauler.fan_count: 3`, `hauler.fan_degrees: 50.0`) at 60 px/s
  (`hauler.shot_speed: 60.0`). They arc down under 120 px/s² of gravity
  (`hauler.shot_gravity: 120.0`).
- It is big enough to block the way: it has to be killed or jumped over (it
  is 24 px tall, half the jump).

### 8. Turret: the wall gun (Stage 2)

A ball mount fixed to a wall, a ceiling or the underside of a gantry.

- It turns to track the Hero at 90°/s (`turret.turn_rate: 90.0`), only
  across the half facing away from its mount.
- Every 2.0 s (`turret.fire_interval: 2.0`) it flashes for 0.35 s
  (`turret.telegraph_seconds: 0.35`) and fires one **aimed** shot at 70 px/s
  (`turret.shot_speed: 70.0`).
- It is always placed above the Hero's head height or more than 64 px away
  (the aimed-shot rule). Stage checks test the placement.
- Turrets are where the Flare and the angled shots pay off: they sit in
  places straight shots can't reach.

### 9. Mote: the swarm (Stage 2)

The Static itself, shared with Cabinets 1 and 2. A cloud of 8
(`mote.formation_size: 8`) within 32 px (`mote.cloud_radius: 32.0`)
appears at the top or the right edge. Each Mote homes on the Hero at 55 px/s
(`mote.speed: 55.0`), turning at 90°/s (`mote.turn_rate: 90.0`). It
dissolves after 8 s (`mote.lifetime: 8.0`) for no points. The same brain,
with its own tuning section.

On foot the Hero can't outrun it the way the Wren could, so the speed is
lower than Cabinet 1's and the cloud is smaller.

### 10. Strider: the set piece (Stage 2)

A tall two-legged walker, the biggest thing in the Cabinet. It is not a boss
(DESIGN.md): it is a Standoff's main wave, and the Standoff's timer runs
under it.

- **Parts:** two legs (Ochre, `Immune`) and a cab on top, 32×20, with an Ice
  core. The core is the only part that takes damage: 300 HP
  (`strider.hp: 300`).
- **Walk:** it strides in from the right at 16 px/s (`strider.speed: 16.0`)
  to x = +72 (`strider.hold_x: 72.0`), then paces back and forth within
  40 px of that (`strider.pace_range: 40.0`).
- **Stomp:** every 4.0 s (`strider.stomp_interval: 4.0`), it raises a leg
  (0.6 s telegraph, `strider.stomp_telegraph_seconds: 0.6`) and stamps. A
  shockwave 8 px tall (`strider.wave_height: 8.0`) runs along the ground
  both ways at 90 px/s (`strider.wave_speed: 90.0`). Jump it. A shockwave is
  a hazard, not a bullet: it doesn't count toward the bullet cap, and the
  40 px rule doesn't apply. It travels only along the ground it starts on.
- **Gun:** between stomps, the cab fires a fan of 5 at chest height and
  above (`strider.fan_count: 5`, `strider.fan_degrees: 40.0`) at 70 px/s
  (`strider.shot_speed: 70.0`), every 2.5 s (`strider.fire_interval: 2.5`),
  with a 0.4 s glow first (`strider.telegraph_seconds: 0.4`). The lowest
  shot of the fan is at chest height, so crouching passes under all five.
- **The two threats alternate,** so there is always one answer at a time:
  jump the wave, crouch under the fan.
- **Time to kill:** the cab is 20 px tall, so angled shots partly miss. #19
  measures the real time with the Hero standing 100 px away; the target is
  under 10 s at typical power (Gun 3: 36 large DPS, about 8.3 s) and about
  6 s at perfect. Its Standoff has a 25 s timer. If the timer runs out, the
  Strider walks off to the right: no points, no drop.

### Hazards

None of these can be destroyed. They are Iron with an Ice warning light, and
they have no Foam outline (see Palette).

| Hazard | Stage | What it does | Keys |
|---|---|---|---|
| **Pit** | 1, 2 | A gap in the ground. Falling in costs a life (see Losing a life). | (terrain) |
| **Crumbling ledge** | 1, 2 | A ledge that shakes and falls 0.5 s after the Hero lands on it, and comes back 3.0 s later. | `run_and_gun.terrain.crumble_seconds`, `crumble_respawn_seconds` |
| **Crusher** | 2 | A press hanging over the ground, 24 px wide. It rises for 1.6 s (`crusher.up_seconds: 1.6`), its light blinks for 0.5 s (`crusher.telegraph_seconds: 0.5`), and it slams down in 0.15 s (`crusher.slam_seconds: 0.15`), staying down for 0.6 s (`crusher.down_seconds: 0.6`). Under it as it slams is a hit. A Crusher's slam also kills enemies (no points). | `run_and_gun.hazards.crusher.*` |
| **Arc floor** | 2 | A ground strip, 32–64 px long, live for 1.0 s (`arc_floor.on_seconds: 1.0`) and off for 1.4 s (`arc_floor.off_seconds: 1.4`). It crackles for 0.4 s before switching on (`arc_floor.telegraph_seconds: 0.4`). Standing on it while live is a hit. Jump it or wait. Its light is the Arc Gate's (Cabinet 1) flicker, in this Cabinet's colours. | `run_and_gun.hazards.arc_floor.*` |
| **Conveyor** | 2 | A ground strip that carries anything on it at 24 px/s (`conveyor.speed: 24.0`), in the direction its spawn entry gives. It doesn't hurt. A conveyor toward the left slows the advance, and one toward a Crusher sets the timing. | `run_and_gun.hazards.conveyor.speed` |

Crushers and arc floors take their phase from the spawn entry's `offset`
param (seconds), so a row of them can be set to ripple. A Restart Point is
never within 48 px of one.

## Stage 1: Wreckfall

- **Theme:** the Underfield under the cloud lid, at dusk.
  - Far layer: the grey underside of the clouds (Bark), with the Spire a thin
    Iron line on the horizon.
  - Mid layer: spoil heaps and the shells of old machines (Bark and Ochre).
  - Ground: packed Ochre dirt with Bark rubble, sinkholes for pits, and the
    fallen fence posts and scaffold planks that make the ledges.
  - The Wren's wreck burns in the mid layer for the first 20 s.
- **Scroll:** at most 40 px/s throughout (Stage file).
- **Length:** 7,200 px. At pace that is 180 s of scrolling plus two Standoffs
  of 14 s and 16 s: **3:30**.
- **Restart Points:** 0:00 (scroll 0), 0:52 (2,080), 1:45 (3,640, just after
  the first Standoff), 2:40 (5,840).
- **Power-ups, in order:** G F G J W G.

| Pace time | Scroll | Beat | What happens |
|---|---|---|---|
| 0:00 | 0 | calm | Restart Point. The Hero stands by the parachute, the wreck burning behind. The music starts. |
| 0:04 | 160 | teach | One Skitter line from the right on flat ground → **G** (gun 2). |
| 0:12 | 480 | teach | The first pit, 24 px. Nothing else. |
| 0:18 | 720 | test | A Skitter line hops a pit toward the Hero, then a second one behind it → Formation bonus. |
| 0:26 | 1,040 | teach | A Kite trio crosses a flat stretch → **F**. |
| 0:36 | 1,440 | test | Low ledges over a wide dip, with a Tick pair hopping across them. |
| 0:46 | 1,840 | breathe | A long flat stretch past a spoil heap. |
| 0:52 | 2,080 | calm | **Restart Point.** |
| 0:56 | 2,240 | teach | One Sentry behind a rubble mound. The crouch lesson. |
| 1:06 | 2,640 | test | A Sentry on a ledge above a Skitter line → **G** (gun 3). |
| 1:16 | 3,040 | teach | One Burrower in loose ground, alone. The jump-the-ankle-shot lesson. |
| 1:24 | 3,360 | breathe | Short. |
| 1:30 | 3,600 | Standoff | **The Sinkhole Ring** (14 s): two small pits either side of the Hero. Skitter lines from both sides (chevrons on the left), a Tick pair, then a Kite trio. Clear: Formation bonus and the time bonus. |
| 1:44 | 3,600 | — | The scroll resumes. |
| 1:45 | 3,640 | calm | **Restart Point.** |
| 1:48 | 3,760 | teach | One Dropper crosses overhead. Aim up. |
| 1:56 | 4,080 | test | Crumbling planks over a 40 px pit, with a Kite trio above → **J**. |
| 2:08 | 4,560 | test | A Dropper over a Burrower pair. Jump bursts and ankle shots at once. |
| 2:20 | 5,040 | twist | A Sentry high on a ledge and a Burrower low, with Kites diving → **W**. |
| 2:34 | 5,600 | breathe | A spoil heap shields the next stretch. |
| 2:40 | 5,840 | calm | **Restart Point.** |
| 2:44 | 6,000 | teach | One Hauler on a long flat stretch. The power check. |
| 2:56 | 6,480 | Standoff | **The Fence Line** (16 s): a Hauler from the right, Skitter lines from behind, then a Kite trio and a Tick pair → **G** (gun 4). |
| 3:12 | 6,480 | run-out | The scroll resumes. Rising ground with no enemies. The Spire's root grows in the far layer. |
| 3:30 | 7,200 | end | The scroll ends at a cable trench: Stage clear (bonuses paid). |

A perfect player leaves Stage 1 with gun 4, Flare, Jet and the Ward. A
typical player finishes at gun 3 with one or two of the others.

## Stage 2: The Roots

- **Theme:** where the Spire meets the ground: gantries, cable trenches and
  machine halls, in Iron and Bark, under the tower's shadow.
  - Far layer: the Spire's base, filling more of the sky as the Stage goes on.
  - Ground: Iron deck plates over Bark trenches (the pits), gantries (thin
    ledges), and machine halls with Crushers, arc floors and conveyors.
- **What's new:**
  - Turrets, Motes, the Strider and the three Stage 2 hazards.
  - Bullets 14% faster.
  - Shorter breathers (about 4 s).
- **Scroll:** at most 44 px/s, slowing to 32 px/s through the Crusher Hall
  (scroll 4,312 to 5,528) so its timing can be read (Stage file).
- **Length:** 7,728 px. At pace that is 186 s of scrolling plus two
  Standoffs of 14 s and 25 s: **3:45**.
- **Restart Points:** 0:00 (0), 0:56 (2,464), 1:52 (4,312), 2:40 (5,968).
- **Power-ups, in order:** G W J G F W. For a perfect player most are
  medals. A player who lost lives gets a way back up.

| Pace time | Scroll | Beat | What happens |
|---|---|---|---|
| 0:00 | 0 | calm | Restart Point. The first deck plates, past the cable trench. |
| 0:04 | 176 | teach | One Turret under a gantry, alone. |
| 0:12 | 528 | test | Two Turrets with a Skitter line along the deck → **G**. |
| 0:24 | 1,056 | teach | The first arc floor, with a Tick pair beyond it. |
| 0:36 | 1,584 | teach | A Mote cloud over open deck → **W**. |
| 0:50 | 2,200 | breathe | Short. |
| 0:56 | 2,464 | calm | **Restart Point.** |
| 1:00 | 2,640 | test | Droppers over a run of trenches, and a Sentry on the far side. |
| 1:10 | 3,080 | Standoff | **The Nest** (14 s): three Turrets in the gantries above, Skitter lines from both sides, then a Mote cloud. Clear: Formation bonus and the time bonus. |
| 1:24 | 3,080 | — | The scroll resumes. |
| 1:28 | 3,256 | test | Two Haulers on a conveyor that runs toward the Hero → **J**. |
| 1:46 | 4,048 | breathe | Short. |
| 1:52 | 4,312 | calm | **Restart Point.** The scroll slows to 32 px/s. |
| 1:56 | 4,440 | teach | The Crusher Hall: three Crushers in a ripple, nothing else. |
| 2:08 | 4,824 | test | Crushers over a conveyor with Skitter lines coming through → **G**. |
| 2:20 | 5,208 | twist | Crumbling gantries over arc floors, with a Burrower on the far side and a Kite trio → Formation bonus. |
| 2:30 | 5,528 | breathe | Out of the hall. The scroll returns to 44 px/s. |
| 2:40 | 5,968 | calm | **Restart Point.** |
| 2:44 | 6,144 | test | The gauntlet: Sentries high and low, a Dropper, then a Mote cloud → **F**. |
| 3:00 | 6,848 | Standoff | **The Strider** (25 s): the Strider walks in from the right, with a Skitter line from behind at 8 s and a Kite trio at 16 s → **W**. |
| 3:25 | 6,848 | run-out | The scroll resumes. The deck ends and the ground rises to the Spire's base. No enemies. |
| 3:45 | 7,728 | end | The scroll ends with the tower's wall filling the right of the screen: Stage clear (bonuses paid). The 3 → 4 Cabinet Transition (the Hero starts to climb) follows. |

## Scoring

| Event | Points | Key |
|---|---|---|
| Each enemy | its score (see the enemy table) | `run_and_gun.enemies.<name>.score` |
| Formation cleared with no Power-up | 100 × members | `run_and_gun.score.formation_bonus_per_enemy: 100` |
| Power-up picked up when already maxed | 1,000 | `run_and_gun.score.maxed_power_up: 1000` |
| Standoff cleared early | 50 per whole second left | `run_and_gun.score.standoff_time_bonus: 50` |
| Stage clear | 3,000 | `run_and_gun.score.stage_clear: 3000` |
| No-miss bonus | 5,000 | `run_and_gun.score.no_miss: 5000` |

The bonuses are paid as in Cabinet 1. A Crusher's kills score nothing.

**Rough targets** (#20 measures the real ones): about 30–40k for a perfect
Stage 1 and 40–50k for Stage 2. A decent player reaches the third extra life
(120,000) in this Cabinet.

## Sound and music (for #20)

- **Stage 1 track:** about 120 BPM, minor key, the slowest of the Run so
  far: a walking bass and a sparse, echoing lead. Lonely and on foot after
  two flying Cabinets.
- **Stage 2 track:** about 150 BPM, mechanical, with a pounding low pulse on
  every beat (the Crushers' rhythm) and noise hats. In the last 20 s it
  climbs in pitch toward the tower.
- **Effects to cover:**
  - Hero: footsteps (soft, every other run frame), jump, Jet puff, land,
    shot, bolt, Flare lob and burst.
  - Power-ups: each letter's chirp, and the maxed medal.
  - Enemy hits: spark (also for `Immune` armour), small and large
    explosions.
  - Warnings: the behind chevron, the Sentry rising, the Burrower dust, the
    Dropper hatch, the Strider's leg lift.
  - Machinery: Crusher slam, arc floor crackle, conveyor hum, crumbling
    ledge creak, the Strider's stomp and shockwave.
  - The Standoff: a lock-in sting as the scroll stops, and a release sting
    as it resumes.
  - Ward break, a fall (a falling whistle, then the life-lost sound), lost
    life, and the "STAGE CLEAR" sting.

## Building it (for #19)

Cabinets 1 and 2's code is the model. Reuse it where the genre is the same,
and write new code only where Cabinet 3 differs.

- **Where it lives:** `src/cabinets/run_and_gun/`, gated by `cabinet_is(2)`,
  laid out like the others: `hero.rs`, `power_ups.rs`, `palette.rs`,
  `tuning.rs`, `terrain.rs`, `standoff.rs` and
  `enemies/{mod,brain,look}.rs`. `tuning.rs` adds one `run_and_gun` field to
  `Tuning`.
- **The kit, unchanged:** Stage timeline and `SpawnRequested`, Restart Point
  rewinds, `Health`, `Hurt`, `Immune`, `EnemyDestroyed`, `projectile`,
  `Shield` (the Ward), `HitStop` / `stage_live`, Formation tracking and the
  fairness gate (as lifted for Cabinet 2), and the Power-up capsule pattern.
- **Two kit additions:**
  - **Driven scroll.** A resource the Cabinet sets each frame, giving how
    far the camera may move. The timeline advances by the smaller of that
    and the Stage's scroll speed × dt. When the resource is absent the kit
    behaves exactly as now, so Cabinets 1 and 2 don't change.
  - **Platform bodies.** Gravity, landing on ground and ledges, one-way
    ledges, and wall stops, against a tile grid. Put it in the kit
    (`kit/platform.rs`), not the Cabinet: Cabinet 4 (the climber) needs the
    same physics. Pure functions over a `TileQuery` trait, with fixed-step
    unit tests for the jump numbers above (height, airtime, distance, coyote
    and buffer).
- **Write new:** the Hero's movement, aiming and crouch; the Flare and Jet;
  the Standoff runner; the terrain format and its drawing; the nine new
  brains (Mote is shared), as pure state machines with fixed-step tests; the
  hazards. Cabinet 3's own `PowerUps` type, with the JSON shape above.
- **Walkers move with the ground.** A walker's position is in world space
  (scroll position + screen x). Its screen position is recomputed each frame
  from the camera, as the terrain is. Flyers are in screen space, as in
  Cabinets 1 and 2.
- **Gallery:** an enemy gallery Stage (`assets/stages/3-gallery.ron`), shown
  with `?gallery`, as before. It includes a flat, a pit, a ledge and each
  hazard.
- **The unattended run (for #20).** The invincible Hero alone can't clear a
  platformer Stage: it has to jump. So, with `debug_keys` on, `?autopilot`
  (and `run_and_gun.debug_autopilot` in tuning) makes the Hero run right,
  fire forward, and jump at every pit and step using the **base** jump and
  the real physics.
  - A pit fall by an invincible Hero puts it back on the far side of the pit
    and counts a **rescue**. The Stage test fails if there are any rescues.
    That makes the run a second check that every gap is passable.
  - `scripts/stages-web.sh 3` then plays both Stages, as for Cabinets 1
    and 2. With Standoffs running to their timers, the clear comes at the
    pace length (3:30 and 3:45). Standoffs end early only if the autopilot
    happens to clear them.

## Stage files and art (for #20)

- **Files:** `assets/stages/3-1.ron` (3:30) and `3-2.ron` (3:45), one line
  per beat of the tables above, each with its pace time in a comment, plus
  the `ground` and `ledges` lists.
- **Enemy kinds and params:** `skitter`, `tick`, `kite`, `sentry`,
  `burrower`, `dropper`, `hauler`, `turret` (`mount`: `"floor"`, `"ceiling"`
  or `"wall"`), `mote`, `strider`. Hazards: `crusher` (`offset`),
  `arc_floor` (`length`, `offset`), `conveyor` (`length`, `dir`). And
  `standoff` (`max_seconds`, `drop`, `waves`). `drop` and `size` work as in
  Cabinet 1.
- **Where things are:** for a walker or a hazard, `pos.y` is ignored. It
  stands on the ground or ledge under `pos.x`, and Stage checks test that
  there is one.
- **`stage_checks.rs`** tests the lengths, Restart Points and their ground,
  the Power-up order, the known kinds and their params, every gap against
  the jump, Turret placement against the aimed-shot rule, and that no
  Restart Point falls inside a Standoff.
- **Scenery** goes in the same files as `scenery:<sprite>` entries, as in
  Cabinet 2. Far layer (cloud underside, the Spire) at 0.05–0.1 of the
  scroll; mid layer (spoil heaps, the wreck, machine shells) at 0.5; ground
  at 1.0.
- **Sprites** are `assets/sprites/c3_*.txt`. The Hero on foot is a new
  sprite (`c3_hero`) in the shared roles: Foam suit, Sprout visor, Ember
  muzzle flash. Frames as listed under [The Hero on foot](#the-hero-on-foot).
  The Sentry has down, rising and up frames; the Burrower buried and risen;
  the Tick squat and hop; the Strider's legs are separate sprites so they can
  step; Crushers and arc floors have an off and a warning frame. Tiles for
  the ground's top, its fill, a ledge, a crumbling ledge (and its shaking
  frame) and a deck plate carry the `#! background` mark.

## Revisit after the playtests

These numbers follow Cabinets 1 and 2, which haven't been played yet. When
#12 or #17 changes the matching number there, check it here too, or record
why not:

- The reaction-time rule behind the bullet caps (70 and 80 px/s).
- The fire rate (8 and 12 volleys/s here, from Cabinet 1's 10) and the Ward.
- Losing one gun level per life, and the rewind replaying Power-ups.
- The role HP and time-to-kill targets.
- How much breathing room the 4–6 s breathers give.

And, for this Cabinet's own playtest (#22): the jump numbers (they are the
whole feel), the camera speed against the run speed, and whether 14–25 s
Standoffs feel like set pieces or like waiting.

## Proposed `tuning.ron` section

For #19 to copy. The values are the ones above.

```ron
run_and_gun: (
    stage2_bullet_speed_scale: 1.14,
    debug_autopilot: false,
    camera: (push_x: -16.0, max_x: 64.0),
    hero: (
        hitbox: (6.0, 14.0), crouch_hitbox: (6.0, 8.0),
        body: (10.0, 20.0), crouch_body: (10.0, 12.0),
        run_speed: 72.0, air_speed: 72.0, edge_margin: 4.0,
        entry_from_y: 132.0, exit_seconds: 1.0,
        respawn_invulnerable_seconds: 2.0, hit_stop_seconds: 0.15,
    ),
    jump: (
        speed: 240.0, gravity_up: 600.0, gravity_down: 900.0, cut: 0.4,
        max_fall: 300.0, coyote_seconds: 0.08, buffer_seconds: 0.1,
    ),
    shot: (
        fire_interval: 0.125, fire_interval_fast: 0.083, speed: 300.0, damage: 1,
        size: (6.0, 2.0), muzzle: (8.0, 4.0), spread_degrees: 12.0,
        bolt_size: (10.0, 3.0), bolt_damage: 2, max_on_screen: 64,
    ),
    flare: (interval: 0.8, size: (4.0, 4.0), launch: (110.0, 160.0),
            damage: 4, radius: 12.0, max_on_screen: 3),
    power_up: (
        size: (10.0, 10.0), bob_amplitude: 2.0, bob_period: 1.0,
        gun_max: 4, jet_speed_fraction: 0.7, hustle_step: 12.0,
        ward_invulnerable_seconds: 1.0,
    ),
    terrain: (tile_size: 8.0, max_gap: 40.0, max_step: 40.0, crumble_seconds: 0.5,
              crumble_respawn_seconds: 3.0),
    enemy: (
        bullet_size: (5.0, 5.0), bullet_hitbox: (2.0, 2.0), hitbox_inset: 1.0,
        fire_grace_seconds: 0.5, min_fire_distance: 40.0, max_bullets: 32,
        no_spawn_radius: 48.0, bullet_lifetime: 10.0, behind_warning_seconds: 0.75,
    ),
    enemies: (
        skitter: (hp: 2, score: 100, formation_size: 4, spacing: 16.0, speed: 60.0,
                  max_hop: 24.0),
        tick: (hp: 6, score: 200, formation_size: 2, hop_height: 24.0, hop_length: 32.0,
               rest_seconds: 0.6, stagger: 0.3, telegraph_seconds: 0.25),
        kite: (hp: 2, score: 150, formation_size: 3, spacing_seconds: 0.4, cruise_y: 80.0,
               speed: 70.0, dive_window: 40.0, telegraph_seconds: 0.3, dive_floor: 14.0,
               dive_speed: 130.0),
        sentry: (hp: 14, score: 500, down_seconds: 1.8, telegraph_seconds: 0.3,
                 up_seconds: 1.4, burst: 2, burst_interval: 0.25, shot_speed: 70.0),
        burrower: (hp: 6, score: 300, wake_distance: 72.0, telegraph_seconds: 0.4,
                   shot_speed: 65.0, fire_interval: 1.6, up_seconds: 3.2, down_seconds: 2.0),
        dropper: (hp: 20, score: 400, cruise_y: 72.0, speed: 40.0, fire_interval: 1.5,
                  telegraph_seconds: 0.4, canister_gravity: 600.0, burst_speed: 60.0,
                  no_drop_window: 24.0),
        hauler: (hp: 54, score: 800, speed: 20.0, plough_depth: 8.0, fire_interval: 2.4,
                 telegraph_seconds: 0.4, fan_count: 3, fan_degrees: 50.0, shot_speed: 60.0,
                 shot_gravity: 120.0),
        turret: (hp: 24, score: 400, turn_rate: 90.0, fire_interval: 2.0,
                 telegraph_seconds: 0.35, shot_speed: 70.0),
        mote: (hp: 1, score: 50, formation_size: 8, cloud_radius: 32.0, speed: 55.0,
               turn_rate: 90.0, lifetime: 8.0),
        strider: (hp: 300, score: 5000, speed: 16.0, hold_x: 72.0, pace_range: 40.0,
                  stomp_interval: 4.0, stomp_telegraph_seconds: 0.6, wave_height: 8.0,
                  wave_speed: 90.0, fan_count: 5, fan_degrees: 40.0, shot_speed: 70.0,
                  fire_interval: 2.5, telegraph_seconds: 0.4),
    ),
    hazards: (
        crusher: (up_seconds: 1.6, telegraph_seconds: 0.5, slam_seconds: 0.15,
                  down_seconds: 0.6),
        arc_floor: (on_seconds: 1.0, off_seconds: 1.4, telegraph_seconds: 0.4),
        conveyor: (speed: 24.0),
    ),
    score: (formation_bonus_per_enemy: 100, maxed_power_up: 1000,
            standoff_time_bonus: 50, stage_clear: 3000, no_miss: 5000),
)
```

## Decisions and reasons

- **The Hero pushes the camera, up to a cap.** A run-and-gun lets the player
  set the pace, but a free camera would let a sprinting player skip the
  Stage and make "3–4 minutes" meaningless. Capping the camera at the
  Stage's scroll speed keeps the kit's position-keyed timeline, the
  Restart Points and the Stage lengths working as in Cabinets 1 and 2. The
  Hero can still run ahead into the free space.
- **Nothing scrolls the Hero off the screen.** A forced auto-scroll in a
  game with pits makes falls feel like the camera's fault (rule 5).
- **Standoffs instead of bosses.** DESIGN.md rules out bosses, but a
  platformer needs moments where you stand and fight. A Standoff has a timer,
  so the Stage length stays fixed, and an early clear pays.
- **Two shot heights, chest and ankle.** Crouch and jump become the dodge,
  in place of a ship's free movement. Each shooter keeps to one height, and
  its sprite shows which.
- **No stomping.** Touching an enemy hurts. The gun is the answer, which
  keeps this a run-and-gun rather than a jumping game.
- **Every gap is cleared by the base jump, and a test proves it.** Jet makes
  jumps easier but is never required. A Power-up lost with a life can't
  strand the player.
- **Five Power-ups, no bomb.** The on-foot controls have no bomb button, so
  the Flare brings area damage into the main fire button. G and W stay as
  in the ship Cabinets. Hustle replaces Speed because the camera caps the
  pace anyway.
- **Lower bullet speeds than Cabinet 2.** Jumping and crouching commit you
  for longer than steering a ship, and the gap from shooter to Hero is
  shorter.
- **Ice bullets, Sprout accent.** Ember stays the Hero's colour across the
  Run, so the accent can't be amber; cyan reads best on brown ground. The
  placeholder palette in `src/sprites/palette.rs` is updated to this table
  in the same change, and its doc test now covers Cabinet 3.
- **The platform physics belong in the kit.** Cabinet 4 is also on foot.
- **An autopilot for the unattended run.** Unlike the ship Cabinets, an
  invincible Hero can't clear a platformer Stage by standing still. The
  autopilot uses only the base jump, so its clean run is also a
  passability check.
- **No new glossary terms.** "Standoff", "walker", "flyer" and "pace time"
  are Cabinet 3 ideas, defined here, like Cabinet 2's "front". The glossary
  keeps to words that hold across the whole Run. The Hero stays "the Hero"
  on foot, with no separate name.
