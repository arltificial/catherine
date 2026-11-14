#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the mediator kata — what a
scripted per-arrow mediator suite catches on a pre-registered fault set,
so the probe arm's unique detections are arithmetic (executed probe
elements minus baseline-identified elements), never asserted.

Responsibilities: load each committed fixture, run the frozen
conventional assertions (every colleague's `hub` resolves to a declared
Mediator row; every interaction's `speaker`/`listener` resolve to
distinct declared Colleague rows; `dispatched_by` resolves to a declared
Mediator row; the documented (speaker, listener) route matches), execute
the probe arm (`catherine check --json`, plus `catherine
underconstraint --max-path-length 2 --json` where the entry asks for the
discovery questions), map both outputs to presentation-level elements
(`colleague:<name>` / `interaction:<name>`), and compare the result
against `fixtures/katas/mediator/faults.json`.

Rationale: the baseline is what a conventional suite asserts when the
mediator is tested arrow by arrow: each cell is well-formed and each
documented route is the documented route. It never composes two arrows,
so `speaker ; hub` and `listener ; hub` are outside it — that
composition is precisely the mediator law under test. It also never
asserts a specific Mediator row, because a second declared mediator is
the migration fault, not a schema violation. Hypothesized
underconstraint findings are recorded verbatim and excluded from unique
detections: a question is not a detection.

Usage:
  python3 prototype/mediator_baseline.py            # report JSON
  python3 prototype/mediator_baseline.py --assert   # frozen gate
  python3 prototype/mediator_baseline.py --hash     # sha256 of this file
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "mediator"
FAULTS = KATA / "faults.json"
CLI = ["cargo", "run", "-q", "-p", "catherine-cli", "--"]


