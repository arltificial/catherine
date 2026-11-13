# Design: declarative keys

One feature cycle (RED → GREEN) plus one refactor cycle. Format 1.3; no
existing document's hash changes. Grounding:
`.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md`
§3 (step 2) and §7 row 2. The two review HIGH findings from that note's
Rule-of-5 pass are resolved here as Decisions 2 (CORR-003, keys scope) and 3
(CORR-002, witness fit); the note is not edited, this design supersedes the
open questions.

## Decision 1 — `keys` is a `schema` section, hashed like equations

Keys are schema constraints, not an additive presentation: `schema_diff` takes
`&Schema`, the canonical writer emits schema sections, and the kernel check
walks schema declarations. So:

```json
"schema": {
  "objects": [...],
  "arrows": [...],
  "equations": [...],
  "keys": [{"object": "capture", "arrows": ["capture_source", "capture_edition_date"], "status": "accepted"}]
}
```

`Schema.keys: Vec<Key>`, `Key { object: Ident, arrows: Vec<Ident>, status:
ConstraintStatus }`. Canonical order `(object, arrows, status)`; input order is
presentation (same discipline as equations, Decision 4 of the model design).
Duplicates are kept — a repeated declaration is presentation, and the enforced
multiset subtracts one for one, exactly as for equations.

The section is omitted from canonical bytes when empty, so an absent `keys`
is invisible: every 1.0/1.1/1.2 document keeps its bytes, its hash, and every
sealed ledger `model_hashes` entry. `KNOWN_MINOR` becomes 3. As with
`schema_map` and `rule_presentation`, section presence is not cross-checked
against the declared minor (existing behavior; a 1.2 document carrying keys is
still represented faithfully, and its canonical bytes preserve 1.2).

## Decision 2 (CORR-003) — keys range over arrows only

A key is injectivity of the arrow-value tuple over one object's rows. That is
the acset-level vocabulary the model already has; it needs no attribute layer.
The measured capture FD `(source, edition_date)` is therefore expressible in
two grounded ways:

1. the value-object convention (one row per distinct value, one arrow per
   column) — what the pilot's model 3 already did, and what key declarations
   make hashed rather than conventional;
2. `rules --project` (step 1) over the instance rows, when the rows already
   carry the columns as arrows to value objects.

Rejected: extending `keys` to `rule_presentation` columns. A rule table's
columns are a presentation-level attribute alphabet with no identity semantics
— a key over them would be an instance constraint whose subject is a separate
section, and `RulePresentation` deliberately does not participate in the
document hash's structural constraints. Attribute columns on acset objects are
step 5's format 1.4 (P8 subobjects); keys will accept them when that layer
exists, as a separate additive minor. This limitation is stated in the
model-document spec and the change close note.

## Decision 3 (CORR-002) — `violation` carries a witness union

```rust
pub enum FindingKind {
    Violation(ViolationWitness),
    // ... four other kinds unchanged
}

#[serde(untagged)]
pub enum ViolationWitness {
    Equation(EquationWitness),
    Key(KeyWitness),
}

pub struct KeyWitness {
    pub key: usize,      // canonical index into Schema::keys
    pub rows: [RowId; 2], // ascending row ids of the colliding pair
}
```

Untagged serde keeps the equation JSON byte-identical
(`{"equation":n,"row":n,"lhs":n,"rhs":n}`) and adds
`{"key":n,"rows":[lo,hi]}` for key violations; the field sets are disjoint, so
deserialization is unambiguous. The kind set stays five; the rule-lens
precedent (overloading `EquationWitness` with indices) is *not* claimed to
fit: in `check` output, equation violations and key violations appear in one
list, so a reused `equation` index would be ambiguous between
`Schema::equations` and `Schema::keys`. The taxonomy's order is variant order
(Equation before Key), then ids, then label — `check` still sorts to a
byte-identical result.

`KeyWitness` names the key by index (as `EquationWitness` does) and both rows;
the shared tuple values are not carried — they are recoverable from the
instance at those rows, and duplicating them would make the witness
instance-dependent while the key declaration is not.

## Decision 4 — check semantics

For each accepted or proposed key, group the object's rows by the tuple of the
key arrows' cells:

- a row with any `None` cell in a key arrow is excluded — the cell is a
  dangling reference or a typing violation and is already reported by its own
  finding (never double-reported);
