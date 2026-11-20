# Sprites

Every `name.txt` here is a sprite, built into the game at compile time
(`build.rs`) and turned into textures at startup (`src/sprites/`).

## Format

- One character per pixel. `.` is transparent; `0`–`7` pick a colour from the
  current Cabinet's palette, by role:

  | 0 | 1 | 2 | 3 | 4 | 5 | 6 | 7 |
  |---|---|---|---|---|---|---|---|
  | backdrop | shadow | base | light | accent | hot | bright | alt |

  The palettes live in one place, `src/sprites/palette.rs`. A sprite drawn in
  roles is baked once per Cabinet, so it changes colour at each Cabinet
  Transition.
- Frames are separated by a blank line and must all be the same size.
- Lines starting with `#` are comments.

## Replacing a sprite with a PNG

Put `name.png` next to `name.txt` and rebuild; no code changes. The PNG holds
the frames side by side (as many as the `.txt` has, or one if there is no
`.txt`), keeps its own colours in every Cabinet, and may be larger than the
grid. If the PNG can't be read, the game logs a warning and uses the `.txt`.

## Reviewing

With `debug_keys` on in `assets/tuning.ron`, press F2 on any screen for a sheet
of every sprite, every frame, in every Cabinet palette.
