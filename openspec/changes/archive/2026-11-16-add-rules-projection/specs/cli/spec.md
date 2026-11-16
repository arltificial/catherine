# cli delta (`rules --project`)

## ADDED Requirements

### Requirement: Instance rows project onto a rule table
The system SHALL provide `catherine rules <model> --project <Object>
--columns <arrow,...> [--action <arrow>] [--values column=v1,v2,...]`, which
synthesizes a `RulePresentation` from the rows of `<Object>`: one rule per row
in canonical row order, predicate = the conjunction of the named columns'
target-row equalities, action = the row's own name or the `--action` arrow's
target-row name, columns typed `string`, policy `DeclaredDisjoint`.

#### Scenario: Incomplete and empty projections are labeled errors
- **WHEN** a projected column or action cell is null, or the projected object
  has no rows
- **THEN** the command fails with the labeled `projection_incomplete`
  (naming row and arrow) or `empty_projection` code under the build-failure
  exit class — never a silent skip and never a vacuous clean run

#### Scenario: Two presentation sources are refused
- **WHEN** `--project` is given for a document that already carries a
  `rule_presentation`, or `--columns`/`--action` names an arrow that is not
  sourced at the projected object
- **THEN** the command fails with a labeled `bad_argument` error — no
  implicit preference and no guessed arrow


#### Scenario: Ambiguous rows are violations
- **WHEN** [describe the action or condition]
- **THEN** [describe the expected observable result]
### Requirement: The projected domain and claim are declared, not invented
The projection's input domain SHALL be the distinct column tuples present in
its rows, unioned with any explicitly declared `--values` product inputs. The
run introduces no new finding kind (overlap → `violation`, gap →
`underconstraint`) and its claim lies outside the model hash and the ledger:
never sealed evidence, and the help text says so.

#### Scenario: Ambiguous rows are violations
- **WHEN** two projected rows share every projected column value and differ
  as rows
- **THEN** exactly one `violation` finding per colliding pair is emitted,
  `executed`, with both row names in the witness, and the command exits with
  the violation class code

#### Scenario: Undeclared absent combinations are silent
- **WHEN** the projected rows cover only some of a larger conceptual set
- **THEN** no `underconstraint` finding is emitted for combinations the
  instance does not contain; declaring such a combination through `--values`
  emits the gap finding naming it