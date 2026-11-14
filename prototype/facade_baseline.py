#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the facade kata — the scripted
recorded-composition replay a team runs on a Facade boundary — plus the
probe arm and the expressibility observations the verdict rests on.

Responsibilities: replay the frozen documented facade contract (the
per-entry-point call multiset, the aggregate outcome, and per-call
transparency) against every committed fixture, execute the kernel's
`check`, `underconstraint`, and `query` surfaces, map both arms onto the
`call:<FacadeOp>/<Action>` / `op:<FacadeOp>` element vocabulary, and compare
every observation with the pre-registered matrix in
`fixtures/katas/facade/faults.json`. `--assert` exits 1 on any drift;
`--hash` prints the script sha256 the README and ledger pin.

Rationale: `unique_detections` must be arithmetic (probe elements minus
baseline-identified elements), never asserted, so the same script derives
both arms from the committed fixtures. The frozen spec is the *baseline's*
own record of the contract, not a read-back of the model's `SpecCall` rows:
a baseline that read the model's documentation would be checking the model
against itself. The expressibility block asserts the surface boundary the
named gaps rest on, so a future kernel capability trips the gate instead of
silently validating an old claim.
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "facade"
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


def pairs(doc: dict) -> Counter:
    """The observed call multiset, keyed by (entry point, subsystem action)."""
    tables = doc["instance"]["tables"]
    return Counter(
        (tables["call_facade"][call], tables["call_action"][call])
        for call in sorted_names(doc, "Call")
    )


def call_row(doc: dict, facade: str, action: str) -> str | None:
    tables = doc["instance"]["tables"]
    for call in sorted_names(doc, "Call"):
        if tables["call_facade"][call] == facade and tables["call_action"][call] == action:
            return call
    return None


def pair_detections(
    facade: str, spec: Counter, observed: Counter
) -> tuple[list[str], set[str]]:
    """Missing, duplicated and unexpected (facade, action) pairs for one facade."""
    detections: list[str] = []
    identified: set[str] = set()
    for pair, count in sorted(spec.items()):
        seen = observed[pair]
        if seen == 0:
            detections.append(f"composition:{pair[0]}/{pair[1]}:missing")
            identified.add(f"call:{pair[0]}/{pair[1]}")
        elif seen > count:
            detections.append(f"composition:{pair[0]}/{pair[1]}:duplicated")
            identified.add(f"call:{pair[0]}/{pair[1]}")
    for pair, seen in sorted(observed.items()):
        if pair[0] == facade and pair not in spec and seen:
            detections.append(f"composition:{pair[0]}/{pair[1]}:unexpected")
            identified.add(f"call:{pair[0]}/{pair[1]}")
    return detections, identified


def aggregate_detection(
    facade: str, tables: dict, documented: dict
) -> tuple[list[str], set[str]]:
    """The facade's aggregated outcome must match the documented aggregate."""
    observed = tables["facade_outcome"][facade]
    expected = documented[facade]["aggregate"]
    if observed == expected:
        return [], set()
    return [f"aggregate:{facade}:{observed}!={expected}"], {f"op:{facade}"}


def composition_detections(
    doc: dict, documented: dict, sample: dict
) -> tuple[list[str], set[str]]:
    """The frozen conventional composition assertions for every sampled facade."""
    tables = doc["instance"]["tables"]
    observed = pairs(doc)
    detections: list[str] = []
    identified: set[str] = set()
    for facade in sample["facades"]:
        spec = Counter((facade, action) for action in documented[facade]["actions"])
        pair_found, pair_named = pair_detections(facade, spec, observed)
        aggregate_found, aggregate_named = aggregate_detection(facade, tables, documented)
        detections += pair_found + aggregate_found
        identified |= pair_named | aggregate_named
    return detections, identified


