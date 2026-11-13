# Design: add-domain-probe-validation

Revision 2 (2026-11-12, post Rule-of-5): Decision 1 revised — the
original `probe` entry kind was rejected in favor of the in-tree
rule-lens pattern after review found the precedent unaddressed.

## Decision 1 (revised) — reuse the kata shape in a separate ledger directory (the rule-lens pattern)

The project has already validated separate-sequence science twice:
`ledger/rule-lens/` is a ledger directory of unmodified `kind: "kata"`
entries with its own self-negating floor control (`rule-lens-near-floor`,
killed), its own rollup, and a recorded **executed** baseline arm
(`prototype/rule_lens_baseline.py`, 45 frozen assertions) — and
`ledger-check ledger/rule-lens` passes green today (executed 2026-11-12:
`ok: 3 entries (2 kata), rollup verdict: survived`). The validator takes
an arbitrary directory (`ledger-check <ledger-dir>`); control-first
ordering and rollup derivation are sequence-relative, not
FizzBuzz-specific.

So domain probes need **no new entry kind**. A probe is: a new ledger
directory `ledger/probes/<name>/`, kata-shaped entries, a probe-specific
floor control, a committed expectation catalog, and — the one genuinely
new rule — a **committed executed baseline arm required for any
non-empty `unique_detections`** (enforced validator-wide, which
retroactively hardens the kata science too; all existing entries comply:
kata unique sets are empty, rule-lens recorded its baseline).

Alternatives rejected:
- *New `probe` entry kind* (revision 1's choice): rejected — the rule-lens
  precedent proves the kata shape carries probe science unmodified; a new
  kind forks entry/validator/rollup code for zero demonstrated gain and
  contradicts the minimal-implementation guardrail. Revisit only if probe
  semantics diverge from the kata fields (e.g., probes needing multi-arm
  baselines the `comparison` shape cannot hold).
- *Separate crate*: the validator/harness/catalog machinery is exactly
  what probes need; a fork would drift.

Known cost of reuse, recorded honestly: kata field vocabulary
(`kata_id`, `series_class`, `series_verdict`) is stretched for probes —
rule-lens already stretched it (its `series_verdict` quotes the
pre-registration, not ct-catlab). The stretch is documented here and in
the probe catalogs; it is a naming wart, not a soundness gap.

## Decision 2 — the floor control is per-probe, and it is a parity floor

The kata ledger's control asks "does the tool self-negate where the
hypothesis says it must?" The probe control asks the probe-specific
version: **on a fault set where the conventional baseline catches
everything, the probe must show zero unique detections.** For migration:
a loss fully visible to row-count/checksum dry run. For rule-lens
extracted tables: the near-floor precedent applies directly. A probe
series whose floor control does not self-negate is refused by the
existing `ControlDidNotSelfNegate` violation — no new machinery.

## Decision 3 — the baseline arm is the pre-registration, and it is a committed artifact

The dispatch synthesis verdict on migration Δ was: capability gap
unanimously confirmed, uniqueness unmeasured. The kata pattern already
solved this shape: expectations committed before the run, comparison
derived mechanically. For probes, the expectation catalog plus a
**committed, version-pinned baseline harness** (frozen script or pinned
tool revision, in-tree like `prototype/rule_lens_baseline.py`) *is* the
pre-registration. `unique_detections` is derived arithmetically (probe
detections minus baseline detections on the same fault set), never
asserted. An uncommitted or unexecuted baseline cannot carry a
uniqueness verdict: cited-only caps at `inconclusive`; absent caps at
parity (empty unique set). This makes "no incumbent does X" executable
instead of rhetorical.

Where the baseline is a human protocol (probe 2's manual review), the
protocol is frozen into scripted assertions **before** the run — the
rule-lens 45-assertion pattern — so the baseline is executed, replayable,
and hash-pinnable. A live unverifiable "we reviewed it" note is not a
baseline arm.

## Decision 4 — probe verdicts do not gate phases

Kata rollup verdicts gated P7/P8 (orchestrator-reserved bead flips).
Probe verdicts gate nothing automatically: they are evidence for roadmap
decisions (which domain to pilot next, which docs claim to scope),
recorded so the decision cites a ledger id. A `killed` probe kills the
*claim under probe*, not the project.

## Test shape (pinned before implementation)

- Validator unit tests: non-empty `unique_detections` without a baseline
  arm → refused; cited-only baseline + unique claim → refused
  (`survived` unreachable); probe sequence without self-negating floor
  control → refused by the EXISTING violations (regression-pinned, not
  re-implemented); green probe fixture ledger validates.
- Byte-identity test: `ledger-check` over `ledger/` output unchanged
  before/after writing probe entries to `ledger/probes/<name>/`.
- Round-trip: record → check → supersede in a probe directory, ids
  content-addressed and recomputable from bytes.
- Catalog: unregistered probe id → empty expectations → harness refuses
  to derive a verdict (the unknown-kata discipline, regression-pinned).
- Fixture-level: each inaugural probe's expectations parse under the
  catalog shape; findings matched via the existing harness
  (`catherine_ledger::harness::run`), never re-implemented.
