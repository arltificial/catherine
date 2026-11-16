# Change: Model composition at a seam (joint check of shared-subschema documents)

> Grounding: cotorreo `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d`
> ("Catherine Feature Gap Test", gap **G1**, 2026-11-15) and
> `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (15-case gap suite, cases
> on joint checking); DESIGN.md evidence row E5 ("composition-as-pushout
> only when justified"). The kimi fixture demonstrates the gap exactly:
> `check billing shipping` → `error: unexpected argument` (exit 2),
> `distinguish` across the two → `schema_mismatch`, and the hand-merged
> document finds the seam violation (exit 3) — the check is executable,
> the tool just cannot perform the composition.

## Why

Two model documents that share a sub-schema (billing and shipping each
model their half of `Order → Region`) each check clean, while order `o3`
is invoiced in `north` and dispatched from `south`. The seam claim spans
both documents and has nowhere to live: composition of models is the
blog thesis the tool's own evidence base cites (E5), and the value
proposition analysis ranked it the #1 gap because it is the only one
with no partial implementation. Today the composition step is a manual,
unvalidated hand-merge (Recipe D) — the merge itself is never checked.

## What Changes

- **`catherine check` accepts two or more documents**: paths are glued
  over their shared declared names (objects, arrows, and rows identified
  by name), and all equations and keys from every input are executed
  against the merged instance. Single-document behavior, exit codes, and
  the JSON envelope are unchanged.
- **Cross-document equations**: an equation in any input may reference
  names declared in another input — the pushout makes them legal. This
  is how the seam claim ("invoiced where dispatched") is stated; it is
  hashed in the document that declares it, so the claim is sealed model
  evidence, not a CLI-side annotation.
- **Conflicts are labeled errors, not silent merges**: a name declared in
  two inputs with disagreeing structure (arrow endpoints, row value for
  the same row id) fails with a stable code naming both documents and
  the node — never a last-writer-wins union.
- **Findings carry source attribution**: on a multi-document run each
  finding names the document that declared its violated constraint
  (additive envelope field, unreferenced by existing requirements).
  Underconstraint questions that exist only after gluing are the seam
  discovery signal.
- **Empty interfaces warn**: a glue with zero shared names proceeds but
  emits a labeled envelope warning naming the inputs — a disjoint union
  is never presented as a seam check.
- **Scope guardrail (the G1 decision)**: the output is findings, never a
  merged-model platform. No document emission, no model algebra; the
  hand-merge recipe stays the only way to materialize the pushout until
  a fixture demands otherwise.
- **Probe**: the composition claim is pre-registered as
  `ledger/probes/composition/` — floor control on a fault the
  single-document check already catches (zero unique detections
  required there), and the committed, executed baseline arm is the
  hand-merged document from the gap fixture. The detection arithmetic is
  expected to land at parity; the value claim under probe — eliminating
  the error-prone merge step — is recorded as a named gap on the entry,
  never as a detection claim.

## Impact

- Affected specs: new `composition` capability; `cli` (check becomes
  variadic over documents); `findings` untouched (no new finding kinds —
  the additive attribution field is unreferenced by an existing
  requirement, the CORR-002-adjacent pattern used by keys for `diff`).
- Affected code: `crates/catherine-core/src/` (gluing as a checked
  operation over parsed documents — schema/instance union with conflict
  detection), `crates/catherine-cli/src/run.rs` (variadic path),
  `crates/catherine-analysis/src/` (post-glue underconstraint
  attribution).
- Evidence: billing/shipping fixture — each doc clean alone, joint run
  fires the seam violation naming both documents; conflicting row value
  → labeled error at the offending node; single-document runs
  byte-identical to today (no format change, so every sealed hash and
  ledger `model_hashes` entry stays valid).
- Beads: feature ticket `catherine-0qz` (RED→GREEN); refactor ticket
  filed at apply.
- Depends on: `add-expressibility-gap-suite` (its composition rows must
  exist before this change's Verification flips them).
- Fixture provenance: the billing/shipping fixtures are re-authored from
  the corpus transcripts (`kimi/1a1dee7a` G1 scenario) — the sandbox
  artifacts referenced there are provider-side and not recoverable.
- Conservative git profile: commits fine, no push/deploy.
