#!/usr/bin/env python3
"""Purpose: the rule-lens semantics oracle (openspec change `add-rule-lens`,
phase A task 1; design.md Decision 5 — Python before Rust).

Responsibilities: implement effective regions E_i = R_i \\ union_{j<i} R_j,
the four bug classes (gap, overlap, shadow, derivable), and table
equivalence (identical total action on X, first differing input as witness)
over a bounded, enumerable input domain, and assert them against
HAND-COMPUTED oracle tables committed below: every expected set was
enumerated by hand before the code existed, and the script exits non-zero on
any divergence — a self-checking oracle, not a demo. A domain is a tuple of
(column, values); an input is the value tuple in column order.

Rationale: predicates use the format 1.2 `rule_presentation` JSON shape
verbatim (kind-tagged cmp/mod/and/or/not), so fixtures/rule_lens/ exercises
the language this sim pins; bounded enumeration stands in for P4 bounded
search. Mapping is design.md Decision 1: gap → underconstraint (hypothesized
until totality is declared), overlap under DeclaredDisjoint → violation,
shadow/derivable → redundant. Overlap with no declared policy is a
diagnostic, deliberately unmapped (Decision 3 forbids implicit precedence).
"""

import itertools
import operator
import sys

GAP = object()  # action undefined: no rule covers the input
AMBIGUOUS = object()  # action underdetermined: overlapping rules, no precedence
OPS = {"eq": operator.eq, "ne": operator.ne, "lt": operator.lt,
       "le": operator.le, "gt": operator.gt, "ge": operator.ge}
PRIORITY = ("PriorityOrder", "FirstMatch")
POLICIES = PRIORITY + ("DeclaredDisjoint",)


def rule(name, predicate, action):
    return {"name": name, "predicate": predicate, "action": action}


def cmp(column, op, value): return {"kind": "cmp", "column": column, "op": op, "value": value}
def mod(column, modulus, equals): return {"kind": "mod", "column": column, "modulus": modulus, "equals": equals}
def conj(*args): return {"kind": "and", "args": list(args)}
def neg(arg): return {"kind": "not", "arg": arg}


def eval_pred(pred, row):
    kind = pred["kind"]
    if kind == "cmp":
        return OPS[pred["op"]](row[pred["column"]], pred["value"])
    if kind == "mod":  # non-negative remainder, per the format's int semantics
        return row[pred["column"]] % pred["modulus"] == pred["equals"]
    if kind == "and":
        return all(eval_pred(arg, row) for arg in pred["args"])
    if kind == "or":
        return any(eval_pred(arg, row) for arg in pred["args"])
    if kind == "not":
        return not eval_pred(pred["arg"], row)
    raise ValueError(f"unknown predicate kind {kind!r}")


def enumerate_domain(domain):
    names = tuple(name for name, _ in domain)
    inputs = [tuple(values) for values in itertools.product(*(v for _, v in domain))]
    return names, inputs


def region(rule_, names, inputs):
    return frozenset(
        inp for inp in inputs if eval_pred(rule_["predicate"], dict(zip(names, inp)))
    )


def effective_regions(table, names, inputs):
    """E_i under the policy; None when no policy is declared (Decision 3)."""
    regions = [region(r, names, inputs) for r in table["rules"]]
    policy = table["policy"]
    if policy in PRIORITY:
        effective, covered = [], frozenset()
        for r in regions:
            effective.append(r - covered)
            covered = covered | r
        return effective
    return list(regions) if policy == "DeclaredDisjoint" else None


def find_overlaps(table, regions):
    policy = table["policy"]
    if policy in PRIORITY:  # legal by construction: order resolves them
        return []
    return [
        (table["rules"][i]["name"], table["rules"][j]["name"], shared)
        for i, j in itertools.combinations(range(len(regions)), 2)
        if (shared := regions[i] & regions[j])
    ]


def find_shadow(table, effective):
    if effective is None:
        return None
    return [table["rules"][i]["name"] for i, eff in enumerate(effective) if not eff]


def find_derivable(table, regions):
    names = [r["name"] for r in table["rules"]]
    return [
        (names[k], (names[i], names[j]))
        for k in range(len(regions))
        for i, j in itertools.combinations([n for n in range(len(regions)) if n != k], 2)
        if regions[k] == regions[i] & regions[j]
    ]


def mapped_kinds(policy, gap, overlaps, shadow, derivable):
    """design.md Decision 1's taxonomy mapping, in finding order."""
    kinds = [("gap", "underconstraint", "hypothesized")] if gap else []
    if overlaps:
        violation = ("overlap", "violation", "accepted")
        kinds.append(violation if policy == "DeclaredDisjoint" else ("overlap", None, None))
    kinds += [("shadow", "redundant", "accepted") for _ in shadow or ()]
    kinds += [("derivable", "redundant", "accepted") for _ in derivable]
    return tuple(kinds)


