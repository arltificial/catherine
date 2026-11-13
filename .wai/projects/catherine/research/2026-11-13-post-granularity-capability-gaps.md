# Post-catherine-8v9 capability gaps — grounded against specs and code

Date: 2026-11-13 · Status: research/design note, no implementation.
Evidence base: the `catherine-8v9` re-model run (`fixtures/probes/granularity-bogart/`,
`prototype/bogart_storage_granularity.py`) and the `catherine-hc6` pilot
(`.wai/projects/catherine/research/catherine-hc6-pilot-findings.md`).
Claims about the code are read from the named symbols; claims about what a
change would cost are marked **[INFERENCE]** unless a test or run exercised them.

## 1. What the four proposed additions are, and what each is worth

| # | Addition | Measured cost of the gap |
|---|---|---|
| A | Composite (n-ary) arrows | `pages.(document_id, version)` is one FK; it had to be encoded as a row-naming convention (`doc_<id>__v_<hex>`) that the canonical hash discards |
| B | Key / functional-dependency constraint | The pilot's only 2 real defects (`bogart-4wt`) were "two captures for one `(source, edition_date)`" — an FD violation with no declaration for it |
| C | Scoped quantification | 67 of 69 pilot findings were one unscopeable totality claim over planner-intent rows (`planned` partitions that must not resolve) |
| D | Extraction bridge | Every real-domain run used a bespoke Python generator; no way to go table → model declaratively |

## 2. A — Composite arrows

**What exists**

- `model.rs::Arrow { src: Ident, tgt: Ident }` — no arity field.
- `model.rs::Instance.values: BTreeMap<Ident, Vec<Option<RowId>>>` — one unary
  partial table per arrow; `instance.rs::TypedInstance` carries the same shape
  (`tables: BTreeMap<Ident, Vec<Option<RowId>>>`).
- Parser: `parse/schema.rs::check_arrows` (name/src/tgt only),
  `parse/mod.rs::check_table_cells` (one target row per cell),
  `parse/mod.rs::check_tables` (allocates `vec![None; n]` per arrow).
- Canonical identity: `hash.rs::model_hash` hashes the canonical schema, and
  `hash.rs::model_hash_is_structure_only` pins arrows as `{name, src, tgt}`.
- JSON Schema artifact: `crates/catherine-core/schema/ct-schema-v1.json`,
  `schema.arrows.items.properties = [name, src, tgt]`.
- Specs: model-document "Malformed presentations…"; kernel-checks
  "Dangling values are checked, not fatal".

**What is absent**: any expression for "these two arrows are one reference".
`grep -rn "arity"` over `catherine-core`/`catherine-analysis` finds nothing.
Consequence seen in 8v9: the pair had to become target-row identity, so the
model is sound only by convention, and consumers that compare arrow identity
(`schema_diff`'s "structural difference → incomparable", `SchemaMap::arrows`
keyed by arrow ident) cannot recover the claim.

**Cheapest grounded design** (additive, 1.3) **[INFERENCE]**

- `Arrow` gains `arity: u8` (default 1, skip-serialized when 1 → identical hash
  for every existing document).
- `instance.tables[arrow]` accepts an array of target rows when `arity > 1`;
  parse rejects arity > 1 arrows inside `equations` (paths stay unary, so
  `rewrite`, `checks` path composition and `search` are untouched).
- Dangling becomes one finding per tuple with any null component —
  `DanglingWitness { arrow, src }` is unchanged, so no finding-shape delta.
- `SchemaMap` (1.1) must state whether a composite source arrow may map to a
  multi-arrow path; recommended: no — map to `null` or to a target composite
  arrow, else labeled `partial_schema_map`.
- Spec deltas: model-document (format, JSON Schema, hash note), kernel-checks
  (tuple dangling semantics), analysis (enforced-constraint set unchanged).
- Tests: `crates/catherine-core/tests/roundtrip.rs` (arity round-trip),
  `check_fixtures.rs` (tuple with one null component), `.espectacular/
  model-document` contract via `ah scenario new`.

## 3. B — Key / FD constraint (highest measured value)

**What exists**

- Exactly five finding kinds (`findings.rs::FindingKind`): violation, dangling,
  redundant, contradiction, underconstraint. The findings spec enumerates them;
  rule-lens spec: "No new finding variants are introduced."
- A working precedent for *overloading* an existing witness rather than adding
  a kind: overlap under `DeclaredDisjoint` is emitted as `violation` with
  `EquationWitness` reused as the disjointness claim (`analysis/rule_lens.rs`
  module docs).
- A complete rule-table engine: `model.rs::RulePresentation{domain, columns,
  rules, policy}`, `Policy::{PriorityOrder, FirstMatch, DeclaredDisjoint}`,
  `catherine_core::rule_lens` analysis, CLI `rules` with `--values`
  (`crates/catherine-cli/src/rules.rs`).

**What is absent**: any declaration of injectivity/keys, and any bridge from
`instance` rows to a `RulePresentation` (the section is parsed but never fed by
instance data; `Instance` has no attribute columns at all).

**Cheapest grounded design** (additive, 1.3) **[INFERENCE]**

`"keys": [{"object": O, "arrows": ["a","b"], "status": "accepted|proposed"}]` —
the tuple of those arrows is injective over `O`'s rows.

