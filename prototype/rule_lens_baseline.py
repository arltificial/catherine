#!/usr/bin/env python3
"""Purpose: the CONVENTIONAL baseline for the rule-lens above-floor kata
(openspec change `add-rule-lens`, phase D task 9; design.md Decision 4) —
the hand-written, behavior-only assertion suite a competent developer would
commit for the same table. It is the comparison arm of the unique-detection
audit, deliberately NOT a rule-lens implementation: it asserts the induced
action on a fixed sample of inputs and nothing else (no gap/overlap/shadow/
derivable analysis).

Responsibilities: evaluate the fixture's predicate language exactly as the
format documents it (int/bool/string comparisons, `mod`, `and`/`or`/`not`),
apply the table's declared policy (rule order for `PriorityOrder` and
`FirstMatch`; `DeclaredDisjoint` yields AMBIGUOUS when more than one rule
matches), and compare against the FROZEN assertions below. Exit 0 when every
assertion holds (the suite did not catch the fixture's fault), exit 1 on the
first mismatch (the suite caught it). The near-floor negative-control
assertions are selected with `--kata near-floor`.

Rationale: the audit must never let the lens grade its own homework. The
assertions are input→action pairs frozen from the clean table before any
fault was injected; a fault is "caught" iff a sampled action changes. This
is the same shape as a table-driven unit test, and it shares no code with
`crates/catherine-*` or `prototype/rule_lens_sim.py`.

Usage:
    python3 prototype/rule_lens_baseline.py <fixture> [--kata pricing|near-floor]
"""

import json
import operator
import sys

OPS = {
    "eq": operator.eq,
    "ne": operator.ne,
    "lt": operator.lt,
    "le": operator.le,
    "gt": operator.gt,
    "ge": operator.ge,
}

GAP = "<gap>"
AMBIGUOUS = "<ambiguous>"


def evaluate(predicate, row):
    kind = predicate["kind"]
    if kind == "cmp":
        return OPS[predicate["op"]](row[predicate["column"]], predicate["value"])
    if kind == "mod":
        return row[predicate["column"]] % predicate["modulus"] == predicate["equals"]
    if kind == "and":
        return all(evaluate(arg, row) for arg in predicate["args"])
    if kind == "or":
        return any(evaluate(arg, row) for arg in predicate["args"])
    if kind == "not":
        return not evaluate(predicate["arg"], row)
    raise ValueError(f"unknown predicate kind {kind!r}")


def induced_action(presentation, row):
    """The action the table claims for one input, under its declared policy."""
    matches = [
        rule for rule in presentation["rules"] if evaluate(rule["predicate"], row)
    ]
    if not matches:
        return GAP
    policy = presentation.get("policy")
    if policy in ("PriorityOrder", "FirstMatch"):
        return matches[0]["action"]
    if policy == "DeclaredDisjoint":
        return matches[0]["action"] if len(matches) == 1 else AMBIGUOUS
    # No policy: the table declares no precedence, so the action on an
    # overlapping input is not determined.
    return matches[0]["action"] if len(matches) == 1 else AMBIGUOUS


def row(age, income, member, tier, region):
    return {
        "age": age,
        "income": income,
        "member": member,
        "tier": tier,
        "region": region,
    }


