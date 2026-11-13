# Design: add-domain-probe-validation

## Decision 1 — lighter sibling, not a widened kata kind

The kata ledger's science is control-first: FizzBuzz must self-negate
before anything else is recorded, and the rollup is sealed. Domain
probes have no negative control to run first (their "control" is the
conventional-baseline arm, per probe) and must never reopen the sealed
project verdict. Widening `kind: "kata"` would force every probe past
the control-first validator or weaken it. So: a new `kind: "probe"`,
same crate, same content-addressing (`catherine_core::hash` port), same
closed verdict vocabulary, separate sequence `ledger/probes/`.

Alternatives rejected:
- *Separate crate*: the validator/harness/catalog machinery is exactly
  what probes need; a fork would drift.
- *Reuse kata entries with a `probe` flag*: control-first ordering and
  rollup derivation both key off `kind == "kata"`; a flag would leak
  probe semantics into the sealed-rollup invariant.

## Decision 2 — the baseline arm is the pre-registration

The dispatch synthesis verdict on migration Δ was: capability gap
unanimously confirmed, demand and uniqueness unmeasured. The kata
pattern already solved this shape: expectations committed before the
run, comparison derived mechanically. For probes the expectation
catalog plus the named baseline arm *is* the pre-registration — a
unique-detection claim is arithmetically derived (probe detections
minus baseline detections on the same fault set), never asserted. No
baseline arm ⇒ empty unique set ⇒ parity ceiling. This makes the
"no incumbent does X" literature claim executable instead of rhetorical.

## Decision 3 — probe verdicts do not gate phases

Kata rollup verdicts gated P7/P8 (orchestrator-reserved bead flips).
Probe verdicts gate nothing automatically: they are evidence for
roadmap decisions (which domain to pilot next, which docs claim to
scope), recorded so the decision cites a ledger id. A `killed` probe
kills the *claim under probe*, not the project.

## Decision 4 — baseline arms are executed or cited, and labeled

Migration baseline: statement-level linting behavior is reproducible
locally (Atlas/Flyway-class tools are installable) — execute where
possible, label EXECUTED. Where a baseline cannot be executed (e.g.,
manual review as the rule-lens probe's baseline), the procedure is
recorded in the entry and labeled accordingly — the same
PROVED/EXECUTED/ILLUSTRATED discipline the kernel enforces on its own
answers. A cited-but-not-executed baseline caps the verdict at
`inconclusive` on uniqueness questions.

## Test shape (pinned before implementation)

- Validator unit tests: each task-1 rejection is one test with a
  mutated fixture ledger; green ledger validates.
- Byte-identity test: `ledger-check` over `ledger/` output unchanged
  before/after writing probe entries to `ledger/probes/`.
- Probe round-trip: record → check → supersede, ids content-addressed
  and recomputable from bytes.
- Fixture-level: each inaugural probe's expectations file parses under
  the catalog schema; findings matched via the existing harness, never
  re-implemented.
