# Tasks: add-execution-scope

One red→green→refactor arc (beads `catherine-kpn`). RED: golden
fixtures and wasm/web binding tests that assert `data.scope` on
check/findings payloads fail before the field exists; the
single-document fixture asserts today's bytes plus exactly the new
field. GREEN: field appears; vacuous fixtures report zeros. REFACTOR:
single shared scope computation, no second path.

## Phase 1 — kernel + CLI half

- [ ] 1. Scope computation in one place: a `ExecutionScope {
      constraints_checked, rows_evaluated, equations_over_zero_rows }`
      value computed from the check walk's own counters, in one shared
      function consumed by both `check` and `findings` command paths —
      never recomputed per subcommand (design Decision 3)
- [ ] 2. Payload construction: thread the scope through
      `crates/catherine-cli/src/render/envelope.rs` — `FindingsData`
      and `SourcedFindingsData` gain the additive `scope` field
      (`#[derive(Serialize)]`, snake_case keys verbatim); both
      envelope paths are the only writers; error envelopes and
      non-executing subcommands (`search`, `diff`, `migrate`,
      `rules`, pattern, naturality, `query`) stay untouched
- [ ] 3. Vacuous runs report zeros: below-floor documents,
      constraint-free documents, and zero-row instances serialize
      `data.scope` with all three keys `0` (design Decision 4) —
      exit codes unchanged for all of them
- [ ] 4. CLI tests (RED first): golden single-document fixture updated
      by adding only the new field (byte-diff proves nothing else
      moved); vacuous-instance and constraint-free fixtures assert
      zero scopes; multi-document composition fixture asserts the
      joint walk's totals; exit-code assertions unchanged
      - Evidence: `ah check --changes add-execution-scope` and the
        CLI test suite green

## Phase 2 — wasm/web half

- [ ] 5. Boundary mirror: `crates/catherine-wasm/src/boundary.rs`
      exposes `data.scope` on the mirrored check/findings payloads
      with the same three snake_case keys — the wasm consumer binds to
      the public contract, not a reshaped copy
- [ ] 6. Web types: `web/src/types/ct-schema-v1.ts` gains the typed
      `scope` object (`constraints_checked: number`,
      `rows_evaluated: number`, `equations_over_zero_rows: number`) on
      the check/findings payload types
- [ ] 7. Web consumer: `web/src/findings.ts` renders (or at minimum
      branches on) the scope so a zero-scope empty result is
      distinguishable in the UI from a checked-and-clean one — the
      axis-C done-when for the web client
      - Evidence: wasm binding test and web-side fixture/test asserting
        the three keys end to end

## REFACTOR + gate

- [ ] 8. No second scope computation exists (grep the CLI crate for
      per-subcommand recounting); fixture diffs show only the added
      field; `openspec validate add-execution-scope --strict`; bead
      status updated + fresh `bd export`
