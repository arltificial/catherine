#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the CoR+Strategy kata
`cor-strategy-rules` (beads ticket `catherine-lgl.7`) — the dispatch
test a team actually scripts over a legacy chain-of-responsibility +
strategy dispatcher, frozen before the probe arm runs so
`unique_detections` is arithmetic (lens findings minus what the scripted
sample caught), never asserted.

Responsibilities: parse each committed fixture's `rule_presentation`,
resolve the pre-registered sample of request cases exactly as the legacy
code resolves them (first match under a priority policy; every match,
hence ambiguity, under a declared-disjoint or absent policy), and report
a detection per sample case that is unhandled, ambiguous, or resolves to
a different action than the recorded expectation. Then execute the probe
arm (`catherine rules --json`) on the same fixtures and compare both
arms with the frozen expectations in `fixtures/katas/cor-strategy-rules/
faults.json`. A mismatch fails loudly; this script never edits the
pre-registration.

Rationale: the claim under test is that a rule table over a legacy
dispatcher finds *structural* defects a behavior sample cannot see —
dead handlers, derivable branches, guard gaps outside the sampled
requests. The sample is a real dispatch suite: one representative
request per handler path, plus (at the near-floor control) the whole
input space, which is why the control must self-negate. The sample is
deliberately not exhaustive above the floor, exactly as a real suite is
not: an unsampled gap, a duplicate handler and a derivable branch are
invisible to it. Nothing here is a detection-rate claim — every
behavior-changing fault in the matrix is caught by the sample.

Usage:
  python3 prototype/cor_strategy_rules_baseline.py            # report JSON
  python3 prototype/cor_strategy_rules_baseline.py --assert   # frozen gate
  python3 prototype/cor_strategy_rules_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "cor-strategy-rules"
MATRIX = KATA / "faults.json"
EXIT_CODES = (0, 3, 6, 7)


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(relative: str) -> dict:
    return json.loads((ROOT / relative).read_text())


def compare(left: object, op: str, right: object) -> bool:
    return {
        "eq": left == right,
        "ne": left != right,
        "lt": left < right,
        "le": left <= right,
        "gt": left > right,
        "ge": left >= right,
    }[op]


def matches(predicate: dict, case: dict) -> bool:
    """Evaluate the fixture predicate language on one request case."""
    kind = predicate["kind"]
    if kind == "cmp":
        return compare(case[predicate["column"]], predicate["op"], predicate["value"])
    if kind == "and":
        return all(matches(arg, case) for arg in predicate["args"])
    if kind == "or":
        return any(matches(arg, case) for arg in predicate["args"])
    if kind == "not":
        return not matches(predicate["arg"], case)
    if kind == "mod":
        return case[predicate["column"]] % predicate["modulus"] == predicate["equals"]
    raise SystemExit(f"predicate kind {kind} outside the dispatch sample's language")


def resolve(rules: list[dict], policy: str | None, case: dict) -> tuple[list[str], str]:
    """The actions a case resolves to, and the anomaly the policy leaves."""
    actions = [rule["action"] for rule in rules if matches(rule["predicate"], case)]
    if not actions:
        return actions, "missing"
    if policy in ("PriorityOrder", "FirstMatch"):
        return actions[:1], ""
    return actions, "ambiguous" if len(actions) > 1 else ""


def render_case(case: dict) -> str:
    return ",".join(f"{name}={value}" for name, value in case.items())


def check_case(rules: list[dict], policy: str | None, entry: dict) -> str:
    actions, anomaly = resolve(rules, policy, entry["case"])
    label = render_case(entry["case"])
    if anomaly:
        return f"sample:{label}:{anomaly}"
    if actions[0] != entry["action"]:
        return f"sample:{label}:wrong-action:{actions[0]}"
    return ""


def sample_detections(doc: dict, sample: list[dict]) -> list[str]:
    """The scripted dispatch suite's detections on one fixture."""
    presentation = doc["rule_presentation"]
    rules, policy = presentation["rules"], presentation.get("policy")
    return sorted(filter(None, (check_case(rules, policy, e) for e in sample)))


def lens(fixture: str, values: dict) -> list[dict]:
    args = ["cargo", "run", "-q", "-p", "catherine-cli", "--", "rules",
            str(ROOT / fixture), "--max-candidates", "64", "--json"]
    for column, items in values.items():
        args.extend(["--values", f"{column}={','.join(items)}"])
    out = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, check=False)
    if out.returncode not in EXIT_CODES:
        raise SystemExit(f"rules exited {out.returncode}: {out.stderr.strip()}")
    return json.loads(out.stdout)["data"].get("findings", [])