def analyze(table, domain):
    names, inputs = enumerate_domain(domain)
    policy = table["policy"]
    if policy is not None and policy not in POLICIES:
        raise ValueError(f"unknown policy {policy!r}")
    regions = [region(r, names, inputs) for r in table["rules"]]
    gap = frozenset(inputs) - (frozenset().union(*regions) if regions else frozenset())
    effective = effective_regions(table, names, inputs)
    overlaps = find_overlaps(table, regions)
    shadow = find_shadow(table, effective)
    derivable = find_derivable(table, regions)
    return {"inputs": inputs, "regions": regions, "effective": effective, "gap": gap,
            "overlaps": overlaps, "shadow": shadow, "derivable": derivable,
            "kinds": mapped_kinds(policy, gap, overlaps, shadow, derivable)}


def induced_action(table, names, inputs):
    """The total action the table claims on X: input → action | GAP | AMBIGUOUS."""
    policy = table["policy"]
    actions = {}
    for inp in inputs:
        row = dict(zip(names, inp))
        matches = [r for r in table["rules"] if eval_pred(r["predicate"], row)]
        if not matches:
            actions[inp] = GAP
        elif len(matches) == 1 or policy in PRIORITY:
            actions[inp] = matches[0]["action"]
        else:
            actions[inp] = AMBIGUOUS
    return actions


def equivalence(left, right, domain):
    """(equivalent, witness, bound): same total action on every input of X?"""
    names, inputs = enumerate_domain(domain)
    a, b = induced_action(left, names, inputs), induced_action(right, names, inputs)
    for inp in inputs:
        if a[inp] != b[inp]:
            return False, inp, len(inputs)
    return True, None, len(inputs)


def fmt_input(inp): return "(" + ", ".join(repr(value) for value in inp) + ")"


def fmt_set(items): return "<not computed>" if items is None else "{" + ", ".join(fmt_input(i) for i in sorted(items)) + "}"


def check(label, actual, expected, failures):
    if actual != expected:
        failures.append(f"{label}: expected {expected!r}, got {actual!r}")
        return False
    return True

# ── Oracle: analysis cases (hand-computed) ────────────────────────────────

AGE = (("age", (10, 20, 30, 70)),)
SIX = (("n", (1, 2, 3, 4, 5, 6)),)
FIFTEEN = (("n", tuple(range(1, 16))),)

FIZZBUZZ_RULES = [
    rule("fizzbuzz", mod("n", 15, 0), "FizzBuzz"),
    rule("fizz", mod("n", 3, 0), "Fizz"),
    rule("buzz", mod("n", 5, 0), "Buzz"),
    rule("plain", cmp("n", "ge", 1), "n"),
]

