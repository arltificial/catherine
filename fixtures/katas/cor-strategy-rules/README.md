# Kata `cor-strategy-rules` — a legacy CoR + Strategy dispatcher as a rule table

Pre-registration for beads ticket `catherine-lgl.7` (P8 rule-lens
reverse-engineering katas; grounding `docs/ref/ploeh-grounding.md` §6
candidate 1). Everything except the section *Observed results* is frozen
before the recorded run; a correction afterwards is a superseding ledger
entry, never an edit here.

## Scope statement (the P8 precedent, stated honestly)

`fixtures/rule_lens/README.md` recorded that an **authored** rule table is
a commodity: the conventional arm catches every behaviour-changing fault
and the lens earns unique detections only on behaviour-preserving
structural redundancy. This kata is a *reverse-engineering transfer* of
that surface onto pattern-shaped legacy code: the question is whether the
rule lens, applied to a **Chain-of-Responsibility + Strategy dispatcher
reconstructed from its handler guards**, names defects that a dispatch
test cannot. It is **not** a detection-rate claim: every injected fault
that changes dispatch behaviour in this matrix is caught by the scripted
sample, and the kata would be misread if the three unique detections were
presented as a win rate. The unique band is exactly the structural
redundancy class — dead handlers and derivable strategy branches.

## Hypothesis

Over a dispatcher whose handler guards are modelled as `rule_presentation`
rules in chain order (predicate = guard, action = handler plus the
strategy that handler selects) and whose strategy branches are the later
rules, the rule lens finds **guard gaps outside the sampled request
classes**, **shadowed handlers**, **derivable strategy branches** and
**false disjointness declarations** witness-first; a conventional scripted
dispatch sample catches every fault that changes what the dispatcher
returns and nothing else.

Expected shape (falsifiable, not asserted): unique detections > 0 above
the floor, 0 at the near-floor control, and the unique set restricted to
`gap`/`shadow`/`derivable` witnesses on faults the sample's cases do not
reach.

## Substrate (authored, and said so)

`dispatch.ct.json` — a legacy request dispatcher modelled as a 7-rule table
over the 30-input domain `kind ∈ {fraud, refund, payout, payment, admin} ×
amount ∈ {0, 50, 5000} × channel ∈ {web, batch}`, `policy: PriorityOrder`
(chain order is the only precedence). The rules, in chain order:

| # | Handler | Guard (region) | Action (handler/strategy) |
|---|---|---|---|
| 0 | `fraud_guard` | `kind = fraud` (6) | `fraud_review/manual` |
| 1 | `refund_guard` | `kind = refund` (6) | `refund/auto_credit` |
| 2 | `payout_guard` | `kind = payout` (6) | `payout/bank_transfer` |
| 3 | `large_amount` | `amount >= 5000` (10) | `manual_review/queue` |
| 4 | `capture_card` | `kind = payment ∧ channel = web` (3) | `capture/card_strategy` |
| 5 | `capture_ach` | `kind = payment ∧ channel = batch` (3) | `capture/ach_strategy` |
| 6 | `else_fallback` | `¬(kind ∈ {fraud, refund, payout, payment})` (6) | `support/escalate` |

The last rule is the code's `else` branch written out in the predicate
language; over the declared value set it is `kind = admin` and it is the
only rule covering `admin` below 5000, so it has a non-empty effective
region and the clean table is total (0 findings — asserted by the frozen
gate, not assumed).

### Mapping (pattern → lens)

| Legacy construct | Rule-lens surface | Class |
|---|---|---|
| handler guard | predicate of the rule at the chain index | — |
| chain order | rule order under `PriorityOrder` | shadow (dead handler = empty effective region) |
| handler + selected strategy | rule `action` | invisible to the lens (no structural content) |
| strategy branch | a later rule whose region refines the chain | derivable (region = intersection of two other rules) |
| "handlers are disjoint" documented policy | `policy: DeclaredDisjoint` | overlap (one `violation` per intersecting pair and shared input) |
| no handler claims a request class | region union ≠ domain | gap (`underconstraint`, `hypothesized`) |

## Fault matrix (pre-registered)

Injected as complete documents under `faults/` (above floor) and
`control/faults/` (control). Counts are the lens findings the analysis is
expected to emit; `baseline` is the frozen sample's verdict.

| Fault | Class | Injected change | Baseline | Lens findings |
|---|---|---|---|---|
| `01-chain-gap-remove-fallback` | gap | `else_fallback` deleted | caught (`admin/0/web` unhandled) | 1 `underconstraint` (4 uncovered inputs) |
| `02-chain-gap-fallback-narrow` | gap | fallback guard narrowed to `admin ∧ web` | **missed** | 1 `underconstraint` (2 uncovered `admin`/`batch` inputs) |
| `03-shadow-duplicate-payout` | shadow | identical duplicate of `payout_guard` inserted after it | **missed** | 1 `redundant` |
| `04-shadow-widen-payout` | shadow | `payout_guard` widened to `payout ∨ payment` | caught (3 sampled wrong actions) | 2 `redundant` (both payment branches dead) |
| `05-derivable-strategy-branch` | derivable | `payout_high_value` inserted before `payout_guard`, same action, region = `payout_guard ∩ large_amount` | **missed** | 1 `redundant` |
| `06-overlap-declared-disjoint` | overlap | `policy: DeclaredDisjoint` on the unchanged table | caught (`payment/5000/web` ambiguous) | 10 `violation` (the `large_amount` intersections: 2+2+2+1+1+2) |
| `07-strategy-selection-mismatch` | strategy | `capture_card` action → `capture/ach_strategy` | caught (`payment/0/web` wrong action) | **0** — the lens reads structure only |

