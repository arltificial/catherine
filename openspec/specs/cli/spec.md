# cli Specification

## Purpose
The `catherine` command surface that agents and CI drive: `check`,
`findings`, `search`, `redundant`, `underconstraint`, `distinguish`,
`diff`, `query`, `migrate`, and `rules`, each speaking JSON and exiting
with stable class codes so a propose → execute → refine loop completes
offline in seconds.

## Requirements

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

#### Scenario: Duplicate document paths are warned
- **WHEN** the same document path is passed more than once to `check`,
  `findings`, or `underconstraint` (including paths that canonicalize to
  the same file), or `naturality` receives the same path for both
  documents
- **THEN** the run proceeds with the documented repeated-declaration
  multiset semantics — every occurrence is still checked and findings are
  not deduplicated — and names the repeated path on the warnings channel
  under the stable `duplicate_document_input` rule in both JSON and human
  modes, never deduplicating the input silently

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

### Requirement: Check and findings payloads carry the execution scope
Every `catherine check` and `catherine find` JSON envelope SHALL carry
`data.scope`, an object with exactly three snake_case keys:
`constraints_checked` (accepted constraints the check walk executed),
`rows_evaluated` (instance rows the walk evaluated), and
`equations_over_zero_rows` (accepted equations that bind zero rows).
The scope SHALL be computed in one place shared by both subcommands so
their reports cannot disagree.

#### Scenario: Every check envelope carries the scope
- **WHEN** `catherine check model.ct.json --json` completes a check
  walk over a document with accepted constraints and rows
- **THEN** the envelope's `data` carries a `scope` object whose keys
  are exactly `constraints_checked`, `rows_evaluated`, and
  `equations_over_zero_rows`, with counts matching the executed walk

#### Scenario: Check and findings agree
- **WHEN** the same document is run through `check --json` and
  `findings --json` in one session
- **THEN** both envelopes report the identical `data.scope` values

### Requirement: A vacuous run reports a zero scope, never a clean-shaped silence
A run whose walk executes nothing — a document below the execution
floor, a constraint-free schema, or a zero-row instance — SHALL report
`data.scope` with all three keys `0` instead of an empty findings
array indistinguishable from a genuine clean pass. The exit code for
such a run is unchanged.

#### Scenario: Zero-row instance is visible in the payload
- **WHEN** a document with accepted equations is checked against an
  instance whose tables contain no rows, and it exits clean
- **THEN** `data.scope` reports `rows_evaluated: 0` (with
  `equations_over_zero_rows` counting the equations bound over the
  empty rows), so the clean exit is visibly vacuous

#### Scenario: Constraint-free document is visible in the payload
- **WHEN** a document with no accepted constraints is checked, and it
  exits clean
- **THEN** `data.scope` reports `constraints_checked: 0`, so nothing
  was checked is distinguishable from clean

### Requirement: The scope field is additive and bytes elsewhere are frozen
Adding `data.scope` SHALL NOT alter any existing payload field, any
other envelope byte, or any exit code. Single-document runs SHALL keep
today's encoding exactly, plus the new field. Commands that do not
execute accepted constraints over an instance (`search`, `diff`,
`migrate`, `rules`, the pattern and naturality lenses, `query`) SHALL
NOT carry a scope field.

#### Scenario: Single-document bytes gain only the new field
- **WHEN** the golden fixture for a single-document check is compared
  byte-for-byte before and after the change
- **THEN** the only difference is the added `data.scope` object;
  findings, warnings, envelope kind, and version fields are identical

#### Scenario: Exit codes are untouched
- **WHEN** the same fixture corpus is run before and after the change
- **THEN** every exit code, clean or finding-classed, is unchanged,
  including the vacuous runs that exit 0 with a zero scope

#### Scenario: Non-executing commands stay scope-free
- **WHEN** `catherine search`, `diff`, `migrate`, `rules`, `query`,
  or the pattern/naturality lenses emit their JSON payloads
- **THEN** no `data.scope` field appears — these commands execute no
  accepted constraints over an instance
