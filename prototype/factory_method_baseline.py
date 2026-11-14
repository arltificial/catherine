#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the `factory-method` kata —
the scripted factory test a legacy team actually runs.

For each **sampled** product kind the test calls the creator's dispatch and
asserts the returned concrete creator (class) against the frozen
expectation. Resolution reads the fixture's own `route`/`handler` tables
(the legacy switch under review); the expected class comes from the frozen
sample in `faults.json` (the test's own assertions), never from the
document's `creator` arrow.

Detections are strings: `sample:<kind>:unrouted` (the switch has no arm),
`sample:<kind>:unresolved:<route>` (the arm has no body), or
`sample:<kind>:wrong-creator:<actual>` (the arm resolves to another
creator, e.g. the silent default).

`--assert` compares every fixture against the frozen `baseline_expected`
in `faults.json` and exits 1 on any mismatch; `--hash` prints the script's
sha256 (pinned into the ledger's executed-baseline artifact).
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "factory-method"
MATRIX = KATA / "faults.json"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dispatch_detections(doc: dict, sample: dict[str, str]) -> list[str]:
    """The sampled test's detections over the declared dispatch tables."""
    tables = doc["instance"]["tables"]
    route = tables.get("route", {})
    handler = tables.get("handler", {})
    detections = []
    for kind, expected in sample.items():
        arm = route.get(kind)
        if arm is None:
            detections.append(f"sample:{kind}:unrouted")
            continue
        resolved = handler.get(arm)
        if resolved is None:
            detections.append(f"sample:{kind}:unresolved:{arm}")
            continue
        if resolved != expected:
            detections.append(f"sample:{kind}:wrong-creator:{resolved}")
    return detections


def check_case(case: dict, samples: dict[str, dict[str, str]]) -> tuple[dict, list[str]]:
    """Run the sampled test on one fixture and compare with its freeze."""
    doc = json.loads((KATA / case["fixture"]).read_text())
    detections = dispatch_detections(doc, samples[case["sample_key"]])
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
    for group in ("clean", "faults", "floor_faults", "gaps"):
        for case in matrix[group]:
            record, problems = check_case(case, samples)
            report.append({"group": group, **record})
            failures.extend(problems)
    # Construction probes are refused by the model parser, not by a factory
    # test: the conventional arm has nothing to say about them. The frozen
    # matrix still pins that they were pre-registered.
    for case in matrix["construction"]:
        if not case.get("baseline_skip"):
            failures.append(f"{case['id']}: construction probe not marked baseline_skip")
        if not (KATA / case["fixture"]).is_file():
            failures.append(f"{case['id']}: fixture {case['fixture']} is missing")
        report.append(
            {"group": "construction", "id": case["id"], "fixture": case["fixture"], "detections": []}
        )
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