#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the composite-null-object kata
(catherine-lgl.1) — the scripted traversal test a team would already run on
a Composite tree with a Null Object, so `unique_detections` is arithmetic
(probe elements minus baseline-identified elements), never asserted.

Responsibilities: model the conventional review as a pre-registered sample
of node records — for each sampled node assert its `parent` cell resolves
and its `top` agrees with its parent's `top`; assert the distinguished
`nil` row is self-looped on `parent` and `top`. Execute the probe arm
(`catherine check --json` / `catherine migrate --json`) to collect the
kernel's elements, compare both arms with the frozen expectations in
`fixtures/katas/composite-null-object/faults.json`, and fail loudly on any
mismatch. The script never edits the pre-registration.

Rationale: a conventional traversal test proves the records it walks; it
does not enumerate an unsampled subtree, and it does not execute a declared
schema map at all. That is the pre-registered sample boundary: the
near-floor one-level composite has every node in the sample (so the
baseline catches every injected check fault), while the above-floor faults
sit outside it. The kernel's check is exhaustive and its migrate surface
names lost cells, so those elements are the unique set. Sampling is the
point, exactly as in the rule-lens extracted-table baseline.

Usage:
  python3 prototype/composite_null_object_baseline.py            # report JSON
  python3 prototype/composite_null_object_baseline.py --assert   # frozen gate
  python3 prototype/composite_null_object_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "composite-null-object"
FAULTS = KATA / "faults.json"


def fixture(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def row_ids(doc: dict) -> dict[str, int]:
    return {name: i for i, name in enumerate(sorted(doc["instance"]["rows"]["Node"]))}


def cell(doc: dict, arrow: str, row: str) -> str | None:
    return doc["instance"]["tables"].get(arrow, {}).get(row)


def check_node(doc: dict, ids: dict[str, int], name: str) -> tuple[list[str], list[str]]:
    """One sampled node: parent resolves, and top agrees with the parent."""
    parent = cell(doc, "parent", name)
    if parent is None:
        element = f"dangling:parent@Node#{ids[name]}"
        return [element], [element]
    if cell(doc, "top", name) != cell(doc, "top", parent):
        return [f"root-mismatch:Node#{ids[name]}"], [f"violation:eq0@Node#{ids[name]}"]
    return [], []


def check_nil(doc: dict, ids: dict[str, int]) -> tuple[list[str], list[str]]:
    """The distinguished Null Object row: self-looped parent and top."""
    detections: list[str] = []
    identified: list[str] = []
    if cell(doc, "parent", "nil") != "nil":
        detections.append("nil-parent:Node#nil")
        identified.append(f"violation:eq0@Node#{ids['nil']}")
    if cell(doc, "top", "nil") != "nil":
        detections.append("nil-top:Node#nil")
    return detections, identified


def baseline_arm(doc: dict, fault: dict) -> tuple[list[str], list[str]]:
    """The conventional scripted traversal test over the pre-registered sample."""
    ids = row_ids(doc)
    detections, identified = check_nil(doc, ids)
    for name in fault["baseline_sample"]:
        found, named = check_node(doc, ids, name)
        detections.extend(found)
        identified.extend(named)
    return sorted(detections), sorted(set(identified))


def run_cli(args: list[str]) -> dict:
    out = subprocess.run(
        ["cargo", "run", "-q", "-p", "catherine-cli", "--", *args],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if out.returncode not in (0, 3, 4, 5, 6, 7, 8):
        raise SystemExit(f"catherine {' '.join(args)} exited {out.returncode}: {out.stderr}")
    return json.loads(out.stdout)


def check_elements(doc: dict, name: str) -> list[str]:
    envelope = run_cli(["check", str(KATA / name), "--json"])
    arrows = {arrow["name"]: arrow["src"] for arrow in doc["schema"]["arrows"]}
    equations = doc["schema"]["equations"]
    elements = []
    for finding in envelope["data"]["findings"]:
        witness = finding["witness"]
        if finding["kind"] == "dangling":
            owner = arrows[witness["arrow"]]
            elements.append(f"dangling:{witness['arrow']}@{owner}#{witness['src']}")
        elif finding["kind"] == "violation":
            owner = arrows[equations[witness["equation"]]["lhs"][0]]
            elements.append(f"violation:eq{witness['equation']}@{owner}#{witness['row']}")
        else:
            raise SystemExit(f"unknown check finding kind {finding['kind']}")
    return sorted(set(elements))


def migrate_elements(name: str) -> list[str]:
    envelope = run_cli(["migrate", str(KATA / name), "--json"])
    elements = []
    for finding in envelope["data"]["lost"]:
        witness = finding["witness"]
        if finding["kind"] == "lost_cell":
            elements.append(f"lost_cell:{witness['arrow']}#{witness['row']}")
        elif finding["kind"] == "lost_row":
            elements.append(f"lost_row:{witness['object']}#{witness['row']}")
        else:
            raise SystemExit(f"unknown migration finding kind {finding['kind']}")
    return sorted(set(elements))


def probe_elements(doc: dict, fault: dict) -> list[str]:
    if fault["surface"] == "migrate":
        return migrate_elements(fault["fixture"])
    return check_elements(doc, fault["fixture"])


def derive_verdict(unique: list[str]) -> str:
    return "killed" if not unique else "survived"


def check_fault(fault: dict) -> tuple[dict, list[str]]:
    doc = fixture(fault["fixture"])
    detections, identified = baseline_arm(doc, fault)
    actual = probe_elements(doc, fault)
    unique = sorted(set(actual) - set(identified))
    derived = derive_verdict(unique)
    failures = []
    if detections != sorted(fault["baseline_expected"]):
        failures.append(f"{fault['id']}: baseline detections {detections} drift")
    if identified != sorted(fault["baseline_identified"]):
        failures.append(f"{fault['id']}: baseline identified elements {identified} drift")
    if actual != sorted(fault["probe_expected"]):
        failures.append(f"{fault['id']}: probe elements {actual} drift")
    if len(unique) != fault["unique_expected"]:
        failures.append(f"{fault['id']}: {len(unique)} unique, expected {fault['unique_expected']}")
    if derived != fault["verdict_expected"]:
        failures.append(f"{fault['id']}: derived verdict {derived} drift")
    report = {
        "id": fault["id"],
        "baseline_detections": detections,
        "baseline_identified": identified,
        "probe_elements": actual,
        "unique": unique,
        "verdict": derived,
    }
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
        total = sum(len(item["unique"]) for item in report)
        print(f"baseline arm frozen: {len(report)} faults, {total} unique detections", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())