def transparency_detections(doc: dict, sample: dict) -> tuple[list[str], set[str]]:
    """Each sampled facade call's outcome must match the action's outcome."""
    tables = doc["instance"]["tables"]
    detections: list[str] = []
    identified: set[str] = set()
    for facade, action in sample["transparency_pairs"]:
        call = call_row(doc, facade, action)
        if call is None:
            continue
        observed_outcome = tables["call_outcome"][call]
        expected_outcome = tables["action_outcome"][action]
        if observed_outcome != expected_outcome:
            detections.append(
                f"transparency:{facade}/{action}:{observed_outcome}!={expected_outcome}"
            )
            identified.add(f"call:{facade}/{action}")
    return detections, identified


def baseline_arm(
    doc: dict, documented: dict, sample: dict
) -> tuple[list[str], set[str]]:
    """The frozen conventional replay: detections and identified elements."""
    composition, composition_named = composition_detections(doc, documented, sample)
    transparency, transparency_named = transparency_detections(doc, sample)
    return sorted(composition + transparency), composition_named | transparency_named


def canonical_equations(doc: dict) -> list[dict]:
    """The parser's canonical equation order: sorted by (lhs, rhs, rationale, status)."""
    return sorted(
        doc["schema"]["equations"],
        key=lambda e: (e["lhs"], e["rhs"], e["rationale"], e.get("status", "accepted")),
    )


def pair_element(doc: dict, obj: str, row: str) -> str:
    tables = doc["instance"]["tables"]
    if obj == "Call":
        return f"call:{tables['call_facade'][row]}/{tables['call_action'][row]}"
    return f"call:{tables['spec_facade'][row]}/{tables['spec_action'][row]}"


def probe_element(doc: dict, finding: dict) -> set[str]:
    """The elements one finding names, in the call:<facade>/<action> vocabulary."""
    arrows = {arrow["name"]: arrow for arrow in doc["schema"]["arrows"]}
    witness = finding["witness"]
    if finding["kind"] == "violation" and "equation" in witness:
        equation = canonical_equations(doc)[witness["equation"]]
        src = arrows[equation["lhs"][0]]["src"]
        return {pair_element(doc, src, row_name(doc, src, witness["row"]))}
    if finding["kind"] == "violation" and "key" in witness:
        key = sorted(
            doc["schema"]["keys"],
            key=lambda k: (k["object"], k["arrows"], k.get("status", "accepted")),
        )[witness["key"]]
        return {pair_element(doc, key["object"], row_name(doc, key["object"], row))
                for row in witness["rows"]}
    if finding["kind"] == "dangling":
        src = arrows[witness["arrow"]]["src"]
        return {pair_element(doc, src, row_name(doc, src, witness["src"]))}
    return set()


def probe_arm(fixture: str, doc: dict) -> dict:
    code, payload = run_cli(["check", str(KATA / fixture), "--json"])
    if code not in (0, 3, 4, 5, 6, 7):
        raise SystemExit(f"{fixture}: check exited {code}, not a finding exit")
    findings = payload["data"]["findings"]
    elements: set[str] = set()
    for finding in findings:
        elements |= probe_element(doc, finding)
    return {"exit": code, "findings": findings, "elements": sorted(elements)}


def run_cli(args: list[str]) -> tuple[int, dict]:
    out = subprocess.run(
        CLI + args, cwd=ROOT, capture_output=True, text=True, check=False
    )
    raw = out.stdout.strip() or out.stderr.strip()
    return out.returncode, json.loads(raw)


def hypothesis_ids(fixture: str) -> list[str]:
    _, payload = run_cli(
        ["underconstraint", str(KATA / fixture), "--max-path-length", "2", "--json"]
    )
    return sorted(
        "parallel_paths:{}={}".format(
            "∘".join(finding["witness"]["detail"]["lhs"]),
            "∘".join(finding["witness"]["detail"]["rhs"]),
        )
        for finding in payload["data"]["findings"]
    )


