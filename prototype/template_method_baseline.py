#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the template-method gap
study — what a real template-method test catches when it can run the
code, next to the only thing the kernel can evaluate when it can only
read a document. The ticket's candidate law is "hook-order declaration
vs declared call order"; this script executes both sides so the gap is
evidence, not an opinion.

Responsibilities: build the conventional arm (an abstract template with
a fixed skeleton, concrete subclasses, and a spy recording every hook
that runs), compute its detections over the pre-registered code faults
(`order-drift`, `omitted-stage`, `dead-override`), transcribe each
committed `.ct.json` fixture back to a hook sequence through the
fixture's own `calls`/`call_rank` tables, execute the kernel arm
(`catherine check --json`) on every fixture, resolve each witness to the
hook element vocabulary of `fixtures/katas/template-method/faults.json`,
and compare everything against that pre-registration. `--assert` exits 1
on any drift; `--hash` prints this file's sha256.

Rationale: the pattern's ground truth is a method body — which subclass
overrides which hook, and which hooks the inherited skeleton invokes.
The conventional arm reads that ground truth directly (it runs the
code); the kernel can only read an authored call log. The `dead-override`
entry makes the asymmetry executable: the clean program and the
defective program share one body, therefore one faithful transcription,
therefore one identical document, and the kernel must stay silent on the
defect while the conventional arm names it. A law that cannot separate
those two programs is not a law of this pattern, so this script is the
evidence for a still-gap close and deliberately records no detection as
unique.

Usage:
  python3 prototype/template_method_baseline.py            # report JSON
  python3 prototype/template_method_baseline.py --assert   # frozen gate
  python3 prototype/template_method_baseline.py --hash     # sha256 of this file
"""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "template-method"
FAULTS = KATA / "faults.json"
CLI = ["cargo", "run", "-q", "-p", "catherine-cli", "--"]

# --- conventional arm: the pattern, executed ---------------------------------

class Spy:
    """Records the hooks a skeleton actually invokes, in order."""

    def __init__(self) -> None:
        self.trace: list[str] = []

    def record(self, step: str) -> None:
        self.trace.append(step)


class Report:
    """The abstract template: `run` is the fixed skeleton."""

    def __init__(self, spy: Spy) -> None:
        self._spy = spy

    def prepare(self) -> None:
        self._spy.record("prepare")

    def work(self) -> None:
        self._spy.record("work")

    def finish(self) -> None:
        self._spy.record("finish")

    def run(self) -> None:
        self.prepare()
        self.work()
        self.finish()


class PlainReport(Report):
    """Clean concrete subclass: overrides nothing."""


class AuditedReport(Report):
    """The defect: a hook the subclass believes the skeleton calls."""

    def audit(self) -> None:
        self._spy.record("audit")


class DriftedReport(Report):
    """The skeleton's order drifts from the declared protocol."""

    def run(self) -> None:
        self.prepare()
        self.finish()
        self.work()


class OmittingReport(Report):
    """The skeleton drops a declared step."""

    def run(self) -> None:
        self.prepare()
        self.finish()


class OneStepTemplate:
    """The near-floor control: one hook, so ordering has no content."""

    def __init__(self, spy: Spy) -> None:
        self._spy = spy

    def only(self) -> None:
        self._spy.record("only")

    def run(self) -> None:
        self.only()


PROTOCOL = ("prepare", "work", "finish")
NEAR_FLOOR_PROTOCOL = ("only",)
BASE_HOOKS = tuple(name for name in vars(Report) if not name.startswith("_"))

VARIANTS = {
    "clean": PlainReport,
    "dead-override": AuditedReport,
    "order-drift": DriftedReport,
    "omitted-stage": OmittingReport,
}
PROTOCOLS = {
    "clean": PROTOCOL,
    "dead-override": PROTOCOL,
    "order-drift": PROTOCOL,
    "omitted-stage": PROTOCOL,
    "near-floor": NEAR_FLOOR_PROTOCOL,
}


def protocol_detections(protocol: tuple[str, ...], trace: list[str]) -> list[str]:
    """Trace-position disagreements, missing protocol steps, undeclared hooks."""
    present = [hook for hook in protocol if hook in trace]
    in_order = [hook for hook in trace if hook in protocol]
    ordered = [f"hook:{e}" for e, a in zip(present, in_order) if e != a]
    ordered += [f"hook:{hook}" for hook in protocol if hook not in trace]
    extra = list(trace)
    for hook in protocol:
        if hook in extra:
            extra.remove(hook)
    return ordered + [f"hook:{hook}" for hook in extra]


