# Rule-lens golden fixtures (P8 phase A)

Four `format_version` 1.2 documents, one per rule-table bug class. Each
parses as a plain model document (schema + instance) plus the additive
`rule_presentation` section, and each is certified semantically by the
hand-computed oracle in `prototype/rule_lens_sim.py` (case named in the
table). Findings emission is phase B (tasks.md task 6); these fixtures pin
the format, the predicate language, and the expected taxonomy mapping.

`fixtures/rule_lens/README.md` is not read by any test at runtime — the
expectations below are asserted by `crates/catherine-core/tests/rule_lens.rs`
(structure), by `crates/catherine-core/tests/rule_lens_{parity,fixtures}.rs`
(semantics, bounded enumeration), and by
`crates/catherine-analysis/tests/rule_lens_findings.rs` (taxonomy
mapping) and `crates/catherine-cli/tests/cli_rules.rs` (the `rules`
surface) as well as by the sim.

## Input domain and policy

`rule_presentation.domain` names the declared schema object `Input`; the
input domain X is the product of the declared `columns`, not the instance
rows. The sim bounds X explicitly (the fixture documents do not carry a
value enumeration — bounded search supplies it). Policies follow
DESIGN §4.4 / design.md Decision 3: `PriorityOrder` and `FirstMatch`
resolve intersections by rule order; `DeclaredDisjoint` declares no
precedence, so every intersection is a violation; an absent policy means
no precedence at all (gap/overlap diagnostics only).

## Expected findings

| Fixture | Bug class | Witness (bounded enumeration) | Mapped taxonomy kind (design.md Decision 1) | Sim case |
|---|---|---|---|---|
| `gap.ct.json` | gap | X = {10, 20, 30, 70}; `age = 70` is covered by no rule | **underconstraint**, `hypothesized` until totality is declared | `gap_age` |
| `overlap-disjoint.ct.json` | overlap under `DeclaredDisjoint` | `amount = 100` satisfies both `small` and `large` | **violation** (declared disjointness failing on a concrete input) | `overlap_disjoint` |
| `shadow.ct.json` | shadow | E(`even`) = ∅: `all` matches every input before `even` | **redundant**, witness = the effective-region proof | `shadow_first_match` |
| `derivable.ct.json` | derivable | R(`fizzbuzz`) = {15} = R(`fizz`) ∩ R(`buzz`) | **redundant**, witness = the combining rules `fizz` ∧ `buzz` | `derivable_fizzbuzz` |

No new finding variants are introduced: the rule lens is a producer over
the existing taxonomy (design.md Decision 1), which is why the mapping is
pinned here before any analysis code exists.

## Policy-dependence (what must *not* be reported)

The same regions under a priority policy are legal. For the overlap
fixture re-declared `PriorityOrder`, `amount = 100` resolves to `small`
and `large`'s effective region is `{200}` — no overlap finding and no
shadow (sim case `overlap_priority_resolves`). A table with no `policy`
key reports overlap as a diagnostic but is mapped to no taxonomy kind,
because Decision 1 pins overlap only under `DeclaredDisjoint`
(sim case `no_policy_overlap_diagnostic`).

## Phase D validation katas (pre-registered, design.md Decision 4)

The above-floor table is `kata/pricing-gold.ct.json`: 16 compound rules
(int `age`/`income`, bool `member`, string `tier`/`region`, `mod`,
`and`/`or`/`not`) under `PriorityOrder` over the 240-input domain
`age=17,25,40,65,80 × income=0,35000,60000,120000 × member=true,false ×
tier=basic,silver,gold × region=us,eu`. It is clean (0 findings). The
conventional baseline is `prototype/rule_lens_baseline.py`: 45 frozen
behavior-only assertions (one minimal witness per rule effective region
plus an age × tier × region grid at `member=false, income=60000`), with
no rule-lens code. Twelve faults are injected as `kata/faults/*.ct.json`;
the audit result committed in `ledger/rule-lens/0002-4ef797a1.json` is:

| Fault | Class | Conventional | Lens | Unique |
|---|---|---|---|---|
| 01-gap-remove-fallback | gap | caught | gap + witness | no |
| 02-gap-fallback-boundary | gap | caught | gap + witness | no |
| 03-gap-fallback-income | gap | caught | gap + witness | no |
| 04-shadow-duplicate-senior | shadow | missed | redundant + witness | **yes** |
| 05-shadow-duplicate-member-basic | shadow | missed | redundant + witness | **yes** |
| 06-shadow-widen-youth | shadow | caught | redundant + witness | no |
| 07-derivable-senior-gold | derivable | missed | redundant + witness | **yes** |
| 08-derivable-young-basic | derivable | caught | redundant + witness | no |
| 09-derivable-senior-silver | derivable | missed | redundant + witness | **yes** |
| 10-derivable-loyalty-regional | derivable | missed | redundant + witness | **yes** |
| 11-overlap-policy-swap | overlap | caught | violation + witness | no |
| 12-overlap-widen-partner | overlap | caught | violation + witness | no |

Conventional 7/12, lens 12/12, unique 5/12. All five unique detections
are behavior-preserving structural redundancy; every behavior-changing
fault was caught by the conventional suite. The matrix is conditional on
the committed 45-assertion baseline (a sparser suite would miss fault 02
too — recorded in the entry's `discrepancies`, not rounded).

The near-floor control is `kata/negative-control/near-floor.ct.json`
(3 rules, 4 inputs) with three behavior-changing faults
(`kata/negative-control/faults/`): the 4-row conventional check catches
3/3 and the lens catches 3/3, so the lens earns zero unique detections
and the control self-negates (`killed`). The katas live in
`ledger/rule-lens/`. The above-floor entry was later corrected by
append-only supersession — `0004` carries the executed
`prototype/rule_lens_baseline.py` arm with the seven faults it caught,
`0005` is the replacement rollup — so the directory holds five files: the
control, the superseded prose-only record, its correction, the superseded
rollup, and the effective rollup. The derived rule-lens verdict is
`survived`, scoped to structural redundancy.