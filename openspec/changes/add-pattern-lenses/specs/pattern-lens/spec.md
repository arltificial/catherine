# pattern-lens delta (pattern lenses)

## ADDED Requirements

### Requirement: Pattern lenses are registered with law-scope metadata
The system SHALL provide a static registry of pattern lenses, each carrying
a stable kebab-case id, the GoF pattern name, its universal abstraction,
one law kind (`identity | totality | coverage | order | delta`), and a scope
string copied from `docs/ref/pattern-kata-dispositions.md`. The registry
SHALL be listable without a model, and lookup SHALL be by exact id in the
registry's fixed order.

#### Scenario: Registry metadata is complete and listable
- **WHEN** the pattern registry is listed
- **THEN** every registered lens reports its id, name, abstraction, law
  kind, and scope string, one entry per executed disposition row, with
  twelve lenses and no duplicate id

#### Scenario: A lens is found by its exact stable id
- **WHEN** the registry is asked for a registered id such as `composite`
  or `chain-of-responsibility`
- **THEN** exactly that lens's metadata is returned, and an id differing
  only by case or spelling does not resolve

### Requirement: Pattern-lens findings reuse the existing taxonomy
The system SHALL emit pattern-lens diagnostics exclusively as the existing
finding kinds and witnesses — gap → `underconstraint` (`hypothesized`,
uncovered inputs witness), overlap under `DeclaredDisjoint` →
`violation` (`executed`), shadow/derivable → `redundant` (`executed`),
equation violations → `violation` (`executed`), lost data → the migration
class's `dangling`/lost findings — sorted by the taxonomy's total order. No
new finding kind, witness shape, or exit class SHALL be introduced.

#### Scenario: A rule-region defect is an existing kind with its witness
- **WHEN** a rule-table lens finds an uncovered input class or a dead rule
- **THEN** the output is an `underconstraint`/`hypothesized` finding with
  the uncovered-inputs witness, or a `redundant`/`executed` finding naming
  the rule and its reducing rules, respectively — never a pattern-specific
  kind

#### Scenario: A bounded non-answer is UNKNOWN with its scope
- **WHEN** a lens's enumeration exhausts its `ScopeCaps` budget before
  deciding
- **THEN** the result is `unknown` carrying the exact scope explored and no
  findings, which is a completed bounded run and is never reported as
  clean

### Requirement: An unknown or unevaluable pattern request fails closed
The system SHALL refuse a pattern request that it cannot serve with a
labeled diagnostic, never with an empty findings list presented as success:
an unregistered or omitted pattern id SHALL be a labeled `bad_argument`
usage error (exit 2) naming the request and listing the known ids, and a
document that parses but lacks the surface the named lens evaluates SHALL
report `unknown` with a scope string naming the missing surface.

#### Scenario: Unknown pattern id is a labeled usage error
- **WHEN** the pattern command is invoked with an id absent from the
  registry
- **THEN** it exits 2 with a labeled `bad_argument` diagnostic naming the
  id and the known ids, writes no findings payload, and never reports
  success

#### Scenario: Missing evaluated surface is not empty success
- **WHEN** the named lens requires a surface the parsed document does not
  carry (no `rule_presentation` for a rule-table lens, a `--values`
  declaration that does not bind the declared columns or names no rule
  surface, no `schema_map` for the Δ lens)
- **THEN** the run reports `unknown` with a scope string naming the missing
  surface and zero findings; a document that fails to parse or type is the
  existing labeled parse/build error (exit 1) instead, and a malformed
  `--values` spelling is the existing labeled `bad_argument` flag error
  (exit 1) — none of the three is an empty success

### Requirement: The replay gate reproduces the recorded kata verdicts
The system SHALL gate every lens on a checked-in expectation table derived
from `ledger/katas/*/`: each recorded fault's fixture, values, and
`(kind, label)` multiset from the entry's `run.findings`, with
`catherine_class: gap` rows and empty recorded multisets asserted silent.
The gate SHALL report parity rows as parity only — it SHALL NOT compute,
emit, or assert a unique-detection count or a detection rate.

