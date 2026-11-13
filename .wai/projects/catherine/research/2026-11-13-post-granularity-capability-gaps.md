# Post-catherine-8v9 capability gaps — grounded against specs and code

Date: 2026-11-13 · Status: research/design note, no implementation.
Evidence base: the `catherine-8v9` re-model run (`fixtures/probes/granularity-bogart/`,
`prototype/bogart_storage_granularity.py`) and the `catherine-hc6` pilot
(`.wai/projects/catherine/research/catherine-hc6-pilot-findings.md`).
Revised after a Rule-of-5 review (findings CORR-001…008, EDGE-002…007, CLAR-001…003).

## 0. Evidence ledger

| Kind | Content |
|---|---|
| **Executed this session** | `catherine check` → 0 findings on 289 pages (slice) and 74,257 pages / 924 documents / 210 editions (full), exit 0; live selector census 1,441 selectors / 3 ambiguous / 67 zero-match; `catherine --help` lists ten subcommands; `pinky audit` ok |
| **Read from code/specs** | every `file::symbol` anchor below; `KNOWN_MINOR = 2`; five finding kinds; `partial_schema_map` = missing entry; `EquationWitness` field set; ledger `model_hashes` |
| **Inferred (design)** | every sentence marked **[INFERENCE]**; no design below has been compiled or tested |

A claim is only as strong as its row here. Numbers in `monospace` under
"Observed" were produced by a run; everything else is reading or design.

## 1. The four proposed additions

| # | Addition | Observed | Consequence (inferred) |
|---|---|---|---|
| A | Composite (n-ary) arrows | `pages.(document_id, version)` is one FK; 8v9 encoded it as a row-naming convention (`doc_<id>__v_<hex>`) | the composite claim is unrecoverable from the model by any consumer; the hash normalizes names away |
| B | Key / functional-dependency constraint | pilot triage: 2 of 69 findings real, both `bogart-4wt` "two captures for one `(source, edition_date)`" | an FD has no declaration, so it can only surface as a dangling cell on a query-time arrow |
| C | Scoped quantification | pilot: 67 of 69 findings are one totality claim over `planned` partitions that must not resolve | totality is per-object and unguardable; the only scope vocabulary is `Policy` |
| D | Extraction bridge | both corpus-scale runs (hc6, 8v9) used bespoke Python generators; the extracted-table probe shipped a hand-curated fixture instead | no table→model importer; an agent must write Python per domain |

## 2. A — Composite arrows

**What exists**

- `model.rs::Arrow { src: Ident, tgt: Ident }` — no arity field.
- `model.rs::Instance.values: BTreeMap<Ident, Vec<Option<RowId>>>` — one unary
  partial table per arrow; `instance.rs::TypedInstance` carries the same shape
  (`tables: BTreeMap<Ident, Vec<Option<RowId>>>`).
- Parser: `parse/schema.rs::check_arrows` (name/src/tgt only),
  `parse/mod.rs::check_table_cells` (one target row per cell),
  `parse/mod.rs::check_tables` (allocates `vec![None; n]` per arrow).
- Canonical identity: `hash.rs::model_hash` hashes the canonical model document;
  `hash.rs::model_hash_is_structure_only` pins arrows as `{name, src, tgt}`.
- JSON Schema artifact: `crates/catherine-core/schema/ct-schema-v1.json`,
  `schema.arrows.items.properties = [name, src, tgt]`.
- Specs: model-document "Malformed presentations are rejected with line-accurate
  errors"; kernel-checks "Dangling values are checked, not fatal".

**What is absent**: any expression for "these two arrows are one reference".
`grep -rn "\barity\b"` over `catherine-core` and `catherine-analysis` returns 0 hits.
Consequence seen in 8v9: the pair had to become target-row identity, so the model
is sound only in the sense that the encoder knew the convention — consumers
comparing arrow identity (`schema_diff`'s structural test, `SchemaMap::arrows`
keyed by arrow ident) cannot recover it.

**Blast radius if arity lands (traced, not assumed)** **[INFERENCE]**

| Consumer | Today | Must change |
|---|---|---|
| `checks.rs::check` | one dangling per cell; equations walk unary paths | one dangling **per null component** (keeps `DanglingWitness`), equation walk unchanged because composite arrows are excluded from paths |
| `analysis/src/query.rs:137,290,332` | `typed.tables().get(name)` then `let Some(to) = cell` over `&Option<RowId>` | composite edges must be skipped or expanded to tuple nodes — decide, do not leave implicit |
| `analysis/src/distinguish.rs` | composes observation paths through the instance | observation paths stay unary; composite arrows unusable in observations |
| `core/src/migrate.rs:143` | iterates `map.arrows` as unary `Path`s | mapping rule below |
| `analysis/src/search.rs` | enumerates candidates, hands them to `check` | untouched only because composite arrows are outside equations; a tuple-valued table still must satisfy the instance builder |

**Cheapest grounded design** (additive, 1.3) **[INFERENCE]**

- `Arrow` gains `arity: u8` (default 1, skip-serialized when 1 → identical hash
  for every existing document; pin that with a case in
  `hash.rs::model_hash_is_structure_only`).
