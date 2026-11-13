#!/usr/bin/env python3
"""Emit the capture-selector slice used by the `rules --project` bridge.

Purpose: ground the step-1 acceptance run of
`.wai/projects/catherine/research/2026-11-13-post-granularity-capability-gaps.md`
in real bogart data. Reads the batch working corpus READ-ONLY and emits a
`.ct.json` model whose instance IS the capture table — one object, two value
objects, two arrows — so a projection over `backfill_captures` can be compared
with the live selector census.

Grounded in the DDL (bogart `corpus.sqlite`):

    backfill_captures(capture_id, source, edition_date, metadata_json, ...)
    backfill_partitions(source, edition_date, state, ...)

The selector universe is `SELECT DISTINCT source, edition_date` over captures
UNION partitions (the pilot's selection semantics); a selector with two or more
matching captures is the `capture_ambiguous` defect the pilot filed as
`bogart-4wt`. The storage model cannot see it (no selector->capture column), but
a projection of the capture rows onto the rule table can: two rows sharing a
(column) predicate and disagreeing on the action are an overlap, which under
`DeclaredDisjoint` is a violation.

Row naming follows the pilot's convention (`cap_bfc_<hex>`, `src_<slug>`,
`ed_<yyyy>_<mm>_<dd>`) because idents must match [A-Za-z_][A-Za-z0-9_]*.

Usage:
  python3 prototype/bogart_capture_slice.py --slice PATH [--since 2026-11-01]
  python3 prototype/bogart_capture_slice.py --census [--since 2026-11-01]
"""

import argparse
import json
import re
import sqlite3
from collections import Counter
from pathlib import Path

CORPUS = Path(__file__).resolve().parents[2] / "bogart" / "data" / "backfill" / "corpus.sqlite"


def ro() -> sqlite3.Connection:
    """Open the corpus read-only (never holds a write lock on live data)."""
    return sqlite3.connect(f"file:{CORPUS}?mode=ro", uri=True)


def slug(text: str) -> str:
    """A representative ident for a database value."""
    return re.sub(r"[^A-Za-z0-9_]+", "_", text).strip("_")


def capture_row(capture_id: str) -> str:
    return "cap_" + slug(capture_id.split(":", 1)[-1])


def source_row(source: str) -> str:
    return "src_" + slug(source)


def edition_row(date: str) -> str:
    return "ed_" + date.replace("-", "_")


def load(since: str):
    """Capture rows inside the window, in canonical row order."""
    c = ro()
    rows = c.execute(
        "SELECT capture_id, source, edition_date FROM backfill_captures "
        "WHERE edition_date >= ? ORDER BY capture_id",
        (since,),
    ).fetchall()
    c.close()
    return rows


def build(since: str) -> dict:
    rows = load(since)
    sources = sorted({source_row(r[1]) for r in rows})
    editions = sorted({edition_row(r[2]) for r in rows})
    captures = [capture_row(r[0]) for r in rows]
    return {
        "format_version": "1.0",
        "schema": {
            "objects": ["backfill_captures", "edition_date", "source"],
            "arrows": [
                {"name": "capture_source", "src": "backfill_captures", "tgt": "source"},
                {
                    "name": "capture_edition_date",
                    "src": "backfill_captures",
                    "tgt": "edition_date",
                },
            ],
            "equations": [],
        },
        "instance": {
            "rows": {
                "backfill_captures": captures,
                "edition_date": editions,
                "source": sources,
            },
            "tables": {
                "capture_source": {
                    capture_row(r[0]): source_row(r[1]) for r in rows
                },
                "capture_edition_date": {
                    capture_row(r[0]): edition_row(r[2]) for r in rows
                },
            },
        },
    }


def census(since: str) -> None:
    """Selector universe and ambiguity histogram, window and full corpus."""
    c = ro()
    caps = c.execute(
        "SELECT capture_id, source, edition_date FROM backfill_captures"
    ).fetchall()
    selectors = set(c.execute("SELECT DISTINCT source, edition_date FROM backfill_captures"))
    selectors |= set(c.execute("SELECT DISTINCT source, edition_date FROM backfill_partitions"))
    c.close()

    def report(name: str, window: bool) -> dict:
        caps_w = [r for r in caps if r[2] >= since] if window else caps
        sel_w = {s for s in selectors if s[1] >= since} if window else selectors
        matches = Counter((r[1], r[2]) for r in caps_w)
        name_map = {k: v for k, v in matches.items()}
        ambiguous = sorted(s for s in sel_w if name_map.get(s, 0) > 1)
        zero = sorted(s for s in sel_w if name_map.get(s, 0) == 0)
        return {
            "set": name,
            "since": since if window else None,
            "captures": len(caps_w),
            "capture_pairs": len(matches),
            "selectors": len(sel_w),
            "ambiguous": [{"source": s, "edition_date": d} for s, d in ambiguous],
            "zero_match": len(zero),
        }

    print(json.dumps(
        {"slice": report("slice", True), "corpus": report("corpus", False)},
        indent=2,
    ))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--slice", metavar="PATH")
    ap.add_argument("--census", action="store_true")
    ap.add_argument("--since", default="2026-11-01")
    a = ap.parse_args()
    if not any((a.slice, a.census)):
        ap.error("one of --slice, --census is required")
    if a.slice:
        model = build(a.since)
        Path(a.slice).write_text(json.dumps(model, indent=2) + "\n")
        rows = model["instance"]["rows"]
        print(f"wrote {a.slice}: " + ", ".join(f"{o}={len(n)}" for o, n in rows.items()))
    if a.census:
        census(a.since)


if __name__ == "__main__":
    main()