def load(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def rows(doc: dict, obj: str) -> list[str]:
    return sorted(doc["instance"]["rows"][obj])


def declared(doc: dict, obj: str) -> set[str]:
    return set(rows(doc, obj))


def table(doc: dict, arrow: str) -> dict:
    return doc["instance"]["tables"].get(arrow, {})


def arrow_src(doc: dict, name: str) -> str:
    for arrow in doc["schema"]["arrows"]:
        if arrow["name"] == name:
            return arrow["src"]
    raise SystemExit(f"arrow `{name}` is not declared")


def row_at(doc: dict, obj: str, index: int) -> str:
    return rows(doc, obj)[index]


def element(doc: dict, obj: str, name: str) -> str:
    return f"{obj.lower()}:{name}"


def colleague_detections(doc: dict, mediators: set[str]) -> tuple[list[str], set[str]]:
    """Every colleague's hub cell must name a declared mediator."""
    detections: list[str] = []
    identified: set[str] = set()
    for colleague in rows(doc, "Colleague"):
        hub = table(doc, "hub").get(colleague)
        if hub is None or hub not in mediators:
            detections.append(f"colleague:{colleague}:hub-unresolved")
            identified.add(element(doc, "Colleague", colleague))
    return detections, identified


def interaction_participants(
    doc: dict, colleagues: set[str], interaction: str
) -> tuple[list[str], set[str], str | None, str | None]:
    """Speaker and listener must be declared colleagues and must differ."""
    detections: list[str] = []
    identified: set[str] = set()
    speaker = table(doc, "speaker").get(interaction)
    listener = table(doc, "listener").get(interaction)
    if speaker is None or speaker not in colleagues:
        detections.append(f"interaction:{interaction}:speaker-unresolved")
        identified.add(element(doc, "Interaction", interaction))
    if listener is None or listener not in colleagues:
        detections.append(f"interaction:{interaction}:listener-unresolved")
        identified.add(element(doc, "Interaction", interaction))
    if speaker is not None and speaker == listener:
        detections.append(f"interaction:{interaction}:self-interaction")
        identified.add(element(doc, "Interaction", interaction))
    return detections, identified, speaker, listener


def interaction_routes(
    doc: dict,
    mediators: set[str],
    documented: dict[str, list[str]],
    interaction: str,
    speaker: str | None,
    listener: str | None,
) -> tuple[list[str], set[str]]:
    """The dispatch cell must name a mediator and the route must match the document."""
    detections: list[str] = []
    identified: set[str] = set()
    dispatch = table(doc, "dispatched_by").get(interaction)
    if dispatch is None or dispatch not in mediators:
        detections.append(f"interaction:{interaction}:dispatch-unresolved")
        identified.add(element(doc, "Interaction", interaction))
    route = documented.get(interaction)
    if route is None:
        detections.append(f"interaction:{interaction}:route-undocumented")
        identified.add(element(doc, "Interaction", interaction))
    elif [speaker, listener] != list(route):
        detections.append(f"interaction:{interaction}:route-mismatch")
        identified.add(element(doc, "Interaction", interaction))
    return detections, identified


def baseline_arm(doc: dict, documented: dict[str, list[str]]) -> tuple[list[str], set[str]]:
    """The frozen conventional suite: per-arrow assertions, all rows."""
    colleagues = declared(doc, "Colleague")
    mediators = declared(doc, "Mediator")
    detections, identified = colleague_detections(doc, mediators)
    for interaction in rows(doc, "Interaction"):
        found, named, speaker, listener = interaction_participants(doc, colleagues, interaction)
        routed, route_named = interaction_routes(
            doc, mediators, documented, interaction, speaker, listener
        )
        detections += found + routed
        identified |= named | route_named
    return sorted(detections), identified


def parsed_equations(doc: dict) -> list[dict]:
    """The kernel's parsed equation order: sorted by (lhs, rhs) path key."""
    return sorted(
        doc["schema"]["equations"], key=lambda equation: (equation["lhs"], equation["rhs"])
    )


def probe_element(doc: dict, finding: dict) -> str:
    witness = finding["witness"]
    if finding["kind"] == "dangling":
        obj = arrow_src(doc, witness["arrow"])
        return element(doc, obj, row_at(doc, obj, witness["src"]))
    if finding["kind"] == "violation":
        equation = parsed_equations(doc)[witness["equation"]]
        obj = arrow_src(doc, equation["lhs"][0])
        return element(doc, obj, row_at(doc, obj, witness["row"]))
    raise SystemExit(f"unexpected finding kind `{finding['kind']}`")


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
        ids.append(
            "parallel_paths:"
            + ",".join(detail["lhs"])
            + "="
            + ",".join(detail["rhs"])
        )
    return sorted(ids)


def probe_arm(run: dict, doc: dict) -> dict:
    _, payload = run_cli(["check", str(KATA / run["fixture"]), "--json"])
    findings = payload["data"]["findings"]
    elements = sorted({probe_element(doc, finding) for finding in findings})
    hypotheses: list[str] = []
    if run.get("discovery"):
        _, under = run_cli(
            [
                "underconstraint",
                str(KATA / run["fixture"]),
                "--max-path-length",
                "2",
                "--json",
            ]
        )
        hypotheses = hypothesis_ids(under["data"]["findings"])
    return {"findings": findings, "elements": elements, "hypotheses": hypotheses}


def check_run(run: dict, documented: dict[str, list[str]]) -> tuple[dict, list[str]]:
    doc = load(run["fixture"])
    detections, identified = baseline_arm(doc, documented)
    probe = probe_arm(run, doc)
    unique = sorted(set(probe["elements"]) - identified)
    report = {
        "fixture": run["fixture"],
        "baseline_detections": detections,
        "baseline_identified": sorted(identified),
        "probe_elements": probe["elements"],
        "unique": unique,
        "hypotheses": probe["hypotheses"],
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
    if run.get("discovery") and probe["hypotheses"] != sorted(
        run["hypotheses_expected"]
    ):
        failures.append(f"{run['fixture']}: discovery hypotheses drift")
    return report, failures


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    matrix = json.loads(FAULTS.read_text())
    documented = matrix["documented_routes"]
    reports, failures = [], []
    for entry in matrix["entries"]:
        entry_report = {"id": entry["id"], "runs": []}
        for run in entry["runs"]:
            report, problems = check_run(run, documented)
            failures.extend(problems)
            entry_report["runs"].append(report)
        reports.append(entry_report)
    print(json.dumps({"baseline_sha256": sha256_of(Path(__file__)), "entries": reports}, indent=2))
    if failures:
        for failure in failures:
            print(f"MISMATCH {failure}", file=sys.stderr)
        return 1
    if "--assert" in sys.argv:
        total = len(
            {u for entry in reports for run in entry["runs"] for u in run["unique"]}
        )
        print(
            f"baseline arm frozen: {len(reports)} entries, {total} unique elements",
            file=sys.stderr,
        )
    return 0


if __name__ == "__main__":
    sys.exit(main())