def expressibility(matrix: dict) -> tuple[dict, list[str]]:
    """The surface boundary the named gaps rest on, asserted, not assumed."""
    spec = matrix["expressibility"]
    doc = load(spec["fixture"])
    facade_index = sorted_names(doc, "FacadeOp").index(spec["entry_point"])
    sourced = sorted(
        arrow["name"] for arrow in doc["schema"]["arrows"] if arrow["src"] == "FacadeOp"
    )
    _, forward = run_cli(
        [
            "query", str(KATA / spec["fixture"]),
            "--seed", f"FacadeOp#{facade_index}",
            "--follow", "facade_outcome", "--json",
        ]
    )
    _, backward = run_cli(
        [
            "query", str(KATA / spec["fixture"]),
            "--seed", f"FacadeOp#{facade_index}",
            "--follow", "call_facade", "--direction", "backward", "--json",
        ]
    )
    observed = {
        "arrows_sourced_at_facadeop": sourced,
        "forward_objects": [node["object"] for node in forward["data"]["nodes"]],
        "backward_call_rows": sorted(
            node["row"] for node in backward["data"]["nodes"] if node["object"] == "Call"
        ),
    }
    failures = []
    for key in ("arrows_sourced_at_facadeop", "forward_objects", "backward_call_rows"):
        if observed[key] != spec[key]:
            failures.append(f"expressibility:{key} drift")
    if "Call" in observed["forward_objects"]:
        failures.append("expressibility: the entry point reaches its call fiber forward")
    return observed, failures


def check_run(run: dict) -> tuple[dict, list[str]]:
    fixture = run["fixture"]
    doc = load(fixture)
    detections, identified = baseline_arm(doc, run["documented"], run["sample"])
    probe = probe_arm(fixture, doc)
    unique = sorted(set(probe["elements"]) - identified)
    report = {
        "fixture": fixture,
        "baseline_detections": detections,
        "baseline_identified": sorted(identified),
        "probe_exit": probe["exit"],
        "probe_findings": probe["findings"],
        "probe_elements": probe["elements"],
        "probe_silent": probe["findings"] == [],
        "silence_is_gap": run.get("silence_is_gap", False),
        "unique": unique,
        "hypotheses": hypothesis_ids(fixture),
    }
    failures = []
    if detections != sorted(run["baseline_expected"]):
        failures.append(f"{fixture}: baseline detections drift")
    if sorted(identified) != sorted(run["baseline_identified"]):
        failures.append(f"{fixture}: baseline identified elements drift")
    if probe["elements"] != sorted(run["probe_expected"]):
        failures.append(f"{fixture}: probe elements drift")
    if unique != sorted(run["unique_expected"]):
        failures.append(f"{fixture}: unique detections drift")
    if report["probe_silent"] != run.get("probe_silent_expected", False):
        failures.append(f"{fixture}: probe silence drift")
    if run.get("silence_is_gap") and not report["probe_silent"]:
        failures.append(f"{fixture}: the facade-defining fault is no longer probe-silent")
    if report["hypotheses"] != sorted(json.loads(FAULTS.read_text())["hypotheses_expected"]):
        failures.append(f"{fixture}: discovery hypotheses drift")
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    surface, failures = expressibility(matrix)
    reports = []
    for entry in matrix["entries"]:
        entry_report = {"id": entry["id"], "control": entry["control"], "runs": []}
        for run in entry["runs"]:
            report, problems = check_run(run)
            failures.extend(problems)
            entry_report["runs"].append(report)
        reports.append(entry_report)
    print(
        json.dumps(
            {
                "baseline_sha256": sha256_of(Path(__file__)),
                "expressibility": surface,
                "entries": reports,
            },
            indent=2,
        )
    )
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        gaps = [run["fixture"] for entry in reports for run in entry["runs"] if run["silence_is_gap"]]
        total = len({u for entry in reports for run in entry["runs"] for u in run["unique"]})
        print(
            f"baseline arm frozen: {len(reports)} entries, {len(gaps)} probe-silent "
            f"facade-defining runs, {total} unique elements",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())