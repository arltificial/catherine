# catherine-core

Catherine's kernel: the `.ct.json` v1 **model document** format, its
parser, canonical serialization, model hash, typed instance tables, and
**executed path-equation checks** — `check(schema, instance)` evaluates
every declared equation on every applicable row (E3: declarations do not
enforce; checks do). Everything downstream (P2 findings taxonomy, P6
ledger content-addressing) consumes the artifacts this crate publishes.
Implemented per openspec changes `add-model-document-kernel` (beads
`catherine-bpz`) and `add-executed-checks` (beads `catherine-0gk`).

## The format (v1)

```json
{
  "format_version": "1.0",
  "schema": {
    "objects": ["A", "B"],
    "arrows":   [{"name": "f", "src": "A", "tgt": "B"}],
    "equations": [{"lhs": ["f"], "rhs": ["f"], "rationale": "why", "status": "accepted"}]
  },
  "instance": {
    "rows":   {"A": ["a1", "a2"], "B": ["b1"]},
    "tables": {"f": {"a1": "b1", "a2": null}}
  }
}
```

- `format_version` — `MAJOR.MINOR`, **required**.
- `schema.objects` / `schema.arrows` — declarations; arrow `name`s are
  unique, `src`/`tgt` must name declared objects (self-loops legal).
- `schema.equations` — path equations `lhs = rhs` over arrow names, with a
  free-form `rationale` and a `status` (`proposed` | `accepted`,
  **defaulting to `accepted` when absent**). `status` is format data only:
  its execution semantics are a P1 concern (`add-executed-checks`) and are
  deliberately absent here. Equations are stored in canonical order
  (sorted by `(lhs, rhs, rationale, status)`) — list order in the file is
  presentation.
- `instance.rows` — row names per object. Row names are per-object.
- `instance.tables` — per-arrow value tables: source row name → target
  row name or `null`. **`null` is a dangling reference — a representable
  value the parser never drops, repairs, or errors on** (dangling
  classification is a check, P1). An unspecified cell parses as `null`.
- Unknown top-level or section keys are ignored: additive minors of v1
  may introduce optional fields without breaking this parser.
- `rule_presentation` — the additive **1.2** section (rule lens):
  `domain` names a declared schema object, `columns` declares the typed
  attribute alphabet (`int` | `bool` | `string`; string order is byte-wise,
  no collation), `rules` is the priority chain (`name`, decidable
  `predicate`, required non-empty `action`), and `policy` is
  `PriorityOrder` | `FirstMatch` | `DeclaredDisjoint`. Predicates are a
  tiny decidable language: typed comparisons (`eq ne lt le gt ge`; bool
  only `eq`/`ne`), `mod` on int columns with a positive modulus, and
  `and`/`or`/`not`. **No implicit precedence**: an absent (or `null`)
  `policy` stays absent, so only gap/overlap diagnostics apply. Every
  rule must assign an action — the action assignment is a required total
  function on X.

Identifiers match `[A-Za-z_][A-Za-z0-9_]*` everywhere.

## `format_version` policy

- **Additive change = minor version** (new optional fields, new instance
  sections that old parsers may ignore). The parser accepts additive
  minors of the current major **only up to `KNOWN_MINOR`** (currently 2,
  i.e. `1.2` — 1.1 added `schema_map`, 1.2 added `rule_presentation`): a
  minor greater than `KNOWN_MINOR` means semantics this
  build doesn't know, so it is rejected with `unsupported_version`
  (no-guessed-parse rule) until the build is updated.
- **Breaking change = a new extension** (a new file extension with its own
  schema), **never an in-place mutation of v1**. v1 is never redefined.
- The parser rejects an unknown major, a malformed version, or an absent
  `format_version` with `unsupported_version` at the version node (or at
  the document root when the field is absent) — never a partial or
  guessed parse.
- **Audit (catherine-348):** the raw document structs
  (`src/parse/schema.rs`: `RawDoc`, `RawSchema`, `RawArrow`,
  `RawEquation`, `RawInstance`; `src/parse/schema_map.rs`:
  `RawSchemaMap`; `src/parse/rule_presentation.rs`: `RawRulePresentation`)
  are serde `Deserialize` **without**
  `deny_unknown_fields` (zero occurrences workspace-wide), so unknown
  JSON keys are silently ignored — the deliberate additive-minor policy
  above, kept and documented per the catherine-51v precedent (no
  parse-behavior change). The negative-input matrix
  (`crates/catherine-cli/tests/negative_matrix.rs`) pins the
  semantics-bearing half of this policy: unknown *versions* are rejected
  with `unsupported_version`, never guessed.

## Errors

Semantic errors carry `{line, column, code, message}` — line/column of the
offending JSON node (1-based; columns in characters), `code` from the
stable, closed set:

| code | fired when |
|---|---|
| `duplicate_ident` | two objects, arrows, or rows (per object) share an ident — reported at the second occurrence; a repeated JSON key in `instance.rows`/`instance.tables` (which a JSON parser would silently deduplicate) is reported at the repeat |
| `unknown_arrow` | an equation path references an undeclared arrow — the message names the path, the position is the offending element |
| `ill_typed_equation` | an equation's path endpoints do not match — the message names both paths and their endpoints, the position is the equation node |
| `invalid_ident` | an identifier violates `[A-Za-z_][A-Za-z0-9_]*` — position at the ident node |
| `invalid_status` | an equation `status` is outside `{proposed, accepted}` |
| `unsupported_version` | `format_version` absent, malformed, or of an unknown major |
| `partial_schema_map` | a schema map (1.1) fails to assign every declared source object or arrow, or assigns an undeclared element — the message names the unmapped element |
| `undeclared_rule_domain` | a rule presentation (1.2) names a `domain` that is not a declared schema object |
| `invalid_column_type` | a rule presentation column's `type` is outside `{int, bool, string}` |
| `unknown_column` | a predicate references a column the presentation does not declare |
| `invalid_predicate` | a predicate is malformed or ill-typed: unknown `kind`/operator, a literal of the wrong JSON type, `mod` outside int columns, a non-positive modulus, or an empty `and`/`or` |
| `invalid_policy` | a present `policy` is outside `{PriorityOrder, FirstMatch, DeclaredDisjoint}` (an absent or `null` policy is legal and means no precedence) |
| `missing_action` | a rule has no action (or an empty one) — the action assignment must be total on X |

