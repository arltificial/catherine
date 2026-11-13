#!/usr/bin/env python3
"""Purpose: the conventional baseline arm of the negative-control-bundle
kata (`catherine-lgl.9`) — a scripted per-fixture test over the same
committed models, so each kernel arm's unique detections are derived
arithmetically instead of asserted. The bundle's subject is honesty:
the kernel must stay silent where its lens is unexpressible, and must
not invent a violation on a path that legitimately differs.

Responsibilities: read the clean and seeded-fault fixtures, execute the
conventional contract of each fixture (NOT a call to `catherine`), and
name the fault witness in the pre-registered element vocabulary. The
Flyweight arm checks row identity (the same instance, not equal data);
the CRUD arm checks read-after-write; the lookalike arm declares the
transformation and records what a naive delegation lens would wrongly
flag.

Rationale: the bundle is a negative control, so the question is not
whether the kernel out-detects the scripted test but whether it remains
sound. Freezing the conventional test before the probe run makes each
disposition arithmetic: the control must self-negate, and every positive
must have an empty unique set with the conventional catch recorded.

Usage:
  python3 prototype/negative_control_bundle_baseline.py            # report JSON
  python3 prototype/negative_control_bundle_baseline.py --assert   # frozen gate
  python3 prototype/negative_control_bundle_baseline.py --hash     # sha256 pin
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
BUNDLE = ROOT / "fixtures" / "katas" / "negative-control-bundle"
FAULTS = BUNDLE / "faults.json"


def sha256_of(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def table(doc: dict, arrow: str) -> dict:
    return doc["instance"]["tables"].get(arrow, {})


def rows(doc: dict, obj: str) -> list[str]:
    return doc["instance"]["rows"].get(obj, [])


def check_control(doc: dict) -> list[str]:
    current = table(doc, "current")
    cached = table(doc, "cached")
    source = table(doc, "source")
    out = []
    for idx, cache in enumerate(rows(doc, "Cache")):
        if current.get(source.get(cache)) != cached.get(cache):
            out.append(f"cache:source.current!=cached@Cache#{idx}")
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


def check_crud(doc: dict) -> list[str]:
    created = table(doc, "created")
    read_back = table(doc, "read_back")
    stored_in = table(doc, "stored_in")
    target = table(doc, "target")
    out = []
    for idx, request in enumerate(rows(doc, "Request")):
        if read_back.get(request) != created.get(request):
            out.append(f"crud:read_back!=created@Request#{idx}")
        if stored_in.get(created.get(request)) != target.get(request):
            out.append(f"crud:stored_in!=target@Request#{idx}")
    return out


def check_lookalike(doc: dict) -> list[str]:
    charged = table(doc, "charged")
    discount = table(doc, "discount")
    list_price = table(doc, "list")
    out = []
    for idx, product in enumerate(rows(doc, "Product")):
        if charged.get(product) != discount.get(list_price.get(product)):
            out.append(f"lookalike:charged!=list.discount@Product#{idx}")
    return out


def naive_lookalike(doc: dict) -> list[str]:
    """What a naive decorator-delegation lens would wrongly flag."""
    charged = table(doc, "charged")
    list_price = table(doc, "list")
    out = []
    for idx, product in enumerate(rows(doc, "Product")):
        if charged.get(product) != list_price.get(product):
            out.append(f"lookalike:charged!=list@Product#{idx}")
    return out


CHECKS = {
    "control": check_control,
    "flyweight": check_flyweight,
    "crud": check_crud,
    "lookalike": check_lookalike,
}


def load(name: str) -> dict:
    return json.loads((BUNDLE / name).read_text())


def evaluate(fault: dict) -> dict:
    check = CHECKS[fault["pattern"]]
    clean = check(load(fault["base_fixture"]))
    seeded = check(load(fault["fixture"]))
    report = {
        "id": fault["id"],
        "pattern": fault["pattern"],
        "control": fault["control"],
        "clean_detections": clean,
        "fault_detections": seeded,
        "baseline_expected": fault["baseline_expected"],
        "clean_ok": clean == fault["clean_expected"],
        "fault_ok": seeded == fault["baseline_expected"],
        "naive_ok": True,
    }
    if fault["pattern"] == "lookalike":
        naive_clean = naive_lookalike(load(fault["base_fixture"]))
        naive_fault = naive_lookalike(load(fault["fixture"]))
        report["naive_clean"] = naive_clean
        report["naive_fault"] = naive_fault
        report["naive_ok"] = (
            naive_clean == fault["naive_lens_expected"]
            and naive_fault == fault["naive_lens_expected"]
        )
    return report


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
        if not (row["clean_ok"] and row["fault_ok"] and row["naive_ok"])
    ]
    if failures:
        print(f"baseline assertion failures: {failures}", file=sys.stderr)
        return 1
    print(f"baseline asserts {len(report)} fixtures (clean + seeded)", file=sys.stderr)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())