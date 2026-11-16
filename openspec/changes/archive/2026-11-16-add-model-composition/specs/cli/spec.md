# cli delta (variadic check)

## MODIFIED Requirements

### Requirement: `catherine check` executes the model document and reports findings
The system SHALL provide `catherine check <model.ct.json>` that parses,
builds, and executes all checks on the document, printing every finding
with its kind, epistemic label, and witness — human-readable by default,
a genesis-vibes JSON envelope under `--json`. The command SHALL accept
two or more document paths, running the joint (composition) check per
the `composition` capability; the exit-code contract and envelope shape
are unchanged.

#### Scenario: Agent loop acceptance
- **WHEN** an agent runs `catherine check bowling.ct.json --json` on a
  model whose accepted equation is violated by one row
- **THEN** the envelope `data.findings` contains exactly one violation
  entry with equation id, row id, and both path values, and the command
  exits with the violation exit code

#### Scenario: Malformed document is a labeled error
- **WHEN** the model document fails parsing (e.g. duplicate ident at
  line 14)
- **THEN** the command fails with a labeled error carrying line/column,
  stable code, and a remediation hint — never a stack trace or silent
  success

#### Scenario: Two documents run the joint check
- **WHEN** an agent runs `catherine check billing.ct.json shipping.ct.json
  --json` over two documents sharing a sub-schema
- **THEN** the run executes the glued constraints of both documents,
  findings carry source attribution naming the declaring document, and
  the exit code follows the existing precedence contract

## ADDED Requirements

### Requirement: `catherine underconstraint` is variadic over documents
The system SHALL accept one or more document paths in
`catherine underconstraint`, raising parallel-path questions over the
glued schema when two or more documents are given. Each question SHALL
carry source attribution naming the documents whose union created the
parallel paths, a disjoint union SHALL carry the
`empty_composition_interface` warning on this surface too, and the
exit-code contract is unchanged (6 when a hypothesis is raised).

#### Scenario: A seam-only question is attributed
- **WHEN** two documents each raise no parallel-path question alone but
  their shared-subschema union creates two parallel paths
- **THEN** the joint `underconstraint` run emits one hypothesized
  finding naming both documents, and exits with the underconstraint
  class code (6)

#### Scenario: A disjoint union on the analysis surface is warned
- **WHEN** two documents that share no declared identity are run jointly
  through `underconstraint`
- **THEN** the run proceeds, reports its per-document questions, and
  labels the disjoint reading with `empty_composition_interface` — never
  a silent seam presentation

#### Scenario: A single path per document raises nothing
- **WHEN** each document alone contributes only one path between its
  shared endpoints
- **THEN** running either document alone raises no question and exits 0
  (the unchanged single-document run)

