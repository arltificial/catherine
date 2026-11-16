# composition Specification

## Purpose
Executed checks at integration seams: several model documents that share a
declared sub-schema are glued by name and every input's equations and keys
run over the merged instance, so a seam claim that spans documents is
checked instead of hand-merged. Conflicts are labeled errors, findings
carry source attribution, and no merged document is ever emitted.

## Requirements

### Requirement: Joint check glues shared-subschema documents by name
The system SHALL accept two or more model documents in a single check
run, glue them over their shared declared names, and execute the enforced
constraints of every input against the merged instance. Two claims about
the same declared structure SHALL agree, or the run SHALL fail with a
labeled error naming the documents and the offending node — never a
last-writer-wins union, a finding no input entails, or a silent drop.

#### Scenario: Seam violation caught across documents
- **WHEN** two documents each check clean alone but a row satisfies both
  (e.g. one order invoiced in one region and dispatched in another),
  and either document declares the seam equation over shared names
- **THEN** the joint run reports the violation with a witness naming the
  declaring document and the row, and exits with the violation code

#### Scenario: Conflicting overlap is a labeled error
- **WHEN** two inputs declare the same arrow with different endpoints,
  the same object with different row sets, or the same row id with
  different values (including `null` against a value) in a
  commonly-declared arrow
- **THEN** the run fails with a stable conflict code (`schema_conflict` /
  `row_conflict`) carrying the offending node's location, the differing
  names, and both document names, and nothing is silently merged or
  fabricated

#### Scenario: Empty-interface glue is warned, never silent
- **WHEN** two or more documents are checked jointly but share no
  declared identity — no object, no arrow, and no `(object, row)` pair
  (a bare row name reused across different objects, or an arrow name
  reused as a row name, is not a shared identity)
- **THEN** the run proceeds with a labeled warning
  (`empty_composition_interface`) naming the inputs — on the machine
  envelope under `--json` and on stderr in human mode, including on the
  joint `underconstraint` surface — never a signal-free clean exit
  presented as a seam check

#### Scenario: Single-document runs are unchanged
- **WHEN** `check` runs with exactly one document path
- **THEN** behavior, findings, exit codes, and JSON bytes are identical
  to the single-document contract

#### Scenario: Arity does not change the first error
- **WHEN** the same document, whose equations resolve against its own
  arrows, is checked once and again as part of a joint run
- **THEN** the first error is the same — the same stable code, the same
  offending-node position, and the same message — because the equation
  slot and the post-glue stage run the parser's own resolver; a document
  whose equations name a sibling's arrows has no standalone parse, and its
  unresolved-path failure is reported at the glue stage

### Requirement: Composition produces findings, not a merged model
The system SHALL restrict the composition surface to findings with
source attribution (the document that declared each violated constraint)
and SHALL NOT emit a merged model document. A cross-document constraint
claim SHALL live in an input document — versioned, reviewed, and enforced
by the joint run. A document is standalone-parseable, and therefore
model-hashable and schema-diffable, only when every name it references is
declared in it.

#### Scenario: Seam claim stays document evidence
- **WHEN** a document declares an equation over names defined in another
  input
- **THEN** the claim travels as ordinary document content: the joint run
  executes it and attributes the violation to the declaring document
  with no CLI-side annotation, while the same document alone is rejected
  with the parser's `unknown_arrow` because it names an arrow it does not
  declare

#### Scenario: A self-contained seam claim is standalone-parseable
- **WHEN** the declaring document also declares every arrow its claim
  names (the committed gap-suite seam-claim document declares all six)
- **THEN** it parses and checks as an ordinary single-document run, and
  only that self-contained form carries a sealed model hash and the diff
  surface

#### Scenario: Underconstraint questions attribute their seam
- **WHEN** a parallel-path question exists only after gluing (no single
  input raises it)
- **THEN** the `underconstraint` finding is reported with seam attribution
  naming the documents whose union created the parallel paths
