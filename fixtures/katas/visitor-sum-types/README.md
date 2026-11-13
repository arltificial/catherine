# Kata — Visitor as a sum type over a legacy dispatch table (pre-registration)

Committed **before** any run. Nothing in this file is edited after the run:
a correction is a superseding ledger entry.

Ticket: `catherine-lgl.3` (P2). Substrate: a fictional legacy AST
Visitor/Interpreter in a language without exhaustiveness checking (the
`docs/ref/ploeh-grounding.md` §6 candidate 3, "Visitor-as-sum-type").
Pattern-equivalence mapping: `docs/ref/ploeh-grounding.md` §2.5 and §6.

## Hypothesis

A legacy Visitor/Interpreter dispatch can be reverse-engineered into a
`rule_presentation` over the finite variant domain (one rule per variant,
`action` = the visitor method, declared policy). Once declared,
`catherine rules` exhaustively enumerates the declared variant domain, so
it names dispatch defects a **sampled** scripted dispatch test misses:

1. a handler shadowed by an earlier broader registration (dead case);
2. a handler derivable from two broader registrations (redundant case);
3. a variant added to the AST that no visitor case covers (uncovered
   input, unsampled because the conventional test predates the variant).

At the near-floor control (two variants, a trivial two-rule table where
the scripted test samples *every* variant) the lens must buy zero unique
detections. Zero unique across the above-floor fault set kills the
transfer claim.

**Honest boundary.** Authored rule tables are commodity: the declared
table is not the artifact under test, and `catherine rules` does not
author it. This kata measures the *reverse-engineering transfer* from an
OOP dispatch shape into the declared rule table plus exact enumeration
over the whole variant domain. It is **not** a detection-rate win over
conventional tooling, and it is not a claim that the lens beats a
conventional test that enumerates every variant and asserts structural
properties (such a test would reach parity). The pre-registered
conventional arm is a *sampled* scripted dispatch test, exactly what the
legacy suite was.

## Pattern-equivalence mapping (fixed before the run)

| Visitor/Interpreter concept | Rule-presentation element |
|---|---|
| AST sum type (variant set) | finite input domain X, declared by `--values kind=…` and committed as `Node` instance rows |
| variant constructor | one input class `kind = <variant>` |
| `visit_<variant>` method | one rule, `predicate` selecting the variant, `action` = method name |
| `accept` dispatch order / double dispatch | rule order under the declared policy (`PriorityOrder`) |
| "these variants are disjoint by construction" | `DeclaredDisjoint` policy |
| default `visit_expr` fallback / range registration | a rule whose predicate is a union of variant classes |
| missing method (non-exhaustive match) | **gap** → `underconstraint` (hypothesized) |
| duplicate/overlapping registration | **overlap** under `DeclaredDisjoint` → `violation` (executed) |
| unreachable case | **shadow** → `redundant` (executed) |
| case equal to a boolean combination of broader cases | **derivable** → `redundant` (executed) |

The rule lens owns the exact enumeration; this kata does not reimplement
any engine logic. The fixtures only declare the table.

## Substrate — `visitor.ct.json` (clean)

`Node` object, one string column `kind`; five variants `num`, `str`,
`add`, `mul`, `neg`; five disjoint rules under `PriorityOrder`:

| rule | predicate (kind) | action |
|---|---|---|
| `visit_num` | `num` | `visit_num` |
| `visit_str` | `str` | `visit_str` |
| `visit_add` | `add` | `visit_add` |
| `visit_mul` | `mul` | `visit_mul` |
| `visit_neg` | `neg` | `visit_neg` |

The clean table has zero findings on the domain
`kind ∈ {num, str, add, mul, neg}`.

## Fault matrix (fixed before the run)

| Fault | Fixture | Injected defect | Affected variant | Pre-registered lens class | Baseline sample |
|---|---|---|---|---|---|
| F1 | `faults/01-gap-missing-num.ct.json` | remove `visit_num` (missing visitor method) | `num` | `underconstraint` (gap) | sampled → caught |
| F2 | `faults/02-overlap-duplicate-add.ct.json` | add a second `add` registration, policy `DeclaredDisjoint` (duplicate visitor) | `add` | `violation` (overlap) | sampled → caught |
| F3 | `faults/03-shadow-literal-covers-str.ct.json` | insert a broad `num \| str` registration before `visit_str` (shadowed handler) | `str` | `redundant` (shadow) | unsampled → missed |
| F4 | `faults/04-derivable-num.ct.json` | re-register `num` as `literal_scope ∩ arith_scope`; behavior preserved (derivable handler) | `num` | `redundant` (derivable) | behavior preserved → missed |
| F5 | `faults/05-missing-variant-div.ct.json` | add the `div` variant to the AST (`Node` row) with no rule (missing-variant coverage) | `div` | `underconstraint` (gap) | unsampled → missed |

