# migration delta (P7)

## ADDED Requirements

### Requirement: Δ migration reshapes without inventing
The system SHALL migrate data along a schema map by reindexing (pullback
along Δ), producing instances of the target schema from instances of the
source schema — creating no rows or values that are not present in the
source, and no phantom arrows.

#### Scenario: Faithful reindex
- **WHEN** a source instance is migrated along a schema map
- **THEN** every target value traces to a source value via the map, and
  round-tripping the reindex report accounts for every source row

### Requirement: Data loss is loud, witnessed, and enumerated
The system SHALL report every row or field a migration discards as an
explicit finding carrying the concrete lost element — a migration report
with dropped elements is never presented as clean, and no drop is
silent.

#### Scenario: Lost-data counterexample
- **WHEN** a schema map drops a source object's rows (object absent from
  the target)
- **THEN** the migration emits one lost-data finding per dropped row with
  its identity and values — countable, diff-stable

### Requirement: Unsupported inverses are proven, not discovered in production
The system SHALL analyze Δ round-trips (migrate along map, attempt return
map) and SHALL report a non-invertible round-trip as a proven finding
naming the offending elements — "this mapping cannot be inverted" is a
checkable claim, not a runtime surprise.

#### Scenario: Non-invertible round-trip proven
- **WHEN** Δ′∘Δ is not the identity on some element within the analyzed
  scope
- **THEN** the report proves non-invertibility by exhibiting the element
  whose return differs, labeled `executed`
