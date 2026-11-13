#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the wrapper-laws kata — the
scripted delegation assertions a team actually writes for an Adapter /
Proxy / Decorator boundary (pre-registered in
`fixtures/katas/wrapper-laws/faults.json` before this script ran).

Responsibilities: model the frozen sample — one assertion per sampled
(wrapper, field, row) cell, comparing the wrapper's observable value to
the value reachable through the wrapper's declared link and the target's
field — and run the probe arm (`catherine check --json`) on the same
fixtures. Unique detections are arithmetic: probe elements minus the
elements the sampled assertions identify, where an element is
`dangling:<arrow>#<row>` or `violation:eq<index>@row<row>` (the kernel's
witness id). A mismatch fails loudly; the script never edits the
pre-registration.

Rationale: the kata's claim is that law preservation is checked on the
declared paths, not on a sampled call set. The conventional arm is
sampled by construction (13 assertions for the above-floor model, 3 for
the floor control) because that is what a delegation test suite is: a
handful of representative calls, not the cross-product of rows and
fields. The sample is frozen here in `faults.json`, not inferred after
the run. The probe arm is executed as a subprocess (the same CLI a user
runs) and parsed mechanically; no finding is copied by hand. The script
also re-checks the two clean fixtures: a law battery that fires on its
own clean model would invalidate the kata.

Usage:
  python3 prototype/wrapper_laws_baseline.py            # report JSON
  python3 prototype/wrapper_laws_baseline.py --assert   # frozen gate (exit 1 on mismatch)
  python3 prototype/wrapper_laws_baseline.py --hash     # sha256 of this script (the pin)
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "wrapper-laws"
FAULTS = KATA / "faults.json"
CHECK_EXITS = (0, 3, 4)


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(fixture: str) -> dict:
    return json.loads((KATA / fixture).read_text())


def dense_rows(doc: dict) -> dict[str, dict[str, int]]:
    """Dense row ids in sorted-name order — the parser's assignment."""
    return {
        obj: {name: index for index, name in enumerate(sorted(names))}
        for obj, names in doc["instance"]["rows"].items()
    }


def cell(doc: dict, arrow: str, row: str) -> str | None:
    return doc["instance"]["tables"].get(arrow, {}).get(row)


def label(value: str | None) -> str:
    return "missing" if value is None else value


def assertion(doc: dict, ids: dict, surface: dict, wrapper: str, field: str, row: int):
    """One sampled delegation assertion: wrapper.field == target.field."""
    names = doc["instance"]["rows"][wrapper]
    name = sorted(names)[row]
    spec = surface[wrapper]["fields"][field]
    linked = cell(doc, surface[wrapper]["link"], name)
    expected = None if linked is None else cell(doc, spec["target"], linked)
    actual = cell(doc, field, name)
    return name, actual, expected, spec


def detection(wrapper: str, field: str, row: int, actual: str | None, expected: str | None):
    if actual == expected:
        return None
    if actual is None:
        return f"delegation:{wrapper}:{field}#{row}:missing"
    return f"delegation:{wrapper}:{field}#{row}:{actual}!={label(expected)}"


def identified(spec: dict, field: str, row: int, actual: str | None) -> str | None:
    if actual is None:
        return f"dangling:{field}#{row}"
    if spec["law"] is None:
        return None
    return f"violation:eq{spec['law']}@row{row}"


def field_arm(doc: dict, ids: dict, surface: dict, wrapper: str, field: str,
              rows: list[int]) -> tuple[list[str], list[str]]:
    """The assertions for one (wrapper, field) cell group."""
    detections: list[str] = []
    elements: list[str] = []
    for row in rows:
        _, actual, expected, spec = assertion(doc, ids, surface, wrapper, field, row)
        found = detection(wrapper, field, row, actual, expected)
        if found is None:
            continue
        detections.append(found)
        element_id = identified(spec, field, row, actual)
        if element_id is not None:
            elements.append(element_id)
    return detections, elements


def baseline_arm(doc: dict, surface: dict, sample: dict) -> tuple[list[str], list[str]]:
    """The sampled assertion suite: detections and identified elements."""
    ids = dense_rows(doc)
    detections: list[str] = []
    elements: list[str] = []
    for wrapper, fields in sample.items():
        for field, rows in fields.items():
            found, named = field_arm(doc, ids, surface, wrapper, field, rows)
            detections.extend(found)
            elements.extend(named)
    return sorted(detections), sorted(elements)


