# model-document delta (coproducts + the 1.5 versioning minor)

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
  (minor <= `KNOWN_MINOR`, currently 5: 1.1 adds `schema_map`, 1.2 adds
  `rule_presentation`, 1.3 adds `schema.keys`, 1.4 adds `instance_map`,
  1.5 adds `schema.coproducts`)
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

## ADDED Requirements

### Requirement: Coproducts declare closed variants over injection arrows
The system SHALL accept an additive `schema.coproducts` list of
declarations `{"object": C, "arrows": [i1, ...], "status": "accepted" |
"proposed"}` where each arrow is a declared injection `V_k → C`. The
declaration asserts coverage and disjointness of `C`'s rows over the
injection images. Coproducts SHALL be stored in canonical `(object,
arrows, status)` order. The section is omitted from canonical bytes
when empty, so documents without it keep their hash.

#### Scenario: Coproducts round-trip and canonicalize
- **WHEN** a document declares several coproducts in an arbitrary input
  order
- **THEN** parsing succeeds, the schema carries them in canonical order,
  and canonical serialization is byte-identical to the same declarations
  in another input order; a repeated declaration is presentation and is
  kept

#### Scenario: Absent coproducts are hash-neutral
- **WHEN** a document without a `coproducts` section is parsed under the
  new minor
- **THEN** canonical bytes, hash, and every sealed ledger `model_hashes`
  entry are unchanged

### Requirement: Malformed coproducts are rejected at their nodes
The system SHALL reject a coproduct declaration that names an undeclared
object, names an undeclared arrow, names an arrow that is not an
injection `V_k → C` — its target is not the coproduct object, or it is
sourced at the coproduct object itself — carries an empty `arrows` list,
or repeats an arrow name, with one new stable code `invalid_coproduct` at
the offending node, mirroring the `invalid_key` discipline.

#### Scenario: Malformed coproduct is rejected at its node
- **WHEN** a coproduct names an undeclared object or arrow, names an
  arrow that is not an injection `V_k → C` (wrong target, or sourced at
  the coproduct object), carries an empty `arrows` list, or repeats an
  arrow name
- **THEN** parsing fails with `invalid_coproduct` carrying the
  line/column of the offending node, and nothing is silently repaired or
  dropped
