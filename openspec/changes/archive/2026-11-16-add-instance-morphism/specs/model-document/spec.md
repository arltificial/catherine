# model-document delta (instance map)

## ADDED Requirements

### Requirement: Instance maps declare claimed row mappings
The system SHALL accept an additive `instance_map` section of
declarations `{"maps": [{"object": O, "rows": {"src": "tgt"}}]}` — a
claimed, total row mapping from the document's instance to another
instance of the same schema. The map SHALL be total over every object
of the schema (an uncovered row is a labeled parse error at that row's
node) and SHALL be stored in canonical `(object, rows)` order. The
section is omitted from canonical bytes when empty, so documents
without it keep their hash.

#### Scenario: Map round-trips and canonicalizes
- **WHEN** a document declares an `instance_map` in an arbitrary input
  order
- **THEN** parsing succeeds, the map is stored in canonical order, and
  canonical serialization is byte-identical to the same map declared in
  another input order

#### Scenario: Incomplete map is rejected at its node
- **WHEN** a map omits a row of any schema object
- **THEN** parsing fails with `incomplete_instance_map` carrying the
  line/column of the offending map node — never an implicit identity on
  the uncovered rows

#### Scenario: Absent map is hash-neutral
- **WHEN** a document without an `instance_map` section is parsed under
  the new minor
- **THEN** canonical bytes, hash, and every sealed ledger `model_hashes`
  entry are unchanged

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
  (minor <= `KNOWN_MINOR`, currently 4: 1.1 adds `schema_map`, 1.2 adds
  `rule_presentation`, 1.3 adds `schema.keys`, 1.4 adds `instance_map`)
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
