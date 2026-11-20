# Design decisions

The settled design from the first design session (2026-11). Terms are defined
in [GLOSSARY.md](../GLOSSARY.md). Per-Cabinet designs live in
[`docs/cabinets/`](cabinets/). Changing a decision here is the user's call.

## Originality

Every Cabinet is an original game in a classic 80s arcade *genre*. Mechanics
may feel familiar; names, characters, art, audio and level layouts are all
original. No ROM assets, no recreations of specific existing games, even
renamed.

## The Run

- Four Cabinets in a fixed order, two Stages each:
  1. Side-scrolling shooter (ship)
  2. Vertical shooter (ship, scrolls upward)
  3. Run-and-gun platformer (on foot, side-scrolling)
  4. Vertical climber (on foot, scrolls upward)
- One Hero for the whole Run: pilots the ship in Cabinets 1–2, ejects, and is
  on foot for 3–4.
- One score and one pool of lives across the Run. 3 lives at the start, an
  extra life at 20,000 then every 50,000, capped at 5.
- Losing a life puts the Hero back at the nearest Restart Point in the
  current Stage, not the Stage start.
- Losing the last life is Game Over: final, the Save is gone.
- After Cabinet 4 the Ending plays and the Run is over. No second lap.
- One fixed difficulty, no difficulty menu. Every gameplay number lives in
  `assets/tuning.ron` so it can be tuned without code changes.

## Stages

- A Stage takes a decent player about 3–4 minutes, so a Run is about 30.
- No bosses. A Stage ends when the Hero reaches the end of its scroll.
- A Stage Card plays between a Cabinet's two Stages.

## Save

- Written only when a Stage is cleared; used up as soon as the next Stage
  begins. There is no save-and-quit from pause; the safe place to stop is
  the Stage Card ("B: QUIT, RESUME LATER").
- Only one Save exists. New Run asks before abandoning it.

## Power-ups

Power-ups belong to their Cabinet: they carry from Stage 1 to Stage 2 (the
Save records them) and are cleared at each Cabinet Transition.

## Cabinet Transitions

Each is its own unique sequence that connects the two Cabinets as one story:

1. **1 → 2:** the ship banks upward and the camera rotates 90°.
2. **2 → 3:** the ship takes a hit, the Hero ejects, the ship crashes and the
   Hero parachutes down.
3. **3 → 4:** the Hero reaches a huge tower and starts climbing; the view pans
   upward.

## Practice

A Cabinet cleared in any Run is unlocked for Practice permanently. Practice
gives fresh lives, shows a score, and never touches the Save or the High
Score Table.

## High Score Table and Attract Mode

Local top ten with three initials, recorded at Game Over or after the
Ending. Attract Mode cycles the title and the High Score Table.

## Controls

Two buttons plus Start, the same everywhere, plus gamepad support. No touch
controls in version 1.

| Input              | Ship Cabinets   | On-foot Cabinets |
|--------------------|-----------------|------------------|
| Move (WASD/arrows) | move            | move / climb     |
| A (Space, Z)       | fire            | jump             |
| B (`;`, X)         | bomb / special  | fire             |
| Start (Enter, Esc) | pause           | pause            |

## Look and sound

- 320×240 internal resolution, integer-scaled with crisp pixels, 4:3.
- Each Cabinet has its own limited palette, so the look changes at every
  Cabinet Transition.
- Art and audio are generated in code first (pixel grids; square/noise chip
  sounds and a small chiptune sequencer). Any piece can later be swapped for
  a CC0 asset. The user does not draw.

## Platforms

Browser first (GitHub Pages, rebuilt on every push to `main`), desktop too.
Saves and scores work in both.

## Naming

"Scroller" is a working title. The real name and the Cabinets' in-game names
come later, once there's something on screen to name.
