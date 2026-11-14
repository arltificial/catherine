#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the flyweight kata
(`catherine-r5k.8`) — a scripted Flyweight test over the same committed
fixtures, so the kernel arm's unique detections are arithmetic (executed
probe elements minus baseline-identified elements), never asserted.

Responsibilities: load each committed fixture, execute the pattern's own
conventional contract (NOT a call to `catherine`), name every detection in
the pre-registered element vocabulary, and compare the result against
`fixtures/katas/flyweight/faults.json`. The contract is the Flyweight
factory test a team would already write:

  handle identity     every client's handle IS the instance the factory
                      returns for that client's key (row identity, not
                      value equality);
  interning pairs     no two declared flyweight rows carry the same
                      intrinsic tuple (one shared instance per intrinsic
                      state), named as ascending unordered pairs;
  pool closure        every declared flyweight row is an instance the
                      factory can return (no leaked/orphaned instance).

Rationale: the slice's honest question is whether the kernel's sharing law
buys anything over the test a team would already write. The three
assertions are the complete conventional contract, deliberately including
the pool-level ones, so a non-empty unique set can only come from the
kernel's own surface being stronger — never from a weakened baseline. The
near-floor control's conventional test catches everything the kernel can
see, so its unique set must be empty.

Usage:
  python3 prototype/flyweight_baseline.py            # report JSON
  python3 prototype/flyweight_baseline.py --assert   # frozen gate
  python3 prototype/flyweight_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
KATA = ROOT / "fixtures" / "katas" / "flyweight"
FAULTS = KATA / "faults.json"

HANDLE = "flyweight:handle-identity@Client#{idx}"
PAIR = "flyweight:interning-pair@Flyweight#{lo}/Flyweight#{hi}"
CLOSURE = "flyweight:pool-closure@Flyweight#{idx}"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def load(name: str) -> dict:
    return json.loads((KATA / name).read_text())


def rows(doc: dict, obj: str) -> list[str]:
    return doc["instance"]["rows"].get(obj, [])


def table(doc: dict, arrow: str) -> dict:
    return doc["instance"]["tables"].get(arrow, {})


def handle_identity(doc: dict) -> list[str]:
    """Every client's handle is the row the factory returns for its key."""
    key, factory, flyweight = table(doc, "key"), table(doc, "factory"), table(doc, "flyweight")
    out = []
    for idx, client in enumerate(rows(doc, "Client")):
        expected = factory.get(key.get(client))
        if flyweight.get(client) != expected:
            out.append(HANDLE.format(idx=idx))
    return out


def interning_pairs(doc: dict) -> list[str]:
    """No two flyweight rows carry the same intrinsic tuple."""
    intrinsic = table(doc, "intrinsic")
    groups: dict[str, list[int]] = {}
    for idx, flyweight in enumerate(rows(doc, "Flyweight")):
        value = intrinsic.get(flyweight)
        if value is None:
            continue
        groups.setdefault(value, []).append(idx)
    out = []
    for group in groups.values():
        for position, lo in enumerate(group):
            for hi in group[position + 1 :]:
                out.append(PAIR.format(lo=lo, hi=hi))
    return out


def pool_closure(doc: dict) -> list[str]:
    """Every declared flyweight row is reachable from the factory."""
    produced = set(table(doc, "factory").values())
    return [
        CLOSURE.format(idx=idx)
        for idx, flyweight in enumerate(rows(doc, "Flyweight"))
        if flyweight not in produced
    ]


def conventional(doc: dict) -> list[str]:
    return handle_identity(doc) + interning_pairs(doc) + pool_closure(doc)


def evaluate(entry: dict) -> dict:
    fixture = entry["fixture"]
    detections = conventional(load(fixture))
    probe = entry["probe_expected"]
    baseline = entry["baseline_expected"]
    unique = [e for e in probe if e not in baseline]
    return {
        "id": entry["id"],
        "control": entry["control"],
        "fixture": fixture,
        "detections": detections,
        "expected": baseline,
        "probe_expected": probe,
        "unique_expected": unique,
        "detections_ok": detections == baseline,
        "unique_consistent": unique == entry["unique_expected"],
    }


def clean_report(fixtures: dict[str, list[str]]) -> list[dict]:
    report = []
    for fixture, expected in fixtures.items():
        detections = conventional(load(fixture))
        report.append(
            {
                "id": f"clean:{fixture}",
                "fixture": fixture,
                "detections": detections,
                "expected": expected,
                "detections_ok": detections == expected,
            }
        )
    return report


def main() -> int:
    if "--hash" in sys.argv:
        print(sha256_of(Path(__file__)))
        return 0
    registered = json.loads(FAULTS.read_text())
    faults = [evaluate(entry) for entry in registered["faults"]]
    clean = clean_report(registered["clean_fixtures"])
    print(json.dumps({"clean": clean, "faults": faults}, indent=2))
    if "--assert" not in sys.argv:
        return 0
    failures = [
        row["id"]
        for row in clean + faults
        if not (row["detections_ok"] and row.get("unique_consistent", True))
    ]
    if failures:
        print(f"baseline assertion failures: {failures}", file=sys.stderr)
        return 1
    print(
        f"baseline asserts {len(clean)} clean + {len(faults)} fault fixtures",
        file=sys.stderr,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())