# model-document delta (pullback declarations)

## ADDED Requirements

### Requirement: Pullbacks declare pullback squares over leg and cospan arrows
The system SHALL accept an additive `schema.pullbacks` list of
declarations `{"apex": P, "legs": [p1, p2], "cospan": [f, g],
"status": "accepted" | "proposed"}` where the legs source at the apex
and the cospan arrows share a target. The declaration asserts the
pullback universal property over the instance (commutes, covers,
unique). Pullbacks SHALL be stored in canonical `(apex, legs, cospan,
status)` order; an empty or absent section is omitted from canonical
bytes, so documents without it keep their hash.

#### Scenario: Pullbacks round-trip and canonicalize
- **WHEN** a document declares several pullbacks in an arbitrary input
  order
- **THEN** parsing succeeds, the schema carries them in canonical
  order, and canonical serialization is byte-identical to the same
  declarations in another input order; a repeated declaration is
  presentation and is kept

#### Scenario: Absent pullbacks are hash-neutral
- **WHEN** a document without a `pullbacks` section is parsed under
  the new minor
- **THEN** canonical bytes, hash, and every sealed ledger
  `model_hashes` entry are unchanged

#### Scenario: The cones field stays rejected
- **WHEN** a document declares a `cones` field
- **THEN** parsing still fails with `unknown_field` at the field node
  — the pullback capability is declarable only through
  `schema.pullbacks`, and no cone namespace is introduced

### Requirement: Malformed pullbacks are rejected at their nodes
The system SHALL reject a pullback declaration that names an undeclared
object, names an undeclared arrow, names a leg not sourced at the
declared apex, names cospan arrows whose targets differ, or repeats an
arrow name, with one new stable code `invalid_pullback` at the
offending node, mirroring the `invalid_coproduct` discipline.

#### Scenario: Malformed pullback is rejected at its node
- **WHEN** a pullback names an undeclared object or arrow, names a leg
  not sourced at the apex, names cospan arrows with different targets,
  or repeats an arrow name
- **THEN** parsing fails with `invalid_pullback` carrying the
  line/column of the offending node, and nothing is silently repaired
  or dropped
