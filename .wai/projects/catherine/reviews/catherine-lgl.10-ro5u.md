# ro5u review — catherine-lgl.10 (GoF disposition table)

Lead-side review. Deliverable: `docs/ref/pattern-kata-dispositions.md`,
linked from the README docs table.

Five passes:

1. Correctness: all 23 GoF patterns plus Null Object have a row, each with
   the abstraction, the kata (if any), the executable law, the ledger path,
   and a disposition. Executed rows cite the recorded ledgers
   (composite-null-object, builder-monoid, visitor-sum-types,
   state-discovery, memento-roundtrip, wrapper-laws, cor-strategy-rules);
   killed rows cite weak-lens-set; unexpressible rows name the missing
   predicate; disposed rows state that no executable invariant was
   identified on this surface (explicitly not a theorem of impossibility).
   Wrapper rows carry the coverage-only caveat (unique → 0 under a complete
   cross-product sample), which is the suite's most important honesty note.
2. Contract drift: docs only. No claim in the table exceeds its ledger;
   HYPOTHESIZED status is stated at the top; P8 scope (authored rule tables
   are commodity) is preserved for the CoR/Strategy row.
3. Tests: N/A (docs); every executed row links to a committed ledger that
   is validated by its slice test.
4. Negative matrix: the honesty gate (`negative-control-bundle`) is cited
   and its result stated: no fabricated findings, all arms killed. The
   table's own "what it does and does not license" section forbids a
   detection-rate reading.
5. Regression risk: one docs file + one README link.

Findings + dispositions:

- [NOTE, ACCEPTED] "Disposed" rows are gap statements, not proofs; the
   re-open rule (name the law first) is written down so a future session
   cannot fill them with a fixture and call it validation.
- [NOTE, RECORDED] Counts are not repeated per row; the table points at the
   ledgers so a reader can audit the arithmetic in place.

max_critical = 0. Verdict: PASS.