#### Scenario: A lens reproduces its recorded finding classes
- **WHEN** the replay harness runs a lens over the fixture the ledger entry
  records
- **THEN** the run's `(kind, label)` multiset equals the entry's recorded
  multiset for that fixture, including the near-floor and clean arms

#### Scenario: Parity rows stay silent
- **WHEN** a fault is one the entry's conventional baseline also catches
  (the ledger's `comparison` records it, and it is absent from
  `unique_detections`)
- **THEN** the gate records it as parity, makes no unique-detection claim,
  and a parity-only result counts as neither a win nor a failure

#### Scenario: Killed and gap rows stay silent
- **WHEN** a fault's recorded `catherine_class` is `gap` or its kata is a
  killed/null row (the `state-discovery` unreachable-state row, the
  `memento-roundtrip` identity control)
- **THEN** the lens emits no finding for that fault and the fixture's
  recorded multiset is matched exactly — silence is asserted, not assumed

#### Scenario: An expectation mismatch fails the gate
- **WHEN** a lens's output differs from its expectation row in kind, label,
  or count
- **THEN** the replay test fails, and the row is never edited to match the
  output — the lens or the spec is corrected instead

### Requirement: The pattern surface is a named CLI command
The system SHALL expose the registry and single-lens runs as
`catherine pattern <model.ct.json> [--id <pattern>] [--values
column=v1,v2,...] [--list]`, with the same findings envelope and exit
classes as the analysis commands: findings exit at the minimum present
class code, `unknown` exits 0 with `outcome: "unknown"` and its scope, and
usage errors exit 2. A run without an id SHALL be refused; there is no
implicit run-every-lens mode.

#### Scenario: One lens runs over a model
- **WHEN** `catherine pattern <model> --id <pattern>` runs over a fixture
  with values that bind its declared domain
- **THEN** the output is the findings envelope (or an `unknown` outcome
  with its scope) for that lens alone, with the class exit code

#### Scenario: List form names the lenses and runs none
- **WHEN** `catherine pattern <model> --list` runs
- **THEN** it prints each registered id with its name, abstraction, law
  kind, and scope, emits no findings, and exits 0 without executing a lens

### Requirement: The composite lens checks identity and totality
The system SHALL provide the lens with id `composite`, name Composite,
abstraction "monoid (identity)", law kind `identity`, and scope
"identity/totality equations; associativity unexpressible". It SHALL check
the identity equation `parent∘top = top` and the totality of parent cells;
associativity is outside the lens's scope.

#### Scenario: Nil as a real mount point violates the identity law
- **WHEN** a composite fixture maps `parent(c5)` to nil so nil becomes a
  real mount point
- **THEN** one `violation`/`executed` finding names the identity equation
  at row c5, and no associativity claim is made

#### Scenario: A broken parent cell is a dangling finding
- **WHEN** a child node's parent cell is null
- **THEN** one `dangling`/`executed` finding names the cell and its source
  row

### Requirement: The null-object lens checks nil-identity equations
The system SHALL provide the lens with id `null-object`, abstraction
"monoid identity element", law kind `identity`, and scope "nil-identity
equations (identity/totality for the nil element)". It SHALL check the nil
element's identity and totality equations and nothing beyond that element.

#### Scenario: A non-neutral nil element is a violation
- **WHEN** the nil-row identity equation fails because the nil element is
  treated as a real leaf
- **THEN** one `violation`/`executed` finding at the nil row names the
  violated identity equation

### Requirement: The builder lens checks declared-step traces and splice order
The system SHALL provide the lens with id `builder`, abstraction "monoid
(accumulation)", law kind `order`, and scope "emitted part traces to a
declared step; splice order equation". It SHALL check that each emitted
part traces to a declared step and that splice order satisfies the
recorded equation; a duplicated-row fault the conventional baseline also
catches SHALL be recorded as parity.

