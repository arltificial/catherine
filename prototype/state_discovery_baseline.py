#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the state-discovery kata —
what a scripted transition-table test catches on a pre-registered sample
of (state, event) cells, so the probe arm's unique detections are
arithmetic (executed probe elements minus baseline-identified elements),
never asserted.

Responsibilities: load each committed fixture, run the frozen cell
sample through conventional assertions (declared cell -> transition
exists, transition source/event/target agree with the cell), execute the
probe arm (`catherine check --json`, plus `catherine underconstraint`
where the run asks for the discovery questions), map both outputs to
presentation-level elements (`cell:<State>/<Event>`), and compare the
result against `fixtures/katas/state-discovery/faults.json`. The
construction probe (a transition naming an undeclared state) is recorded
as an executed parse rejection but never counted in `unique_detections`:
it is not a finding.

Rationale: a conventional team tests the cells it wrote tests for. The
sample is pre-registered per fixture, so a fault outside it is a fair
miss rather than a strawman. The probe arm is executed as a subprocess
(the user's CLI) and parsed mechanically; no finding is copied by hand.
Hypothesized underconstraint findings are recorded verbatim and excluded
from unique detections — a question is not a detection.

Usage:
  python3 prototype/state_discovery_baseline.py            # report JSON
  python3 prototype/state_discovery_baseline.py --assert   # frozen gate
  python3 prototype/state_discovery_baseline.py --hash     # sha256 of this file
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "state-discovery"
FAULTS = KATA / "faults.json"
CLI = ["cargo", "run", "-q", "-p", "catherine-cli", "--"]


def load(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def sorted_names(doc: dict, obj: str) -> list[str]:
    return sorted(doc["instance"]["rows"][obj])


def row_name(doc: dict, obj: str, index: int) -> str:
    return sorted_names(doc, obj)[index]


def cell_element(doc: dict, cell: str) -> str:
    tables = doc["instance"]["tables"]
    return f"cell:{tables['cell_state'][cell]}/{tables['cell_event'][cell]}"


def transition_cell(doc: dict, transition: str) -> str:
    for cell, value in doc["instance"]["tables"]["cell_transition"].items():
        if value == transition:
            return cell
    raise SystemExit(f"transition `{transition}` has no cell")


def cell_for(doc: dict, state: str, event: str) -> str | None:
    tables = doc["instance"]["tables"]
    for cell in sorted_names(doc, "Cell"):
        if tables["cell_state"][cell] == state and tables["cell_event"][cell] == event:
            return cell
    return None


def baseline_cell(doc: dict, state: str, event: str) -> str | None:
    """One conventional cell assertion; a detection string or None."""
    tables = doc["instance"]["tables"]
    cell = cell_for(doc, state, event)
    if cell is None:
        return f"sample:{state}/{event}:cell-undeclared"
    transition = tables["cell_transition"].get(cell)
    if transition is None:
        return f"sample:{state}/{event}:transition-missing"
    target = tables["to"].get(transition)
    if target is None:
        return f"sample:{state}/{event}:target-unresolved"
    if target not in set(sorted_names(doc, "State")):
        return f"sample:{state}/{event}:unknown-target"
    if target != tables["cell_target"][cell]:
        return f"sample:{state}/{event}:target-mismatch"
    if tables["from"][transition] != state:
        return f"sample:{state}/{event}:source-mismatch"
    if tables["on"][transition] != event:
        return f"sample:{state}/{event}:event-mismatch"
    return None


def baseline_arm(doc: dict, sample: list[list[str]]) -> tuple[list[str], set[str]]:
    detections: list[str] = []
    identified: set[str] = set()
    for state, event in sample:
        detection = baseline_cell(doc, state, event)
        if detection is not None:
            detections.append(detection)
            identified.add(f"cell:{state}/{event}")
    return sorted(detections), identified


def probe_element(doc: dict, finding: dict) -> str | None:
    witness = finding["witness"]
    if finding["kind"] == "dangling":
        arrow = witness["arrow"]
        obj = "Cell" if arrow == "cell_transition" else "Transition"
        cell = row_name(doc, obj, witness["src"])
        if obj == "Transition":
            cell = transition_cell(doc, cell)
        return cell_element(doc, cell)
    if finding["kind"] == "violation":
        return cell_element(doc, row_name(doc, "Cell", witness["row"]))
    return None


def construction_element(doc: dict, message: str) -> str | None:
    match = re.search(r"maps `([^`]+)`", message)
    if match is None:
        return None
    return cell_element(doc, transition_cell(doc, match.group(1)))


def run_cli(args: list[str]) -> tuple[int, dict]:
    out = subprocess.run(
        CLI + args, cwd=ROOT, capture_output=True, text=True, check=False
    )
    raw = out.stdout.strip() or out.stderr.strip()
    return out.returncode, json.loads(raw)


def hypothesis_ids(findings: list[dict]) -> list[str]:
    ids = []
    for finding in findings:
        detail = finding["witness"]["detail"]
        ids.append(f"parallel_paths:{'='.join(detail['lhs'] + detail['rhs'])}")
    return sorted(ids)


def probe_arm(run: dict, doc: dict) -> dict:
    code, payload = run_cli(["check", str(KATA / run["fixture"]), "--json"])
    if run.get("construction"):
        expected = run["construction"]
        if code != 1 or payload["data"]["code"] != expected["code"]:
            raise SystemExit(f"{run['fixture']}: construction probe did not reject")
        return {
            "findings": [],
            "elements": [],
            "construction": construction_element(doc, payload["data"]["message"]),
            "hypotheses": [],
        }
    findings = payload["data"]["findings"]
    elements = sorted({element for f in findings if (element := probe_element(doc, f))})
    hypotheses: list[str] = []
    if run.get("underconstraint"):
        _, under = run_cli(
            ["underconstraint", str(KATA / run["fixture"]), "--max-path-length", "1", "--json"]
        )
        hypotheses = hypothesis_ids(under["data"]["findings"])
    return {
        "findings": findings,
        "elements": elements,
        "construction": None,
        "hypotheses": hypotheses,
    }


def reachable_states(doc: dict, initial: str) -> set[str]:
    tables = doc["instance"]["tables"]
    reached = {initial}
    frontier = [initial]
    while frontier:
        state = frontier.pop()
        for cell in sorted_names(doc, "Cell"):
            if tables["cell_state"][cell] != state:
                continue
            transition = tables["cell_transition"].get(cell)
            target = None if transition is None else tables["to"].get(transition)
            if target is not None and target not in reached:
                reached.add(target)
                frontier.append(target)
    return reached


def check_run(run: dict) -> tuple[dict, list[str]]:
    doc = load(run["fixture"])
    detections, identified = baseline_arm(doc, run["sample"])
    probe = probe_arm(run, doc)
    unique = sorted(set(probe["elements"]) - identified)
    report = {
        "fixture": run["fixture"],
        "baseline_detections": detections,
        "baseline_identified": sorted(identified),
        "probe_elements": probe["elements"],
        "unique": unique,
        "hypotheses": probe["hypotheses"],
        "construction": probe["construction"],
    }
    failures = []
    if detections != sorted(run["baseline_expected"]):
        failures.append(f"{run['fixture']}: baseline detections drift")
    if sorted(identified) != sorted(run["baseline_identified"]):
        failures.append(f"{run['fixture']}: baseline identified elements drift")
    if probe["elements"] != sorted(run["probe_expected"]):
        failures.append(f"{run['fixture']}: probe elements drift")
    if unique != sorted(run["unique_expected"]):
        failures.append(f"{run['fixture']}: unique detections drift")
    if run.get("underconstraint") and probe["hypotheses"] != sorted(
        run["hypotheses_expected"]
    ):
        failures.append(f"{run['fixture']}: discovery hypotheses drift")
    if run.get("construction") and probe["construction"] != run["construction"]["cell"]:
        failures.append(f"{run['fixture']}: construction element drift")
    return report, failures


def observation(run: dict, doc: dict, report: dict) -> list[str]:
    expected = run.get("unreachable_expected")
    if expected is None:
        return []
    reached = reachable_states(doc, run["initial_state"])
    unreachable = sorted(set(sorted_names(doc, "State")) - reached)
    report["unreachable_observed"] = unreachable
    return [] if unreachable == sorted(expected) else ["unreachable observation drift"]


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    reports, failures = [], []
    for entry in matrix["entries"]:
        entry_report = {"id": entry["id"], "runs": []}
        for run in entry["runs"]:
            report, problems = check_run(run)
            failures.extend(problems)
            failures.extend(observation(run, load(run["fixture"]), report))
            entry_report["runs"].append(report)
        reports.append(entry_report)
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)), "entries": reports}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        total = len({u for entry in reports for run in entry["runs"] for u in run["unique"]})
        print(f"baseline arm frozen: {len(reports)} entries, {total} unique elements", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())