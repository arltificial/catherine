# Change: Project instance rows onto a rule table (`rules --project`)

> Grounding: `.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md`
> step 1 (§3 "Design, step 1 — projection bridge", §7 row 1). Successor to the
> closed `catherine-8v9` re-model.

## Why

The storage-granularity re-model found 0 findings on bogart's FTS anchor and
structurally excluded both real defects the pilot ever found there
(`bogart-4wt`: one `(source, edition_date)` selector matching two captures).
The cause is not a missing check — it is a missing **bridge**: the rule-lens
engine (P8) already encodes predicates, actions, and `DeclaredDisjoint`
overlap, but a document can only feed it a hand-written
`rule_presentation`. An instance of stored rows cannot reach it, so an agent
with a `.ct.json` of a real table must hand-author the table it already holds
as data.

The measured blind spot is small and precise: of bogart's 1,441 live selectors,
3 are ambiguous and 67 are zero-match (census reproduced in
`fixtures/probes/capture-selector/`). The ambiguous three are unreachable by
`catherine check` because no storing column records the match; they are
reachable by projecting the capture rows — two rows agreeing on
`(source, edition_date)` and disagreeing on which capture they are is exactly
an overlap under `DeclaredDisjoint`.

## What Changes

- `catherine rules <model> --project <Object> --columns <arrow,...>
  [--action <arrow>] [--values ...]` synthesizes a `RulePresentation` from the
  instance rows of `<Object>` and analyzes it with the existing rule lens —
  no new finding kinds, no format change, no new analysis code path.
- Rules are built one per instance row (canonical row order): the predicate is
  the conjunction of column equalities (column = an arrow from `<Object>` to a
  value object; the value is the target row's name), and the action is the
  row's own name (identity) or the target row's name of `--action`.
- Policy is fixed to `DeclaredDisjoint`: a projection asserts that its rows
  are a function of the projected columns, so a collision is a declared
  contradiction, never a precedence question. There is no `--policy` flag —
  a `PriorityOrder` projection would silently resolve exactly the defect class
  this bridge exists to surface.
- The domain is the **observed** tuple set (the distinct column tuples of the
  projected rows), optionally unioned with explicitly declared
  `--values column=v1,v2,...` product inputs. A projection never invents an
  input, so selectors with no row produce no finding unless the caller
  declares them.
- Refusals stay labeled errors, never silent skips: a null cell in a projected
  column or action (`projection_incomplete`), an empty projection
  (`empty_projection`), a document that already carries a `rule_presentation`
  while `--project` is given (`bad_argument`), and malformed
  `--columns`/`--action` (`bad_argument`).
- The envelope shape is unchanged: findings and warnings only. The projection
  is visible in the invocation and in each rule's name (the row name), so an
  acceptance run needs no new payload field.
- **Boundary, stated wherever the result is cited**: the projected claim lives
  outside the document hash and the ledger. It is evidence about a projection,
  not a sealed model claim.

## Impact

- Affected specs: `cli` (ADDED requirement). `analysis` is deliberately
  untouched — the projection feeds the shipped rule-lens analysis unmodified,
  which is the whole point; `model-document` is untouched (no format change,
  `KNOWN_MINOR` stays 2).
- Affected code: `crates/catherine-cli/src/{cli.rs,rules.rs}` (+ a projection
  module), `crates/catherine-cli/tests/`.
- Fixtures: `fixtures/probes/capture-selector/` (generator
  `prototype/bogart_capture_slice.py`, committed before the run).
- Beads: `catherine-x94` (this change). The two spec drifts fixed in step 0
  were `catherine-rgh` and `catherine-0hn` (closed).
- No capability is gated on this: the bridge adds evidence, it does not
  validate or invalidate the granularity rule.