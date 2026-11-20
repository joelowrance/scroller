# Roadmap

The plan for finishing Scroller, written so an agent can work through it
without the user. The work is tracked as GitHub issues in
[joelowrance/scroller](https://github.com/joelowrance/scroller/issues), grouped
into milestones. Read [docs/DESIGN.md](docs/DESIGN.md) (the settled design)
and [GLOSSARY.md](GLOSSARY.md) (the domain language) before the first ticket.

## Order

Work the milestones in order and, within a milestone, the issues in number
order. An issue may start once the issues it names are closed.

| Milestone | Issues | Notes |
|---|---|---|
| Cabinet 1: Side-scrolling shooter | #1–#12 | #1–#5 are the shared foundation every later Cabinet builds on. #6 (design doc) comes before #7–#11. |
| Cabinet 2: Vertical shooter | #13–#17 | Design doc, gameplay, Stages/audio, the 1→2 Cabinet Transition, playtest. |
| Cabinet 3: Run-and-gun platformer | #18–#22 | Same shape, with the 2→3 Cabinet Transition. |
| Cabinet 4: Vertical climber | #23–#27 | Same shape, with the 3→4 Cabinet Transition. |
| Ending and polish | #28–#35 | The Ending, Attract Mode demo, presentation, release hardening, performance. |

Issues labelled `needs-human` (#12, #17, #22, #27, #33–#35) are for the user.
Skip them; the next milestone starts without waiting for them. Before starting
each Cabinet's design doc, read the comments on every `needs-human` issue and
act on any feedback first.

## Working a ticket

1. **Claim it:** read the issue and its comments, and any design doc it names.
2. **Build it** to the issue's scope, in the domain language. Every gameplay
   number goes in `assets/tuning.ron`.
3. **Prove it** against the definition of done below.
4. **Ship it:** commit to `main` with a message ending `Closes #N`, push, and
   watch the Pages run (`gh run watch`) until it is green. The game at
   http://joelowrance.com/scroller/ is then the user's way to see the work.
5. **Report it:** comment on the issue with what was built, any decision made
   along the way, and anything left for later. Open a new issue for follow-up
   work rather than leaving it in a comment.

## Definition of done

A ticket is done when every box in its "Done when" list is ticked and all of
these pass:

- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `scripts/smoke-web.sh` (once #1 exists), with its screenshots looked at,
  not just generated
- the CI run for the push is green and deployed

## Deciding alone

The user has handed this work off, so make design calls without asking.

- **Within the settled design:** decide, and record the decision and its reason
  in the relevant design doc and in the issue comment. The user vetoes by
  commenting; until then the decision stands.
- **Changes to the settled design** (anything in `docs/DESIGN.md` or the
  glossary's meanings): pick the most reversible option that keeps the current
  design, carry on, and open a `needs-human` issue describing the choice.
- **Originality is a hard line:** every name, character, enemy, level layout,
  sprite and tune is original. Genre conventions are fine; recreating a
  specific existing game, even renamed, is not.
- When the user is back and a question needs them, ask one question at a time.

## When stuck

After three genuinely different attempts at the same problem, stop. Comment on
the issue with what was tried and what failed, label it `blocked`, and move to
the next issue that doesn't depend on it. If `main` is broken, fixing it comes
before anything else.

## Handing off

To start or resume autonomous work, the user can say:

> Work through ROADMAP.md. Take the next open issue that isn't `needs-human`
> or `blocked`, finish it to the definition of done, and keep going until
> nothing is left or you're blocked.
