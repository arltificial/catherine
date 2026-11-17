---
tags: [lean, assurance, parity, proofs]
bead: catherine-xvh
---

# Lean twin implementation learnings (catherine-xvh)

Ticket: `catherine-xvh` — external Lean 4 reference semantics of the
catherine kernel, parity oracle, soundness/completeness proofs.
Change: `openspec/changes/add-lean-kernel-model` (14/14 tasks).
Recorded: 2026-11-17, after the ro5u gate reached `max_critical: 0`
(verdict READY) and the stop gate recorded **hold**
(`ledger/lean/0001-cbef9219.json`).
Code tree at recording: `ba06672` (the five-commit change series; these
docs uncommitted at the time of writing).

This is the transferable record: what the exercise taught about assurance
artifacts, proof engineering in Lean, and the gates around them — not the
delivery log.

## Evidence base

| Fact | Value |
|---|---|
| Corpus enumerated | 190 `*.ct.json` under `fixtures/{katas,parity,probes}` |
| Compared against `catherine check --json` | 180 |
| Parse/build-rejected (recorded `skipped_parse`, dynamic) | 10 |
| Trivially agreeing (no findings on either side) | 102 |
| Findings-bearing agreement | 79 |
| Divergences / build-report mismatches (7 parity fixtures) | 0 / 0 |
| Lean kernel modules | 803 lines |
| Proof tree (`Catherine/Proofs/*`) | 2124 lines, 49 private helper lemmas |
| Oracle + mirror | 1161 lines |
| Public theorems audited | 18 (listed in `Catherine/Axioms.lean`), `sorry`-free, axioms ⊆ `{propext, Classical.choice, Quot.sound}` |
| ro5u findings | 1 HIGH, 3 MEDIUM, 7 LOW; HIGH and all MEDIUMs fixed, 1 LOW deferred |
| Stop gate | hold — parity had already pinned the semantics; proofs added universal statements without surfacing a defect or contract hole |

Gates at the final tree: `lake build` (24 jobs), `lean-parity` and
`--check-build` exit 0, `scripts/check-lean-axioms.sh` exit 0,
`just ci` exit 0, `pretender check --mode gate` exit 0,
`openspec validate add-lean-kernel-model --strict` valid, `ah check`
clean.

Reproduce: `(cd lean && lake build && lake exe lean-parity &&
lake exe lean-parity --check-build)` and
`scripts/check-lean-axioms.sh`; the oracle report is written with
`--report PATH`.

Of the 10 `skipped_parse` documents, nine are malformed or unbuildable
fixtures; the tenth (`parity/typing_violation`) is the deliberate
presentation-route fixture, compared Lean-only against its vendored
`expected.json` outcomes.

## Lessons

### 1. Parity green is not coverage — report trivial agreement

102 of the 181 agreements are both-sides-empty (54% of the 190
enumerated documents). A single pass/fail bit
would let 56% of the agreements assert nothing while the oracle looks
stronger than it is. The harness therefore reports
`trivially_agreeing` and `findings_bearing_agreeing` separately, and a
zero findings-bearing count is a red gate, not a green one.

Corollary: enumerate the corpus dynamically and surface what was
rejected (`skipped_parse`) rather than filtering it away. The 10 rejected
documents are a fact about the parser contract, not noise to hide.

### 2. The assurance gate is the weak link, not the semantics

The only HIGH finding in this change's ro5u (CORR-001) was in the new
axiom audit, not in the kernel or the proofs:

- Lean never echoes `axiom` declaration lines, and `#print axioms`
  reports the bare name (`[cheat]`), so the original
  `sorryAx` substring test plus its `^axiom` source scan accepted a
  proof built on a user-declared axiom.
- A second gap (CORR-002): only four declarations were audited, while
  `lake build` accepts `sorry` with a warning.

The fix asserts the axiom *set* against an allowlist
(`{propext, Classical.choice, Quot.sound}`), fails on any
`declaration uses 'sorry'` build warning, and scans every
`lean/Catherine/**/*.lean` for `axiom` declarations. The script also grew
a `--file` seam so the gate can be tested with planted bad files — which
immediately exposed a relative-path bug (EDGE-003, fixed).

Rule: a gate is a claim. Give it a red-test (`[cheat]`, `[sorryAx]`,
empty audit) before trusting it, and never audit a subset without naming
why.

### 3. One-sided implications hide behind symmetric prose

`DanglingDiscipline.no_violation_through_unresolved` proved
`lhs = some ∧ rhs = none`, while the docstring and README claimed “an
equation path that short-circuits … emits no violation”. The mirror
orientation and the both-`none` case were unproved until ro5u CORR-003.
The kernel was correct in all three cases; the *certificate* was
narrower than the claim.

Rule: review each theorem statement against every clause of the prose it
cites, in both directions. A hypothesis set that forces one side to
resolve is a red flag.

### 4. Proof cost is list/ordering plumbing, not mathematics

2.6× the kernel's line count, almost all of it membership bridges over
`mapIdx` / `filterMap` / `foldl`-grouped rows and the taxonomy order —
not category theory. The equation-completeness pass was the cheapest
(about 2.5 minutes of agent wall-clock); the Soundness and
DanglingDiscipline passes were the expensive ones (~16 and ~25 minutes,
including reading and iteration), with the key/coproduct membership
lemmas and the builder trichotomy the bulk.

