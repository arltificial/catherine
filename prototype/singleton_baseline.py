#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the singleton gap kata
(`catherine-r5k.7`) — the test a team writing a singleton would already
have, frozen before the probe run so the probe's unique detections are
derived arithmetically instead of asserted.

Responsibilities: read the committed singleton fixtures as JSON (never
calling `catherine`), execute the singleton's own conventional contract —
exactly one `Engine` row, every access path resolving to it, both access
paths naming the *same* row, and the observed mode agreeing with the
instance's mode — and name each fault in the pre-registered element
vocabulary. `--assert` gates the run against `faults.json`.

Rationale: the kata's honest question is whether the kernel's surfaces buy
anything over the conventional count-and-identity test. Freezing that test
first makes the answer arithmetic: the near-floor control's conventional
test catches everything the kernel catches (unique must be empty), and the
cardinality faults the kernel cannot say are `killed` null results with a
named expressibility gap, never fabricated findings.

Usage:
  python3 prototype/singleton_baseline.py            # report JSON
  python3 prototype/singleton_baseline.py --assert   # frozen gate
  python3 prototype/singleton_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PROBE = ROOT / "fixtures" / "katas" / "singleton"
FAULTS = PROBE / "faults.json"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((PROBE / name).read_text())


def rows(doc: dict, obj: str) -> list[str]:
    return doc["instance"]["rows"].get(obj, [])


def table(doc: dict, arrow: str) -> dict:
    return doc["instance"]["tables"].get(arrow, {})


def check_singleton(doc: dict) -> list[str]:
    """The conventional singleton contract, in the frozen element order.

    Cardinality first (one element per fault, not per row pair), then one
    pass per client row: accessor resolution, row identity, mode coherence.
    """
    out: list[str] = []
    engines = rows(doc, "Engine")
    if len(engines) > 1:
        out.append("singleton:engine-duplicate")
    elif len(engines) == 0:
        out.append("singleton:engine-missing")

    engine = table(doc, "engine")
    fallback = table(doc, "fallback")
    mode = table(doc, "mode")
    seen = table(doc, "seen")
    for idx, client in enumerate(rows(doc, "Client")):
        primary = engine.get(client)
        secondary = fallback.get(client)
        if primary not in engines or secondary not in engines:
            out.append(f"singleton:accessor-dangling@Client#{idx}")
            continue
        if primary != secondary:
            out.append(f"singleton:engine!=fallback@Client#{idx}")
        if seen.get(client) != mode.get(primary):
            out.append(f"singleton:engine.mode!=seen@Client#{idx}")
    return out


def evaluate(entry: dict) -> list[dict]:
    report = []
    for run in entry["runs"]:
        doc = load(run["fixture"])
        report.append(
            {
                "entry": entry["id"],
                "control": entry.get("control", False),
                "fixture": run["fixture"],
                "detections": check_singleton(doc),
                "baseline_expected": run["baseline_expected"],
            }
        )
    return report


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    entries = json.loads(FAULTS.read_text())["entries"]
    report = [row for entry in entries for row in evaluate(entry)]
    print(json.dumps(report, indent=2))
    if "--assert" not in sys.argv:
        return 0
    failures = [
        f"{row['entry']}:{row['fixture']}"
        for row in report
        if row["detections"] != row["baseline_expected"]
    ]
    if failures:
        print(f"baseline assertion failures: {failures}", file=sys.stderr)
        return 1
    print(
        f"baseline asserts {len(report)} fixture runs (3 entries, 1 control)",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())