Pre-registered unique detections: **3** (`02`, `03`, `05`). Every
behaviour-changing fault is in the baseline's caught column; no unique
detection is a behaviour change.

## Near-floor control (two-handler chain)

`control/two-handler.ct.json` — `kind ∈ {card, cash} × amount ∈ {0, 100}`
(4 inputs), `policy: DeclaredDisjoint`, rules `card_guard` (`kind = card`)
→ `charge/card` and `cash_fallback` (`¬kind = card`, i.e. the `else`
branch) → `charge/cash`. Two handlers, four inputs, clean (0 findings).

The scripted dispatch sample is **the whole input space** at the floor,
which is the point: at this size a conventional test enumerates it. Three
faults, all caught by that 4-row test:

| Fault | Class | Baseline | Lens findings |
|---|---|---|---|
| `floor-01-gap-remove-fallback` | gap | caught (both cash cases unhandled) | 1 `underconstraint` |
| `floor-02-overlap-total-fallback` | overlap | caught (both card cases ambiguous) | 2 `violation` |
| `floor-03-strategy-selection-mismatch` | strategy | caught (both card cases wrong action) | 0 |

Pre-registered control unique detections: **0** → verdict `killed`
(the control must self-negate; a non-empty set refutes the
triviality-floor argument and the kata is recorded as such).

## Baseline protocol (frozen before the run)

`prototype/cor_strategy_rules_baseline.py`, committed and version-pinned
by sha256, is the conventional arm: a scripted dispatch sample of **one
representative request per handler path** above the floor (seven cases),
plus all four floor inputs at the control. Each case resolves under the
document's declared policy (first match under `PriorityOrder`;
all matches, hence ambiguity, under `DeclaredDisjoint`), and is reported
as `missing`, `ambiguous`, or `wrong-action:<got>` against the frozen
expected action.

The above-floor sample is deliberately not exhaustive — that is what a
real dispatch suite looks like, and it is why `02` (unsampled `admin`
batch class), `03` (behaviour-preserving duplicate) and `05`
(behaviour-preserving derivable branch) are invisible to it. Sampling is
the measured quantity, not a strawman: `01`, `04`, `06` and `07` are
caught by the same seven cases.

The baseline arm also executes the probe arm (`catherine rules --json`,
`--max-candidates 64`) on the same documents and prints the arithmetic;
`--assert` fails loudly on any drift between the two arms and the frozen
`faults.json` expectations.

## Kill condition and falsifiers

- The control's unique set is non-empty → the near-floor argument fails;
  the kata is recorded `killed`/`inconclusive`, never smoothed.
- Any above-floor unique detection on a fault the baseline caught → the
  uniqueness arithmetic is broken; re-derive, do not re-describe.
- Lens findings outside the pre-registered counts or kinds — the fixture
  or the analysis changed; record the discrepancy, never edit this file.
- The abstract story this kata must be able to fail: if the lens earned
  nothing the sample missed (parity), the null result is the deliverable
  and the reverse-engineering claim is `killed` exactly as kata-6 killed
  the schema-lens rule table.

## Observed results (recorded after the executed run)

`python3 prototype/cor_strategy_rules_baseline.py --assert` → exit 0:
10 faults, 0 unique detections at the control, 3 above the floor.
Control verdict `killed`; above-floor verdict `survived`, scoped to
structural redundancy on an authored table.

| Fault | Baseline | Lens | Unique |
|---|---|---|---|
| `01-chain-gap-remove-fallback` | caught | 1 `underconstraint` | no |
| `02-chain-gap-fallback-narrow` | missed | 1 `underconstraint` | **yes** |
| `03-shadow-duplicate-payout` | missed | 1 `redundant` | **yes** |
| `04-shadow-widen-payout` | caught | 2 `redundant` | no |
| `05-derivable-strategy-branch` | missed | 1 `redundant` | **yes** |
| `06-overlap-declared-disjoint` | caught | 10 `violation` | no |
| `07-strategy-selection-mismatch` | caught | 0 | no |

Unique: **3** above the floor, **0** at the control. The scope caveat
stands: the unique band is behaviour-preserving structural redundancy on a
table authored for this kata, not a detection-rate advantage over a
dispatch suite.

## Execution

```
python3 prototype/cor_strategy_rules_baseline.py --assert
cargo run -q -p catherine-ledger --bin ledger-check -- ledger/katas/cor-strategy-rules
cargo test -q -p catherine-ledger --test cor_strategy_rules_kata
```