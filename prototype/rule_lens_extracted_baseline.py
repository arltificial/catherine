#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of probe 2 — rule lens on an
EXTRACTED table (`add-domain-probe-validation` task 9, beads
`catherine-1od`).

Responsibilities: model the review a team actually performs on a table
extracted from a policy PDF — a frozen, sampled value check (one witness
per sampled (scope, date) cell against the printed cap, resolving the
sample under the document's declared policy) — and compare it with the
rule lens's executed findings on the same injected fault set. Faults the
lens catches and the sampled review misses are the unique detections;
the arithmetic is per fault, never asserted.

Rationale: the manual review protocol is frozen into scripted assertions
BEFORE the run (the rule-lens pattern): four sampled cells for the
above-floor extracted table, one for the near-floor single-row table.
Sampling is the point — a conventional reviewer spot-checks values, not
the whole rule space, so a dropped unsampled row, a dead duplicate and a
widened rule are invisible to it. The lens's findings are the real CLI's,
parsed verbatim.

Usage:
  python3 prototype/rule_lens_extracted_baseline.py            # report JSON
  python3 prototype/rule_lens_extracted_baseline.py --assert   # frozen gate
  python3 prototype/rule_lens_extracted_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ROOT / "fixtures" / "probes" / "rule_lens"
FAULTS = PROBE / "faults.json"

# The frozen sample: (scope, date) cells checked against the printed cap.
# GENERAL's three rows plus one ZD row — the rest of the ZD column is not
# reviewed, exactly as a spot check would leave it.
SAMPLE = {
    ("general", "2022-10-01"): "tope_49192134",
    ("general", "2022-12-01"): "tope_53207958",
    ("general", "2023-01-01"): "tope_58528749",
    ("zona_desfavorable", "2022-10-01"): "tope_59030562",
}
FLOOR_SAMPLE = {("2024-10-01",): "tope_641748216"}


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def matches(predicate: dict, cell: dict) -> bool:
    """Evaluate the tiny predicate language on one domain cell."""
    kind = predicate["kind"]
    if kind == "cmp":
        left, op, right = cell[predicate["column"]], predicate["op"], predicate["value"]
        return {
            "eq": left == right, "ne": left != right,
            "lt": left < right, "le": left <= right,
            "gt": left > right, "ge": left >= right,
        }[op]
    if kind == "and":
        return all(matches(arg, cell) for arg in predicate["args"])
    if kind == "or":
        return any(matches(arg, cell) for arg in predicate["args"])
    if kind == "not":
        return not matches(predicate["arg"], cell)
    raise SystemExit(f"predicate kind {kind} outside the frozen review's language")


def resolve(rules: list[dict], policy: str | None, cell: dict) -> list[str]:
    """The actions a cell resolves to under the declared policy."""
    hits = [rule["action"] for rule in rules if matches(rule["predicate"], cell)]
    if policy == "PriorityOrder" or policy == "FirstMatch":
        return hits[:1]
    return hits[:1] if len(hits) == 1 else hits


def sample_detections(doc: dict, sample: dict) -> list[str]:
    """The sampled review's detections: missing, ambiguous, or wrong cap."""
    presentation = doc["rule_presentation"]
    policy = presentation.get("policy")
    detections = []
    for names, expected in sample.items():
        cell = dict(zip(presentation_columns(presentation), names))
        resolved = resolve(presentation["rules"], policy, cell)
        label = ":".join(names)
        if not resolved:
            detections.append(f"sample:{label}:missing")
        elif len(resolved) > 1:
            detections.append(f"sample:{label}:ambiguous")
        elif resolved[0] != expected:
            detections.append(f"sample:{label}:wrong-cap:{resolved[0]}")
    return sorted(detections)


def presentation_columns(presentation: dict) -> list[str]:
    return [column["name"] for column in presentation["columns"]]


def lens_findings(fixture: str, domain: dict) -> list[dict]:
    args = ["cargo", "run", "-q", "-p", "catherine-cli", "--", "rules",
            str(PROBE / fixture), "--max-candidates", "64", "--json"]
    for column, values in domain.items():
        args.extend(["--values", f"{column}={','.join(values)}"])
    out = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, check=False)
    if out.returncode not in (0, 3, 6, 7):
        raise SystemExit(f"rules exited {out.returncode}: {out.stderr.strip()}")
    return json.loads(out.stdout)["data"].get("findings", [])


def element(finding: dict) -> str:
    return json.dumps(finding, sort_keys=True, separators=(",", ":"))


def check_fault(fault: dict, samples: dict, domains: dict) -> tuple[dict, list[str]]:
    """Run both arms on one fault and compare them with its expectations."""
    doc = json.loads((PROBE / fault["fixture"]).read_text())
    baseline = sample_detections(doc, samples[fault["table"]])
    caught = bool(baseline)
    failures = []
    if caught != fault["baseline_catches"]:
        failures.append(f"{fault['id']}: baseline caught={caught}, preregistered "
                        f"{fault['baseline_catches']}")
    findings = lens_findings(fault["fixture"], domains[fault["table"]])
    record = {"id": fault["id"], "table": fault["table"],
              "catalog_fault": fault["catalog_fault"],
              "baseline_detections": baseline, "baseline_catches": caught,
              "lens_elements": [element(f) for f in findings],
              "unique_eligible": not caught}
    return record, failures


def unique_count_failures(report: list[dict], expected: dict) -> list[str]:
    """The unique arithmetic per table against the recorded count."""
    failures = []
    for table, expected_count in expected.items():
        got = sum(len(f["lens_elements"]) for f in report
                  if f["unique_eligible"] and f["table"] == table)
        if got != expected_count:
            failures.append(f"{table}: {got} unique detections, recorded {expected_count}")
    return failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    samples = {"above_floor": SAMPLE, "floor": FLOOR_SAMPLE}
    domains = {"above_floor": matrix["domain"], "floor": matrix["floor_domain"]}
    report, failures = [], []
    for fault in matrix["faults"]:
        record, problems = check_fault(fault, samples, domains)
        report.append(record)
        failures.extend(problems)
    failures.extend(unique_count_failures(report, matrix.get("unique_expected", {})))
    unique = [e for f in report if f["unique_eligible"] for e in f["lens_elements"]]
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)),
                      "faults": report, "unique": len(unique)}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        print(f"baseline arm frozen: {len(report)} faults, {len(unique)} unique detections",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())