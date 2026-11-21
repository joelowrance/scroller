# Sprites

Every `name.txt` here is a sprite, built into the game at compile time
(`build.rs`) and turned into textures at startup (`src/sprites/`).

## Format

- One character per pixel. `.` (or `0`) is transparent; `1`–`8` pick a
  colour from the current Cabinet's palette, by role:

  | 1 | 2 | 3 | 4 | 5 | 6 | 7 | 8 |
  |---|---|---|---|---|---|---|---|
  | backdrop | shade | fill | accent | bright | warm | danger | extra |

  The palettes live in one place, `src/sprites/palette.rs`. Each Cabinet's
  design doc names its colours and says what may use them (Cabinet 1:
  `docs/cabinets/1-side-shooter.md`). A sprite is baked once per Cabinet
  palette, so a shared sprite changes colour at each Cabinet Transition.
- Background and terrain sprites use only roles 1, 2, 3 and 8, so nothing
  behind the action looks like the Hero, an enemy or a bullet. Mark them with
  a `#! background` line and a test enforces it.
- Frames are separated by a blank line and must all be the same size.
- Lines starting with `#` are comments.

## Replacing a sprite with a PNG

Put `name.png` next to `name.txt` and rebuild; no code changes. The PNG holds
the frames side by side (as many as the `.txt` has, or one if there is no
`.txt`), keeps its own colours in every Cabinet, and may be larger than the
grid. If the PNG can't be read, the game logs a warning and uses the `.txt`; if the
`.txt` is broken, the PNG is used as one frame.

## Reviewing

With `debug_keys` on in `assets/tuning.ron`, press F2 on any screen for a sheet
of every sprite, every frame, in every Cabinet palette.
