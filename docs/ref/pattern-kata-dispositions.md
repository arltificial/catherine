# GoF pattern → universal-abstraction disposition table

Status: the suite is a **HYPOTHESIZED** reverse-engineering hypothesis
(`catherine-lgl`, ploeh dictionary `docs/ref/ploeh-grounding.md` §6). Every
row states what was actually executed, where the evidence lives, and what
the disposition is. Nothing here claims a detection-rate win: parity with
conventional tests is the expected outcome (E4), and several rows are
explicit kills or gaps.

Dispositions:

- **executed** — a recorded kata with a control that self-negates, a
  committed conventional baseline arm, and a ledger entry (scoped).
- **killed** — explored; the conventional test caught the same faults
  (parity). Recorded as a null result, not a failure.
- **unexpressible** — no predicate/law for the pattern on the current
  kernel surface; the gap is named.
- **disposed** — no executable structural invariant identified for this
  surface (construction/convenience patterns). Not a proof of
  impossibility; a re-open needs a new law, not a new fixture.

| Pattern | Universal abstraction (ploeh) | Kata | Executable law on the surface | Evidence | Disposition |
|---|---|---|---|---|---|
| Abstract Factory | product families / functor | — | construction only; no invariant on a finite instance | — | disposed |
| Builder | monoid (accumulation) | `builder-monoid` | emitted part traces to a declared step; splice order equation | `ledger/katas/builder-monoid/` | executed, scoped (duplicate-row case parity → killed) |
| Factory Method | construction dispatch | — | construction only | — | disposed |
| Prototype | copy/equality | `weak-lens-set` | field equality after clone | `ledger/katas/weak-lens-set/` | killed (parity) |
| Singleton | unique element | — | object identity has no predicate | `weak-lens-set` (identity gap) | unexpressible |
| Adapter | wrapper law preservation | `wrapper-laws` | link-level delegation/field laws | `ledger/katas/wrapper-laws/` | executed, coverage-only (unique → 0 under a complete cross-product sample) |
| Bridge | two-axis delegation | `weak-lens-set` | delegation path agreement | `ledger/katas/weak-lens-set/` | killed (parity) |
| Composite | monoid (identity) | `composite-null-object` | identity/totality equations; associativity unexpressible | `ledger/katas/composite-null-object/` | executed, scoped |
| Decorator | wrapper law preservation | `wrapper-laws` | stacking transparency equation | `ledger/katas/wrapper-laws/` | executed, coverage-only |
| Facade | convenience delegation | — | no invariant | — | disposed |
| Flyweight | sharing / identity | `weak-lens-set` | none — identity is data | `ledger/katas/weak-lens-set/` | unexpressible (0 findings, gap recorded) |
| Proxy | wrapper law preservation | `wrapper-laws` | identity-preservation equation | `ledger/katas/wrapper-laws/` | executed, coverage-only |
| Chain of Responsibility | catamorphism / handler order | `cor-strategy-rules` | rule-lens gap/shadow/overlap/derivable over handler order | `ledger/katas/cor-strategy-rules/` | executed, scoped (3 unique; 4 parity) |
| Command | reification / inverse | `weak-lens-set` | inverse law | `ledger/katas/weak-lens-set/` | killed (parity) |
| Interpreter | sum type / recursive dispatch | `visitor-sum-types` | coverage of the variant domain | `ledger/katas/visitor-sum-types/` | executed, scoped |
| Iterator | traversal protocol | — | traversal state is behavioral, not a doc invariant | — | disposed |
| Mediator | star topology | — | wiring has no equation with a truth value; a discovery question is plausible but unvalidated | — | disposed (candidate for a discovery-only kata) |
| Memento | Δ save/restore pair | `memento-roundtrip` | lost-data enumeration + round-trip non-invertibility proof | `ledger/katas/memento-roundtrip/` | executed, scoped (control self-negates) |
| Observer | coherence / broadcast | `weak-lens-set` | subject/observer coherence | `ledger/katas/weak-lens-set/` | killed (parity) |
| State | state shape / discovery questions | `state-discovery` | transition laws, declared-cell totality; unreachable-state detection unexpressible | `ledger/katas/state-discovery/` | executed, scoped (one honest null kill) |
| Strategy | rule lens above the floor | `cor-strategy-rules` | branch selection as rule regions | `ledger/katas/cor-strategy-rules/` | executed, scoped |
| Template Method | inheritance ordering | — | no surface (method bodies are not modeled) | — | disposed |
| Visitor | sum type / exhaustiveness | `visitor-sum-types` | variant coverage + handler classes | `ledger/katas/visitor-sum-types/` | executed, scoped (3 unique) |
| Null Object | monoid identity element | `composite-null-object` | nil-identity equations | `ledger/katas/composite-null-object/` | executed, scoped |

## Honesty gate

`ledger/katas/negative-control-bundle/` is the suite's adversarial check:
pattern-lookalike and procedural fixtures where the kernel must stay
silent, plus a Flyweight identity case where no lens exists. A fabricated
finding there would fail the bundle; its recorded results are the
evidence that the dispositions above are not bought with false positives.

## What the table does and does not license

- It licenses a **structural** claim: where a pattern's equivalence was
  proven in the dictionary, an executable check often exists, with the
  law's scope named (identity/totality vs associativity; coverage vs
  behavior).
- It does **not** license a detection-rate claim: the executed rows are
  scoped, several uniques are sampling gaps, and two whole classes
  (construction and identity) are disposed/unexpressible.
- A row marked disposed or unexpressible is a gap statement, not a
  theorem. Re-opening one requires naming the law first.