#### Scenario: An out-of-order splice violates the order equation
- **WHEN** a product-local provenance splice violates the declared splice
  order
- **THEN** one `violation`/`executed` finding names the order equation and
  the Step row

#### Scenario: A dropped step loses its rows and cells
- **WHEN** the migration drops Step rows and their valued cells
- **THEN** the lost rows and cells are enumerated as the migration class's
  findings, one per dropped row or valued cell

#### Scenario: The duplicated-row case is parity
- **WHEN** a duplicated Step row is caught by the conventional trace diff
  as well as by the lens
- **THEN** the finding is recorded as parity with no unique-detection claim

### Requirement: The adapter lens checks link-level delegation and fields
The system SHALL provide the lens with id `adapter`, abstraction "wrapper
law preservation", law kind `totality`, and scope "link-level
delegation/field laws; coverage-only". It SHALL report dropped delegated
fields and broken links; the coverage-only scope SHALL be part of its
metadata because a complete cross-product sample collapses the unique set.

#### Scenario: A dropped translated field is dangling per row
- **WHEN** an adapter drops the translated auxiliary field on every row of
  its model
- **THEN** one `dangling`/`executed` finding per dropped cell names the
  cell and its source row

### Requirement: The decorator lens checks stacking transparency
The system SHALL provide the lens with id `decorator`, abstraction "wrapper
law preservation", law kind `order`, and scope "stacking transparency
equation; coverage-only". It SHALL check that a decorator stack returns the
same payload independent of wrapping order and SHALL NOT widen the law
beyond transparency.

#### Scenario: A reversed stack order violates transparency
- **WHEN** reversing two decorators' stacking order returns a different
  payload
- **THEN** one `violation`/`executed` finding names the stacking equation
  and the differing row

### Requirement: The proxy lens checks identity preservation
The system SHALL provide the lens with id `proxy`, abstraction "wrapper law
preservation", law kind `identity`, and scope "identity-preservation
equation; coverage-only". It SHALL check that delegation preserves
identity even when the result path agrees.

#### Scenario: A rewired identity is a violation
- **WHEN** a proxy rewires the identity of a wrapped row while the result
  path still agrees
- **THEN** one `violation`/`executed` finding at that row names the
  identity-preservation equation

### Requirement: The visitor lens checks variant coverage and handler classes
The system SHALL provide the lens with id `visitor`, abstraction "sum type
/ exhaustiveness", law kind `coverage`, and scope "variant coverage +
handler classes; no evaluation semantics". It SHALL report uncovered
variants, duplicate registrations under `DeclaredDisjoint`, shadowed
handlers, and derivable handlers; evaluation semantics are outside the
lens's scope.

#### Scenario: A missing visit method leaves a variant uncovered
- **WHEN** a visitor has no rule for one variant of the finite variant
  domain
- **THEN** one `underconstraint`/`hypothesized` finding names the uncovered
  variant class with its dense example indices

#### Scenario: Shadowed and derivable handlers are redundant
- **WHEN** a handler's effective region is empty under the declared policy,
  or its region is the intersection of two other handlers with the same
  action
- **THEN** one `redundant`/`executed` finding per handler names the handler
  and the rules that reduce it

#### Scenario: Duplicate registration is a violation under disjointness
- **WHEN** two registrations share an input under a `DeclaredDisjoint`
  policy
- **THEN** one `violation`/`executed` finding per shared input names the
  two colliding rules

### Requirement: The interpreter lens checks variant-domain coverage
The system SHALL provide the lens with id `interpreter`, abstraction "sum
type / recursive dispatch", law kind `coverage`, and scope "coverage of the
variant domain". It SHALL report a variant with no dispatch rule as an
uncovered input and SHALL make no evaluation or exhaustiveness-of-behavior
claim.

#### Scenario: An unhandled variant is a coverage gap
- **WHEN** a variant exists in the instance's sum type with no dispatch
  rule
- **THEN** one `underconstraint`/`hypothesized` finding names the variant
  class as uncovered, with no claim about how it would evaluate

