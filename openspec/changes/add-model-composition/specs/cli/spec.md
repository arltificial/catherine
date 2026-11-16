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
