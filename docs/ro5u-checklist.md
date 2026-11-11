# ro5u review checklist

The standing completion-gate checklist for rule-of-5-universal reviews.
Ticket briefs (`§8 ro5u review completion gate`) reference this file; the
per-change review artifact lands at
`openspec/changes/<change>/ro5u-review.md` (findings + dispositions).

Before declaring a ticket done, every item holds:

- [ ] rule-of-5-universal review of the **full diff** run (invoke
      explicitly: `/skill:rule-of-5-universal`), gate: **max_critical = 0**
      (critical/high fixed; medium fixed-or-deferred-with-reason; lows
      deferred only with written reasons).
- [ ] The review artifact (findings + dispositions) is recorded and
      cited in the implementor report — ro5u is a completion criterion:
      without it the ticket is not done regardless of test status.
- [ ] Every behavior-impacting fix adds or updates a test (structural
      outcome assertions, parity/property tests, or CLI envelope/exit
      assertions — never snapshots).
- [ ] **A new flag or surface without a negative-matrix row is
      incomplete** (catherine-348): every CLI surface or flag added or
      changed gains rows in
      `crates/catherine-cli/tests/negative_matrix.rs` covering every
      applicable malformed-input class, each row asserting a labeled
      error (exit 1, stderr envelope, stable code) or an envelope
      warning — never silence.