def dead_hook_detections(cls: type, trace: list[str]) -> list[str]:
    """Hooks a concrete class defines that the inherited skeleton never invokes."""
    declared = [h for h in vars(cls) if not h.startswith("_") and h not in ("run", *BASE_HOOKS)]
    return [f"hook:{hook}" for hook in declared if hook not in trace]


def conventional_detections(variant: str) -> tuple[list[str], list[str]]:
    """The conventional test: run the report, compare trace to protocol.

    A hook is identified when it is out of declared order, missing from the
    trace, or defined by a concrete subclass the inherited skeleton never calls.
    """
    protocol = PROTOCOLS[variant]
    spy = Spy()
    cls = OneStepTemplate if variant == "near-floor" else VARIANTS[variant]
    cls(spy).run()
    trace = spy.trace
    detections = protocol_detections(protocol, trace)
    if variant != "near-floor":
        detections += dead_hook_detections(cls, trace)
    return list(dict.fromkeys(detections)), trace

# --- transcription arm: what the document actually records ------------------

def load(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def row_names(doc: dict, obj: str) -> list[str]:
    """Dense row ids are sorted-name order (the parser's row assignment)."""
    return sorted(doc["instance"]["rows"][obj])


def arrow_src(doc: dict, arrow: str) -> str:
    for declared in doc["schema"]["arrows"]:
        if declared["name"] == arrow:
            return declared["src"]
    raise KeyError(f"arrow {arrow!r} is not declared")


def arrow_tgt(doc: dict, arrow: str) -> str:
    for declared in doc["schema"]["arrows"]:
        if declared["name"] == arrow:
            return declared["tgt"]
    raise KeyError(f"arrow {arrow!r} is not declared")


def transcription(doc: dict) -> list[str]:
    """The hook order the document records, read only from its own tables."""
    equation = doc["schema"]["equations"][0]
    call_arrow = equation["lhs"][0]
    rank_arrow = equation["rhs"][0]
    tables = doc["instance"]["tables"]
    call_row = row_names(doc, arrow_src(doc, call_arrow))
    rank_row = {name: index for index, name in enumerate(row_names(doc, arrow_tgt(doc, rank_arrow)))}
    calls = tables[call_arrow]
    ranks = tables[rank_arrow]
    ordered = sorted(call_row, key=lambda call: rank_row[ranks[call]])
    return [calls[call] for call in ordered]


def uncovered_hooks(doc: dict) -> list[str]:
    """Hooks whose declared coverage cell is unresolved (the dangling surface)."""
    coverage = "hook_call" if "hook_call" in doc["instance"]["tables"] else "stage_action"
    table = doc["instance"]["tables"][coverage]
    hooks = row_names(doc, arrow_src(doc, coverage))
    return [hook for hook in hooks if table.get(hook) is None]

# --- kernel arm: what the surface can see -----------------------------------

def kernel_run(name: str) -> dict:
    result = subprocess.run(
        CLI + ["check", str(KATA / name), "--json"],
        cwd=ROOT,
        capture_output=True,
        text=True,
    )
    payload = json.loads(result.stdout)
    return {
        "exit": result.returncode,
        "findings": payload["data"]["findings"],
        "ok": payload["ok"],
    }


def kernel_elements(doc: dict, findings: list[dict]) -> list[str]:
    """Resolve each witness to its hook element (the pre-registered vocabulary)."""
    elements: list[str] = []
    tables = doc["instance"]["tables"]
    for finding in findings:
        witness = finding["witness"]
        if "arrow" in witness:
            source = arrow_src(doc, witness["arrow"])
            hook = row_names(doc, source)[witness["src"]]
        else:
            equation = doc["schema"]["equations"][witness["equation"]]
            call_arrow = equation["lhs"][0]
            call = row_names(doc, arrow_src(doc, call_arrow))[witness["row"]]
            hook = tables[call_arrow][call]
        elements.append(f"hook:{hook}")
    return elements

# --- comparison -------------------------------------------------------------

def entry_record(entry: dict, cache: dict[str, dict]) -> dict:
    """One report entry splicing the declarative, conventional and kernel arms."""
    fixture = entry["fixture"]
    if fixture not in cache:
        doc = load(fixture)
        run = kernel_run(fixture)
        cache[fixture] = {
            "transcription": transcription(doc),
            "uncovered": uncovered_hooks(doc),
            "kernel": run,
            "elements": kernel_elements(doc, run["findings"]),
        }
    observed = cache[fixture]
    variant = entry["code_variant"]
    baseline, trace = conventional_detections(variant) if variant else ([], [])
    elements, counted = observed["elements"], entry.get("counted", True)
    return {
        "id": entry["id"],
        "fixture": fixture,
        "code_variant": variant,
        "counted": counted,
        "transcription": observed["transcription"],
        "uncovered_hooks": observed["uncovered"],
        "conventional_trace": trace,
        "baseline_detections": baseline,
        "kernel_exit": observed["kernel"]["exit"],
        "kernel_findings": observed["kernel"]["findings"],
        "kernel_elements": elements,
        "unique": sorted(set(elements) - set(baseline)) if counted else [],
        "probe_elements": elements if not counted else [],
        "gap": entry.get("gap"),
        "note": entry.get("note"),
    }


def collapse_record(cache: dict[str, dict]) -> dict:
    """The executable core of the gap: one document, two programs, one defect."""
    clean, dead = conventional_detections("clean"), conventional_detections("dead-override")
    return {
        "document": "template.ct.json",
        "clean_trace": clean[1],
        "dead_override_trace": dead[1],
        "clean_detections": clean[0],
        "dead_override_detections": dead[0],
        "same_body": PlainReport.run is AuditedReport.run,
        "same_transcription": clean[1] == dead[1],
        "kernel_silent_on_both": cache["template.ct.json"]["kernel"]["findings"] == [],
    }


def build_report() -> dict:
    faults = json.loads(FAULTS.read_text())
    cache: dict[str, dict] = {}
    entries = [entry_record(entry, cache) for entry in faults["entries"]]
    collapse = collapse_record(cache)
    return {"kata": faults["kata"], "status": faults["status"], "collapse": collapse, "entries": entries}


def entry_problems(report: dict, faults: dict) -> list[str]:
    """Each arm of each entry against its pre-registered expectation."""
    expected = {entry["id"]: entry for entry in faults["entries"]}
    problems: list[str] = []
    for observed in report["entries"]:
        want = expected[observed["id"]]
        checks = {
            "transcription": (observed["transcription"], want["transcription_expected"]),
            "baseline": (observed["baseline_detections"], want["baseline_expected"]),
            "kernel": (observed["kernel_elements"], want["kernel_expected"]),
            "unique": (observed["unique"], want["unique_expected"]),
        }
        for label, (got, wanted) in checks.items():
            if got != wanted:
                problems.append(f"{observed['id']}: {label} {got!r} != pre-registered {wanted!r}")
    return problems


def collapse_problems(collapse: dict) -> list[str]:
    """The shared-body defect must stay invisible to both programs."""
    problems: list[str] = []
    if not collapse["same_body"]:
        problems.append("collapse: clean and dead-override no longer share the skeleton body")
    if collapse["clean_detections"] != []:
        problems.append(f"collapse: clean program is no longer clean: {collapse['clean_detections']!r}")
    if collapse["dead_override_detections"] != ["hook:audit"]:
        problems.append(f"collapse: dead override not caught: {collapse['dead_override_detections']!r}")
    if not collapse["kernel_silent_on_both"]:
        problems.append("collapse: the kernel is no longer silent on the shared document")
    return problems


def gap_close_problems(entries: list[dict]) -> list[str]:
    """The kernel must add no surface the conventional arm lacks; unique sets stay empty."""
    problems: list[str] = []
    for observed in entries:
        if observed["id"] == "dead-override" and observed["kernel_elements"] != []:
            problems.append("dead-override: the kernel must be silent (no surface)")
        if observed["unique"]:
            problems.append(f"{observed['id']}: a non-empty unique set contradicts the gap close")
    return problems


def assert_report(report: dict) -> list[str]:
    faults = json.loads(FAULTS.read_text())
    return (
        entry_problems(report, faults)
        + collapse_problems(report["collapse"])
        + gap_close_problems(report["entries"])
    )


def main(argv: list[str]) -> int:
    if "--hash" in argv:
        digest = hashlib.sha256(Path(__file__).read_bytes()).hexdigest()
        print(digest)
        return 0

    report = build_report()
    print(json.dumps(report, indent=2, sort_keys=True))

    if "--assert" not in argv:
        return 0
    problems = assert_report(report)
    if problems:
        for problem in problems:
            print(f"FAIL {problem}", file=sys.stderr)
        return 1
    print(
        "ok: conventional arm, transcription arm and kernel arm match the "
        "pre-registration; every unique set is empty",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))