def element(finding: dict) -> str:
    witness = finding["witness"]
    if finding["kind"] == "dangling":
        return f"dangling:{witness['arrow']}#{witness['src']}"
    if finding["kind"] == "violation":
        return f"violation:eq{witness['equation']}@row{witness['row']}"
    raise SystemExit(f"finding kind {finding['kind']} outside the kata's law classes")


def probe_arm(fixture: str) -> list[dict]:
    args = ["cargo", "run", "-q", "-p", "catherine-cli", "--", "check",
            str(KATA / fixture), "--json"]
    out = subprocess.run(args, cwd=ROOT, capture_output=True, text=True, check=False)
    if out.returncode not in CHECK_EXITS:
        raise SystemExit(f"check exited {out.returncode}: {out.stderr.strip()}")
    return json.loads(out.stdout)["data"]["findings"]


def verdict(unique: int) -> str:
    """Mirror harness::derive_probe_verdict: empty unique = killed."""
    return "killed" if unique == 0 else "survived"


def sample_failures(table: str, sample: dict, expected_size: int) -> list[str]:
    total = sum(len(rows) for fields in sample.values() for rows in fields.values())
    if total != expected_size:
        return [f"{table}: {total} sampled assertions, preregistered {expected_size}"]
    return []


def check_clean(record: dict, surface: dict, sample: dict) -> tuple[dict, list[str]]:
    doc = load(record["fixture"])
    detections, elements = baseline_arm(doc, surface, sample)
    probe = [element(finding) for finding in probe_arm(record["fixture"])]
    failures = []
    if detections != record["baseline_expected"]:
        failures.append(f"{record['fixture']}: clean baseline fires {detections}")
    if elements:
        failures.append(f"{record['fixture']}: clean baseline identifies {elements}")
    if probe != record["probe_expected"]:
        failures.append(f"{record['fixture']}: clean probe {probe} != {record['probe_expected']}")
    report = {"fixture": record["fixture"], "baseline_detections": detections, "probe": probe}
    return report, failures


def check_fault(fault: dict, surface: dict, sample: dict) -> tuple[dict, list[str]]:
    """Compare one fault's two arms against its frozen expectations."""
    doc = load(fault["fixture"])
    detections, elements = baseline_arm(doc, surface, sample)
    findings = probe_arm(fault["fixture"])
    probe = sorted(element(finding) for finding in findings)
    unique = sorted(set(probe) - set(elements))
    derived = verdict(len(unique))
    failures = []
    if detections != sorted(fault["baseline_expected"]):
        failures.append(f"{fault['id']}: baseline {detections} != {sorted(fault['baseline_expected'])}")
    if elements != sorted(fault["baseline_identified"]):
        failures.append(f"{fault['id']}: baseline identified {elements} drift")
    if probe != sorted(fault["probe_expected"]):
        failures.append(f"{fault['id']}: probe {probe} != preregistered {sorted(fault['probe_expected'])}")
    if len(unique) != fault["unique_expected"]:
        failures.append(f"{fault['id']}: {len(unique)} unique, preregistered {fault['unique_expected']}")
    if derived != fault["verdict_expected"]:
        failures.append(f"{fault['id']}: verdict {derived}, preregistered {fault['verdict_expected']}")
    record = {"id": fault["id"], "control": fault["control"],
              "baseline_detections": detections, "baseline_identified": len(elements),
              "probe": probe, "unique": unique, "verdict": derived, "findings": findings}
    return record, failures


def run_matrix(matrix: dict) -> tuple[list[dict], list[str]]:
    report, failures = [], []
    for table, surface in matrix["surface"].items():
        failures.extend(sample_failures(table, matrix["sample"][table],
                                        matrix["sample_size"][table]))
        for record in matrix["clean"]:
            if record["table"] == table:
                clean, problems = check_clean(record, surface, matrix["sample"][table])
                report.append(clean)
                failures.extend(problems)
    for fault in matrix["faults"]:
        surface = matrix["surface"][fault["table"]]
        record, problems = check_fault(fault, surface, matrix["sample"][fault["table"]])
        report.append(record)
        failures.extend(problems)
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    report, failures = run_matrix(matrix)
    payload = {"baseline_sha256": sha256_of(Path(__file__)), "kata": matrix["kata"],
               "records": report}
    print(json.dumps(payload, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        total = sum(len(record["unique"]) for record in report if "unique" in record)
        faults = sum(1 for record in report if "unique" in record)
        print(f"baseline arm frozen: {faults} faults, {total} unique detections",
              file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())