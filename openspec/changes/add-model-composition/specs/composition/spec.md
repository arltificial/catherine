# composition delta (model composition at a seam)

## ADDED Requirements

### Requirement: Joint check glues shared-subschema documents by name
The system SHALL accept two or more model documents in a single check
run, glue them over their shared declared names (objects, arrows,
equations, keys, rows identified by name), and execute the enforced
constraints of every input against the merged instance. A declaration
that overlaps with a disagreeing structure SHALL fail with a labeled
error naming the documents and the offending node — never a
last-writer-wins union or a silent drop.

#### Scenario: Seam violation caught across documents
- **WHEN** two documents each check clean alone but a row satisfies both
  (e.g. one order invoiced in one region and dispatched in another),
  and either document declares the seam equation over shared names
- **THEN** the joint run reports the violation with a witness naming the
  declaring document and the row, and exits with the violation code

#### Scenario: Conflicting overlap is a labeled error
- **WHEN** two inputs declare the same arrow with different endpoints, or
  the same row id with different values in a commonly-declared arrow
- **THEN** the run fails with a stable conflict code (`schema_conflict` /
  `row_conflict`) carrying the node location and both document names,
  and nothing is silently merged

#### Scenario: Empty-interface glue is warned, never silent
- **WHEN** two or more documents are checked jointly but share no
  declared names (a disjoint union — often a typo'd interface name)
- **THEN** the run proceeds with a labeled envelope warning
  (`empty_composition_interface`) naming the inputs — never a
  signal-free clean exit presented as a seam check

#### Scenario: Single-document runs are unchanged
- **WHEN** `check` runs with exactly one document path
- **THEN** behavior, findings, exit codes, and JSON bytes are identical
  to the single-document contract

### Requirement: Composition produces findings, not a merged model
The system SHALL restrict the composition surface to findings with
source attribution (the document that declared each violated
constraint); the operation SHALL NOT emit a merged model document, and
cross-document constraint claims SHALL live in an input document so
they are hashed and schema-diffable like any other declaration.

#### Scenario: Seam claim is sealed document evidence
- **WHEN** a document declares an equation over names defined in another
  input, and a later `diff` compares that document against a version
  without the equation
- **THEN** the diff classifies the change through the existing scan (the
  claim needs no CLI-side annotation to be evidence)

#### Scenario: Underconstraint questions attribute their seam
- **WHEN** a parallel-path question exists only after gluing (no single
  input raises it)
- **THEN** the `underconstraint` finding is reported with seam attribution
  naming the documents whose union created the parallel paths
