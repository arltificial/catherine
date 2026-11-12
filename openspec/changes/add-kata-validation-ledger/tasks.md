# Tasks: add-kata-validation-ledger

One red→green→refactor cycle (beads `catherine-9vv`).

## RED — the ledger refuses bad science

- [x] 1. Ledger validator rejects: kata entry without a prior negative
      control; verdict outside the closed vocabulary; entry without
      model-hash references — `crates/catherine-ledger/src/validate.rs`
      names all three (`NegativeControlMissing`, `InvalidVerdict`,
      `MissingModelHashes`) plus `ControlDidNotSelfNegate`,
      `IdMismatch`, `SeqOrder`, `BadSupersession`, `RollupMisplaced`,
      `RollupIncomplete`; rejections are total (any violation refuses
      the ledger); pinned by the crate's RED tests (14 + 3 passing).

## GREEN

- [x] 2. Negative control: port FizzBuzz (kata-6) fixture; run; record
      verdict (expected: no categorical advantage) as ledger entry #1
- [x] 3. Port Bowling (kata-1): temporal-semantics fixture; expected
      partial result recorded honestly
- [x] 4. Port Gilded Rose: state-shape invariants as path equations +
      totality (entry #3 `ledger/0003-ee446895.json`: survived — the two
      structural catches [F-sulfuras, F-backstage] fired as executed
      violations; 4 behavioral faults recorded as gaps, never papered
      over; `ledger-check` green)
- [x] 5. Port Vending Machine: strongest structural result (6/6)
      (entry #4 `ledger/0004-04d57a83.json`: survived, scoped — F2/F5
      fired executed, F3 construction-time vocabulary rejection; the
      series ledger REFUTED the 6/6 rounding [F6 killed by an executed
      MR≠RM counterexample], discrepancy recorded in the entry and the
      catalog; `ledger-check` green)
- [x] 6. Browser fixtures wired into the web view (load, check, findings
      panel per kata) — `web/index.html` gains a `#kata-preset` select;
      `web/src/app.ts` lists the four `fixtures/katas/*.ct.json` base
      models and fetches the chosen fixture via `fetch("../fixtures/katas/…")`
      into the textarea, reusing the EXISTING load→`runCheck`→findings
      panel path (no framework). Serves from the repo root
      (`python3 -m http.server`; page-relative path). Served bundle
      rebuilt (`web/app.js`, esbuild, same command shape as `build`);
      `tsc --noEmit` green; vitest 3/3; smoke: served page and fixture
      both 200.
- [x] 7. Ledger entries content-addressed; append-only; supersession
      supported — ids are `sha256:` over canonical content bytes,
      recomputed by the validator on every `ledger-check` (entry-file
      id mismatch = `IdMismatch`, the integrity contract); the four
      kata filenames carry their id prefixes
      (0001-0466d547 / 0002-48f21fc0 / 0003-ee446895 / 0004-04d57a83);
      `ledger-record` refuses a record that breaks the directory and
      reverts it (append-only enforced at write time); `supersedes`
      targets are checked real-and-earlier (`BadSupersession`) with the
      later-killed-control correction path
      (`ControlDidNotSelfNegate`); id-change-on-any-change pinned by the
      entry.rs unit tests (`equal_content_equal_id_changed_content_changed_id`).
- [x] 8. Project-verdict rollup entry; gates wired: P7/P8 beads blocked
      unless `survived` — entry #5 `ledger/0005-6079e36b.json` derived
      STRICTLY via `catherine_ledger::rollup::derive` over the four
      sealed entries; verdict **survived** (scoped): control gate passed
      (FizzBuzz killed, zero unique detections), all three positive
      katas survived; PROCEDURE embedded verbatim (7 steps);
      `kata_entry_ids` are the real sealed ids; gates
      `p7_catherine_alr` / `p8_catherine_nq5` = "unblock only on project
      survived (derived: survived); the bead flip is
      orchestrator-reserved"; `ledger-check` green: `ok: 5 entries
      (4 kata), rollup verdict: survived`.
- [x] 9. Expressibility-gap field recorded per kata (injectivity-flavored
      invariants named, not papered over) — verified: every kata entry
      carries non-empty `expressibility_gap` entries, each naming the
      invariant and why the v1 surface cannot say it: kata-6 (divisibility/
      coverage/precedence; domain bounds), kata-1 (history-dependence of
      scoring; frame-count bounds; per-frame pin sums), kata-2 (quality
      arithmetic deltas/tiers; time decay incl. frozen sellIn; live-pass
      quality bounds), kata-3 (operation commutativity MR≠RM; boundary
      wrapping; operation sequencing).

## REFACTOR

- [x] 10. Extract kata-fixture harness shared with P8/P9 work —
      `crates/catherine-ledger/src/harness.rs`: `run(fixture)` parses a
      fixture, builds the `TypedInstance`, executes the kernel checks on
      the ONE path the CLI and wasm boundary drive, and returns the
      `KataRun` inventory (parsed flag — the construction-time probe
      class — plus findings verbatim) a ledger entry records; consumes
      only `catherine-core` public artifacts, so P8/P9 reuse needs no
      ledger coupling.

## Validation

- [x] 11. All four ledger entries present, ordered, validator green;
      `just ci` green; `openspec validate add-kata-validation-ledger
      --strict`; `bd close catherine-9vv` + fresh export —
      `ledger-check`: `ok: 5 entries (4 kata), rollup verdict: survived`
      (seqs 1–5, rollup last, coverage complete); `openspec validate
      add-kata-validation-ledger --strict` green; scoped
      `cargo test -p catherine-ledger` 17/17. ORCHESTRATOR-RESERVED
      (pending): `just ci` (workspace gate, out of worker scope) and
      `bd close catherine-9vv` + fresh export.
