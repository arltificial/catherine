# ro5u review — catherine-1od (P11 domain-probe validation)

Lead-side review of the executed probe science. Deliverables: two probe
ledgers (`ledger/probes/migration/`, `ledger/probes/rule-lens-extracted/`),
their committed fault sets and pre-registrations
(`fixtures/probes/{migration,rule_lens}/`), two executed, version-pinned
baseline arms (`prototype/migration_baseline.py`,
`prototype/rule_lens_extracted_baseline.py`), frozen assertions
(`probe_migration_fixtures.rs`, `probe_ledger_entries.rs`,
`probe_rule_lens_extracted.rs`), docs claim-scoping, and the wired
spectacular contracts. Commits `c234a41`, `354212a`, `5748fc5`, `e60956e`.

Orchestrator verification (its own commands): both baseline arms green
with their frozen gates (`--assert`, 3 faults/11 unique; 7 faults/4
unique); `ledger-check` ok on both probe ledgers (3 and 2 entries, no
rollup — probe verdicts gate nothing) and unchanged on `ledger/` and
`ledger/rule-lens/`; `cargo test -p catherine-ledger` green; `ah check
--changes add-domain-probe-validation --run-tests` = 90 passed, 0
findings; `openspec validate --changes --strict` 2/2; workspace suite
green with the pre-existing slow property skipped (see blocker).

Five passes:

1. Correctness: the arithmetic is mechanical, not asserted — unique
   detections are probe elements minus baseline-identified elements, and
   the migration baseline identifies elements only where a dropped table
   makes the row set exact. Control fixtures self-negate (verdict
   killed, empty unique) exactly where the baseline catches everything.
   Probe 1 found 10 named cell losses + 1 round-trip witness the
   conventional lint/dry run cannot identify; probe 2 found an unsampled
   gap plus three dead/widened rules a sampled conventional review
   misses. The derivable expectation is recorded as unreached with its
   reason, never forced.
2. Contract drift: no new entry kind (rule-lens reuse held); the probe
   ids and expectation rows are the ones committed in tasks 3-4, and
   `CatalogHashMismatch` is enforced on entry, `ledger-record`, and the
   recorded entries. The kata ledger stayed byte-identical (existing
   digest test plus untouched files); the validator changed only for
   clippy (`filter().next_back()` → `rfind`, same behavior).
3. Tests: expectations were committed before the runs; both arms are
   replayable (script `--assert`, `cargo test`); fixture model hashes
   are pinned; baseline scripts are hash-pinned inside the entries and
   have since been refactored — entries cite the new hashes
   (`ebdf6614…` migration, `7e5cdf83…` rule lens), and the earlier
   pins were amended out before any push.
4. Negative matrix: two self-negating controls; the fabricated-table
   refusal held (the bogart serving corpus has no extractable policy
   table — `corpus_attachments` empty, 0 pages with the values — so an
   alternative real annex source was selected and recorded, with its
   PDF-text hazard and absence from the corpus surface in the entry
   discrepancies). The openrouter/mistral/nemotron dispatch numbers
   remain uncited.
5. Regression risk: kernel crates untouched; changes are ledger
   tests/fixtures, two prototype scripts, docs, and contracts. The only
   behavioral-code edit is the clippy rfind (identical semantics).

Findings + dispositions:

- [NOTE, ACCEPTED] Fixtures were authored at reduced row counts (8 docs,
   16 pages, 6 captures) instead of the pilot corpus's hundreds: the
   kernel's canonical relabel (used by `model_hash`) exhausts
   individualization branches on symmetric instances, so hashing a
   120-row path-shaped fixture did not terminate. The probe still
   exercises the pilot *schema* shape; the scale claim in the task text
   is softened and the defect is filed separately (below).
- [NOTE, RECORDED] Probe 2's above-floor table has six rules, so its
   unique band is scoped to an extracted table of that size; the entry
   says "extracted table", never a corpus-level or DMN-parity claim.
- [NOTE, RECORDED] Probe 2's source annex is a recon PDF extraction, not
   a corpus document; recorded in the entry discrepancies so no
   corpus-ingest claim is implied.
- [BLOCKER, PRE-EXISTING, FILED] `cargo test --workspace` hangs on
   `catherine-core`'s `presentation_row_names_never_reach_the_hash`
   proptest (canonical relabel exponential on symmetric models): the
   test alone exceeded 180 s repeatedly; the full suite is green with
   that property skipped. Not caused by this ticket; filed as a beads
   bug with the repro.
- [DEFERRED, LOW] `ah check` positional-argument error seen once from
   the pre-commit wrapper (clap: "--changes requires a value") while the
   same `ah check` passes standalone and via `lefthook run`; not
   reproducible on retry. Not a catherine gate condition.

max_critical = 0. Verdict: PASS (scoped: probe evidence recorded; full
workspace gate pending the pre-existing canonicalization defect).