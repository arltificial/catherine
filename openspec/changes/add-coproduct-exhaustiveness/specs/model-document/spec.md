# model-document delta (coproducts)

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
object, names an undeclared arrow, names an arrow whose target is not
its object, carries an empty `arrows` list, or repeats an arrow name —
with one new stable code at the offending node, mirroring the
`invalid_key` discipline.

#### Scenario: Malformed coproduct is rejected at its node
- **WHEN** a coproduct names an undeclared object or arrow, names an
  arrow whose target is not its object, carries an empty `arrows` list,
  or repeats an arrow name
- **THEN** parsing fails with `invalid_coproduct` carrying the
  line/column of the offending node, and nothing is silently repaired or
  dropped
