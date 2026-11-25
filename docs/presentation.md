# Presentation

How the game looks and feels around the play itself, from the presentation
pass (#30). It sits inside the settled "Look and sound" design in
[DESIGN.md](DESIGN.md); the numbers are in the `presentation` and `kit`
sections of `assets/tuning.ron`.

## Title logo

"SCROLLER" is pixel art generated in code (`src/presentation/logo.rs`) at
the canvas's own resolution, not the 5×7 font scaled up: chunky letters
drawn on a 7×9 grid at 3 pixels a cell, slanted forward one pixel every four
rows, filled with a sunset-chrome gradient (pale gold to orange, a dark
seam, then hot pink to violet), outlined and drop-shadowed. A white glint
sweeps across it every few seconds. "Scroller" is still a working title;
the letters are a small table, so the real name drops in the same way.

## Screen shake and hit-stop

Two moments get both:

- **The Hero going down:** each Cabinet's `hit_stop_seconds` freeze, then
  the explosion and `kit.shake_on_hero_down` (unchanged).
- **A big enemy destroyed** (worth `kit.big_explosion_score` or more): a
  shake of `kit.shake_on_big_explosion` and a `kit.big_explosion_hit_stop`
  freeze. Small enemies get neither, so the effect stays special. See
  [kit.md](kit.md).

## CRT effect

Scanlines plus a slight vignette, drawn at window resolution over the
scaled-up canvas, so the art underneath is untouched. Off by default.
**V** toggles it anywhere (it is not one of the game's buttons, so the
two-button-plus-Start controls are unchanged); "CRT ON"/"CRT OFF" shows
briefly and the choice is stored as the `display` setting. The title shows
`V: CRT` small in the corner.

- The dark third of every canvas pixel row is a scanline (at least one
  window row). The texture is rebuilt only when the integer scale changes.
  Below 2× there's no room for a scanline, so only the vignette shows.
- `crt_scanline_alpha` and `crt_vignette_alpha` set the strength.

## Fades between screens

Every screen change fades the new screen in from black over `fade_seconds`
(eased). It fades in only. A fade out would have to keep the old screen
running after it has decided to leave, and every screen (and the Stage
flow's Save rules) would have to wait for it. At 0.35 s the fade-in reads
as a smooth change and every screen switch stays instant underneath.

## HUD

One HUD for every Cabinet (`src/screens/hud.rs`): a dark strip across the
12 pixels above each Cabinet's play area (`PLAY_TOP`), edged underneath with
a 1-pixel line in the Cabinet's accent colour; grey `SCORE` and `LIVES`
labels with white numbers; the Stage (`1-2`, `PRACTICE 3-1`, `DEMO`) in the
accent colour in the middle. The strip is slightly see-through, so a
bullet passing behind it is still visible. Each Cabinet's own indicators
(bomb stock) stay where the Cabinet puts them.