F4's table is behavior-preserving on *every* variant: the specific
`visit_num` handler fires first; `literal_scope` (num | str → `visit_str`),
`arith_scope` (num | add → `visit_add`), `binary_scope` (add | mul →
`visit_mul`) are priority registrations whose effective remainders are the
single variants their actions name. The specific handler is exactly
`R(literal_scope) ∩ R(arith_scope)`, so it is structurally derivable while
dispatch is unchanged.

F5 commits the new variant as a `Node` instance row; the run binds
`--values kind=num,str,add,mul,neg,div`. The CLI never infers X from the
instance rows — the committed inventory and the `--values` list are the
same declared data, stated twice for reproducibility.

## Near-floor control (fixed before the run)

`floor/two-variant.ct.json`: two variants (`lit`, `neg`), two rules
(`visit_lit`, `visit_neg`), zero findings. Three injected floor faults:

| Fault | Fixture | Defect | Pre-registered class |
|---|---|---|---|
| NF1 | `floor/faults/01-gap-remove-lit.ct.json` | remove `visit_lit` | `underconstraint` (gap) |
| NF2 | `floor/faults/02-overlap-duplicate-lit.ct.json` | duplicate `lit` registration + `DeclaredDisjoint` | `violation` (overlap) |
| NF3 | `floor/faults/03-shadow-widen-lit.ct.json` | broad `lit \| neg` registration before `visit_neg` | `redundant` (shadow) |

The floor sample is **both** variants (`lit`, `neg`): every injected floor
fault changes a sampled dispatch or removes/duplicates a sampled
registration, so the scripted test catches all three and the lens must
record zero unique detections → control verdict `killed`.

**Floor scope (recorded, not smoothed).** All three floor faults are
behavior-changing. A behavior-preserving *derivable* handler cannot be
expressed on two variants with distinct actions (a broad rule covering
both variants and preserving both actions would need two actions), so the
derivable class is above-floor only. The zero-unique result is scoped to
behavior-changing faults, not a claim that structural redundancy is
undetectable at any floor.

## Conventional baseline protocol (committed before the run)

`prototype/visitor_sum_types_baseline.py` models the review a team
actually performs on this legacy visitor: a scripted dispatch test over a
pre-registered sample of variants, asserting the resolved action against
the recorded expectation. Resolution follows the document's declared
policy (first match under `PriorityOrder`; every match under
`DeclaredDisjoint`, where more than one match is ambiguous).

Frozen sample (above-floor): **`num`, `add`, `neg`** — the arithmetic
nodes the legacy interpreter's test suite exercised; `str` and `mul` were
not sampled. Frozen sample (floor): **`lit`, `neg`** (all variants).

Detections are strings:

- `sample:<variant>:missing` — no rule matched;
- `sample:<variant>:ambiguous:<a>|<b>` — two or more rules matched under
  `DeclaredDisjoint` (actions in rule order);
- `sample:<variant>:wrong:<actual>` — the resolved action differs from the
  frozen expectation.

Pre-registered baseline behaviour: catches F1 and F2 (sampled faults),
misses F3 (unsampled `str`), F4 (behaviour preserved everywhere), F5
(unsampled `div`). Catches all three floor faults.

The baseline supports `--assert` (compare every fixture's computed
detections to the frozen `baseline_expected` in `faults.json`; exit 1 on
mismatch) and `--hash` (sha256 of the script, pinned into the positive
ledger entry's baseline artifact).

## Kill condition and falsifiers

- **Kill:** zero unique detections across F1–F5 or across the floor set
  when it was pre-registered to be zero. A null result is recorded with
  the control `killed`; the honest boundary above stays in force.
- Falsifier: any lens finding outside the pre-registered class sets, any
  baseline `--assert` mismatch, or a floor fault the baseline catches
  that a positive fault's parity depends on — record the discrepancy,
  never edit this file.

## Execution order

```
python3 prototype/visitor_sum_types_baseline.py --assert   # conventional arm first
cargo run -q -p catherine-cli -- rules … --json            # probe arm
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/visitor-sum-types
cargo test -q -p catherine-ledger --test visitor_sum_types_kata
```