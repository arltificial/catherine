# Rule-lens golden fixtures (P8 phase A)

Four `format_version` 1.2 documents, one per rule-table bug class. Each
parses as a plain model document (schema + instance) plus the additive
`rule_presentation` section, and each is certified semantically by the
hand-computed oracle in `prototype/rule_lens_sim.py` (case named in the
table). Findings emission is phase B (tasks.md task 6); these fixtures pin
the format, the predicate language, and the expected taxonomy mapping.

`fixtures/rule_lens/README.md` is not read by any test at runtime — the
expectations below are asserted by `crates/catherine-core/tests/rule_lens.rs`
(structure) and by the sim (semantics, bounded enumeration).

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