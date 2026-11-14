#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the iterator kata — a scripted
smoke walk of the declared traversal over a pre-registered position sample,
so the probe arm's unique detections are arithmetic (executed probe elements
minus baseline-identified elements), never asserted.

Responsibilities: load each committed fixture, derive the declared element
sequence and the emitted sequence from the fixture's own tables (asserting
both against the pre-registered values in `faults.json`, so fixture drift is
a failure, not a silent re-baseline), run the frozen position sample through
conventional assertions, execute the probe arm (`catherine check --json`),
map every finding to a presentation-level element (`element:<row name>`),
and compare everything against the frozen expectations. `--assert` exits 1
on any drift; `--hash` prints this script's sha256 (pinned in the ledger's
`comparison.baseline.artifact`).

Rationale: a conventional team tests the traversal it wrote a test for. The
sample is pre-registered per fixture, so a fault outside it is a fair miss
rather than a strawman; the control's sample is the complete window, and
the `duplicate-covered` run repeats the duplicate fault with a complete
sample to record the parity ceiling explicitly. The probe arm is executed as
a subprocess (the user's CLI) and parsed mechanically; no finding is copied
by hand.

Usage:
  python3 prototype/iterator_baseline.py            # report JSON
  python3 prototype/iterator_baseline.py --assert   # frozen gate
  python3 prototype/iterator_baseline.py --hash     # sha256 of this file
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "iterator"
FAULTS = KATA / "faults.json"
CLI = ["cargo", "run", "-q", "-p", "catherine-cli", "--"]

SENTINEL_ELEMENT = "e9_end"
SENTINEL_STEP = "s9_end"


def load(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def row_names(doc: dict, obj: str) -> list[str]:
    """Every row of an object in dense-id order (the parser's sorted names)."""
    return sorted(doc["instance"]["rows"][obj])


def row_name(doc: dict, obj: str, index: int) -> str:
    return row_names(doc, obj)[index]


def element_sequence(doc: dict) -> list[str]:
    """The declared sequence: Element rows in dense-id order, sentinel excluded."""
    return [name for name in row_names(doc, "Element") if name != SENTINEL_ELEMENT]


def step_sequence(doc: dict) -> list[str]:
    """The traversal's step rows in dense-id (emission) order, sentinel excluded."""
    return [name for name in row_names(doc, "Step") if name != SENTINEL_STEP]


def emitted_sequence(doc: dict) -> list[str | None]:
    """The emitted element per step, read from the fixture's own `yields` table."""
    yields = doc["instance"]["tables"]["yields"]
    return [yields.get(step) for step in step_sequence(doc)]


def element_id(name: str | None) -> str | None:
    """A presentation-level element id; sentinel and null rows are outside it."""
    if name is None or name == SENTINEL_ELEMENT:
        return None
    return f"element:{name}"


def baseline_arm(doc: dict, sample: list[int]) -> tuple[list[str], set[str]]:
    """The scripted smoke walk: one equality assertion per sampled position.

    A failed assertion names both sides, exactly as the failure message would.
    The walk asserts nothing about total length, termination, or the tail.
    """
    declared = element_sequence(doc)
    emitted = emitted_sequence(doc)
    detections: list[str] = []
    identified: set[str] = set()
    for index in sample:
        if index >= len(emitted):
            detection = f"sample:pos{index}:no-emission"
            if index < len(declared):
                identified.add(f"element:{declared[index]}")
        elif index >= len(declared):
            detection = f"sample:pos{index}:unexpected-emission"
            if emitted[index] is not None:
                identified.add(f"element:{emitted[index]}")
        elif emitted[index] is None:
            detection = f"sample:pos{index}:emission-missing"
            identified.add(f"element:{declared[index]}")
        elif emitted[index] != declared[index]:
            detection = (
                f"sample:pos{index}:expected-{declared[index]}-got-{emitted[index]}"
            )
            identified.add(f"element:{declared[index]}")
            identified.add(f"element:{emitted[index]}")
        else:
            continue
        detections.append(detection)
    return sorted(detections), identified


def canonical_equations(doc: dict) -> list[dict]:
    """The parser's canonical equation order: (lhs, rhs, rationale, status)."""
    return sorted(
        doc["schema"]["equations"],
        key=lambda eq: (eq["lhs"], eq["rhs"], eq["rationale"], eq["status"]),
    )


def witness_element(
    doc: dict, finding: dict, tables: dict, arrows: dict, equations: list[dict]
) -> str | None:
    """The element one finding names, or None when its witness is unmapped."""
    witness = finding["witness"]
    if "key" in witness:
        step = row_name(doc, "Step", witness["rows"][0])
        return element_id(tables["yields"].get(step))
    if finding["kind"] == "dangling":
        if witness["arrow"] == "back":
            return element_id(row_name(doc, "Element", witness["src"]))
        return None
    if finding["kind"] == "violation":
        equation = equations[witness["equation"]]
        source = arrows[equation["lhs"][0]]["src"]
        if source == "Step":
            step = row_name(doc, "Step", witness["row"])
            return element_id(tables["yields"].get(step))
        if source == "Element":
            return element_id(row_name(doc, "Element", witness["row"]))
    return None


def probe_elements(doc: dict, findings: list[dict]) -> tuple[list[str], list[dict]]:
    """Map each finding to the element it names; unmapped findings are recorded."""
    tables = doc["instance"]["tables"]
    arrows = {arrow["name"]: arrow for arrow in doc["schema"]["arrows"]}
    equations = canonical_equations(doc)
    elements: list[str] = []
    unmapped: list[dict] = []
    for finding in findings:
        name = witness_element(doc, finding, tables, arrows, equations)
        if name is None:
            unmapped.append(finding)
        else:
            elements.append(name)
    return sorted(set(elements)), unmapped


def run_probe(fixture: str) -> tuple[int, list[dict]]:
    """Execute the probe arm: the user's CLI, findings parsed mechanically."""
    out = subprocess.run(
        CLI + ["check", str(KATA / fixture), "--json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    try:
        payload = json.loads(out.stdout)
    except json.JSONDecodeError as error:
        raise SystemExit(f"probe output is not JSON for `{fixture}`: {error}")
    return out.returncode, payload["data"]["findings"]


def verdict_for(unique: list[str]) -> str:
    """The recorded rule: empty unique = killed; unique + executed arm = survived."""
    return "killed" if not unique else "survived"


def sequence_failures(run: dict, declared: list[str], emitted: list[str]) -> list[str]:
    """The declared and emitted element sequences against the recording."""
    failures: list[str] = []
    if declared != run["declared"]:
        failures.append(f"{run['fixture']}: declared sequence drifted")
    if emitted != run["emitted"]:
        failures.append(f"{run['fixture']}: emitted sequence drifted")
    return failures


def baseline_checks(run: dict, doc: dict) -> tuple[list[str], set[str], list[str]]:
    """Baseline detections and identified set against the frozen expectations."""
    detections, identified = baseline_arm(doc, run["sample"])
    failures: list[str] = []
    if detections != run["baseline_expected"]:
        failures.append(f"{run['fixture']}: baseline detections drifted")
    if sorted(identified) != run["baseline_identified"]:
        failures.append(f"{run['fixture']}: baseline identified set drifted")
    return detections, identified, failures


def probe_checks(run: dict, doc: dict, identified: set[str]) -> tuple[dict, list[str]]:
    """The probe arm against the frozen expectations."""
    exit_code, findings = run_probe(run["fixture"])
    elements, unmapped = probe_elements(doc, findings)
    unique = sorted(set(elements) - identified)
    failures: list[str] = []
    if exit_code != run["exit_expected"]:
        failures.append(
            f"{run['fixture']}: probe exit {exit_code} != {run['exit_expected']}"
        )
    if findings != run["findings_expected"]:
        failures.append(f"{run['fixture']}: probe findings drifted")
    if elements != run["probe_expected"]:
        failures.append(f"{run['fixture']}: probe element set drifted")
    if unique != run["unique_expected"]:
        failures.append(f"{run['fixture']}: unique detections drifted")
    return {
        "probe_exit": exit_code,
        "probe_findings": findings,
        "probe_elements": elements,
        "unmapped_findings": unmapped,
        "unique_detections": unique,
    }, failures


def check_run(run: dict) -> tuple[dict, list[str]]:
    doc = load(run["fixture"])
    declared = element_sequence(doc)
    emitted = emitted_sequence(doc)
    baseline_detections, identified, baseline = baseline_checks(run, doc)
    probe, probe_failures = probe_checks(run, doc, identified)
    failures = sequence_failures(run, declared, emitted) + baseline + probe_failures

    report = {
        "fixture": run["fixture"],
        "declared": declared,
        "emitted": emitted,
        "sample": run["sample"],
        "probe_exit": probe["probe_exit"],
        "probe_findings": probe["probe_findings"],
        "baseline_detections": baseline_detections,
        "baseline_identified": sorted(identified),
        "probe_elements": probe["probe_elements"],
        "unmapped_findings": probe["unmapped_findings"],
        "unique_detections": probe["unique_detections"],
        "verdict": verdict_for(probe["unique_detections"]),
    }
    return report, failures


def entry_report(entry: dict) -> tuple[dict, list[str]]:
    """Run every recorded arm of one entry and select its primary report."""
    runs: list[dict] = []
    failures: list[str] = []
    for run in entry["runs"]:
        report, run_failures = check_run(run)
        failures.extend(run_failures)
        runs.append(report)
    primary = next(
        report
        for report, run in zip(runs, entry["runs"])
        if run["fixture"] == entry["primary_fixture"]
    )
    return {
        "id": entry["id"],
        "control": entry["control"],
        "primary_fixture": entry["primary_fixture"],
        "runs": runs,
        "unique_detections": primary["unique_detections"],
        "verdict": primary["verdict"],
    }, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0

    faults = json.loads(FAULTS.read_text())
    failures: list[str] = []
    entries: list[dict] = []
    for entry in faults["entries"]:
        report, entry_failures = entry_report(entry)
        failures.extend(entry_failures)
        entries.append(report)

    report = {
        "kata": faults["kata"],
        "baseline_sha256": sha256_of(Path(__file__)),
        "baseline_protocol": faults["baseline_protocol"],
        "entries": entries,
        "failures": failures,
        "ok": not failures,
    }
    print(json.dumps(report, indent=2))

    if "--assert" in sys.argv and failures:
        for failure in failures:
            print(f"FAIL {failure}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())