Implication: budget proofs by walk complexity (families, grouping,
ordering), not by conceptual depth. Design the walk so its families are
nameable (see 6).

### 5. Freeze the shared lemma contract before parallel proof work

Three agents on disjoint proof files duplicated the same bridges —
`mem_danglingFindings_iff` vs `mem_danglingFindings_imp`, two
`mem_coproductFindings` families, duplicated `mapIdx`/`range` plumbing.
Parallelism helped throughput but paid twice in duplicated proof text
(EXCL-001, deferred with named consolidation targets).

Next time: define `Proofs/Membership.lean` (the membership/ordering
lemmas every family proof consumes) as a contract *before* fanning out;
each proof file then imports it. This is the same “shared contract”
discipline as a type interface.

### 6. Proof-friendly API is part of the kernel's testability surface

`Check.equationFindings` inlines an anonymous `mapIdx` lambda, so a
proof cannot name the per-equation family; the completeness proof
restated the body as `equationFamily` to get a handle on it. Public (or
at least named, private) walk helpers — one per finding family — remove
that restatement and make statements read like the spec.

### 7. Write the data contract before the harness

`design.md` originally described the Lean input as the `expected.json`
mirror shape. Implementation needed `{schema, instance: {rows, values}}`:
`expected.json` has no schema, and the twin must build the instance
itself to test the builder. The design was corrected during ro5u
(CLAR-001). The harness also reproduces Rust's interning rule (dense row
ids in sorted row-name order, cell resolution target-first then first
declared object) in Lean — duplicated normalization, covered only by
parity.

Next time: write the mirror JSON contract (fields, ordering, resolution
rule) as a document/type *first*, then the harness against it.

### 8. Schema growth is broad and gate-visible

Adding `Option<DecisionRecord>` to the ledger entry:

- forced `decision: None` into ten constructor sites across seven
  existing test files (serde defaults do not help Rust struct
  literals);
- pushed `entry.rs` (423) and `validate.rs` (444) past the 400-line
  pretender gate, forcing an unplanned `decision.rs` split and two
  behavior-preserving compactions.

The gate did its job, but the split should have been planned with the
schema addition: one module per entry kind keeps the file-size and
complexity budgets aligned with the data model.

### 9. Deferring key/coproduct completeness was correct — and needs a named caveat

Soundness covers their findings; the completeness theorems were deferred
by design. `KeyCollision` deliberately omits the dense-row-space bounds
the walk supplies, so `mem_keyFindings_forward` is one-directional. That
is fine for soundness, but the predicate is *not* strong enough for a
future key-completeness theorem. Weaker predicates are acceptable only
when the docstring names what they do not support.

### 10. Process at the gate: freeze, then review

Code was still moving (ledger module split, doc fixes) while the reviewer
cited line numbers; the review had to be re-run after fixes, and its
first-pass line anchors went stale. The two-pass review worked, but
freeze → fix known gate failures → review → fix → re-verify is cheaper.

Related: subagents skipped formatting; the first `just ci` failed at
`cargo fmt --check` on the new ledger tests. One orchestrator-level
`cargo fmt --all` after the parallel wave, then one gate run, is the
right shape.

### 11. TDD in Lean uses the axiom audit as the test harness

The RED state was not a failing test but `sorry`-bearing theorems whose
`#print axioms` output contained `sorryAx`; the audit script exiting 1
was the observed red. That gave a concrete, reproducible red→green
transition for task 7 → tasks 8–10, and the same command is the
completion criterion.

### 12. The stop gate produced an honest negative result

Parity found zero semantic divergences; the proofs found no kernel
defect and no contract hole. The evidence therefore funded neither an
extension (search/migration) nor a retirement (the tree is std-only and
cheap to keep). Recording `hold` with extend/retire counterfactuals kept
“proofs landed” from silently becoming “therefore invest more”.

## What to do differently next time

1. Freeze `Proofs/Membership.lean` (family membership + ordering lemmas)
   before parallel proof work.
2. Write the mirror/data contract first, then the oracle.
3. Build the gate with red self-tests from day one; assert axiom sets,
   not substrings.
4. Review statements against prose clause-by-clause, both orientations.
5. Plan the module seam when extending a schema (one file per entry
   kind).
6. Report assurance coverage (non-trivial agreement share) as a
   first-class number in every parity report.

## Open / deferred

- EXCL-001: consolidate the duplicated private membership lemmas
  (`Soundness` ↔ `DanglingDiscipline`) into a shared module; named
  candidates are recorded in
  `openspec/changes/add-lean-kernel-model/ro5u-review.md`.
- Key and coproduct completeness theorems remain deferred; a future
  statement needs `KeyCollision` strengthened with row-space bounds.
- Maintenance risk: the twin mirrors Rust shapes by hand; the CI parity
  gate is the only thing keeping the mirror honest — findings over the
  180 Rust-compared documents, build reports over the 7 parity fixtures.

## Pointers

- Durable practice distilled from these lessons:
  `.wai/resources/reflections/assurance-gates-are-claims.md`.
- `openspec/changes/add-lean-kernel-model/{design.md,ro5u-review.md}`
- `lean/README.md` — what the twin claims and does not
- `ledger/lean/0001-cbef9219.json` — stop-gate decision
- `scripts/check-lean-axioms.sh` — the proof gate, with its self-test seam
