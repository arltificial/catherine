# kernel-checks delta (declarative keys)

## ADDED Requirements

### Requirement: Declared keys are executed as injectivity checks
The system SHALL evaluate every declared key against the rows of its object
and SHALL emit one `violation` finding per unordered pair of rows whose key
arrow tuple agrees — `executed` for an accepted key, `hypothesized` for a
proposed one. A row with an unresolved cell on a key arrow SHALL be
excluded from grouping: that cell is already reported by its own finding,
never double-reported. The witness SHALL name the key's canonical index and
both colliding rows.

#### Scenario: Colliding tuple carries its witness
- **WHEN** two rows of a key's object agree on every key arrow's value
- **THEN** checking emits exactly one `violation` for that pair, naming the
  key index and both row ids (ascending), labeled `executed` for an accepted
  key and `hypothesized` for a proposed one

#### Scenario: Three-way collision is three pairs
- **WHEN** three rows agree on the key tuple
- **THEN** checking emits three findings, one per unordered pair — each pair
  is a demonstrated failure with an exact witness, not a summary count

#### Scenario: Dangling key cell is not a collision
- **WHEN** a row's key arrow value is dangling
- **THEN** checking emits the dangling finding for that cell and no key
  violation involving that row

#### Scenario: Clean key emits nothing
- **WHEN** every row of the key's object resolves its key arrows to distinct
  tuples
- **THEN** checking emits no key finding — repeated checks serialize
  byte-identically under the taxonomy's canonical order