# Tasks: add-pattern-lenses

One feature cycle per child ticket, red first: each item turns green only
when the lens reproduces the per-fault classes recorded in `ledger/katas/`
(parity and killed/gap rows stay silent) against the expectation table
shipped complete by `catherine-mx1.2`. No item edits `ledger/katas/`
(sealed) or `tests/pattern_expectations.rs`; the implemented-status edit in
`docs/ref/pattern-kata-dispositions.md` is made by the orchestrator at
ship, one row per landed lens, to keep parallel lens tickets file-disjoint.

## RED → GREEN (one cycle per child ticket)

- [ ] 1. `catherine-mx1.2` — registry, law-scope metadata, and reference
      lens seam: red — `find("unknown")` errors, a duplicate id in the
      registry literal is rejected, the reference lens's recorded Visitor
      classes fail, the replay harness reads an (initially empty)
      expectation table; green — `pattern/mod.rs` (`LawKind`,
      `PatternMeta`, `LensInput`, `LensOutcome`, `PatternLens`,
      `LensError`, `registry`, `find`, `run`), test-only reference lens in
      `tests/pattern_reference.rs`, `tests/pattern_replay.rs` +
      `tests/pattern_expectations.rs`, `catherine pattern` CLI with
      `--list` and fail-closed unknown id. Law scope: the seam only —
      registry metadata, taxonomy reuse, fail-closed lookup, replay gate;
      no pattern law ships here (the shipped Visitor lens is item 9).

- [ ] 2. `catherine-mx1.3` — `composite` lens: red — identity/totality
      replay rows (`violation` at the nil mount, `dangling` for the broken
      parent link, lost cells on the flatten schema map); green —
      `pattern/composite.rs`. Law scope: identity/totality equations;
      associativity unexpressible. Evidence:
      `ledger/katas/composite-null-object/`.

- [ ] 3. `catherine-mx1.4` — `null-object` lens: red — the nil-identity
      row (`parent(nil) ≠ root` at the nil row) and the silent clean arm;
      green — `pattern/null_object.rs`. Law scope: nil-identity equations
      (identity/totality for the nil element). Evidence:
      `ledger/katas/composite-null-object/`.

- [ ] 4. `catherine-mx1.5` — `builder` lens: red — declared-step trace
      (`dangling`/lost rows and cells) and the splice-order `violation`,
      with the duplicated-row row asserted as parity (no unique claim);
      green — `pattern/builder.rs`. Law scope: emitted part traces to a
      declared step; splice order equation. Evidence:
      `ledger/katas/builder-monoid/`.

- [ ] 5. `catherine-mx1.6` — `adapter` lens: red — link-level
      delegation/field rows, one `dangling` per dropped cell; green —
      `pattern/adapter.rs`. Law scope: link-level delegation and field
      laws; coverage-only (a complete cross-product sample collapses the
      unique set). Evidence: `ledger/katas/wrapper-laws/`.

- [ ] 6. `catherine-mx1.7` — `decorator` lens: red — the stacking-order
      `violation` at the unsampled stack row and the clean arm; green —
      `pattern/decorator.rs`. Law scope: stacking transparency equation;
      coverage-only. Evidence: `ledger/katas/wrapper-laws/`.

- [ ] 7. `catherine-mx1.8` — `proxy` lens: red — the identity-rewire
      `violation` while the result path still agrees; green —
      `pattern/proxy.rs`. Law scope: identity-preservation equation;
      coverage-only. Evidence: `ledger/katas/wrapper-laws/`.

- [ ] 8. `catherine-mx1.9` — `visitor` lens: red — missing-method gap
      (`underconstraint`/`hypothesized`), duplicate registration under
      `DeclaredDisjoint` (`violation`), shadowed and derivable handlers
      (`redundant`), new-variant coverage, with F1/F2 asserted as parity;
      green — `pattern/visitor.rs`. Law scope: variant coverage and
      handler-class shape; no evaluation semantics. Evidence:
      `ledger/katas/visitor-sum-types/`.

- [ ] 9. `catherine-mx1.10` — `interpreter` lens: red — variant-domain
      coverage rows on the sum-type fixture (the missing-variant gap and
      the above-floor clean arm); green — `pattern/interpreter.rs`. Law
      scope: coverage of the variant domain only; no evaluation semantics.
      Evidence: `ledger/katas/visitor-sum-types/` (the shared sum-type
      surface; distinct from item 8's handler-class scope).

- [ ] 10. `catherine-mx1.11` — `memento` lens: red — lost-cell rows per
      dropped field, the optional-field and version-skew rows, the
      non-invertibility proof, and the identity control asserted silent;
      green — `pattern/memento.rs`. Law scope: lost-data enumeration +
      round-trip non-invertibility proof. Evidence:
      `ledger/katas/memento-roundtrip/`.

- [ ] 11. `catherine-mx1.12` — `state` lens: red — dangling transition
      targets, declared-cell totality rows, drift `violation` on the
      unsampled cell, schema-level questions as `hypothesized`, and the
      unreachable-state row asserted silent (honest null kill); green —
      `pattern/state.rs`. Law scope: transition laws and declared-cell
      totality only; unreachable-state detection unexpressible. Evidence:
      `ledger/katas/state-discovery/`.

- [ ] 12. `catherine-mx1.13` — `chain-of-responsibility` lens: red —
      handler-order rule-lens rows: gap `underconstraint`, shadow and
      derivable `redundant`, `DeclaredDisjoint` overlap `violation`, with
      the behavioural/parity rows asserted as parity; green —
      `pattern/chain_of_responsibility.rs`. Law scope: rule-lens
      gap/shadow/overlap/derivable over declared handler order. Evidence:
      `ledger/katas/cor-strategy-rules/`.

- [ ] 13. `catherine-mx1.14` — `strategy` lens: red — branch selection as
      rule regions above the triviality floor (the derivable branch row
      `redundant`), with the near-floor control asserted as a parity
      result, not a win; green — `pattern/strategy.rs`. Law scope:
      branch selection as rule regions, above the floor. Evidence:
      `ledger/katas/cor-strategy-rules/`.

## REFACTOR

- [ ] 14. `catherine-mx1.15` — extract shared law-check helpers after
      Composite, Builder, and Memento have landed: finding construction,
      variant/row enumeration, witness formatting, and the scope metadata
      carrier; remove the duplication across lens modules with no
      behaviour change (each lens's replay stays identical per fault, no
      new findings), keep every `//!` header, and stay under the
      `pretender.toml` complexity/size/nesting caps. Law scope: refactor
      only — no law, scope string, or expectation row changes.