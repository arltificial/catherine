#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of probe 1 (migration Δ at
pilot scale, `add-domain-probe-validation` task 6, beads
`catherine-1od`) — what the ordinary tooling a team already runs finds
on the committed fault set, so `unique_detections` is arithmetic
(probe elements minus baseline-identified elements), never asserted.

Responsibilities: statement-level migration linting (Atlas/Flyway class:
DROP TABLE, DROP COLUMN, table merge, non-transportable path) plus a
row-count/checksum dry run over the declared schema map; execution of
the probe arm (`catherine migrate --json` and the round-trip fixture
test) to collect its elements; comparison against the frozen,
pre-registered expectations in `fixtures/probes/migration/faults.json`
— `baseline_expected`, `baseline_identified`, `probe_expected`,
`unique_expected`, `verdict_expected`. A mismatch fails loudly; the
script never edits the pre-registration.

Rationale: the probe's claim is measured against tooling a team would
actually run. The dry run reads only declared structure and row sets,
never column payloads: dropped columns are named by lint (column, not
row), count deltas name tables and counts, and no conventional check
expresses Δ∘Δ′ at all — exactly the identification granularity the
preregistration fixes. The probe arm is executed as a subprocess (the
same CLI a user runs) and parsed mechanically; no finding is copied by
hand.

Usage:
  python3 prototype/migration_baseline.py            # report JSON
  python3 prototype/migration_baseline.py --assert   # frozen gate (exit 1 on mismatch)
  python3 prototype/migration_baseline.py --hash     # sha256 of this script (the pin)
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ROOT / "fixtures" / "probes" / "migration"
FAULTS = PROBE / "faults.json"


def fixture(name: str) -> dict:
    return json.loads((PROBE / name).read_text())


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def dropped_objects(doc: dict) -> tuple[list[str], dict[str, list[int]]]:
    """Dropped tables: lint names the table, the count names every row."""
    schema, rows, mapping = doc["schema"], doc["instance"]["rows"], doc["schema_map"]
    detections: list[str] = []
    identified: dict[str, list[int]] = {}
    for obj in schema["objects"]:
        if mapping["objects"].get(obj) is None:
            detections.append(f"lint:drop-table:{obj}")
            detections.append(f"dry-run:row-count-delta:{obj}:{len(rows[obj])}->0")
            identified[f"lost_row:{obj}"] = list(range(len(rows[obj])))
    return detections, identified


def arrow_lint(doc: dict) -> list[str]:
    """Statement lint for dropped and non-transportable arrows."""
    detections = []
    for arrow in doc["schema"]["arrows"]:
        path = doc["schema_map"]["arrows"].get(arrow["name"])
        if path is None:
            detections.append(f"lint:drop-column:{arrow['name']}")
        elif len(path) > 1:
            detections.append(f"lint:non-transportable-path:{arrow['name']}")
    return detections


def count_deltas(doc: dict) -> list[str]:
    """The dry run: row-count deltas and merges per kept target object."""
    rows = doc["instance"]["rows"]
    preimages: dict[str, list[str]] = {}
    for src, tgt in doc["schema_map"]["objects"].items():
        if tgt is not None:
            preimages.setdefault(tgt, []).append(src)
    detections = []
    for tgt, group in sorted(preimages.items()):
        group.sort()
        base = len(rows[group[0]])
        source_total = sum(len(rows[src]) for src in group)
        if len(group) > 1:
            detections.append(f"lint:table-merge:{tgt}<-{','.join(group)}")
        if source_total != base:
            detections.append(f"dry-run:row-count-delta:{tgt}:{source_total}->{base}")
    return detections


def lint_and_dry_run(doc: dict) -> tuple[list[str], dict[str, list[int]]]:
    """The baseline: statement lint + row-count/checksum dry run.

    Returns the detections (sorted) and the elements the detections
    identify. Only a dropped table identifies elements: the table is
    absent, so its lost row set is exactly its row set. Dropped
    columns, count deltas, and merges name no elements.
    """
    detections, identified = dropped_objects(doc)
    detections += arrow_lint(doc) + count_deltas(doc)
    return sorted(detections), identified


