#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the Builder-as-monoid kata
(`catherine-lgl.2`) — the review a team actually runs on a legacy
builder: a scripted accumulation diff over sampled steps.

Responsibilities: replay the committed accumulation trace, assert for
each sampled step index that the step row exists and is the recorded
step, and that its `emits` cell is the recorded part; compare the
baseline's detections and the `catherine` probe arm's findings against
the frozen expectations in `fixtures/katas/builder-monoid/faults.json`.
The baseline never inspects a `schema_map` (a migration edit is not a
runtime accumulation event) and never runs the redundancy analysis (a
conventional test does not prove declarations redundant). An unmapped
detection or finding is a mismatch, never silently ignored.

Rationale: the kata's `unique_detections` is arithmetic — probe
elements minus baseline-identified elements — so both arms must run
over the same committed fixtures and the same canonical element map.
The probe arm is executed as a subprocess through the shipped CLI
(`check` / `migrate` / `redundant`), and no finding is copied by hand.
The script is committed and sha256-pinned before the probe run; a
correction is a superseding ledger entry, never an edit.

Usage:
  python3 prototype/builder_monoid_baseline.py            # report JSON
  python3 prototype/builder_monoid_baseline.py --assert   # frozen gate (exit 1 on mismatch)
  python3 prototype/builder_monoid_baseline.py --hash     # sha256 of this script (the pin)
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "builder-monoid"
FAULTS = KATA / "faults.json"
EXIT_CODES = range(0, 9)


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load_fault(fault: dict) -> dict:
    return json.loads((KATA / fault["fixture"]).read_text())


def run_cli(command: str, path: Path) -> dict:
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "catherine-cli", "--", command,
         str(path), "--json"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    if out.returncode not in EXIT_CODES:
        raise SystemExit(f"{command} exited {out.returncode}: {out.stderr.strip()}")
    return json.loads(out.stdout)["data"]


def witness(finding: dict) -> str:
    """One finding's witness, spelled in the pre-registered vocabulary."""
    kind, cells = finding["kind"], finding["witness"]
    if kind == "violation":
        return f"violation:eq{cells['equation']}@row{cells['row']}"
    if kind == "dangling":
        return f"dangling:{cells['arrow']}@row{cells['src']}"
    if kind == "redundant":
        return "redundant:" + ";".join(cells["derivable"])
    if kind == "lost_row":
        return f"lost_row:{cells['object']}#{cells['row']}"
    if kind == "lost_cell":
        return f"lost_cell:{cells['arrow']}#{cells['row']}"
    raise SystemExit(f"finding kind {kind} outside the pre-registered vocabulary")


def probe_raw(fault: dict) -> list[str]:
    """Execute the pre-registered surfaces; the findings, verbatim."""
    path = KATA / fault["fixture"]
    found: list[str] = []
    for surface in fault["surfaces"]:
        if surface == "check":
            found += [witness(f) for f in run_cli("check", path)["findings"]]
        elif surface == "redundant":
            found += [witness(f) for f in run_cli("redundant", path)["findings"]]
        elif surface == "migrate":
            found += [witness(f) for f in run_cli("migrate", path)["lost"]]
        else:
            raise SystemExit(f"surface {surface} outside the pre-registered set")
    return sorted(set(found))


def sample_detections(fault: dict, trace: list[dict]) -> list[str]:
    """The scripted accumulation diff: sampled step rows and emissions."""
    doc = load_fault(fault)
    rows = doc["instance"]["rows"].get("Step", [])
    emits = doc["instance"].get("tables", {}).get("emits", {})
    detections: list[str] = []
    for index in fault["sample"]:
        if index >= len(rows):
            detections.append(f"sample:step#{index}:missing")
        elif index >= len(trace) or rows[index] != trace[index]["step"]:
            detections.append(f"sample:step#{index}:unexpected={rows[index]}")
        elif emits.get(rows[index]) != trace[index]["emits"]:
            expected = trace[index]["emits"]
            detections.append(f"sample:step#{index}:emits={emits.get(rows[index])}:"
                              f"expected={expected}")
    return detections


def map_elements(raw: list[str], mapping: dict[str, str]) -> set[str]:
    """Map every witness through the frozen map; an unmapped one fails."""
    elements: set[str] = set()
    for item in raw:
        if item not in mapping:
            raise SystemExit(f"detection {item} not in the pre-registered element map")
        elements.add(mapping[item])
    return elements


def verdict(unique: list[str]) -> str:
    """Mirror harness::derive_probe_verdict with an executed baseline."""
    return "killed" if not unique else "survived"


def compare_arms(fault: dict, trace: list[dict]) -> tuple[dict, list[str]]:
    detections = sample_detections(fault, trace)
    baseline = map_elements(detections, fault["baseline_element_map"])
    raw = probe_raw(fault)
    probe = map_elements(raw, fault["probe_element_map"])
    unique = sorted(probe - baseline)
    derived = verdict(unique)
    failures: list[str] = []
    if detections != sorted(fault["baseline_expected"]):
        failures.append(f"{fault['id']}: baseline {detections} != "
                        f"{sorted(fault['baseline_expected'])}")
    if baseline != map_elements(fault["baseline_expected"],
                                fault["baseline_element_map"]):
        failures.append(f"{fault['id']}: baseline elements drift")
    if raw != sorted(fault["probe_expected"]):
        failures.append(f"{fault['id']}: probe {raw} != "
                        f"{sorted(fault['probe_expected'])}")
    if len(unique) != fault["unique_expected"]:
        failures.append(f"{fault['id']}: {len(unique)} unique, preregistered "
                        f"{fault['unique_expected']}")
    if derived != fault["verdict_expected"]:
        failures.append(f"{fault['id']}: verdict {derived}, preregistered "
                        f"{fault['verdict_expected']}")
    report = {"id": fault["id"], "baseline_detections": detections,
              "baseline_elements": sorted(baseline),
              "probe_elements": sorted(probe), "unique": unique,
              "verdict": derived}
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    report, failures = [], []
    for fault in matrix["faults"]:
        trace = fault.get("trace", matrix["trace"])
        record, problems = compare_arms(fault, trace)
        report.append(record)
        failures.extend(problems)
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)),
                      "faults": report}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        total = sum(len(record["unique"]) for record in report)
        print(f"baseline arm frozen: {len(report)} faults, {total} unique "
              f"detections", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())