- Check: group rows by tuple; a group of size > 1 is one finding per colliding
  pair (canonical order), `Violation(EquationWitness)` when accepted (label
  `executed`), `hypothesized` when proposed — exactly the rule-lens overloading
  precedent, so **no new kind, no new exit code** (violation = 3, precedence
  unchanged per cli spec).
- `schema_diff` must add keys to its "enforced constraints" set, or a key
  addition classifies as `cosmetic` — that spec requirement currently defines
  the enforced set as accepted equations' `(lhs, rhs)` pairs only.
- Hash: keys are structure, so they must enter the canonical form (and
  `model_hash_is_structure_only` gains a case); absent = skip-serialized, so
  existing hashes and all sealed ledger entries stay valid.
- Cheaper bridge without a format change **[INFERENCE]**: `catherine rules
  --project <Object> --columns c1,c2 --action <arrow>` deriving rules whose
  predicate is column equality and whose action is the row id; a collision is
  then an overlap → violation under `DeclaredDisjoint`. Uses only existing
  machinery, but the claim stays outside the document's hash and ledger.
  Recommended as step 1; the declarative `keys` section as step 2.

## 4. C — Scoped quantification (largest, least grounded)

**What exists**: `SearchBounds`/`SearchScope` and first-class `UNKNOWN`
(`analysis/src/bounds.rs`, analysis spec); `UnderconstraintWitness::
{ParallelPaths, UncoveredInputs}`; `Policy` as the only scoping vocabulary;
dangling is global — `checks.rs::dangling_findings` walks every cell with no
guard; `ConstraintStatus` is two-valued (`accepted|proposed`).

**What is absent**: predicates over acset objects. The predicate language
(`Predicate`, `ColumnType`, `Literal`) belongs to `RulePresentation` only —
acset objects have no attributes, so "total on the rows that must resolve"
cannot be spelled. This is the P8 "subobjects + resolution policy" extension
already flagged as HYPOTHESIZED in the 8v9 handoff.

**Design consequence** **[INFERENCE]**: it needs declared attribute columns per
object (format 1.4) plus per-equation/per-arrow guards, and it changes the
meaning of a finding ("violation on the guarded subobject") — a spec-level
change to kernel-checks, not a tweak. Recommendation: keep documenting the
boundary as a recorded gap; do not schedule before A and B.

## 5. D — Extraction bridge (cheapest ergonomically, no kernel impact)

**What exists**: the real CLI surface is ten subcommands — `check`, `findings`,
`search`, `redundant`, `underconstraint`, `distinguish`, `diff`, `query`,
`migrate`, `rules` (verified against `catherine --help`) — all model-in, none
table-in. Fixtures for every real domain were generated by bespoke scripts:
`prototype/*.py` (13 files), bogart's `scripts/pilot_ct_instances.py`, and now
`prototype/bogart_storage_granularity.py`.

**What is absent**: any table→model importer, any declarative mapping, any
reproducible path an agent can drive without writing Python.

**Design** **[INFERENCE]**: `*.extract.json` (source: sqlite/csv; `tables`,
`objects`, `arrows`, `columns`, `row_naming`) consumed by a CLI-side
`catherine extract` that emits a `.ct.json`. CLI-only, so the kernel crates stay
genesis-free and serde-free; spec delta is a new cli capability + JSON Schema
for the mapping file; tests are generator round-trips over the 8v9 fixtures.

## 6. Invariants any of these must respect (already spec'd)

| Invariant | Anchor |
|---|---|
| Additive minors only; unknown majors rejected | model-document req. 1; `parse/schema.rs::check_version` |
| Canonical, input-order-independent hash; row names never reach it | model-document req. 4; `hash.rs::model_hash` |
| No finding without a witness; five kinds only | findings req. 1; rule-lens req. 2 |
| Exit codes and class precedence are contract | cli req. 2 |
| `UNKNOWN` carries its explored scope; never "no violation found" | analysis req. 1 |

**Drift found while grounding** (two items, both spec-side):

1. The deployed model-document spec states the known minor is "currently 0",
   but `parse/schema.rs::KNOWN_MINOR = 2` (1.1 `schema_map`, 1.2
   `rule_presentation`) and the CLI error hint names 1.2.
2. The deployed cli spec's Purpose lists six subcommands (`check`, `findings`,
   `search`, `diff`, `migrate`, `rules`); the binary ships ten (adds
   `redundant`, `underconstraint`, `distinguish`, `query` — each introduced by
   an archived change whose delta did not amend the Purpose sentence).

Both are one-line spec deltas with no code change; any format-touching change
should fold them in rather than leave the deployed specs describing an older
tool.

## 7. Sequencing (evidence-weighted)

1. **B bridge** (`rules --project`): closes the measured blind spot using
   machinery that already exists; no format change.
2. **B declarative keys** (format 1.3) + schema_diff enforced-set delta: makes
   the pilot's defect class a first-class, hashed, ledger-able claim.
3. **A composite arrows** (same 1.3 bump): honest FK models; needed before the
   8v9-style encoding is recommended to anyone else.
4. **D extraction**: ergonomics and agent-drivability; do after the vocabulary
   settles so the mapping file does not need a second version.
5. **C scoping**: only when a domain pays for it; record the boundary until then.