### Requirement: The memento lens checks lost data and round-trip invertibility
The system SHALL provide the lens with id `memento`, abstraction "Δ
save/restore pair", law kind `delta`, and scope "lost-data enumeration +
round-trip non-invertibility proof". It SHALL enumerate lost rows and cells
introduced by the restore map and SHALL report the non-invertibility proof
when a saved shape cannot be recovered.

#### Scenario: A dropped restore field loses its cells
- **WHEN** the restore schema map assigns null to a saved field
- **THEN** one lost-cell finding per lost valuable cell names that cell and
  its row

#### Scenario: A collapsed shape is proven non-invertible
- **WHEN** a new shape has no place for saved page data
- **THEN** the run names the lost page cells and reports the round-trip
  non-invertibility proof for the collapsed pages

#### Scenario: The identity round trip is silent
- **WHEN** the restore map is the identity
- **THEN** the lens emits zero findings, matching the recorded control
  self-negation

### Requirement: The state lens checks transitions and declared-cell totality
The system SHALL provide the lens with id `state`, abstraction "state shape
/ discovery questions", law kind `totality`, and scope "transition laws,
declared-cell totality; unreachable-state detection unexpressible". It
SHALL report unresolved transition targets, unfilled declared cells, and
equation drift; unreachable-state detection is outside the lens's scope
and SHALL stay silent rather than approximate.

#### Scenario: An unresolved target is dangling
- **WHEN** a transition names a target state the model does not resolve
- **THEN** one `dangling`/`executed` finding names the target cell and its
  source row

#### Scenario: An unfilled declared cell is dangling with schema questions
- **WHEN** a declared transition cell is unfilled
- **THEN** the cell is reported `dangling`/`executed`, and each undecided
  schema choice is an `underconstraint`/`hypothesized` question that is
  never counted as a detection

#### Scenario: Drift on an unsampled cell is a violation
- **WHEN** a transition equation fails on a cell the conventional sample
  omits
- **THEN** one `violation`/`executed` finding names the equation and the
  row

### Requirement: The chain-of-responsibility lens checks handler order as rule regions
The system SHALL provide the lens with id `chain-of-responsibility`,
abstraction "catamorphism / handler order", law kind `order`, and scope
"rule-lens gap/shadow/overlap/derivable over handler order". It SHALL
report a handler-order gap as an uncovered input class, a dead handler as
redundant, a derivable handler as redundant, and an overlap under
`DeclaredDisjoint` as a violation.

#### Scenario: A handler-order gap is an uncovered class
- **WHEN** a request class is covered by no handler in the declared order
- **THEN** one `underconstraint`/`hypothesized` finding names the uncovered
  class with its dense example indices

#### Scenario: A dead or derivable handler is redundant
- **WHEN** a handler is unreachable under the declared policy, or its
  region equals the intersection of two factors with the same action
- **THEN** one `redundant`/`executed` finding names the handler and its
  reducing rules

#### Scenario: Behaviour-changing faults are parity
- **WHEN** a reordered or narrowed handler changes dispatch and the
  conventional dispatch sample catches it
- **THEN** the finding is recorded as parity with no unique-detection claim

### Requirement: The strategy lens checks branch selection above the floor
The system SHALL provide the lens with id `strategy`, abstraction "rule
lens above the floor", law kind `coverage`, and scope "branch selection as
rule regions". It SHALL report structural redundancy among strategy
branches above the triviality floor and SHALL NOT claim to detect
behaviour-changing faults the conventional dispatch sample catches.

#### Scenario: A derivable strategy branch is redundant
- **WHEN** a branch's region equals the intersection of two other branches'
  regions and it carries the same action
- **THEN** one `redundant`/`executed` finding names the branch and the two
  branches that combine to it

#### Scenario: A near-floor control buys nothing
- **WHEN** the strategy table is at the triviality floor and a scripted
  conventional sample enumerates its whole input space
- **THEN** the lens's result is recorded as a killed/parity outcome with no
  unique detections