# wasm-ui delta (P5)

## ADDED Requirements

### Requirement: The wasm boundary is JSON-only and logic-free
The system SHALL expose kernel and analysis functionality to the browser
exclusively through wasm-bindgen functions that take and return JSON —
no business logic in the wasm crate — and SHALL enforce a row-count cap
at the boundary with a typed error rather than unbounded work.

#### Scenario: Oversized instance rejected at the boundary
- **WHEN** a document exceeding the declared row cap is loaded in the
  browser
- **THEN** the boundary returns a typed cap error naming the cap — the
  kernel is never invoked on the oversized instance

#### Scenario: Kernel stays pure
- **WHEN** the workspace builds for the `wasm32` target
- **THEN** `catherine-core`/`-analysis` compile with no wasm-bindgen or
  genesis dependency

### Requirement: The web view renders the model and the findings faithfully
The system SHALL render the schema as an SVG graph (objects, arrows,
equation paths) and the findings list grouped by kind with epistemic
labels and witnesses — the same finding taxonomy and encoding the CLI
emits, generated from the shared JSON Schema artifact.

#### Scenario: Browser and CLI agree
- **WHEN** the same `.ct.json` is checked by the CLI and loaded in the
  web view
- **THEN** both show the same findings with the same kinds, labels, and
  witnesses

#### Scenario: The view is never the source of truth
- **WHEN** a model is edited in the JSON textarea and saved
- **THEN** the saved artifact is a plain `.ct.json` file identical in
  format to what the kernel parses — the UI adds no side-car state
