#!/usr/bin/env python3
"""Arm A of the three-arm selector→capture bench (ticket catherine-kw9.11).

Version pin: committed in kw9.11; the ledger entry
`ledger/probes/bench/selector-state/0002-*.json` records this file's
commit hash as `comparison.conventional_baseline`. Census semantics are
reused verbatim from the committed, executed conventional baseline
precedent `prototype/bogart_storage_granularity.py --census`: a capture
row `(src, ed, aliases)` matches a selector `(s, e)` iff `src == s` and
(`ed == e` or `e` is in the capture's recorded edition-date alias set).

Detection criterion (pinned, single criterion,
fixtures/probes/bench/arms.md): a detection is a selector matched by
>=2 stored capture rows. No secondary modes; the duplicate-`sha256`
view is a description of the known defect shape, not a criterion.

Instance custody (arms.md scoring hygiene): this script reads
`slice.json`-shaped rows only — it never reads `census.json`; scoring
compares its output to the census ground truth out-of-band. The frozen
files are read-only for this script; `--seed` applies the deterministic
seeding of arms.md to an in-memory copy and never writes it back.

Modes:
  (default)            census detections over the given instance
  --floor              collision-free sub-slice (selectors-only instance)
  --seed s1            seeded 4th collision (arms.md S1)
  --seed s2            seeded dangling reference (arms.md S2; N/A here —
                       the census makes no referential claim)
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_instance(path: Path) -> dict:
    """Read a slice-shaped instance: captures with edition-date aliases,
    optional partitions, optional explicit selectors."""
    return json.loads(path.read_text())


def selectors_of(instance: dict) -> list[tuple[str, str]]:
    """Selector set: explicit rows plus DISTINCT (source, edition_date)
    from captures and partitions — the census rule verbatim."""
    selectors = {(s, e) for s, e in instance.get("selectors", [])}
    selectors |= {(c["source"], c["edition_date"]) for c in instance.get("captures", [])}
    selectors |= {(p["source"], p["edition_date"]) for p in instance.get("partitions", [])}
    return sorted(selectors)


def matches_for(captures: list[dict], src: str, ed: str) -> list[str]:
    """Capture ids counting for the selector, under the census match
    semantics reused verbatim from the baseline precedent."""
    return [
        c["capture_id"]
        for c in captures
        if c["source"] == src and (c["edition_date"] == ed or ed in c.get("edition_dates_alias", []))
    ]


def seed_s1(instance: dict) -> dict:
    """arms.md S1 — a seeded 4th collision: a new selector
    `bench-seed/s1 2026-11-03` holding a duplicated capture pair (copies
    of the slice's first capture, `pba/JUDICIAL 2026-11-03`, sha256
    d9ff31c3…, under the new source), plus the selector row."""
    seeded = json.loads(json.dumps(instance))
    first = seeded["captures"][0]
    for suffix, revision in (("", "bench-seed-s1"), ("-2", "bench-seed-s1-2")):
        row = json.loads(json.dumps(first))
        row["capture_id"] = f"bfc:bench-seed-s1{suffix}"
        row["source"] = "bench-seed/s1"
        row["revision"] = revision
        seeded["captures"].append(row)
    seeded.setdefault("selectors", []).append(["bench-seed/s1", "2026-11-03"])
    return seeded


def seed_s2(instance: dict) -> dict:
    """arms.md S2 — the first capture's sha256 repointed to a
    nonexistent artifact. The census makes no referential claim, so this
    is out of scope here; the run must still exit 0 with the unseeded
    output."""
    seeded = json.loads(json.dumps(instance))
    seeded["captures"][0]["sha256"] = "bench-seed-s2-dangling"
    return seeded


def evaluate(instance: dict) -> dict:
    """The census over the instance: histogram + detections
    (selectors matched by >=2 stored capture rows)."""
    captures = instance.get("captures", [])
    hist = {0: 0, 1: 0, 2: 0}
    detections = []
    for src, ed in selectors_of(instance):
        n = len(matches_for(captures, src, ed))
        hist[min(n, 2)] = hist.get(min(n, 2), 0) + 1
        if n > 1:
            detections.append({"selector": [src, ed], "match_count": n})
    return {
        "instance": str(INSTANCE_PATH),
        "captures": len(captures),
        "selectors": len(selectors_of(instance)),
        "histogram": {"0-match": hist[0], "1-match": hist[1], ">=2-match": hist[2]},
        "detections": detections,
        "detection_count": len(detections),
    }


INSTANCE_PATH = Path("<set in main>")


def main() -> None:
    global INSTANCE_PATH
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--instance", type=Path, required=True,
                    help="slice-shaped instance JSON (frozen slice, floor, …)")
    ap.add_argument("--floor", action="store_true",
                    help="evaluate the census criterion over the collision-free sub-slice")
    ap.add_argument("--seed", choices=["s1", "s2"],
                    help="apply the deterministic arms.md seeding in memory")
    args = ap.parse_args()
    INSTANCE_PATH = args.instance
    instance = load_instance(args.instance)
    if args.seed == "s1":
        instance = seed_s1(instance)
        INSTANCE_PATH = Path(f"{args.instance}+seed:s1")
    elif args.seed == "s2":
        instance = seed_s2(instance)
        INSTANCE_PATH = Path(f"{args.instance}+seed:s2")
    if args.floor:
        # Floor discipline: the sub-slice must be defect-free by
        # construction (no captures); any detection is a scoring alarm.
        if instance.get("captures"):
            sys.exit("floor instance carries captures — not collision-free")
    print(json.dumps(evaluate(instance), indent=2))


if __name__ == "__main__":
    main()
