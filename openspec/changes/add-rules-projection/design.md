# Design: `rules --project`

One red→green→refactor cycle, beads `catherine-x94`. No format change, so no
`KNOWN_MINOR` bump and no sealed artifact is disturbed.

## Decision 1 — the bridge lives in the CLI crate

`catherine-core` types (`RulePresentation`, `Rule`, `Predicate`, `Literal`,
`ColumnType`) are already public and constructible; the rule-lens analysis is
`catherine_analysis::rule_lens::rule_lens`. The projection is therefore a pure
translation `Model -> RulePresentation`, plus a domain, and it belongs in
`catherine-cli` beside the runner that already owns `--values` binding.

Consequence to accept: the kernel never learns the word "projection". A future
`*.extract.json` (step 4) can reuse the same CLI-side seam.

## Decision 2 — the domain is the observed tuple set

`RuleDomain` is normally a product: the caller supplies values per column and
the engine enumerates the Cartesian product. That is right for a
hand-authored table and **wrong** for a projection of real rows: bogart's
1,441 selectors live in a 5-source × ~500-date rectangle, so the product
(≈2,500 inputs) invents ≈1,100 inputs that are not selectors, and the gap
finding would name them as real uncovered classes.

So for a projection:

- the domain is the set of **distinct column tuples present in the projected
  rows** — no invented input, gap impossible by construction;
- `--values` remains available and is **unioned** in: an explicitly declared
  input that no row covers is a genuine gap, reported as `underconstraint`
  with the declared class as the witness. The help text states that a product
  declaration can contain combinations no selector has; that is the declared
  scope, not a claim about the data.

`--values` for a projection with no `--project` keeps today's product
semantics unchanged.

## Decision 3 — rules are rows; action is identity by default

One rule per instance row, in canonical row order (row ids are assigned in
sorted-name order at parse, so the rule indices — which the overlap witness
carries — are deterministic).

- predicate = `And[Cmp{column: c, op: Eq, value: String(target row name)}]` for
  every `--columns` entry, in the order given (order is presentation only; the
  conjunction is commutative and the analysis never depends on it);
- column name = the arrow name; column type = `String` always. Row names are
  text; guessing `Int` from a numeric-looking name would make the domain
  bindable only by accident;
- action = the row's own name (identity) unless `--action <arrow>` is given, in
  which case it is that arrow's target row name. Identity is the projection's
  normal reading: *two rows share the columns and are still different rows*;
- an arrow named in `--columns`/`--action` must exist, must be sourced at the
  projected object, and its target must have a value for every row — otherwise
  `bad_argument` / `projection_incomplete` (Decision 5).

## Decision 4 — policy is fixed to `DeclaredDisjoint`

A projection's claim is exactly "these rows are a function of these columns".
`PriorityOrder` or `FirstMatch` would resolve a collision by rule order, i.e.
silently pick one of the two real captures behind an ambiguous selector — the
one thing the acceptance run must never do. No `--policy` flag is offered;
adding one later is a deliberate, reviewed decision, not a knob.

Overlap therefore maps through the shipped witness encoding: `violation`,
`executed`, `EquationWitness { equation: lower rule index, row: shared input
index, lhs/rhs: the two rule indices }` (`catherine-analysis::rule_lens`
module docs). One finding per (pair, shared input) — the witness set stays
exact, never a summary.

## Decision 5 — refusals are labeled errors, never silent skips

| Situation | Outcome |
|---|---|
| `--project` on a document that carries `rule_presentation` | `bad_argument`: two sources of the same presentation, never an implicit preference |
| `--project` without `--columns`, or an empty/duplicate column entry | `bad_argument` |
| unknown project object, or an arrow not sourced at it | `bad_argument` |
| a projected column/action cell is null | `projection_incomplete` naming the row and arrow — a row that cannot be projected is not silently dropped from either rules or domain |
| the projected object has zero rows | `empty_projection` — an empty projection supports no claim, and a clean exit would be the vacuous-clean defect |

All are labeled errors with exit code 1 (the parse/build-failure class), not
findings: they describe the invocation, not the model.

## Decision 6 — the claim is not sealed

Projection output is not part of the model hash, is not recorded in the ledger,
and cannot be cited as sealed evidence. The reading is: *on this snapshot of
rows, projected this way, these collisions exist.* The CLI help says so and the
spec requirement states it, because the temptation to treat a projection run as
a ledger-grade claim is exactly the failure the P6 stop condition exists to
prevent.

## Verification

- Hermetic: `crates/catherine-cli/tests/cli_rules_projection.rs` — three-row
  fixture with one collision (violation + witness pair), a clean projection
  (exit 0, zero findings), a declared-but-uncovered `--values` input (gap,
  exit 6), an `--action` projection, and each refusal.
- Grounded acceptance (`fixtures/probes/capture-selector/`, pre-registered in
  its README): exactly 3 violations on the slice, both capture rows in each
  witness, no gap on the 30 undeclared zero-match selectors, exit 3; declaring
  `source=nacional/primera edition_date=2026-11-07` adds the gap.
- Regression: the shipped `rules` scenarios (`tests/cli_rules.rs`) are
  untouched — the projection path must not change the presentation path.

## Risks

- The acceptance is a *snapshot* (live corpus, 2026-11-13). The fixture is
  committed with its census, so the evidence is reproducible from the artifact,
  not from the live database.
- Rule count 1:1 with rows means a large projection is a large rule list;
  budget and enumeration stay proportional to the domain (rows), not to rules,
  so this is a size concern for payloads, not an algorithmic one. State the
  bound in the help text; do not pre-optimize.