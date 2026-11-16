# model-document delta (declarative keys)

## ADDED Requirements

### Requirement: Keys declare injectivity over arrow tuples
The system SHALL accept an additive `schema.keys` list of declarations
`{"object": O, "arrows": [a, ...], "status": "accepted" | "proposed"}`
asserting that the arrow-value tuple is injective over `O`'s rows. Keys
SHALL be stored in canonical `(object, arrows, status)` order. Keys range
over arrows only: a column-level functional dependency uses the value-object
convention or the `rules --project` bridge, never an attribute
reinterpretation of this section.

#### Scenario: Keys round-trip and canonicalize
- **WHEN** a document declares several keys in an arbitrary input order
- **THEN** parsing succeeds, the schema carries them in canonical
  `(object, arrows, status)` order, and canonical serialization is
  byte-identical to the same keys declared in another input order; a
  repeated declaration is presentation and is kept

#### Scenario: Malformed key is rejected at its node
- **WHEN** a key names an undeclared object, names an undeclared arrow, names
  an arrow not sourced at its object, carries an empty `arrows` list, or
  repeats an arrow name
- **THEN** parsing fails with `invalid_key` carrying the line/column of the
  offending node (`/schema/keys/<i>`), and no key is silently repaired or
  dropped

#### Scenario: Proposed key is preserved as a question
- **WHEN** a key declares `status: "proposed"`
- **THEN** the parsed schema carries it like any other key, and the schema's
  accepted-key set is unchanged

## MODIFIED Requirements

### Requirement: The model document format is versioned and additive-only
The system SHALL parse `.ct.json` model documents carrying a required
`format_version` field, SHALL accept additive minor versions of the
current major, and SHALL reject unknown major versions with a labeled
error — breaking changes ship as a new extension, never as in-place
mutation of v1 (wild accretion-laws discipline).

#### Scenario: Current major parses
- **WHEN** a `.ct.json` document with `format_version` of the current
  major is parsed and its minor is one this build knows
  (minor <= `KNOWN_MINOR`, currently 3: 1.1 adds `schema_map`, 1.2 adds
  `rule_presentation`, 1.3 adds `schema.keys`)
- **THEN** parsing succeeds and the document's schema and instance are
  fully represented

#### Scenario: Unknown additive minor is rejected
- **WHEN** a document declares `format_version` of the current major
  with a minor greater than the build's `KNOWN_MINOR`
- **THEN** parsing fails with `unsupported_version` carrying line/column
  of the `format_version` node — never a partial or guessed parse

#### Scenario: Unknown major is rejected
- **WHEN** a document declares a `format_version` major the parser does
  not know, or omits the field
- **THEN** parsing fails with `unsupported_version` carrying line/column
  of the `format_version` node — never a partial or guessed parse

### Requirement: Malformed presentations are rejected with line-accurate errors
The system SHALL reject, at parse time: duplicate object or arrow
identifiers; identifiers that do not match `[A-Za-z_][A-Za-z0-9_]*`;
equations referencing unknown arrows; ill-typed equations
(path endpoints that do not match); equation `status` outside
{proposed, accepted}; keys whose object or arrows are undeclared, whose
arrows are not sourced at the key's object, or whose arrow list is empty
or repeats a name — each with the line and column of the offending
JSON node and a stable error code. A parsed document SHALL never contain
a silently-repaired or dropped element.

#### Scenario: Duplicate identifier
- **WHEN** two objects (or two arrows) share one identifier
- **THEN** parsing fails with `duplicate_ident` pointing at the second
  occurrence, and neither occurrence is stored

#### Scenario: Ill-typed equation
- **WHEN** an equation's lhs and rhs paths have different endpoint
  objects, or any path references an undeclared arrow
- **THEN** parsing fails with `ill_typed_equation` or `unknown_arrow`
  respectively, naming the offending path

#### Scenario: Ill-formed key
- **WHEN** a key's object or arrow is undeclared, an arrow is not
  sourced at the key's object, or its arrow list is empty or repeats a
  name
- **THEN** parsing fails with `invalid_key` at the key's node, naming
  the exact defect

### Requirement: Serialization is canonical and hash-identifiable
The system SHALL serialize parsed documents in canonical sorted order
(byte-identical regardless of input key order) and SHALL compute a
model identity as sha256 over the canonical JSON (wild `formats.rs`
discipline), so two files expressing the same model produce equal bytes
and equal hashes. Canonical form SHALL normalize row-id naming (dense
per-object ids in sorted first-reference order), so the hash depends only
on the model's structure — not on presentation names in the input file.
Keys SHALL be emitted in canonical order, and an empty key list SHALL be
omitted entirely, so every pre-1.3 document keeps its exact canonical
bytes and hash.

#### Scenario: Byte-stable across input order
- **WHEN** the same model is written twice with different JSON key
  ordering
- **THEN** canonical serialization of both parses is byte-identical and
  their hashes are equal

#### Scenario: Absent keys are hash-neutral
- **WHEN** a pre-1.3 document carries no `keys` section
- **THEN** its canonical bytes and model hash are identical to the bytes
  and hash it produced before keys existed, and a document that adds keys
  hashes differently

#### Scenario: Round-trip preserves the model
- **WHEN** an arbitrary valid model is serialized and re-parsed
  (property test, generated)
- **THEN** the re-parsed model equals the original

### Requirement: A published JSON Schema validates the format's syntactic surface
The system SHALL publish a JSON Schema artifact for `.ct.json` v1
alongside the parser, covering the document's syntactic surface without
the Rust kernel. Semantic checks that cross-reference user declarations
remain kernel-side; the artifact SHALL document this division.

#### Scenario: Editor-side validation
- **WHEN** a `.ct.json` document is validated against the published
  artifact
- **THEN** schema-expressible errors (missing required fields, JSON type
  mismatches, bad `status` enum, ident-pattern violations, duplicate
  array-identifiers, a key with an empty or duplicate `arrows` list) are
  detected without the kernel, and the artifact documents that
  unknown-arrow, ill-typed-equation, and key declaredness/source checks
  remain kernel-side