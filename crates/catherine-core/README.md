# catherine-core

Catherine's kernel skeleton: the `.ct.json` v1 **model document** format,
its parser, canonical serialization, and model hash. Everything downstream
(P1 executed checks, P6 ledger content-addressing) consumes the artifact
this crate publishes. Implemented per openspec change
`add-model-document-kernel` (beads `catherine-bpz`).

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

Identifiers match `[A-Za-z_][A-Za-z0-9_]*` everywhere.

## `format_version` policy

- **Additive change = minor version** (new optional fields, new instance
  sections that old parsers may ignore). The parser accepts any additive
  minor of the current major.
- **Breaking change = a new extension** (a new file extension with its own
  schema), **never an in-place mutation of v1**. v1 is never redefined.
- The parser rejects an unknown major, a malformed version, or an absent
  `format_version` with `unsupported_version` at the version node (or at
  the document root when the field is absent) — never a partial or
  guessed parse.

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

Structural malformations outside this set (missing required fields, JSON
type mismatches, references to undeclared objects/rows, empty equation
paths) are `Syntax` errors with a human message and a position; the six
codes above are the stable semantic contract. Check order is the document
order: version → objects → arrows → equations → rows → tables.

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
  their post-seed refinement profile matches an explored branch). The
  labeling is a deterministic, name-free function of the structure, so
  it is invariant under *any* renaming of rows — including
  order-reversing ones — and the hash is the identity of the model's
  structure, not of its presentation. One documented bound: the pruning
  treats profile-equal ties as interchangeable (true for every structure
  this format is expected to carry); adversarially crafted
  refinement-indistinguishable structures could, in principle, collide —
  a follow-up for P6 if the ledger ever admits untrusted documents.

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
| wild (`charly/wild`, Apache-2.0) | `src/formats.rs` | canonical-JSON writer rules (recursively sorted keys, no whitespace, base-ten integers, lowercase `\u00xx` control escapes, unescaped solidus, literal UTF-8) and the `sha256:` + 64-lowercase-hex digest format, with wild's normative examples ported verbatim; sha256 itself is in-crate per FIPS 180-4 because this crate's dependencies are locked to serde/serde_json/proptest |

Dropped as specodelic-domain: parity/refinement/typing-table predicates,
the fixed five-object closed set, the lossless text writer (JSON model
docs, not markdown), and `query.rs` (a P1/P4 donor).

## Development

```
just ci                                        # fmt + clippy -D warnings + tests
cargo test -p catherine-core                   # unit + golden fixtures + properties
cargo test -p catherine-core --test golden_fixtures
cargo test -p catherine-core --test roundtrip
```

Dependencies are locked to `serde`, `serde_json` (deps) and `proptest`
(dev-deps) per the change contract; adding more is a follow-up ticket
(brief §10), never a silent addition.
