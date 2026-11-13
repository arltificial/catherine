#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the weak-lens-set kata
(`catherine-lgl.8`) — a scripted per-pattern test over the same committed
fixtures, so each candidate structural lens's unique detections are
derived arithmetically instead of asserted.

Responsibilities: read the clean and seeded-fault fixtures, execute the
pattern's own conventional contract (NOT a call to `catherine`), and name
the fault witness in the pre-registered element vocabulary. The Flyweight
arm checks instance identity (same row), the one claim the kernel cannot
express.

Rationale: the slice's honest question is whether a weak structural lens
buys anything over the test a team would already write. Freezing that
conventional test before the probe run makes the answer arithmetic; the
near-floor control's conventional test catches everything, so its unique
set must be empty.

Usage:
  python3 prototype/weak_lens_set_baseline.py            # report JSON
  python3 prototype/weak_lens_set_baseline.py --assert   # frozen gate
  python3 prototype/weak_lens_set_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ROOT / "fixtures" / "katas" / "weak-lens-set"
FAULTS = PROBE / "faults.json"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def table(doc: dict, arrow: str) -> dict:
    return doc["instance"]["tables"].get(arrow, {})


def rows(doc: dict, obj: str) -> list[str]:
    return doc["instance"]["rows"].get(obj, [])


def check_observer(doc: dict, prefix: str) -> list[str]:
    event = table(doc, "event")
    seen = table(doc, "seen")
    subject = table(doc, "subject")
    out = []
    for idx, observer in enumerate(rows(doc, "Observer")):
        delivered = seen.get(observer)
        current = event.get(subject.get(observer))
        if delivered != current:
            out.append(f"{prefix}:seen!=subject.event@Observer#{idx}")
    return out


def check_bridge(doc: dict) -> list[str]:
    impl = table(doc, "impl")
    impl_op = table(doc, "impl_op")
    op = table(doc, "op")
    out = []
    for idx, abstraction in enumerate(rows(doc, "Abstraction")):
        if op.get(abstraction) != impl_op.get(impl.get(abstraction)):
            out.append(f"bridge:op!=impl_op.impl@Abstraction#{idx}")
    return out


def check_flyweight(doc: dict) -> list[str]:
    key = table(doc, "key")
    factory = table(doc, "factory")
    flyweight = table(doc, "flyweight")
    out = []
    for idx, client in enumerate(rows(doc, "Client")):
        if flyweight.get(client) != factory.get(key.get(client)):
            out.append(f"flyweight:duplicate-instance@Client#{idx}")
    return out


def check_command(doc: dict) -> list[str]:
    before = table(doc, "before")
    after = table(doc, "after")
    undo = table(doc, "undo")
    redo = table(doc, "redo")
    out = []
    for idx, command in enumerate(rows(doc, "Command")):
        if undo.get(after.get(command)) != before.get(command):
            out.append(f"command:after.undo!=before@Command#{idx}")
        if redo.get(before.get(command)) != after.get(command):
            out.append(f"command:before.redo!=after@Command#{idx}")
    return out


def check_prototype(doc: dict) -> list[str]:
    clone = table(doc, "clone")
    intrinsic = table(doc, "intrinsic")
    source = table(doc, "source")
    out = []
    for idx, copy in enumerate(rows(doc, "Copy")):
        if intrinsic.get(clone.get(copy)) != intrinsic.get(source.get(copy)):
            out.append(f"prototype:clone.intrinsic!=source.intrinsic@Copy#{idx}")
    return out


CHECKS = {
    "control": lambda doc: check_observer(doc, "observer-control"),
    "observer": lambda doc: check_observer(doc, "observer"),
    "bridge": check_bridge,
    "flyweight": check_flyweight,
    "command": check_command,
    "prototype": check_prototype,
}


def load(name: str) -> dict:
    return json.loads((PROBE / name).read_text())


def evaluate(fault: dict) -> dict:
    check = CHECKS[fault["pattern"]]
    clean = check(load(fault["base_fixture"]))
    seeded = check(load(fault["fixture"]))
    return {
        "id": fault["id"],
        "pattern": fault["pattern"],
        "control": fault["control"],
        "clean_detections": clean,
        "fault_detections": seeded,
        "baseline_expected": fault["baseline_expected"],
        "clean_ok": clean == [],
        "fault_ok": seeded == fault["baseline_expected"],
    }


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    faults = json.loads(FAULTS.read_text())["faults"]
    report = [evaluate(fault) for fault in faults]
    print(json.dumps(report, indent=2))
    if "--assert" not in sys.argv:
        return 0
    failures = [
        row["id"]
        for row in report
        if not (row["clean_ok"] and row["fault_ok"])
    ]
    if failures:
        print(f"baseline assertion failures: {failures}", file=sys.stderr)
        return 1
    print(f"baseline asserts {len(report)} fixtures (clean + seeded)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())