# Roadmap

The plan for finishing Scroller, written so an agent can work through it
without the user. The work is tracked as GitHub issues in
[joelowrance/scroller](https://github.com/joelowrance/scroller/issues), grouped
into milestones. Read [docs/DESIGN.md](docs/DESIGN.md) (the settled design)
and [GLOSSARY.md](GLOSSARY.md) (the domain language) before the first ticket.

## Order

Work the milestones in order. Within a milestone, issues whose dependencies
are closed run in parallel, one developer agent each. An issue may start
once the issues it names are closed.

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

## Process

This runs as an IT project: every change is tracked by an issue, lands
through a pull request, and is code-reviewed before it merges. Nobody pushes
to `main` directly, and branch protection enforces it.

**Roles**

- **Project manager** (the coordinating agent): assigns issues, runs the
  developer and reviewer agents, and merges pull requests.
- **Developer** (one agent per issue): builds the issue on its own branch in
  its own git worktree.
- **Reviewer** (a standing agent): reviews every pull request.
- **The user** owns the `needs-human` issues and can veto anything by
  commenting.

**Working a ticket (developer)**

1. **Claim it:** read the issue and its comments, plus any design doc it
   names, and comment that work has started.
2. **Build it** on branch `t<N>-<slug>` (slug: a few kebab-case words from
   the issue title) in the worktree you were given. Use
   the domain language, and put every gameplay number in `assets/tuning.ron`.
3. **Prove it** against the definition of done below.
4. **Open a pull request:** rebase on `origin/main`, push the branch, and open
   a PR whose body starts `Closes #N`. It says what changed, which decisions
   were made, and how it was verified (commands run, screenshots looked at).
   Follow-up work becomes a new issue, not a note in a comment.
5. **Address the review:** fix every blocking finding, push, and reply on the
   PR. Repeat until the reviewer's verdict is APPROVED.

**Reviewing (reviewer)**

Check the PR against its issue's "Done when" list, the design docs, the
glossary, correctness, tests for the logic, every gameplay number living in
`assets/tuning.ron`, and originality. Post findings as a PR review comment ending in
one verdict line: `Verdict: APPROVED` or `Verdict: CHANGES REQUESTED`. All
agents share one GitHub account, so GitHub's own approve button can't be
used. The verdict line is the approval.

**Merging (project manager)**

Merge with squash only when the latest verdict is APPROVED and the PR's CI
check is green, rebasing first if `main` moved. Until the PR check from #2
exists, the gate is the developer's and the reviewer's local runs of the
definition of done. Then confirm the deploy run on `main` succeeded. The live game at
http://joelowrance.com/scroller/ shows the result.

## Definition of done

A ticket is done when every box in its "Done when" list is ticked and all of
these pass:

- `cargo test`
- `cargo clippy --all-targets -- -D warnings`
- `scripts/smoke-web.sh` (once #1 exists), with its screenshots looked at,
  not just generated
- CI is green on the PR, and the deploy run on `main` is green after merge

## Deciding alone

The user has handed this work off, so make design calls without asking.

- **Within the settled design:** decide, and record the decision and its reason
  in the relevant design doc and in the PR body. The user vetoes by
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

> You are the project manager. Work through ROADMAP.md: run developer agents
> on the open issues that aren't `needs-human` or `blocked`, have every pull
> request reviewed, merge what is approved, and keep going until nothing is
> left or everything remaining is blocked.
