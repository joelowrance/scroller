# The Ending

The "you finished" sequence after Cabinet 4's last Stage (#28). It replaces
the old text-only screen. Afterwards the Run is over, with no second lap
(`docs/DESIGN.md`, "The Run"). The code is `src/screens/ending.rs`.

## The sequence

About 13 seconds, then it waits for the player.

1. **The summit** (0–7 s), in Cabinet 4's colours. Stage 2 ended on the
   Crown's deck at dawn (`docs/cabinets/4-climber.md`, "Premise"). The Hero
   vaults up onto the deck and runs to the beacon that beats out the
   Static's signal. The Hero aims up and shoots its lamp out. The stars
   fade, the sky warms, and the sun rises out of the cloud sea while the
   Hero turns to watch it. Captions: "THE CROWN FALLS SILENT" and "THE
   STATIC IS STILL".
2. **Looking back** (from 7 s): "YOU FINISHED!" and one panel per Cabinet,
   popping in one after another. Each panel is drawn in its own Cabinet's
   palette with that Cabinet's colours for the frame. The panels call back
   to the whole Run:
   - Cabinet 1: the Wren over the night sea, past the lighthouse.
   - Cabinet 2: the Wren climbing nose-up over the clouds.
   - Cabinet 3: the Hero coming down under the parachute.
   - Cabinet 4: the Hero beside the dark beacon, with the sun up.

   Under each panel is the points earned in that Cabinet.
3. **The tally** (from 10.8 s): the lives left and the lives bonus. The
   bonus counts into the total with a medal chime per life. Then "THE END"
   and "PRESS A".

After that, the flow is the same as after Game Over: initials if the score
makes the High Score Table, otherwise the High Score Table.

## Decisions

- **Shooting the beacon out.** The Cabinet 4 doc left the Hero's action at
  the Crown to this ticket. Shooting the lamp uses the Hero's own gun and
  reads at 320×240 without new sprites (the lamp has a dark frame).
- **A lives bonus: 10,000 points per life left** (`ending_life_bonus` in
  `assets/tuning.ron`). It rewards finishing with lives in hand, as the
  Stage clear and no-miss bonuses reward a clean Stage. It is added the
  moment the Ending opens, so skipping can't lose it. Setting it to 0 turns
  it off. Extra lives no longer matter at this point, so the bonus doesn't
  count toward them.
- **Per-Cabinet tally.** `Session::cabinet_scores` records each Cabinet's
  points as they're earned. A Save is a Session, so the tally survives
  Continue. A Save made before this change loads with zeros.
- **Skippable.** After 1 s (the same input delay as Game Over), A or Start
  jumps straight to the tally. The next press moves on.
- **Music: "Daybreak"** (`Song::Daybreak`), an original 116 BPM tune in
  C major. It picks up from CrownRise's held C chord, opens with a slow
  fanfare, and loops while the tally waits for the player.
- **The Hero sprite** is `c3_hero` in Cabinet 4's palette, the same as
  Cabinet 4's gameplay uses.

## Testing

- `cargo test`: the timeline (the Hero's path never jumps, the sky, the
  count-up), the bonus being paid, skipping, and moving on.
- `tests/web/ending.mjs`, run by `scripts/smoke-web.sh`, plays a full Run
  with the developer keys (C clears each Stage, P adds points in each
  Cabinet). It screenshots the Ending once a second and checks that no Save
  is left and that the score reaches the High Score Table.