CASES = [{
        "name": "gap_age",
        "domain": AGE,
        "table": {"policy": "PriorityOrder", "rules": [rule("kid", cmp("age", "lt", 18), "kid"),
            rule("adult", conj(cmp("age", "ge", 18), cmp("age", "lt", 65)), "adult")]},
        # Hand: R_kid = {10}; R_adult = {20, 30}; X = {10, 20, 30, 70}.
        "expect": {
            "regions": ({(10,)}, {(20,), (30,)}),
            "effective": ({(10,)}, {(20,), (30,)}),
            "gap": {(70,)}, "overlaps": (), "shadow": (), "derivable": (),
            "kinds": (("gap", "underconstraint", "hypothesized"),)},
    }, {
        "name": "overlap_disjoint",
        "domain": (("amount", (50, 100, 200)),),
        "table": {"policy": "DeclaredDisjoint", "rules": [rule("small", cmp("amount", "le", 100), "small"),
            rule("large", cmp("amount", "ge", 100), "large")]},
        # Hand: R_small = {50, 100}; R_large = {100, 200}; shared = {100}.
        "expect": {
            "regions": ({(50,), (100,)}, {(100,), (200,)}),
            "effective": ({(50,), (100,)}, {(100,), (200,)}),
            "gap": set(), "overlaps": (("small", "large", {(100,)}),),
            "shadow": (), "derivable": (),
            "kinds": (("overlap", "violation", "accepted"),)},
    }, {
        "name": "overlap_priority_resolves",
        "domain": (("amount", (50, 100, 200)),),
        "table": {"policy": "PriorityOrder", "rules": [rule("small", cmp("amount", "le", 100), "small"),
            rule("large", cmp("amount", "ge", 100), "large")]},
        # Hand: same regions, order resolves 100 for `small`, E_large = {200}.
        "expect": {
            "regions": ({(50,), (100,)}, {(100,), (200,)}),
            "effective": ({(50,), (100,)}, {(200,)}),
            "gap": set(), "overlaps": (), "shadow": (), "derivable": (), "kinds": ()},
    }, {
        "name": "shadow_first_match",
        "domain": SIX,
        "table": {"policy": "FirstMatch", "rules": [rule("all", cmp("n", "ge", 1), "default"),
            rule("even", mod("n", 2, 0), "even")]},
        # Hand: R_all = X; E_all = X; E_even = {2,4,6} \ X = {}.
        "expect": {
            "regions": ({(1,), (2,), (3,), (4,), (5,), (6,)}, {(2,), (4,), (6,)}),
            "effective": ({(1,), (2,), (3,), (4,), (5,), (6,)}, set()),
            "gap": set(), "overlaps": (), "shadow": ("even",), "derivable": (),
            "kinds": (("shadow", "redundant", "accepted"),)},
    }, {
        "name": "derivable_fizzbuzz",
        "domain": FIFTEEN,
        "table": {"policy": "FirstMatch", "rules": FIZZBUZZ_RULES},
        # Hand: R_fizzbuzz = {15} = R_fizz ∩ R_buzz;
        # R_fizz = {3,6,9,12,15}; R_buzz = {5,10,15}; R_plain = X.
        "expect": {
            "regions": ({(15,)}, {(3,), (6,), (9,), (12,), (15,)},
                        {(5,), (10,), (15,)},
                        {(1,), (2,), (3,), (4,), (5,), (6,), (7,), (8,), (9,),
                         (10,), (11,), (12,), (13,), (14,), (15,)}),
            "effective": ({(15,)}, {(3,), (6,), (9,), (12,)}, {(5,), (10,)},
                          {(1,), (2,), (4,), (7,), (8,), (11,), (13,), (14,)}),
            "gap": set(), "overlaps": (), "shadow": (),
            "derivable": (("fizzbuzz", ("fizz", "buzz")),),
            "kinds": (("derivable", "redundant", "accepted"),)},
    }, {
        "name": "types_bool_string",
        "domain": (("member", (True, False)), ("tier", ("basic", "gold"))),
        "table": {"policy": "PriorityOrder", "rules": [
            rule("gold", conj(cmp("member", "eq", True), cmp("tier", "eq", "gold")), "gold"),
            rule("basic", cmp("tier", "lt", "gold"), "basic"),  # byte-wise: "basic" < "gold"
            rule("guest", neg(cmp("member", "eq", True)), "guest")]},
        # Hand: E_gold = {(T,gold)}; E_basic = {(T,basic),(F,basic)};
        # E_guest = {(F,basic),(F,gold)} minus covered = {(F,gold)}.
        "expect": {
            "regions": ({(True, "gold")}, {(True, "basic"), (False, "basic")},
                        {(False, "basic"), (False, "gold")}),
            "effective": ({(True, "gold")}, {(True, "basic"), (False, "basic")},
                          {(False, "gold")}),
            "gap": set(), "overlaps": (), "shadow": (), "derivable": (), "kinds": ()},
    }, {
        "name": "no_policy_overlap_diagnostic",
        "domain": (("age", (10, 20, 70)),),
        "table": {"policy": None, "rules": [rule("young", cmp("age", "le", 20), "young"),
            rule("old", cmp("age", "gt", 20), "old"), rule("vip", cmp("age", "ge", 20), "vip")]},
        # Hand: R_young = {10,20}, R_old = {70}, R_vip = {20,70}; union = X;
        # overlaps at 20 and 70; no precedence, so effective/shadow are None.
        "expect": {
            "regions": ({(10,), (20,)}, {(70,)}, {(20,), (70,)}),
            "effective": None, "gap": set(),
            "overlaps": (("young", "vip", {(20,)}), ("old", "vip", {(70,)})),
            "shadow": None, "derivable": (),
            "kinds": (("overlap", None, None),)},
    }]

# ── Oracle: equivalence cases (hand-computed) ─────────────────────────────

FIZZBUZZ_SWAPPED = {"policy": "FirstMatch", "rules": [
    rule("fizzbuzz", mod("n", 15, 0), "FizzBuzz"),
    rule("buzz", mod("n", 5, 0), "Buzz"),
    rule("fizz", mod("n", 3, 0), "Fizz"),
    rule("plain", cmp("n", "ge", 1), "n")]}

DEFAULT_ALL = {"policy": "FirstMatch", "rules": [rule("all", cmp("n", "ge", 1), "default")]}
DEFAULT_THEN_EVEN = {"policy": "FirstMatch", "rules": [
    rule("all", cmp("n", "ge", 1), "default"), rule("even", mod("n", 2, 0), "even")]}
