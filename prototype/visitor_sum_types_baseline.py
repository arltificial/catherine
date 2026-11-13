#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the `visitor-sum-types` kata —
the scripted dispatch test a team actually runs on a legacy Visitor.

The test resolves a pre-registered sample of variants through the
document's declared dispatch policy and compares the resolved action to
the frozen expectation. It is deliberately a sample: the legacy suite
exercised the arithmetic nodes, not every variant.

Detections are strings: `sample:<variant>:missing`,
`sample:<variant>:ambiguous:<a>|<b>` (DeclaredDisjoint, behaviorally
distinct actions in rule order), or `sample:<variant>:wrong:<actual>`.

`--assert` compares every fixture against the frozen `baseline_expected`
in `faults.json` and exits 1 on any mismatch; `--hash` prints the
script's sha256 (pinned into the ledger's executed-baseline artifact).
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "visitor-sum-types"
MATRIX = KATA / "faults.json"

OPS = {
    "eq": lambda left, right: left == right,
    "ne": lambda left, right: left != right,
    "lt": lambda left, right: left < right,
    "le": lambda left, right: left <= right,
    "gt": lambda left, right: left > right,
    "ge": lambda left, right: left >= right,
}

# The same declared domain keys the pre-registration names; the sample is
# selected by the fault's probe domain, never inferred from the fixture.
SAMPLE_KEYS = {
    "domain": "sample",
    "fault_domain": "sample",
    "floor_domain": "floor_sample",
}


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def evaluate(predicate: dict, row: dict) -> bool:
    """Evaluate the rule lens's tiny predicate language on one input row."""
    kind = predicate["kind"]
    if kind == "cmp":
        return OPS[predicate["op"]](row[predicate["column"]], predicate["value"])
    if kind == "and":
        return all(evaluate(arg, row) for arg in predicate["args"])
    if kind == "or":
        return any(evaluate(arg, row) for arg in predicate["args"])
    if kind == "not":
        return not evaluate(predicate["arg"], row)
    raise ValueError(f"predicate kind {kind!r} outside the frozen review's language")


def resolve(rules: list[dict], policy: str | None, row: dict) -> list[str]:
    """The actions an input dispatches to under the declared policy."""
    matched = [rule["action"] for rule in rules if evaluate(rule["predicate"], row)]
    if policy == "DeclaredDisjoint":
        return matched
    return matched[:1]


def sample_detections(doc: dict, sample: dict[str, str]) -> list[str]:
    """The sampled test's detections: missing, ambiguous, or wrong action."""
    presentation = doc["rule_presentation"]
    policy = presentation.get("policy")
    detections = []
    for variant, expected in sample.items():
        matches = resolve(presentation["rules"], policy, {"kind": variant})
        if not matches:
            detections.append(f"sample:{variant}:missing")
        elif len(set(matches)) > 1:
            detections.append(f"sample:{variant}:ambiguous:{'|'.join(matches)}")
        elif matches[0] != expected:
            detections.append(f"sample:{variant}:wrong:{matches[0]}")
    return detections


def check_case(case: dict, samples: dict[str, dict[str, str]]) -> tuple[dict, list[str]]:
    """Run the sampled test on one fixture and compare with its freeze."""
    doc = json.loads((KATA / case["fixture"]).read_text())
    detections = sample_detections(doc, samples[SAMPLE_KEYS[case["probe_domain"]]])
    failures = []
    if detections != case["baseline_expected"]:
        failures.append(
            f"{case['id']}: detections {detections} != "
            f"frozen {case['baseline_expected']}"
        )
    if "baseline_catches" in case and case["baseline_catches"] != bool(detections):
        failures.append(
            f"{case['id']}: caught={bool(detections)}, "
            f"preregistered {case['baseline_catches']}"
        )
    record = {"id": case["id"], "fixture": case["fixture"], "detections": detections}
    return record, failures


def main() -> int:
    if "--hash" in sys.argv[1:]:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(MATRIX.read_text())
    samples = {"sample": matrix["sample"], "floor_sample": matrix["floor_sample"]}
    report, failures = [], []
    for group in ("clean", "faults", "floor_faults"):
        for case in matrix[group]:
            record, problems = check_case(case, samples)
            report.append({"group": group, **record})
            failures.extend(problems)
    hash_value = sha256_of(Path(__file__))
    print(json.dumps({"baseline_sha256": hash_value, "cases": report}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv[1:]:
        print(
            f"baseline arm frozen: {len(report)} fixtures, "
            f"{sum(len(r['detections']) for r in report)} detections",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())