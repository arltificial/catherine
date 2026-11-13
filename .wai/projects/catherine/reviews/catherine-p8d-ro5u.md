# ro5u review — catherine-p8d (docs status drift)

Lead-side review of the status-drift fix (chore). Deliverables: `README.md`
status section and rule-lens row; `docs/USING.md` captured-run section
(the banner flip landed with catherine-1od's claim-scoping commit
`e60956e`, per the coordination note: 1od owns claim positioning and the
banner evidence, p8d owns status drift).

Five passes:

1. Correctness: every status statement now matches the repo. The roadmap
   block said "implementation not started" while P0–P10 were closed and
   P11 recorded; it now states that and keeps the historical phase map.
   The rule-lens row's "(P8, hypothesized)" became "(P8, executed; scoped
   to extracted tables)", matching `ledger/rule-lens/` (survived, scoped)
   and the probe entries.
2. Contract drift: none — docs only; `ah check` remains green over the
   deployed specs.
3. Tests: captured runs are verbatim CLI output on committed fixtures
   (check on `fixtures/katas/bowling-fault-miswired-slot.ct.json`; migrate
   and rules runs already cited in the 1od ledger entries). No example is
   presented as EXECUTED unless it was actually executed.
4. Negative matrix: the pre-format equipment sketch stays explicitly
   ILLUSTRATED and is not runnable; the banner says so. No synthetic output
   is passed off as captured.
5. Regression risk: zero code.

Findings + dispositions:

- [NOTE, ACCEPTED] The worked example still uses the pre-P0 document shape
   (arrows as an object, nested equations). Replacing it with a real
   `.ct.json` would rewrite the pedagogy; it is labeled ILLUSTRATED and a
   captured `check` run now sits beside it. Replacing the sketch is
   deferred to a docs pass if wanted.
- [NOTE, RECORDED] The banner flip happened in the 1od commit rather than
   this one; coordination note honored (status-drift edits are separate
   from claim repositioning).

max_critical = 0. Verdict: PASS.