- `instance.tables[arrow]` accepts an array of target rows when `arity > 1`;
  parse rejects arity > 1 arrows inside `equations`. **Consequence to state in
  the spec:** a composite FK can then be checked for totality and, with §3, for
  injectivity — but no path claim can cross it, so the pilot's lost
  `block → document → edition` equation cannot be re-expressed over a composite.
- Dangling: one finding per null component.
- `SchemaMap` (1.1): every declared source arrow still needs an entry (totality
  is enforced at parse). A composite source arrow may map to a composite target
  arrow of equal arity or to `null` (explicit drop); any other assignment is a
  **new** labeled code (`composite_map_unsupported`) — `partial_schema_map` is
  specifically the *missing entry* error (`parse/schema_map.rs:43-45`) and must
  not be overloaded.
- Spec deltas: model-document (format, JSON Schema, hash note), kernel-checks
  (tuple dangling semantics), migration (schema-map rule), analysis (query/
  distinguish behavior), cli (none).
- Tests: `crates/catherine-core/tests/roundtrip.rs` (arity round-trip),
  `check_fixtures.rs` (tuple with one null component),
  `migration.rs` (composite assignment matrix), `.espectacular/model-document`
  contract via `ah scenario new`.

## 3. B — Key / FD constraint (highest measured value)

**What exists**

- Exactly five finding kinds (`findings.rs::FindingKind`): violation, dangling,
  redundant, contradiction, underconstraint. The findings spec enumerates them;
  rule-lens spec: "No new finding variants are introduced."
- A working precedent for *overloading* an existing witness rather than adding
  a kind: overlap under `DeclaredDisjoint` is emitted as `violation` with
  `EquationWitness` reused as the disjointness claim (`analysis/rule_lens.rs`
  module docs) — it works because all four fields are overloaded with
  *indices*.
- A complete rule-table engine: `model.rs::RulePresentation{domain, columns,
  rules, policy}`, `Policy::{PriorityOrder, FirstMatch, DeclaredDisjoint}`,
  `catherine_core::rule_lens` analysis, CLI `rules` with `--values`
  (`crates/catherine-cli/src/rules.rs`, `args.rs::parse_domain_values`).

**What is absent**: any declaration of injectivity/keys, and any bridge from
`instance` rows to a `RulePresentation` (the section is parsed but never fed by
instance data; `Instance` has no attribute columns at all).

**Design, step 1 — projection bridge** **[INFERENCE]**

`catherine rules --project <Object> --columns c1,c2 --action <arrow>`: build a
`RulePresentation` from instance rows (predicate = column equality, action = the
row's arrow value) and analyze it unmodified; a collision is an overlap →
`violation` under `DeclaredDisjoint`. Two costs must be stated in the CLI help:

- the domain must be supplied (`--values column=v1,v2,...`) and `rule_lens`
  enumerates the **product** of the column value lists — for the capture case
  that is ~5 sources × ~290 dates ≈ 1,450 candidates for 1,441 real selectors
  **[INFERENCE]**, inside `ScopeCaps::max_candidates = 256` only after a budget
  raise;
- the claim lives outside the document hash and the ledger, so it cannot be
  cited as sealed evidence.

**Design, step 2 — declarative keys** (additive, 1.3) **[INFERENCE]**

`"keys": [{"object": O, "arrows": ["a","b"], "status": "accepted|proposed"}]` —
the tuple of those arrow values is injective over `O`'s rows.

- **Scope limit to state up front:** keys range over *arrows*, so a column-level
  FD (`(source, edition_date)`) is expressible only under the value-object
  convention (one row per distinct value, one arrow per column) — which is
  exactly what the pilot's model 3 did. Where typed columns already exist (rule
  presentations), step 1 is more faithful and cheaper.
- **Check semantics:** group rows by tuple; dangling cells are excluded from
  grouping (already reported by the dangling finding, not double-reported), and
  a group of size > 1 emits one finding per colliding pair in canonical order.
  Accepted → `Violation`, `executed`; proposed → `hypothesized`.
- **Witness:** `EquationWitness { equation: usize, row: RowId, lhs: RowId,
  rhs: RowId }` (`findings.rs:79-88`) carries one failing row plus two composed
  values — it cannot carry two colliding rows *and* their shared tuple. State
  the overload explicitly: `equation` = key index, `row` = the canonically lower
  colliding row, `lhs` = the higher row, `rhs` = the lower row again (sentinel),
  and accept that the tuple values are not in the witness, or propose a
  `Violation(KeyWitness)` payload variant with its own findings-spec delta. Do
  not claim the rule-lens precedent "fits exactly".
- **Search interaction:** accepted keys are enforced constraints and should
  judge satisfiability in bounded search — the analysis spec currently says only
  "accepted equations" do, so that sentence needs the delta; proposed keys never
  judge.