# Frozen from fixtures/rule_lens/kata/pricing-gold.ct.json before any fault
# was injected: one minimal witness per rule effective region (16) plus a
# table-driven age x tier x region sweep at fixed member=false, income=60000
# (30 rows, deduplicated with the witnesses). 45 assertions total.
PRICING_ASSERTIONS = [
    (row(17, 120000, True, "gold", "eu"), "staff_40"),
    (row(17, 0, True, "gold", "us"), "partner_35"),
    (row(65, 0, True, "basic", "us"), "hardship_30"),
    (row(65, 35000, True, "gold", "eu"), "senior_gold_25"),
    (row(65, 35000, True, "basic", "us"), "senior_20"),
    (row(17, 0, True, "basic", "us"), "youth_15"),
    (row(25, 120000, True, "basic", "us"), "high_value_20"),
    (row(40, 0, True, "basic", "us"), "milestone_8"),
    (row(25, 0, True, "gold", "eu"), "loyalty_gold_15"),
    (row(25, 0, True, "silver", "us"), "loyalty_silver_10"),
    (row(25, 0, True, "basic", "eu"), "regional_eu_10"),
    (row(25, 0, True, "basic", "us"), "member_basic_5"),
    (row(25, 0, False, "basic", "us"), "nonmember_basic_8"),
    (row(25, 0, False, "gold", "us"), "tier_gold_10"),
    (row(25, 0, False, "silver", "us"), "tier_silver_5"),
    (row(25, 60000, False, "basic", "eu"), "standard"),
    (row(17, 60000, False, "basic", "us"), "youth_15"),
    (row(17, 60000, False, "basic", "eu"), "youth_15"),
    (row(17, 60000, False, "silver", "us"), "youth_15"),
    (row(17, 60000, False, "silver", "eu"), "youth_15"),
    (row(17, 60000, False, "gold", "us"), "youth_15"),
    (row(17, 60000, False, "gold", "eu"), "youth_15"),
    (row(25, 60000, False, "basic", "us"), "nonmember_basic_8"),
    (row(25, 60000, False, "silver", "us"), "tier_silver_5"),
    (row(25, 60000, False, "silver", "eu"), "tier_silver_5"),
    (row(25, 60000, False, "gold", "us"), "tier_gold_10"),
    (row(25, 60000, False, "gold", "eu"), "tier_gold_10"),
    (row(40, 60000, False, "basic", "us"), "nonmember_basic_8"),
    (row(40, 60000, False, "basic", "eu"), "standard"),
    (row(40, 60000, False, "silver", "us"), "tier_silver_5"),
    (row(40, 60000, False, "silver", "eu"), "tier_silver_5"),
    (row(40, 60000, False, "gold", "us"), "tier_gold_10"),
    (row(40, 60000, False, "gold", "eu"), "tier_gold_10"),
    (row(65, 60000, False, "basic", "us"), "senior_20"),
    (row(65, 60000, False, "basic", "eu"), "senior_20"),
    (row(65, 60000, False, "silver", "us"), "senior_20"),
    (row(65, 60000, False, "silver", "eu"), "senior_20"),
    (row(65, 60000, False, "gold", "us"), "senior_20"),
    (row(65, 60000, False, "gold", "eu"), "senior_gold_25"),
    (row(80, 60000, False, "basic", "us"), "senior_20"),
    (row(80, 60000, False, "basic", "eu"), "senior_20"),
    (row(80, 60000, False, "silver", "us"), "senior_20"),
    (row(80, 60000, False, "silver", "eu"), "senior_20"),
    (row(80, 60000, False, "gold", "us"), "senior_20"),
    (row(80, 60000, False, "gold", "eu"), "senior_gold_25"),
]

# The near-floor negative control: all four inputs of the trivial table, a
# one-line check per row (nothing above the kata-6 triviality floor).
NEAR_FLOOR_ASSERTIONS = [
    ({"tier": "basic", "member": True}, "member"),
    ({"tier": "basic", "member": False}, "basic"),
    ({"tier": "gold", "member": True}, "gold"),
    ({"tier": "gold", "member": False}, "gold"),
]


def main(argv):
    if len(argv) < 2:
        print(__doc__.strip().splitlines()[-1], file=sys.stderr)
        return 2
    fixture = argv[1]
    kata = "pricing"
    if "--kata" in argv:
        kata = argv[argv.index("--kata") + 1]
    assertions = (
        NEAR_FLOOR_ASSERTIONS if kata == "near-floor" else PRICING_ASSERTIONS
    )
    model = json.loads(open(fixture, encoding="utf-8").read())
    presentation = model["rule_presentation"]
    mismatches = []
    for expected_row, expected_action in assertions:
        actual = induced_action(presentation, expected_row)
        if actual != expected_action:
            mismatches.append((expected_row, expected_action, actual))
    print(
        f"baseline: {fixture}: {len(assertions)} assertion(s), "
        f"{len(mismatches)} mismatch(es)"
    )
    for expected_row, expected_action, actual in mismatches[:5]:
        print(
            f"  mismatch {expected_row}: expected {expected_action!r}, got {actual!r}"
        )
    if len(mismatches) > 5:
        print(f"  ... {len(mismatches) - 5} more")
    return 1 if mismatches else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))