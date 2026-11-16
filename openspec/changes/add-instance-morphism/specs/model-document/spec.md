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
