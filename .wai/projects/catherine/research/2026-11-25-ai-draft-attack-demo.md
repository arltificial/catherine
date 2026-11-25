# Demo: the AI-draft → catherine-attack loop, executed

Date: 2026-11-25 · protocol: `docs/ref/ai-draft-attack-recipe.md` ·
domain: **library lending desk** · verdict: **loop completed, controls pass**.

Executed with the debug build via `cargo run --quiet --bin catherine --`
(catherine CLI, repo root). All commands below were actually run; outputs
are verbatim, exit codes recorded.

## Files

- `fixtures/attack-demo/prose.md` — the prose requirements (LLM's only draft input)
- `fixtures/attack-demo/draft-v1.ct.json` — LLM draft, revision 1
- `fixtures/attack-demo/draft-v2.ct.json` — LLM draft, revision 2 (final)
- `fixtures/attack-demo/target.ct.json` — naturality target instance
- `fixtures/attack-demo/control-overconstraint.ct.json` — negative control A

## The prose (input)

> A lending desk tracks physical copies of books and the loans of those
> copies to members. Every loan is for exactly one physical copy, and
> every copy is a copy of exactly one catalog book. When a loan is
> recorded, the desk writes down both which copy was handed out and (for
> shelf reports) which catalog book the loan counts against — these must
> always agree: the book a loan counts against is exactly the book of the
> copy that was handed out. Each loan is taken out by exactly one member.

## (b) LLM draft, revision 1

Draft v1 encodes four arrows (`copy_of: Copy→Book`, `loan_copy:
Loan→Copy`, `loan_book: Loan→Book`, `loan_member: Loan→Member`), a small
instance, and **no equations** — the classic LLM failure: the prose rule
"the loan's book is the copy's book" is left implicit as two parallel
paths Loan→Book.

## (c) Battery on draft-v1

```sh
$ cargo run --quiet --bin catherine -- check fixtures/attack-demo/draft-v1.ct.json --human
```
(no output) — exit **0**. The declared instance satisfies every declared
constraint; the model's emptiness of constraints is not `check`'s business.

```sh
$ cargo run --quiet --bin catherine -- underconstraint fixtures/attack-demo/draft-v1.ct.json --max-path-length 3 --human
underconstraint: parallel paths loan_book / loan_copy∘copy_of (hypothesized)
  should these agree?
```
exit **6** (underconstraint). The parallel-path question fired exactly on
the unasserted prose rule.

```sh
$ cargo run --quiet --bin catherine -- search fixtures/attack-demo/draft-v1.ct.json --rows Loan=1 --rows Copy=1 --rows Book=1 --rows Member=1 --max-candidates 256 --human
search: witness within bounds
```
exit **0**. A witness exists within the bounds — the draft is satisfiable,
which is a bounded outcome, not a clean bill.

(`naturality` skipped at v1: no `instance_map` declared yet.)

## (d)–(e) Revision → draft-v2

Fed the exit-6 finding back; the revision asserts the path equation

```json
{"lhs": ["loan_book"], "rhs": ["loan_copy", "copy_of"], "status": "accepted",
 "rationale": "…the prose rule… closing the parallel-path question raised on draft v1"}
```

and adds an `instance_map` (row renaming to the target vocabulary) so
naturality can run. No other edits.

## (c) Battery on draft-v2

```sh
$ cargo run --quiet --bin catherine -- check fixtures/attack-demo/draft-v2.ct.json --human
```
exit **0** (silent).

```sh
$ cargo run --quiet --bin catherine -- underconstraint fixtures/attack-demo/draft-v2.ct.json --max-path-length 3 --human
```
exit **0** (silent — the question is now answered by a declared equation).

```sh
$ cargo run --quiet --bin catherine -- search fixtures/attack-demo/draft-v2.ct.json --rows Loan=1 --rows Copy=1 --rows Book=1 --rows Member=1 --human
search: witness within bounds
```
exit **0**.

Naturality, first attempt — real refusal, recorded honestly:

```sh
$ cargo run --quiet --bin catherine -- naturality fixtures/attack-demo/draft-v2.ct.json fixtures/attack-demo/target.ct.json --human
error[schema_mismatch]: the second document's schema differs from the first (`fixtures/attack-demo/target.ct.json`)
  hint: distinguish compares two instances of the same schema; give two documents with identical schemas
```
exit **1**. The target had been written with an empty equation set while
the source asserts one; naturality requires the *whole* schema —
equations included — to be identical. Fix: the target document mirrors
the v2 schema verbatim (instance rows stay the renamed target vocabulary).

```sh
$ cargo run --quiet --bin catherine -- naturality fixtures/attack-demo/draft-v2.ct.json fixtures/attack-demo/target.ct.json --human
```
exit **0** (silent): `map ∘ arrow = arrow ∘ map` on every arrow and mapped
row.

## (f) Negative controls

### Control A — injected overconstraint (must be caught)

`control-overconstraint.ct.json` = draft v2's schema and equation, with a
second loan injected that breaks it: loan `l2` is recorded against book
`b1`, but its copy `c2` is a copy of `b2`.

```sh
$ cargo run --quiet --bin catherine -- check fixtures/attack-demo/control-overconstraint.ct.json --human
violation: eq-0 loan_book ≠ loan_copy∘copy_of
  at row Loan#1: lhs=Book#0 rhs=Book#1
```
exit **3** (violation). **Caught** — the battery is not vacuous: the same
equation it enforced silently on the clean draft fires on the broken row.

### Control B — clean control (must pass silent)

Full battery on the unmodified draft-v2:

```sh
$ cargo run --quiet --bin catherine -- check fixtures/attack-demo/draft-v2.ct.json --human
$ cargo run --quiet --bin catherine -- underconstraint fixtures/attack-demo/draft-v2.ct.json --max-path-length 3 --human
$ cargo run --quiet --bin catherine -- naturality fixtures/attack-demo/draft-v2.ct.json fixtures/attack-demo/target.ct.json --human
$ cargo run --quiet --bin catherine -- search fixtures/attack-demo/draft-v2.ct.json --rows Loan=1 --rows Copy=1 --rows Book=1 --rows Member=1 --human
search: witness within bounds
```
check/underconstraint/naturality exit **0** silent; search exit **0**,
its witness line being a payload outcome, not a finding. **Passed silent.**

## Verdict

- Loop executed end to end: prose → draft v1 → battery (underconstraint
  fires, exit 6) → revision → draft v2 → battery clean.
- Control A: injected overconstraint **caught** (violation, exit 3).
- Control B: clean draft **passed silent** on all four verbs.
- Stopping rule of the recipe satisfied: clean battery + both controls
  demonstrated.
- Honest notes: the first naturality invocation was refused
  (`schema_mismatch`) because the target's equation set differed; the
  refusal itself is recorded as part of the transcript. `search`'s
  "witness" is a bounded successful outcome, never claimed as "clean".