def element(finding: dict) -> str:
    return json.dumps(finding, sort_keys=True, separators=(",", ":"))


def kind_counts(findings: list[dict]) -> dict[str, int]:
    counts: dict[str, int] = {}
    for finding in findings:
        key = f"{finding['kind']}:{finding['label']}"
        counts[key] = counts.get(key, 0) + 1
    return counts


def frozen_counts(pairs: list[list]) -> dict[str, int]:
    return {f"{kind}:{label}": count for kind, label, count in pairs}


def clean_failures(label: str, table: dict) -> list[str]:
    """The clean table is clean for both arms; a drift is a fixture bug."""
    failures = []
    findings = lens(table["clean"], table["values"])
    if findings != table["clean_findings"]:
        failures.append(f"{label}: clean table findings {findings} != preregistered "
                        f"{table['clean_findings']}")
    detections = sample_detections(load(table["clean"]), table["sample"])
    if detections:
        failures.append(f"{label}: clean table broke the scripted sample: {detections}")
    return failures


def check_fault(fault: dict, table: dict) -> tuple[dict, list[str]]:
    """Run both arms on one fault and compare them with its expectations."""
    detections = sample_detections(load(fault["fixture"]), table["sample"])
    caught = bool(detections)
    findings = lens(fault["fixture"], table["values"])
    unique = [] if caught else [element(finding) for finding in findings]
    failures = []
    if detections != sorted(fault["baseline_detections"]):
        failures.append(f"{fault['id']}: baseline detections {detections} != preregistered "
                        f"{sorted(fault['baseline_detections'])}")
    if caught != fault["baseline_catches"]:
        failures.append(f"{fault['id']}: baseline caught={caught}, preregistered "
                        f"{fault['baseline_catches']}")
    if kind_counts(findings) != frozen_counts(fault["lens_findings"]):
        failures.append(f"{fault['id']}: lens findings {kind_counts(findings)} != preregistered "
                        f"{frozen_counts(fault['lens_findings'])}")
    if len(unique) != fault["unique_expected"]:
        failures.append(f"{fault['id']}: {len(unique)} unique detections, preregistered "
                        f"{fault['unique_expected']}")
    record = {"id": fault["id"], "class": fault["class"],
              "baseline_detections": detections, "baseline_catches": caught,
              "lens_elements": [element(f) for f in findings], "unique": unique}
    return record, failures


def run_table(label: str, table: dict, report: list[dict]) -> list[str]:
    """One table's clean check plus every fault, and its unique arithmetic."""
    failures = clean_failures(label, table)
    for fault in table["faults"]:
        record, problems = check_fault(fault, table)
        record["table"] = label
        report.append(record)
        failures.extend(problems)
    total = sum(len(record["unique"]) for record in report if record["table"] == label)
    if total != table["unique_expected"]:
        failures.append(f"{label}: {total} unique detections, preregistered "
                        f"{table['unique_expected']}")
    return failures


def verdict(unique: int) -> str:
    """Mirror harness::derive_probe_verdict: an empty unique set is parity
    (`killed`); a non-empty one with an executed baseline arm survives."""
    return "killed" if unique == 0 else "survived"


def summarize(report: list[dict], matrix: dict) -> tuple[dict, dict, list[str]]:
    """Per-table unique arithmetic and the derived verdicts."""
    unique = {
        table: sum(len(record["unique"]) for record in report if record["table"] == table)
        for table in ("above_floor", "control")
    }
    verdicts = {table: verdict(count) for table, count in unique.items()}
    failures = []
    if verdicts["control"] != matrix["control"]["verdict_expected"]:
        failures.append(f"control: verdict {verdicts['control']}, preregistered "
                        f"{matrix['control']['verdict_expected']}")
    return unique, verdicts, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(MATRIX.read_text())
    report: list[dict] = []
    failures = run_table("above_floor", matrix["above_floor"], report)
    failures += run_table("control", matrix["control"], report)
    unique, verdicts, summary = summarize(report, matrix)
    failures += summary
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)),
                      "faults": report, "verdicts": verdicts, "unique": unique}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        print(f"baseline arm frozen: {len(report)} faults, {unique['control']} unique "
              f"detections at the control, {unique['above_floor']} above the floor",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())