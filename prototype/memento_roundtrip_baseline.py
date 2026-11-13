#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the memento-roundtrip kata
(catherine-lgl.5) — what a field-by-field save/restore review catches.

The legacy Memento pipeline is modelled in the probe fixtures as a pair
of delta schema maps (save: Document -> Snapshot, restore: Snapshot ->
Document). The conventional team does not execute a delta: it walks a
checklist over sampled (field, row) cells of the saved instance and
asserts that the restore map does not assign the field to null. A loss
at an unsampled field, or at a sampled field on an unsampled row, is
invisible; row counts and object collapses are not modelled at all.

This artifact is committed and version-pinned (sha256) BEFORE the probe
run. `--assert` compares its detections and identified elements against
the pre-registered matrix in fixtures/katas/memento-roundtrip/faults.json
and recomputes unique = probe_expected minus identified; a mismatch
exits 1. `--hash` prints the sha256 of this file.
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "memento-roundtrip"
FAULTS = KATA / "faults.json"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def fixture(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def sampled_losses(doc: dict, sample: list[dict]) -> tuple[list[str], list[str]]:
    """The checklist: only a sampled valued cell exposes a null field."""
    declared = {arrow["name"]: arrow for arrow in doc["schema"]["arrows"]}
    rows_of = doc["instance"]["rows"]
    tables = doc["instance"]["tables"]
    map_arrows = doc["schema_map"]["arrows"]
    detections: list[str] = []
    identified: list[str] = []
    for entry in sample:
        arrow = entry["arrow"]
        row_names = rows_of[declared[arrow]["src"]]
        table = tables.get(arrow, {})
        for index in entry["rows"]:
            if index >= len(row_names) or table.get(row_names[index]) is None:
                continue
            if map_arrows[arrow] is not None:
                continue
            detections.append(f"field-drop:{arrow}#{index}")
            identified.append(f"lost_cell:{arrow}#{index}")
    return sorted(set(detections)), sorted(set(identified))


def identified_elements(fault: dict) -> set[str]:
    return {
        f"{prefix}#{row}"
        for prefix, rows in fault["baseline_identified"].items()
        for row in rows
    }


def expected_elements(fault: dict) -> set[str]:
    """The pre-registered probe elements (a pre-registration check only:
    no probe arm is executed here)."""
    expected = fault["probe_expected"]
    elements: set[str] = set()
    for obj, rows in expected["lost_rows"].items():
        elements.update(f"lost_row:{obj}#{row}" for row in rows)
    for arrow, rows in expected["lost_cells"].items():
        elements.update(f"lost_cell:{arrow}#{row}" for row in rows)
    elements.update(expected["round_trip"])
    return elements


def derive_verdict(unique: int) -> str:
    """Mirror harness::derive_probe_verdict with an executed arm."""
    return "killed" if unique == 0 else "survived"


def check_fault(fault: dict, sample: list[dict]) -> tuple[dict, list[str]]:
    """Compare one fixture's baseline arm and pre-registration arithmetic."""
    detections, identified = sampled_losses(fixture(fault["fixture"]), sample)
    identified_set = identified_elements(fault)
    unique = sorted(expected_elements(fault) - identified_set)
    verdict = derive_verdict(len(unique))
    failures: list[str] = []
    if detections != sorted(fault["baseline_expected"]):
        failures.append(f"{fault['id']}: baseline detections {detections} != "
                        f"{sorted(fault['baseline_expected'])}")
    if identified != sorted(identified_set):
        failures.append(f"{fault['id']}: identified elements {identified} != "
                        f"{sorted(identified_set)}")
    if unique != sorted(fault["unique_expected"]):
        failures.append(f"{fault['id']}: unique {unique} != "
                        f"{sorted(fault['unique_expected'])}")
    if verdict != fault["verdict_expected"]:
        failures.append(f"{fault['id']}: verdict {verdict} != "
                        f"{fault['verdict_expected']}")
    report = {
        "id": fault["id"],
        "control": fault["control"],
        "baseline_detections": detections,
        "identified": len(identified_set),
        "probe_elements": len(expected_elements(fault)),
        "unique": unique,
        "verdict": verdict,
    }
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    reports, failures = [], []
    for fault in matrix["faults"]:
        report, problems = check_fault(fault, matrix["sample"])
        reports.append(report)
        failures.extend(problems)
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)),
                      "faults": reports}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        counts = [len(report["unique"]) for report in reports]
        print(f"baseline arm frozen: {len(reports)} faults, unique {counts}",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())