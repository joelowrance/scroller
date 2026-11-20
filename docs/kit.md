# Gameplay kit

The shared engine pieces in `src/kit/`, built once so each Cabinet holds only
its own rules and content. The module docs in `src/kit/mod.rs` describe how
a Cabinet plugs in. This page records the format and the decisions.

## Stage files

A Stage is a RON file under `assets/stages/`, compiled into the binary with
`include_str!` (like `tuning.ron`) so the web build needs no asset loading.

```ron
(
    length: 900.0,                                   // scroll position where the Stage clears
    speed: [(from: 0.0, speed: 40.0), (from: 450.0, speed: 60.0)],
    restart_points: [(at: 0.0, hero: (-110.0, 0.0)), (at: 300.0, hero: (-110.0, 0.0))],
    spawns: [(at: 40.0, kind: "drone", pos: (176.0, 40.0))],
)
```

- Every `at`/`from` is a **scroll position**: pixels scrolled since the Stage
  began.
- `pos` and `hero` are canvas coordinates (origin at the centre, ±160 × ±120).
- `kind` is a free string; the Cabinet maps it to an enemy, pickup or terrain.
- A Restart Point at 0 is added if the file has none.

## Frame order

`KitSet::Timeline → Move → Collide → Resolve → Effects`, only while a Stage is
playing and not paused. Cabinet systems that spawn from `SpawnRequested` run
after `Timeline`; movement and AI run before `Move`.

## Decisions

- **Scroll speed is keyed by scroll position, not seconds.** The issue says
  "speed over time"; keying by position gives the same effect but means a
  rewind to a Restart Point always replays the same speeds.
- **Rewinding** goes to the latest Restart Point at or before the current
  scroll position. Spawns at or after that point fire again; enemies, shots
  and pickups on screen are removed; terrain stays. The Hero moves to the
  Restart Point's `hero` position and blinks for `kit.invulnerable_seconds`.
- **One life per frame:** any number of hits in the same frame cost one life.
- **Touching boxes don't collide**; boxes must share area.
- **Hit flash** tints the sprite white. Image sprites from #4 may need a
  different flash later.
- **An enemy touching the Hero** costs a life but the enemy survives (the
  rewind clears it anyway).
- **Debug aids** (`invincible_hero`, `time_scale` in `tuning.ron`) only act
  when `debug_keys` is on. The K key still costs a life with an invincible
  Hero, so the flow can be tested. In the browser, `?invincible&time_scale=8`
  on the URL overrides both, so a smoke test can play a whole Stage
  unattended.
