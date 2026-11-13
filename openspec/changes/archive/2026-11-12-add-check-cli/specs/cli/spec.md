# cli delta (P3)

## ADDED Requirements

### Requirement: `catherine check` executes the model document and reports findings
The system SHALL provide `catherine check <model.ct.json>` that parses,
builds, and executes all checks on the document, printing every finding
with its kind, epistemic label, and witness — human-readable by default,
a genesis-vibes JSON envelope under `--json`.

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

### Requirement: Exit codes mirror finding classes with defined precedence
The system SHALL define stable exit codes per finding class (clean = 0;
parse/build failure distinct from any finding) so CI and agents can branch
on exit status without parsing output text. When multiple finding classes
are present, the process exits with the code of the highest-severity class
under a fixed precedence (violation > dangling > contradiction >
underconstraint > redundant — the order is part of the contract); the full
class set is always present in the output payload.

#### Scenario: Multiple classes resolve by precedence
- **WHEN** a check run produces both dangling and violation findings
- **THEN** the exit code is the violation-class code, and the output
  payload still lists both finding classes

#### Scenario: CI branches without parsing
- **WHEN** a check run produces only dangling findings
- **THEN** the process exit code is the dangling-class code, and a clean
  model exits 0

### Requirement: The envelope is the charly-vibes contract
The system SHALL emit `--json` output as the shared suite envelope
(`ok`, `envelope_kind`, `data`, `warnings`, `hints`) via genesis-vibes
stable modules, consumed by `catherine-cli` only — the kernel crates
remain genesis-free.

#### Scenario: Envelope shape matches suite tools
- **WHEN** any `catherine` command runs with `--json`
- **THEN** the output parses as the same envelope shape turu/whisper/ddl
  emit, with findings under `data.findings` and no genesis dependency
  linked into `catherine-core`