def run_migrate(fixture_name: str) -> list[dict]:
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "catherine-cli", "--", "migrate",
         str(PROBE / fixture_name), "--json"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    if out.returncode not in (0, 8):
        raise SystemExit(f"migrate exited {out.returncode}: {out.stderr.strip()}")
    return json.loads(out.stdout)["data"]["lost"]


def run_round_trip() -> dict:
    out = subprocess.run(
        ["cargo", "test", "-q", "-p", "catherine-ledger", "--test",
         "probe_migration_fixtures", "collapsing_map", "--", "--nocapture"],
        cwd=ROOT, capture_output=True, text=True, check=False,
    )
    if out.returncode != 0:
        raise SystemExit(f"round-trip arm failed:\n{out.stderr}")
    match = re.search(r"^PROBE_ROUND_TRIP (\{.*\})$", out.stdout, re.MULTILINE)
    if match is None:
        raise SystemExit("round-trip arm printed no machine-readable witness")
    return json.loads(match.group(1))


def probe_elements(fault: dict) -> set[str]:
    elements: set[str] = set()
    if fault["probe_expected"]["round_trip"]:
        proof = run_round_trip()
        elements.add(proof["element"])
        return elements
    for finding in run_migrate(fault["fixture"]):
        witness = finding["witness"]
        if finding["kind"] == "lost_row":
            elements.add(f"lost_row:{witness['object']}#{witness['row']}")
        elif finding["kind"] == "lost_cell":
            elements.add(f"lost_cell:{witness['arrow']}#{witness['row']}")
        else:
            raise SystemExit(f"unknown migration finding kind {finding['kind']}")
    return elements


def expected_elements(fault: dict) -> set[str]:
    expected = fault["probe_expected"]
    elements = set()
    for obj, dropped in expected["lost_rows"].items():
        elements.update(f"lost_row:{obj}#{row}" for row in dropped)
    for arrow, dropped in expected["lost_cells"].items():
        elements.update(f"lost_cell:{arrow}#{row}" for row in dropped)
    elements.update(f"round_trip:{item.split(':', 1)[1]}" for item in expected["round_trip"])
    return elements


def verdict(unique: int, executed: bool) -> str:
    """Mirror harness::derive_probe_verdict: empty unique = killed
    (parity); unique with an executed arm = survived; cited-only caps at
    inconclusive."""
    if unique == 0:
        return "killed"
    return "survived" if executed else "inconclusive"


def check_fault(fault: dict) -> tuple[dict, list[str]]:
    """Compare one fault's two arms against its frozen expectations."""
    doc = fixture(fault["fixture"])
    detections, identified = lint_and_dry_run(doc)
    identified_elements = {
        f"{key}#{row}" for key, rows in identified.items() for row in rows
    }
    actual, expected = probe_elements(fault), expected_elements(fault)
    unique = sorted(actual - identified_elements)
    derived = verdict(len(unique), executed=True)
    failures = []
    if detections != sorted(fault["baseline_expected"]):
        failures.append(f"{fault['id']}: baseline detections {detections} != "
                        f"{sorted(fault['baseline_expected'])}")
    if identified != {k: sorted(v) for k, v in fault["baseline_identified"].items()}:
        failures.append(f"{fault['id']}: baseline identified elements drift")
    if actual != expected:
        failures.append(f"{fault['id']}: probe elements {sorted(actual)} != preregistered "
                        f"{sorted(expected)}")
    if len(unique) != fault["unique_expected"]:
        failures.append(f"{fault['id']}: {len(unique)} unique detections, preregistered "
                        f"{fault['unique_expected']}")
    if derived != fault["verdict_expected"]:
        failures.append(f"{fault['id']}: derived verdict {derived}, preregistered "
                        f"{fault['verdict_expected']}")
    report = {"id": fault["id"], "baseline_detections": detections,
              "baseline_identified": len(identified_elements),
              "probe_elements": len(actual), "unique": unique, "verdict": derived}
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    report, failures = [], []
    for fault in matrix["faults"]:
        record, problems = check_fault(fault)
        report.append(record)
        failures.extend(problems)
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)), "faults": report}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        total = sum(len(f["unique"]) for f in report)
        print(f"baseline arm frozen: {len(report)} faults, {total} unique detections", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())