Structural malformations outside this set (missing required fields, JSON
type mismatches, references to undeclared objects/rows, empty equation
paths) are `Syntax` errors with a human message and a position; the codes
above are the stable semantic contract. Check order is the document
order: version → objects → arrows → equations → rows → tables →
schema map → rule presentation.

## Canonical form and hashing

Parsing normalizes presentation away: rows become dense per-object ids,
and every collection is keyed in sorted order. Two serializations exist,
by design:

- `serialize(model)` — a valid `.ct.json` v1 document (row names `_<id>`
  in id order) that re-parses to the same model. Byte-identical for any
  two files that parse to the same model: key order, equation order, and
  declaration order never reach the bytes.
- `canonical_bytes(model)` — `serialize` of the model **re-labeled with
  canonical row ids**: dense per-object ids derived from the model's
  *structure* by name-free canonical labeling (iterative refinement over
  each row's outgoing cells and incoming preimage classes per named
  arrow; remaining ties broken by individualization with a
  lexicographically-minimal serialization, with branches pruned when
  their post-seed refined partition equals an explored branch's
  partition, compared exactly). The labeling is a
  deterministic, name-free function of the structure, so it is invariant
  under *any* renaming of rows — including order-reversing ones — and
  the hash is the identity of the model's structure, not of its
  presentation. Supersedes the earlier "[CORR-001] could collide" bound:
  an initial version pruned branches by class-size multisets, which does
  not imply equal partitions and made the tie-break sensitive to the
  input's row order (rename-variant hashes); pruning now keys on the
  exact refined partition, which is a sound equivalence — equal
  partitions have identical deterministic continuations, so deduping
  them cannot change the chosen minimum — so there is no collision
  class and no documented bound remains.

`model_hash(model)` = `sha256:` + 64 lowercase hex digits over the
canonical bytes (wild's identity discipline). This is what makes ledger
content-addressing (P6) sound: models equal up to row renaming hash
equal, and any structural difference changes the hash.

## JSON Schema artifact

`schema/ct-schema-v1.json` publishes the format's syntactic surface for
editors/agents (JSON Schema draft 2020-12): required fields, JSON types,
the `status` enum, ident patterns, and duplicate array-identifiers. It
deliberately does **not** decide semantic, cross-referential questions —
unknown arrows in equations, ill-typed equation endpoints, and whether a
`format_version` major is known remain kernel-side. The artifact's
`description` documents this division.

## Attribution

Adapted, not authored, per the change's "adapt, don't author" contract.
Both donors are Apache-2.0 and ship no NOTICE file, so attribution is
recorded here and in per-file headers.

| Upstream | Files | Taken |
|---|---|---|
| specodelic (`charly/specodelic`, Apache-2.0) | `src/acset/instance.rs`, `src/acset/schema.rs` | canonical sorted enumeration (`canonical_order`), byte-stable deterministic builds (`build_deterministic`), dense interned ids with sorted-order assignment, partial morphism storage (`None` = unresolved, never dropped) |
| specodelic (`charly/specodelic`, Apache-2.0) | `src/acset/instance.rs` → `src/instance.rs` (P1, `add-executed-checks`) | the typed instance builder behind the parity gate (`tests/parity_gate.rs` over `fixtures/parity/`): dense per-object id spaces, one partial morphism table per arrow (`None` = dangling, a representable value never dropped), build-time typing violations reported and never stored as morphism values, total builder — dropped as specodelic-domain: `graph::build` parity machinery, walk-order edge recording, attribute cells, markdown `Spec` parsing. Path-equation execution (`src/checks.rs`) is new code (E10) with its own property tests |
| wild (`charly/wild`, Apache-2.0) | `src/formats.rs` | canonical-JSON writer rules (recursively sorted keys, no whitespace, base-ten integers, lowercase `\u00xx` control escapes, unescaped solidus, literal UTF-8) and the `sha256:` + 64-lowercase-hex digest format, with wild's normative examples ported verbatim; sha256 itself is in-crate per FIPS 180-4 because this crate's dependencies are locked to serde/serde_json/proptest |

Dropped as specodelic-domain: parity/refinement/typing-table predicates,
the fixed five-object closed set, the lossless text writer (JSON model
docs, not markdown), and `query.rs` (a P1/P4 donor).

## Development

```
just ci                                        # fmt + clippy -D warnings + tests
cargo test -p catherine-core                   # unit + golden fixtures + parity gate + properties
cargo test -p catherine-core --test golden_fixtures
cargo test -p catherine-core --test check_fixtures
cargo test -p catherine-core --test parity_gate
cargo test -p catherine-core --test checks_properties
cargo test -p catherine-core --test roundtrip
```

Dependencies are locked to `serde`, `serde_json` (deps) and `proptest`
(dev-deps) per the change contract; adding more is a follow-up ticket
(brief §10), never a silent addition.