- a group of size > 1 emits one finding per unordered pair of rows (ascending
  row ids), in the taxonomy's canonical order; a 3-way collision is three
  findings — each pair is a distinct demonstrated failure, and a summary
  finding would carry no exact witness;
- label: `accepted` → `executed`, `proposed` → `hypothesized` (the equation
  rule, verbatim).

Keys are evaluated after equations and before the sort; `check` still returns
the taxonomy-sorted list, so repeated checks are byte-identical.

## Decision 5 — `schema_diff` enforced set = accepted equations ∪ accepted keys

`EnforcedEquation` becomes `EnforcedConstraint`:

```rust
pub enum EnforcedConstraint {
    Equation { lhs: Path, rhs: Path },
    Key { object: Ident, arrows: Vec<Ident> },
}
```

- `enforced(schema)` collects accepted equations and accepted keys; proposed
  keys are presentation, like proposed equations.
- Classification is unchanged structurally: dropped → `classify_dropping`;
  only added → `accreted`. `is_clean_typed` already derives from `check`, and
  accepted key violations are `executed`, so the bounded scan enforces keys
  without a second predicate. Proposed key violations are `hypothesized` and
  never block a witness (the existing label rule).
- The weakening witness is the existing `violation`/`executed` finding from
  the v1 walk — now possibly a `KeyWitness` for a dropped key. The CLI renders
  it against v1 and the `added` payload gains an untagged key shape
  (`{"object", "arrows"}`); the equation shape is unchanged and exit codes are
  untouched.
- `incomparable` still keys on objects/arrows only: equal vocabularies make
  keys comparable, and a key cannot reference an object or arrow the schema
  does not declare (parse guarantees it).

## Decision 6 — parse validation and the published JSON Schema

`RawSchema` gains `keys: Vec<RawKey>` (`object: String`, `arrows: Vec<String>`,
`status: Option<String>`). The new pass runs after equations, before the
instance (`/schema/keys/<i>` pointers):

- `object` must be declared and an ident → `invalid_key`;
- every arrow must be declared and `src == object` → `invalid_key`;
- `arrows` non-empty and duplicate-free within one key → `invalid_key`;
- status outside `{accepted, proposed}` → `invalid_status` (reused);
- absent status defaults to `accepted` (the equation rule).

One new stable code (`invalid_key`) rather than four: every one of these is
"this key is not a well-formed injectivity claim over this schema", and the
message names the exact defect at its node. The code set stays closed except
for this ticket, per `error.rs`'s contract.

`ct-schema-v1.json` gains `schema.keys` (`object`/`arrows` required, `arrows`
`uniqueItems`, `status` enum, ident patterns, `additionalProperties: false`);
the artifact's description keeps the kernel-side division (declaredness and
source typing remain kernel checks). `web/src/types/ct-schema-v1.ts` is
regenerated. `schema_map.target_schema` runs the same passes, so a target
schema's keys are validated, not silently dropped by serde.

## Decision 7 — untouched surfaces

- **Migration**: schema maps assign objects and arrows; keys are not mapped
  elements and the totality rule is unchanged. A target schema may declare
  keys; they are validated and carried in the target schema's canonical bytes.
- **`redundant` / `underconstraint` / `distinguish` / `query`**: they operate
  on equation paths, observations, and 1.1 sections; keys are not paths and
  do not participate in rewriting. No spec or code delta.
- **Wasm boundary / CLI**: no new subcommand, no exit-class change. The
  findings JSON is additive (union witness); the web panel's TS mirror gains
  the key witness branch so a key violation renders with its rows instead of
  `undefined` fields.
- **Ledger**: unchanged code; the hash-neutrality of absent keys is the
  invariant that keeps sealed entries valid.

## Falsifiers (from the note, made executable)

| Claim | Test |
|---|---|
| Accepted key, duplicated tuple → one `violation`/`executed` per colliding pair | core `checks` fixture, exact witness set |
| Clean key → zero findings | core fixture |
| Proposed key collision → `hypothesized`, never blocks search cleanliness | core/analysis fixture |
| Absent keys hash identically | hash test: 1.2 document before/after the change |
| Dropped accepted key → `weakened` with a key witness | analysis `schema_diff` fixture pair |
| Added accepted key → `accreted` naming it | analysis fixture + CLI payload |
| Dangling cell in a key arrow → no key finding, only the dangling finding | core fixture |
| Malformed key → `invalid_key`/`invalid_status` at the node's line/column | parser fixture |