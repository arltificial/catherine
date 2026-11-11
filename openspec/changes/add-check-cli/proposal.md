# Change: Add the `catherine` CLI — check and findings commands for agents and CI
> IDs: E1–E10 = docs/DESIGN.md §0 evidence base · C1–C9 = docs/PROPOSAL.md §2 conclusions

## Why

Agents are the primary user (PROPOSAL §4): the propose → execute → refine
loop must complete in seconds, fully offline, over JSON. The kernel has no
human/agent surface yet; without the CLI nothing can drive it from a
terminal, CI, or an agent tool-call. Beads ticket: `catherine-86i` (P3;
blocked by `catherine-1x7`; unblocks `catherine-1ay` and, with it,
`catherine-hc6`).

## What Changes

- New crate `crates/catherine-cli`: `catherine check <model.ct.json>` and
  `catherine findings <model.ct.json>`
- `check`: parse + build + execute all checks; print findings
  (human-readable by default, `--json` for agents); **exit codes mirror
  finding classes with a fixed precedence for multi-class runs** (highest
  severity wins; full class set always in the payload) so CI and agents
  can branch without parsing text
- `findings`: findings-only view (no check re-run semantics differences —
  same execution, different presentation)
- genesis-vibes adoption **in this crate only** (stable modules: envelope
  `ok/envelope_kind/data/warnings/hints`, verbosity/format clap structs):
  agents get the exact JSON envelope semantics every charly-vibes tool
  already speaks; the kernel stays genesis-free
- Parse errors and typing violations print as labeled errors with
  line/column and remediation hints — never stack traces
- The DESIGN §5 agent-loop walkthrough becomes the acceptance scenario:
  check bowling-style model → violation with witness printed → agent
  proposes equation → re-check → clean

## Impact

- Affected specs: `cli` (new capability; consumes `findings`,
  `kernel-checks`, `model-document`)
- Affected code: new `crates/catherine-cli`; workspace gains a binary
  target; `catherine-core` unchanged (dependency direction `cli → core`
  via findings/checks APIs)
- Beads: implements `catherine-86i`; unblocks `catherine-1ay` (P4) and
  co-unblocks `catherine-hc6` (P9 pilot with P4)
- Naming open question (DESIGN §8.3: `catherine` vs short bin) decided
  here: binary is `catherine`; aliases, if any, later
