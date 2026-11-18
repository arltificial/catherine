# Expressibility dispositions: known limitation → executable case

Status: every row below pairs a limitation claim from an investigation
with an executable case under
`crates/catherine-cli/tests/expressibility_gaps/`. The suite runs each
case's fixtures through the real `catherine` binary and compares the
structural observation (exit code, finding kinds/labels, labeled-error
codes) against the row's disposition — in both directions. A row is
**truth at registration time**: the observation records what HEAD does,
not what the 0.1.0-era chat believed (openspec design Decision 2).

Provenance: the claims come from the investigations cited per row
(provider/conversation-id, synthesis note, or review finding): two
conversation transcripts in the research corpus,
`claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (the 15-case runnable
suite) and `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d` (gaps G1–G3:
composition, naturality, coproduct), plus the synthesis notes
`aleandro-2bc` (`docs/synthesis.org` §3) for the pullback row. The
claims are cited for provenance; the `Verified HEAD observation` column
is the truth.

Dispositions:

- **gap** — the limitation is observable at HEAD; the case asserts its
  signature and the suite passes while it holds.
- **closed** — the limitation no longer holds at HEAD; the case asserts
  the changed behavior and the row cites the fixing evidence. A
  `closed` row whose case regresses fails the suite.

Flip protocol: a capability change that fixes a `gap` flips that row's
Disposition cell to `closed` and fills its Sealing evidence in the *same
change*; a regression flips `closed` back to `gap` with the regression
recorded. The case observation is written so the flip itself is a
one-cell table edit: `gap` rows fail the suite as soon as their fixture
stops showing the limitation, and `closed` rows fail as soon as their
fixture shows it again. Rows whose Sealing evidence names a sibling
change (composition, naturality, coproduct) flip in the PR that lands
that change.

Machine-read contract: `runner.rs` parses this table's first column
(case id) and fifth column (disposition, bare `gap` or `closed`). Cells
carry no `|`; the command spellings are exact invocations that were
executed at RED time.

## Table

| Case id | Claimed limitation | Source conversation | Verified HEAD observation | Disposition | Sealing evidence |
|---|---|---|---|---|---|
| identity-empty-path-equations | An equation side cannot be empty, so identity and inverse laws (`neg∘neg = id`, `restore∘save = id`) cannot be stated | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (cases g1a, g1b) | `catherine check identity_involution.ct.json` → exit 1, `error[syntax]` "equation path must be non-empty"; the memento fixture is rejected identically | gap | Missing: an identity path form (an empty equation side or a named identity arrow) the parser accepts and the checker executes |
| optional-references | An arrow cannot be declared optional; a legitimately absent value is reported `dangling` | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (case g4) | `catherine check optional_reference.ct.json` → exit 4, one `dangling` finding, witness `{arrow: address, src: 1}`; the fixture's former `optional: true` key is now itself an `unknown_field` rejection (catherine-bak), so optionality still cannot be declared | gap | Missing: an optional-arrow declaration with defined semantics through absent values |
| unknown-top-level-fields | Unknown document, schema and instance fields are silently ignored | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (case g3a); `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d` (unknown `instance_map` probe) | `catherine check unknown_top_level_fields.ct.json` → exit 1, `error[unknown_field]` at line 3 naming `notes` (the other two unknown keys are rejected the same way once the earlier ones are removed); previously exit 0 with empty `findings`/`warnings` | closed | `crates/catherine-core/tests/unknown_fields.rs` |
| declared-cones-ignored | A `schema.cones` declaration is accepted and ignored; the methodology's cone language has no execution | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (case g5) | `catherine check declared_cone.ct.json` → exit 1, `error[unknown_field]` at line 7 naming `cones`; previously exit 0 with empty `findings`/`warnings`. Cross-link: pullback squares are now declarable through `schema.pullbacks` (`add-pullback-checks`) and execute as three verdicts (`pullback-absence`), while `schema.cones` remains rejected as `unknown_field` | closed | `crates/catherine-core/tests/unknown_fields.rs` |
| pullback-absence | A pullback-shaped claim cannot be declared or executed; `schema.cones` is rejected as `unknown_field` | `aleandro-2bc` (docs/synthesis.org §3) | `catherine check pullback_claims.ct.json` → exit 3, one `violation`/`executed` finding with the commutes witness `{pullback: 0, row: 0, lhs: 0, rhs: 1}` over the square's only apex row; previously a pullback claim was unstatable (`pullback-absence`), so the square could not be written down | closed | `crates/catherine-core/tests/pullbacks_checks.rs` |
| non-functorial-schema-map | `schema_map` is not checked for functoriality: an arrow may map to a path with the wrong endpoints | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (case g2a) | `catherine migrate nonfunctorial_schema_map.ct.json` → exit 0, `data.lost: []` (0 lost data findings) though `f: A→B` maps to `h: Y→Y` under `A↦X, B↦Y` | gap | Missing: endpoint checking of mapped arrow paths and equation preservation across the map |
| dropped-equations-on-migrate | A schema map whose target forgets a source equation is accepted without a finding | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (case g2b) | `catherine migrate dropped_equation_migration.ct.json` → exit 0, `data.lost: []` though the target declares `equations: []` and drops the source triangle | gap | Missing: a migration requirement that the target schema preserve the source's accepted equations |
| dropped-pullbacks-on-migrate | A schema map that drops a pullback support arrow whose cells are all null is accepted without a finding | rule-of-5 review 2026-11-18 | `catherine migrate dropped_pullback_migration.ct.json` → exit 0, `data.lost: []` (0 lost data findings) though the map assigns the square's `p1` leg to `null` and the target schema carries no `pullbacks` | gap | Missing: a migration finding for a lost constraint declaration |
| vacuous-clean-instances | A zero-row instance checks clean with no warning; an equation evaluated over zero rows is never evaluated | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (cases g3a, g3b) | `catherine check zero_rows_instance.ct.json` → exit 0, `findings: []`, `warnings: []` | gap | Missing: a vacuity diagnostic for equations evaluated over zero rows. The wrong-shaped instance this row used to cover is now refused as `unknown_field` naming the unrecognized key (catherine-bak; originally `syntax` under catherine-3en, pinned by `crates/catherine-cli/tests/negative_matrix_instance.rs`), not a gap |
| distinguish-observation-scope | `distinguish` requires a hand-picked observation set and reports `equivalent-in-scope` even when an unobserved arrow differs | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (cases g7a, g7b) | `catherine distinguish distinguish_left.ct.json distinguish_right.ct.json` → exit 2 (missing `--observe`); `--observe 'f@a#0'` → exit 0 `equivalent-in-scope`; `--observe 'g@a#0'` → exit 0 `distinguishable` | gap | Missing: an automatic or reported observation scope (path enumeration up to a declared depth) |
| associativity-scaffolding | Associativity is expressible only with hand-built Triple rows and wiring equations; omitting the failing triples makes the model clean | `claude/b417f19b-fdbc-4f73-ae08-c3eb79a1dcf4` (cases g6a, g6b) | `catherine check associativity_scaffolded.ct.json` → exit 3, 18 `violation` findings at equation 7 over the 27 hand-built Triple rows; `catherine check associativity_omitted_triples.ct.json` → exit 0, `findings: []` | gap | Missing: a finite-carrier operation declaration with an executable associativity law and carrier completeness |
| composition-absence | Two documents cannot be checked jointly (`check a b` is an arity error); the seam claim is executable only after manual merge | `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d` (gap G1) | `catherine check composition_billing.ct.json composition_shipping.ct.json composition_seam_claim.ct.json` → exit 3, one `violation`/`executed` witness `{equation: 0, row: 2, lhs: 0, rhs: 1}` with `source` naming all three documents; each document alone → exit 0, `findings: []` | closed | ledger/probes/composition/0002-ee6e97ab.json |
| naturality-absence | A claimed row map between two same-schema instances cannot be declared or checked; carried as an instance field it is silently dropped | `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d` (gap G2) | `catherine naturality crates/catherine-cli/tests/fixtures/naturality/crm.ct.json crates/catherine-cli/tests/fixtures/naturality/warehouse_break.ct.json` → exit 3, one `violation`/`executed` witness `{arrow: account_owner, source_row: 2, lhs: 1, rhs: 0}`; the additive `instance_map` (format 1.4) is parsed, hashed, and executed by the `naturality` surface (`check` still ignores it) | closed | ledger/probes/naturality/0002-42f47295.json |
| coproduct-absence | A closed variant set cannot be declared; adding a whole alternative to the instance stays presentation-only | `kimi/1a1dee7a-3ad2-8579-8000-0999f885b87d` (gap G3) | `catherine check coproduct_payments.ct.json` → exit 3, two `violation`/`executed` findings: `{coproduct: 0, row: 2, injections: [card_inj, transfer_inj]}` (doubly covered `shared`) and `{coproduct: 0, row: 4}` (uncovered `voucher`), against the declared three-injection `Method` coproduct | closed | ledger/probes/coproduct/0003-2709a990.json |

At registration the suite opened with 12 rows, all `gap` (0 `closed`). That
is history, not the current split: a row flips with the capability change
that lands it, and the suite asserts the live split, never this sentence.

## What the table licenses and what it does not

- It licenses a **structural** claim about the current build: the
  observation column was produced by the exact invocation it quotes.
  Nothing here is a theorem about impossibility; re-opening a `gap`
  requires landing the capability (or naming a new law) first.
- It does **not** license stale claims: a row whose case stops showing
  its limitation fails the suite until the row flips (design Decision
  1). The suite is executable both ways, so limitations cannot ossify
  into folklore.
- The sibling changes `add-model-composition`,
  `add-instance-morphism`, `add-coproduct-exhaustiveness` and
  `add-pullback-checks` were authored with these rows as their flip
  targets; the fixtures for those rows use the planned surface shapes so
  the flip is one table cell, not a fixture rewrite.