- `schema_diff` must add accepted keys to its enforced-constraint set
  (`schema_diff.rs` `EnforcedConstraint` is "an accepted equation's two path
  sides"), or a key addition classifies as `cosmetic`. Proposed keys stay
  presentation, like proposed equations.
- **Format checklist:** `Arrow`/model types, parser (`parse/mod.rs`,
  `parse/schema.rs`), `crates/catherine-core/schema/ct-schema-v1.json`, the
  canonical writer and `model_hash_is_structure_only`, and `KNOWN_MINOR`
  2 → 3. Absent = skip-serialized, so existing documents' hashes and every
  sealed ledger entry's `model_hashes` stay valid (`ledger/src/entry.rs` carries
  them).

## 4. C — Scoped quantification (largest, least grounded)

**What exists**: `SearchBounds`/`SearchScope` and first-class `UNKNOWN`
(`analysis/src/bounds.rs`, analysis "Bounded search returns a witness or an
honest UNKNOWN"); `UnderconstraintWitness::{ParallelPaths, UncoveredInputs}`;
`Policy` as the only *declarative* scoping vocabulary in a document; dangling is
global — `checks.rs::dangling_findings` walks every cell with no guard;
`ConstraintStatus` is two-valued (`accepted|proposed`).

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
table-in. Fixture provenance: the two corpus-scale runs (hc6, 8v9) used bespoke
generators (`prototype/*.py`, 13 pre-existing files, plus
`prototype/bogart_storage_granularity.py`; bogart's
`scripts/pilot_ct_instances.py`); the extracted-table probe used a hand-curated
extraction instead.

**What is absent**: any table→model importer, any declarative mapping, any
reproducible path an agent can drive without writing Python.

**Design** **[INFERENCE]**: `*.extract.json` (versioned; `sources: [{id, path,
sha256}]` so a run is reproducible and joins across files are explicit; `tables`,
`objects`, `arrows`, `columns`, `row_naming`) consumed by a CLI-side
`catherine extract` that emits a `.ct.json`. CLI-only, so the kernel crates stay
genesis-free and serde-free; spec delta is a new cli capability + a JSON Schema
for the mapping file; tests are generator round-trips that reproduce the 8v9
fixtures byte-for-byte.

## 6. Invariants any of these must respect (already spec'd)

| Invariant | Anchor (requirement heading) |
|---|---|
| Additive minors only; unknown majors rejected | model-document "The model document format is versioned and additive-only"; `parse/schema.rs::check_version` |
| Canonical, input-order-independent hash; row names never reach it | model-document "Serialization is canonical and hash-identifiable"; `hash.rs::model_hash` |
| No finding without a witness; five kinds only | findings "Findings are typed and carry witnesses"; rule-lens "Rule-lens findings map onto the existing taxonomy" |
| Exit codes and class precedence are contract | cli "Exit codes mirror finding classes with defined precedence" |
| `UNKNOWN` carries its explored scope; never "no violation found" | analysis "Bounded search returns a witness or an honest UNKNOWN" |

**Spec drift found while grounding** (two items, both spec-side, both
undispositioned before this revision):

1. Model-document "Current major parses" states the known minor is "currently
   0"; `parse/schema.rs::KNOWN_MINOR = 2` (1.1 `schema_map`, 1.2
   `rule_presentation`) and the CLI hint names 1.2.
2. The cli spec's Purpose lists six subcommands (`check`, `findings`, `search`,
   `diff`, `migrate`, `rules`); the binary ships ten (adds `redundant`,
   `underconstraint`, `distinguish`, `query` — each introduced by an archived
   change whose delta did not amend the Purpose sentence).

Disposition: **step 0** of any change below — fix both texts in the same
change, since steps 1 and 3 both touch these surfaces.

## 7. Sequencing (evidence-weighted, each step with its falsifier)

| Step | Work | Touched specs | Falsifier |
|---|---|---|---|
| 0 | Correct the two drifting spec texts | model-document, cli | `openspec validate --strict` green and the texts match `KNOWN_MINOR` / `--help` |
| 1 | `rules --project` bridge | cli (new scenario), analysis (projection note) | run it on the 8v9 capture slice: 3 ambiguous selectors must produce 3 violations, and no finding on the 67 zero-match ones unless separately declared |
| 2 | Declarative `keys` + schema_diff enforced-set delta | model-document, kernel-checks, findings (witness note), analysis (diff enforced set + search), migration (unchanged) | a fixture with a duplicated `(source, edition_date)` must emit one `violation`/`executed` per colliding pair; a clean key must emit nothing; an absent `keys` must hash identically to today |
| 3 | Composite arrows (same 1.3 bump) | model-document, kernel-checks, migration, analysis | the 8v9 model re-expressed with `arity: 2` must reproduce 0 findings, and a phantom `version` must still yield exactly one dangling finding |
| 4 | Extraction bridge | cli (new capability) | regenerating the 8v9 fixtures from `*.extract.json` must be byte-identical (sha256) |
| 5 | Scoped quantification | model-document (1.4), kernel-checks | not scheduled; recorded as a boundary |

Ordering rationale: step 1 closes the measured blind spot with machinery that
already exists; step 2 makes it a hashed, ledger-able claim; step 3 removes the
naming-convention workaround before anyone else copies it; step 4 is ergonomics
and should follow a settled vocabulary; step 5 needs an attribute layer that
does not exist.