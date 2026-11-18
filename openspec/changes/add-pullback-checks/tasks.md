# Tasks: add-pullback-checks

> TDD per repo policy: every task maps to a red→green→refactor cycle.
> Grounding: beads `catherine-4vs`; aleandro `synthesis.org` §3,
> `relation-inventory.org`, `findings.org` (first falsifiable
> experiment). Scope guardrail: one pullback shape, no cone machinery;
> gaps found along the way go to the findings ledger.

## 1. RED — the capability cannot even be declared

- [x] 1.1 Add fixture `pullback_square.ct.json` (apex relation object
      over a two-leg/cospan square, status accepted) and a contract
      test asserting today's behavior: `catherine check` fails with
      `error[unknown_field]` at the `pullbacks` node, exit 1 — the
      red gate from the corrected `catherine-4vs` acceptance criteria
- [x] 1.2 Pin the RED in `unknown_fields.rs` / `negative_matrix.rs`
      expectations (currently they know nothing of `pullbacks`) and an
      expressibility-gap evidence row for "pullback-shaped claims
      unstatable"

## 2. GREEN — declare (format 1.6)

- [x] 2.1 `Pullback` model struct (`apex`, `legs`, `cospan`,
      `ConstraintStatus`, status defaulting to accepted when absent)
      with canonical sort key `(apex, legs, cospan, status)`, duplicates
      kept as presentation (mirror `Coproduct`)
- [x] 2.2 Parser: accept `schema.pullbacks`, additive minor 1.6,
      empty/absent section omitted from canonical bytes (hash-neutral
      roundtrip test), canonical-order roundtrip test
- [x] 2.3 `invalid_pullback` node rejections: undeclared object,
      undeclared arrow, leg not sourced at apex, cospan target
      mismatch, leg/cospan lists whose length is not exactly two,
      repeated arrow names (mirror `invalid_coproduct` tests)
- [x] 2.4 `ct-schema-v1.json` + `web/src/types/ct-schema-v1.ts`
      surface the new field; editor validation stays shape-only, the
      kernel remains the version authority

## 3. GREEN — execute (the three checks)

- [x] 3.1 `checks/pullbacks.rs`: commutes (per apex row, reusing path
      composition with dangling short-circuit), covers (per resolvable
      fiber pair, canonical order), unique (joint-injectivity bucket
      pass) — one witness-bearing `violation`/`executed` per offending
      row or pair, per design Decision 2
- [x] 3.2 `PullbackWitness` finding shape + serialization (findings
      delta scenario: square index + verdict-specific offender),
      golden fixture per the every-kind-has-a-fixture rule
- [x] 3.3 Status rule: proposed pullbacks fail as `hypothesized`,
      countable separately; accepted set judges bounded search
      (analysis delta scenario)
- [x] 3.4 Decidability discipline: dangling leg keeps commutativity
      undefined; dangling cospan cell excludes the pair from decidable
      coverage; never double-reported with dangling findings;
      zero-row apex over empty fiber is evaluated-clean
- [x] 3.5 CLI envelope: exit 3 on pullback violations, exit 0 clean;
      `docs/USING.md` check documentation with a worked example
      (witnesses for all three verdicts)

## 4. Integration

- [x] 4.1 `schema_diff`: accepted pullbacks join the enforced set;
      accretion/weakening scan covers dropped pullbacks
- [x] 4.2 Bounded search judges candidates against accepted pullbacks
      (proposed failures surface as `hypothesized`, never blocking)
- [x] 4.3 Schema-map transport parity with coproducts: target schemas
      may declare pullbacks, validated by the same passes; a map
      dropping an apex or cospan arrow is loud (dropped-equations
      discipline) — implementation task only, no migration spec delta
      (design Decision 5)
- [x] 4.4 Expressibility-gap suite: the RED evidence row flips to
      closed; `declared-cones-ignored` stays (cones remain rejected —
      model-document delta scenario); gap-suite observer asserts the
      pullback fixture now checks with witness-bearing violations

## 5. Refactor (separate commit)

- [x] 5.1 Dedupe witness construction with the taxonomy's existing
      paths (migrate.rs refactor-task-9 pattern)
- [x] 5.2 Property tests in `checks_properties`: coverage/unique
      verdicts are order-stable under row permutation; findings
      diff-stable under declaration reordering
- [x] 5.3 Parity/golden fixture corpus extended (feeds the
      `add-lean-kernel-model` parity oracle when the twin formalizes —
      coordination note there, no blocker)
- [x] 5.4 Walk-extraction decision: skipped. The declaration wrapper
      loops are identical but content-free; the table resolvers differ
      on endpoint, payload, arity, and failure policy; the per-row
      verdicts are three distinct algorithms. Only the pairwise emitter
      is verbatim and has two instances (rule of three unmet). Extract
      a shared `pair_findings` when a third pairwise family appears.
      `label_for` remains the one real shared rule.

## 6. Documentation

- [x] 6.1 `docs/THEORY.md` limits paragraph: pullback squares over
      declared cospan arrows are now executed; general limits/cones
      remain absent, `cones` still rejected
- [x] 6.2 CHANGELOG entry (additive 1.6)
- [x] 6.3 Cross-link the `declared-cones-ignored` disposition to this
      change as the declarable path; note in `catherine-4vs` that the
      aleandro-2bc demand-side gate lifts on archive
