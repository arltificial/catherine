# findings delta

## ADDED Requirements

### Requirement: Findings payloads carry the execution scope
The findings payload SHALL carry `data.scope`, an object with exactly
three snake_case keys — `constraints_checked`, `rows_evaluated`,
`equations_over_zero_rows` — reporting what the producing walk actually
executed, so an empty `data.findings` array with a zero scope
(nothing was checked) is distinguishable from an empty array with a
non-zero scope (checked and clean). The field is part of the public
payload contract consumed by the wasm/web client.

#### Scenario: Empty findings with a zero scope is not a pass
- **WHEN** a findings payload serializes with an empty
  `data.findings` array and `data.scope` of all zeros
- **THEN** the payload states nothing was executed — the emptiness is
  attributable to a vacuous run, not to a verified clean result

#### Scenario: Clean pass is distinguishable from vacuity
- **WHEN** a findings payload serializes with an empty
  `data.findings` array and a non-zero `constraints_checked` and
  `rows_evaluated`
- **THEN** the payload states the walk ran over real constraints and
  rows and found nothing — a genuine clean result

#### Scenario: The web consumer binds to the scope
- **WHEN** the wasm boundary mirrors a findings payload to the web
  client
- **THEN** the mirrored payload carries `data.scope` with the same
  three keys, and the client can branch on it without parsing
  findings or warnings

### Requirement: The scope field is additive and stable
`data.scope` SHALL serialize deterministically alongside the existing
payload fields without changing any of them, their order-independent
encoding, or any exit code; repeated runs of the same document SHALL
produce byte-identical scopes.

#### Scenario: Repeated runs are diff-stable including scope
- **WHEN** the same document is checked twice
- **THEN** the serialized payloads, including `data.scope`, are
  byte-identical, and the pre-existing fields' bytes are unchanged
  from before this field existed