EVEN_THEN_DEFAULT = {"policy": "FirstMatch", "rules": [
    rule("even", mod("n", 2, 0), "even"), rule("all", cmp("n", "ge", 1), "default")]}

AGE_TWO_RULES = CASES[0]["table"]
AGE_THREE_RULES = AGE_TWO_RULES | {"rules": AGE_TWO_RULES["rules"] + [
    rule("senior", cmp("age", "ge", 65), "senior")]}

EQUIVALENCE_CASES = [
    {
        "name": "reordered_fizzbuzz_is_equivalent",
        "domain": FIFTEEN,
        "left": {"policy": "FirstMatch", "rules": FIZZBUZZ_RULES},
        "right": FIZZBUZZ_SWAPPED,
        # Hand: 15 → FizzBuzz first in both; 3,6,9,12 → Fizz; 5,10 → Buzz;
        # the rest → n. fizz/buzz order is immaterial behind fizzbuzz.
        "expect": {"equivalent": True, "witness": None},
    },
    {
        "name": "shadowed_rule_drops_without_witness",
        "domain": SIX,
        "left": DEFAULT_THEN_EVEN,
        "right": DEFAULT_ALL,
        # Hand: `all` matches every input first, so `even` never fires;
        # both tables induce "default" everywhere.
        "expect": {"equivalent": True, "witness": None},
    },
    {
        "name": "order_matters_when_earlier_rule_narrows",
        "domain": SIX,
        "left": EVEN_THEN_DEFAULT,
        "right": DEFAULT_ALL,
        # Hand: 1 → default on both; 2 → "even" vs "default" (first witness).
        "expect": {"equivalent": False, "witness": (2,)},
    },
    {
        "name": "gap_input_distinguishes_tables",
        "domain": AGE,
        "left": AGE_TWO_RULES,
        "right": AGE_THREE_RULES,
        # Hand: 70 is GAP on the left and "senior" on the right.
        "expect": {"equivalent": False, "witness": (70,)},
    },
]

# ── Runner ────────────────────────────────────────────────────────────────


def check_analysis(case, failures):
    analysis = analyze(case["table"], case["domain"])
    expect = case["expect"]
    as_tuple = lambda value: None if value is None else tuple(value)
    checks = [
        check("regions", tuple(analysis["regions"]), expect["regions"], failures),
        check("effective", as_tuple(analysis["effective"]), expect["effective"], failures),
        check("gap", analysis["gap"], expect["gap"], failures),
        check("overlaps", tuple(analysis["overlaps"]), expect["overlaps"], failures),
        check("shadow", as_tuple(analysis["shadow"]), as_tuple(expect["shadow"]), failures),
        check("derivable", tuple(analysis["derivable"]), expect["derivable"], failures),
        check("kinds", analysis["kinds"], expect["kinds"], failures),
    ]
    return analysis, all(checks)


def print_analysis(case, analysis, ok):
    print(f"  {case['name']}: {'ok' if ok else 'FAIL'}  (|X| = {len(analysis['inputs'])})")
    print(f"    regions     {[fmt_set(r) for r in analysis['regions']]}")
    effective = analysis["effective"]
    print(f"    effective   {None if effective is None else [fmt_set(e) for e in effective]}")
    print(f"    gap         {fmt_set(analysis['gap'])}")
    print(f"    overlaps    {[(i, j, fmt_set(s)) for i, j, s in analysis['overlaps']]}")
    print(f"    shadow      {analysis['shadow']}")
    print(f"    derivable   {analysis['derivable']}")
    print(f"    kinds       {analysis['kinds']}")


def check_equivalence(case, failures):
    equivalent, witness, bound = equivalence(case["left"], case["right"], case["domain"])
    ok = check("equivalent", equivalent, case["expect"]["equivalent"], failures)
    ok = check("witness", witness, case["expect"]["witness"], failures) and ok
    print(f"  {case['name']}: {'ok' if ok else 'FAIL'}  "
          f"(equivalent={equivalent}, witness="
          f"{fmt_input(witness) if witness else None}, |X| = {bound})")


def main():
    failures = []
    print("analysis cases (hand-computed oracle)")
    for case in CASES:
        analysis, ok = check_analysis(case, failures)
        print_analysis(case, analysis, ok)
    print("equivalence cases (hand-computed oracle)")
    for case in EQUIVALENCE_CASES:
        check_equivalence(case, failures)
    if failures:
        print(f"\nFAIL: {len(failures)} oracle mismatch(es)")
        for failure in failures:
            print(f"  - {failure}")
        return 1
    print(f"\nPASS: {len(CASES)} analysis cases + {len(EQUIVALENCE_CASES)} "
          "equivalence cases match the hand-computed oracle")
    return 0


if __name__ == "__main__":
    sys.exit(main())