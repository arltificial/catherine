# kernel-checks delta (coproduct execution)

## ADDED Requirements

### Requirement: Accepted coproducts execute coverage with witnesses
The system SHALL execute accepted coproduct declarations against the
instance: a row of the declared object in no injection image SHALL
produce one `violation` finding labeled `executed` naming the uncovered
row. Proposed coproducts SHALL produce the same findings labeled
`hypothesized`. No new finding kinds are introduced.

#### Scenario: Rogue variant is a violation with a witness
- **WHEN** an instance carries a row of the declared object outside every
  injection image (the natural data evolution the declaration exists to
  catch)
- **THEN** exactly one `violation`/`executed` finding names the row, and
  the run exits with the violation code — never a clean exit on a
  declared-closed type

#### Scenario: Clean closed table is silent
- **WHEN** every row of the declared object is in exactly one injection
  image
- **THEN** zero coproduct findings are produced

### Requirement: Coproduct disjointness violations name the colliding injections
The system SHALL produce one `violation` finding labeled `executed` for
each row of the declared object that lies in the image of two declared
injections, with a witness naming the row and both injections. Rows
with a dangling or typing-violated cell in any declared injection arrow
SHALL be excluded from coproduct findings (already reported, never
double-reported).

#### Scenario: Dual coverage is a violation naming both injections
- **WHEN** a row lies in the image of two declared injections
- **THEN** exactly one `violation`/`executed` finding names the row and
  both injections as the disjointness witness

#### Scenario: Dangling injection cells are not double-reported
- **WHEN** a variant row's injection cell is dangling
- **THEN** only the dangling finding is produced — the row is excluded
  from coverage and disjointness findings
