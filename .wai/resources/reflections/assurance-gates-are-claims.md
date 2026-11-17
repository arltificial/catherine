# Practice: assurance gates are claims — audit the checker, report coverage

**Rule:** a gate that decides whether an assurance artifact is trustworthy
is itself a claim. Give it adversarial red-tests (planted bad input must
fail), audit every artifact it names, and report what the gate actually
covered — not just pass/fail.

## Why

Assurance artifacts (parity oracles, proof audits, mutation checks)
are reviewed for the claim they make. Their *gates* are usually written
once, trusted because they are simple, and never given a failing input.
A gate with a blind spot converts an unproved claim into a green check —
the worst failure mode, because the artifact is then believed.

## Evidence

- `catherine-xvh` (Lean twin, 2026-11-17): the axiom audit
  (`scripts/check-lean-axioms.sh`) failed only on its `sorryAx`
  substring test and its `^axiom` source scan. Lean never echoes
  declaration lines and
  `#print axioms` names the axiom bare (`[cheat]`), so a proof on a
  user-declared axiom passed. Found by ro5u as HIGH CORR-001; fixed by
  asserting the axiom set against an allowlist, failing on build
  `declaration uses 'sorry'` warnings, and scanning every source for
  `axiom`. A `--file` seam now lets the gate be tested with planted
  `[cheat]` / `[sorryAx]` / empty-audit files.
- Same change, CORR-002: only four declarations were audited, while the
  decision entry cited four additional exclusion theorems as landed
  proofs; a `sorry` in an unaudited theorem was invisible. Coverage must
  be stated, not implied.
- Same change, CORR-003: the dangling-discipline statement proved one
  short-circuit orientation while the prose claimed both. A certificate
  narrower than its prose is a silent weakening.
- Same change, parity: 102 of 181 agreements were both-sides-empty. A
  pass/fail oracle would have reported “green” for a corpus where 56% of
  the comparisons assert nothing; the harness now reports
  `trivially_agreeing` vs `findings_bearing_agreeing`, and zero
  findings-bearing agreement is red.

## Gate

For any new assurance gate:

1. **Red-test it.** Plant the defect it is supposed to catch (an axiom, a
   `sorry`, an empty input, a deleted witness) and show the gate fails.
   Keep the planted-input path (a `--file`/fixture seam) so the test is
   repeatable.
2. **State coverage.** Name every artifact/declaration the gate checks
   and the exclusion boundary (e.g. private helpers are covered only
   through the public theorems that depend on them); fail explicitly if
   the audited set is empty. Report trivial vs substantive agreement,
   not just the total.
3. **Fail closed.** An unexpected exit, empty output, or unparsable line
   is a failure, not a skip.
4. **Re-run after the fix and reproduce the red**, not just the green.

## Decision procedure

1. New gate or gate change? → write the planted-bad-input test first.
2. Gate audits a list? → make the list explicit and machine-derived;
   empty list = failure.
3. Gate reports a count? → split it into trivial and substantive; a
   substantive count of zero is red.
4. Statement/prose claims symmetry? → check both orientations before
   calling the proof done.
5. Cross-language parity? → compare normalized witnesses (kind + label +
   witness), never bytes, and disclose what was skipped and why.

## Links

- Research record:
  `.wai/projects/catherine/research/2026-11-17-lean-twin-implementation-learnings-catherine-xv.md`
- Review evidence:
  `openspec/changes/add-lean-kernel-model/ro5u-review.md`
  (CORR-001/002/003)
- Gate: `scripts/check-lean-axioms.sh`; oracle: `lean/Catherine/Parity.lean`.