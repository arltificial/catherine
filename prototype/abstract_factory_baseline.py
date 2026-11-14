#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the abstract-factory kata —
what a scripted factory test catches on a pre-registered sample of
(factory, kind) realization sites, so the probe arm's unique detections
are arithmetic (executed probe elements minus baseline-identified
elements), never asserted.

Responsibilities: load each committed fixture, run the frozen sample
through conventional factory assertions (the (factory, kind) site is
declared exactly once, its product resolves, the product's kind equals
the requested kind, the product's family equals the factory's family,
and every sampled kind is realized somewhere), execute the probe arm
(`catherine check --json`), map both outputs to presentation-level
elements (`cell:<Factory>/<Kind>` and `kind:<Kind>`), and compare the
result against `fixtures/katas/abstract-factory/faults.json`. The
construction probe (a cell naming an undeclared product) is recorded as
an executed parse rejection but never counted in `unique_detections`: it
is a refusal, not a finding.

Rationale: a conventional team tests the factory calls it wrote tests
for. The sample is pre-registered per fixture, so a fault outside it is a
fair miss rather than a strawman. The probe arm is executed as a
subprocess (the user's CLI) and parsed mechanically; no finding is copied
by hand. The baseline arm always runs before the probe arm inside this
process. The scaffold this script needs (a declared Cell per (factory,
kind) pair) is authored data — the kernel checks the declared grid, never
the cross-product — and that boundary is recorded as G1 in the kata
README, not hidden.

Usage:
  python3 prototype/abstract_factory_baseline.py            # report JSON
  python3 prototype/abstract_factory_baseline.py --assert   # frozen gate
  python3 prototype/abstract_factory_baseline.py --hash     # sha256 of this file
"""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "abstract-factory"
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


def row_index(doc: dict, obj: str, name: str) -> int:
    return sorted_names(doc, obj).index(name)


def tables(doc: dict) -> dict:
    return doc["instance"]["tables"]


def cell_element(doc: dict, cell: str) -> str:
    """The presentation element of one declared realization site."""
    t = tables(doc)
    factory = t["cell_factory"].get(cell)
    kind = t["cell_kind"].get(cell)
    if factory is None or kind is None:
        return f"cell@row:{row_index(doc, 'Cell', cell)}"
    return f"cell:{factory}/{kind}"


def cells_for(doc: dict, factory: str, kind: str) -> list[str]:
    t = tables(doc)
    return [
        cell
        for cell in sorted_names(doc, "Cell")
        if t["cell_factory"].get(cell) == factory and t["cell_kind"].get(cell) == kind
    ]


def baseline_pair(doc: dict, factory: str, kind: str) -> str | None:
    """One conventional factory-call assertion; a detection string or None."""
    t = tables(doc)
    cells = cells_for(doc, factory, kind)
    if not cells:
        return f"sample:{factory}/{kind}:cell-undeclared"
    if len(cells) > 1:
        return f"sample:{factory}/{kind}:duplicate-cell"
    cell = cells[0]
    product = t["cell_product"].get(cell)
    if product is None:
        return f"sample:{factory}/{kind}:product-missing"
    if product not in sorted_names(doc, "Product"):
        return f"sample:{factory}/{kind}:product-undeclared"
    if t["product_kind"].get(product) != kind:
        return f"sample:{factory}/{kind}:kind-mismatch"
    if t["product_family"].get(product) != t["factory_family"].get(factory):
        return f"sample:{factory}/{kind}:family-mismatch"
    return None


def baseline_kind(doc: dict, kind: str) -> str | None:
    """One conventional coverage assertion over a sampled kind."""
    t = tables(doc)
    if kind not in sorted_names(doc, "Kind"):
        return f"sample:kind:{kind}:undeclared"
    realized = any(
        t["cell_kind"].get(cell) == kind and t["cell_product"].get(cell) is not None
        for cell in sorted_names(doc, "Cell")
    )
    if not realized:
        return f"sample:kind:{kind}:unrealized"
    return None


def baseline_arm(
    doc: dict, sample: list[list[str]], sample_kinds: list[str]
) -> tuple[list[str], set[str]]:
    detections: list[str] = []
    identified: set[str] = set()
    for factory, kind in sample:
        detection = baseline_pair(doc, factory, kind)
        if detection is not None:
            detections.append(detection)
            identified.add(f"cell:{factory}/{kind}")
    for kind in sample_kinds:
        detection = baseline_kind(doc, kind)
        if detection is not None:
            detections.append(detection)
            identified.add(f"kind:{kind}")
    return sorted(detections), identified


def dangling_element(doc: dict, witness: dict) -> str:
    """One dangling witness resolved through its declared arrow vocabulary."""
    arrow = witness["arrow"]
    if arrow == "kind_witness":
        return f"kind:{row_name(doc, 'Kind', witness['src'])}"
    if arrow in ("cell_product", "cell_factory", "cell_kind"):
        return cell_element(doc, row_name(doc, "Cell", witness["src"]))
    return f"{arrow}@row:{witness['src']}"


def violation_elements(doc: dict, witness: dict) -> set[str]:
    """A violation witness names its cell either by row or by a row list."""
    if "equation" in witness:
        return {cell_element(doc, row_name(doc, "Cell", witness["row"]))}
    if "key" in witness:
        return {cell_element(doc, row_name(doc, "Cell", row)) for row in witness["rows"]}
    return {"violation:unmapped"}


def finding_elements(doc: dict, finding: dict) -> set[str]:
    """The pre-registered elements one kernel finding contributes."""
    if finding["kind"] == "dangling":
        return {dangling_element(doc, finding["witness"])}
    if finding["kind"] == "violation":
        return violation_elements(doc, finding["witness"])
    return {f"{finding['kind']}:unmapped"}


def probe_elements(doc: dict, findings: list[dict]) -> set[str]:
    elements: set[str] = set()
    for finding in findings:
        elements |= finding_elements(doc, finding)
    return elements


def construction_element(doc: dict, message: str) -> str | None:
    match = re.search(r"maps `([^`]+)`", message)
    if match is None:
        return None
    return cell_element(doc, match.group(1))


def run_cli(args: list[str]) -> tuple[int, dict]:
    out = subprocess.run(
        CLI + args, cwd=ROOT, capture_output=True, text=True, check=False
    )
    raw = out.stdout.strip() or out.stderr.strip()
    return out.returncode, json.loads(raw)


def probe_arm(run: dict, doc: dict) -> dict:
    code, payload = run_cli(["check", str(KATA / run["fixture"]), "--json"])
    if run.get("construction"):
        expected = run["construction"]
        if code != 1 or payload["data"]["code"] != expected["code"]:
            raise SystemExit(f"{run['fixture']}: construction probe did not reject")
        return {
            "fixture": run["fixture"],
            "exit": code,
            "findings": [],
            "elements": [],
            "construction": construction_element(doc, payload["data"]["message"]),
        }
    findings = payload["data"]["findings"]
    return {
        "fixture": run["fixture"],
        "exit": code,
        "findings": findings,
        "elements": sorted(probe_elements(doc, findings)),
        "construction": None,
    }


def check_run(run: dict) -> tuple[dict, list[str]]:
    doc = load(run["fixture"])
    if run.get("construction"):
        # The conventional arm loads the document before it can call a
        # factory; an unloadable document is not a model the suite can run
        # against, so the arm is recorded as not run (see the kata README's
        # Observed section: the first implementation inspected the raw JSON
        # and identified the sampled cell — recorded as a discrepancy, not
        # hidden; the unique set is empty under either reading).
        detections, identified = [], set()
    else:
        detections, identified = baseline_arm(doc, run["sample"], run["sample_kinds"])
    probe = probe_arm(run, doc)
    unique = sorted(set(probe["elements"]) - identified)
    report = {
        "fixture": run["fixture"],
        "exit": probe["exit"],
        "baseline_detections": detections,
        "baseline_identified": sorted(identified),
        "probe_elements": probe["elements"],
        "unique": unique,
        "findings": probe["findings"],
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
    if run.get("construction") and probe["construction"] != run["construction"]["cell"]:
        failures.append(f"{run['fixture']}: construction element drift")
    return report, failures


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