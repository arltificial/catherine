# model-document delta (P0)

## ADDED Requirements

### Requirement: The model document format is versioned and additive-only
The system SHALL parse `.ct.json` model documents carrying a required
`format_version` field, SHALL accept additive minor versions of the
current major, and SHALL reject unknown major versions with a labeled
error — breaking changes ship as a new extension, never as in-place
mutation of v1 (wild accretion-laws discipline).

#### Scenario: Current major parses
- **WHEN** a `.ct.json` document with `format_version` of the current
  major is parsed and its minor is one this build knows
  (minor <= KNOWN_MINOR, currently 0)
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
{proposed, accepted} — each with the line and column of the offending
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

### Requirement: Dangling references are preserved as values
The system SHALL store every instance arrow value, including unresolved
(`null`) references, as representable values — the parser never drops,
repairs, or errors on a dangling reference at parse time (dangling
classification is a check, not a parse failure).

#### Scenario: Null reference survives parsing
- **WHEN** an instance assigns `null` as an arrow value for a row
- **THEN** the parsed instance carries that value as dangling, and
  canonical serialization round-trips it as `null`

### Requirement: Serialization is canonical and hash-identifiable
The system SHALL serialize parsed documents in canonical sorted order
(byte-identical regardless of input key order) and SHALL compute a
model identity as sha256 over the canonical JSON (wild `formats.rs`
discipline), so two files expressing the same model produce equal bytes
and equal hashes. Canonical form SHALL normalize row-id naming (dense
per-object ids in sorted first-reference order), so the hash depends only
on the model's structure — not on presentation names in the input file.

#### Scenario: Byte-stable across input order
- **WHEN** the same model is written twice with different JSON key
  ordering
- **THEN** canonical serialization of both parses is byte-identical and
  their hashes are equal

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
  array-identifiers) are detected without the kernel, and the artifact
  documents that unknown-arrow and ill-typed-equation checks remain
  kernel-side
