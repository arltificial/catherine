#!/usr/bin/env python3
"""Generate the storage-granularity re-model of bogart's FTS-anchor subdomain.

Purpose: execute the second-domain re-model protocol
(`.wai/projects/catherine/research/2026-11-13-granularity-rule-calibration.md`,
ticket catherine-8v9). Reads bogart's frozen serving projection READ-ONLY and
emits a `.ct.json` model whose arrows are exactly the stored references the
rule admits.

Storage model (grounded in the DDL, not the pilot's derived shape):

    backfill_read_documents(document_id, version, source, edition_date, ...)
      PRIMARY KEY(document_id, version)
    backfill_read_pages(id, document_id, version, page, status, text, ...)
      UNIQUE(document_id, version, page)

- `document`      read_pages -> read_documents. Storing columns:
                  `backfill_read_pages.document_id` + `backfill_read_pages.version`
                  (the pair is the composite FK into the documents primary key;
                  a single-column arrow cannot express the composite, so the
                  document row identity is the pair).
- `edition_date`  read_documents -> edition_date. Storing column:
                  `backfill_read_documents.edition_date`.

Dropped against the pilot model 1 (catherine-hc6): `hit_edition_date` and the
equation `document_id ; edition_date = hit_edition_date`. No per-hit edition
column exists in serving; that arrow is derived in `read_service`, so at storage
granularity the equation collapses to a tautology and the pilot's drift half is
unexpressible. This is the rule's point, and also its recall cost.

Also emits the selector census used to test the recall claim: the pilot's two
real defects (bogart-4wt: one (source, edition_date) selector matching two
captures) are a property of a query-time match, and `backfill_captures` /
`backfill_partitions` store no selector->capture column, so a storage model
cannot see them. `--census` prints the live ambiguity count.

Usage:
  python3 prototype/bogart_storage_granularity.py --slice PATH
  python3 prototype/bogart_storage_granularity.py --full PATH
  python3 prototype/bogart_storage_granularity.py --census
"""

import argparse
import json
import re
import sqlite3
import sys
from pathlib import Path

BOGART = Path("/var/home/sasha/para/areas/dev/gh/at/bogart")
SERVING = BOGART / "data/backfill/delivery-2026-11-11-november/serving.sqlite"
BATCH = BOGART / "data/backfill/corpus.sqlite"
SLICE_EDITION = "2026-11-11"  # the pilot's committed slice edition


def ident(prefix: str, raw: str) -> str:
    """catherine idents are [A-Za-z_][A-Za-z0-9_]*; bogart ids carry ':' and '-'."""
    return prefix + re.sub(r"[^A-Za-z0-9_]", "_", raw)


def document_row(document_id: str, version: str) -> str:
    return ident("doc_", document_id) + "__" + ident("v_", version)


def ro(path: Path) -> sqlite3.Connection:
    return sqlite3.connect(f"file:{path}?mode=ro", uri=True)


def documents(c: sqlite3.Connection, edition: str | None):
    sql = ("SELECT document_id, version, edition_date FROM backfill_read_documents"
           + (" WHERE edition_date=?" if edition else "") + " ORDER BY document_id, version")
    return c.execute(sql, (edition,) if edition else ()).fetchall()


def build(edition: str | None) -> dict:
    c = ro(SERVING)
    docs = documents(c, edition)
    pairs = {(d, v) for d, v, _ed in docs}
    if edition is None:
        # Full corpus: every page participates; a page whose (document_id,
        # version) pair has no documents row gets a null cell — the dangling
        # finding carries the witness.
        pages = c.execute(
            "SELECT id, document_id, version FROM backfill_read_pages ORDER BY id").fetchall()
    else:
        # Slice: pages on slice pairs only. Pages on other editions reference
        # documents outside the instance by construction — that is the slice
        # boundary, not a finding (same discipline as the pilot's slice).
        docs_in = sorted({d for d, _v, _e in docs})
        marks = ",".join("?" * len(docs_in)) or "NULL"
        pages = c.execute(
            f"SELECT id, document_id, version FROM backfill_read_pages "
            f"WHERE document_id IN ({marks}) ORDER BY id", docs_in).fetchall()
    c.close()

    rows = {"read_pages": set(), "read_documents": set(), "edition_date": set()}
    tables = {"document": {}, "edition_date": {}}
    if edition is not None:
        # Slice boundary: keep only pages whose pair is a slice pair.
        pages = [p for p in pages if (p[1], p[2]) in pairs]
    for d, v, ed in docs:
        name = document_row(d, v)
        rows["read_documents"].add(name)
        rows["edition_date"].add(ident("ed_", ed))
        tables["edition_date"][name] = ident("ed_", ed)
    for pid, d, v in pages:
        name = ident("pg_", str(pid))
        rows["read_pages"].add(name)
        tables["document"][name] = document_row(d, v) if (d, v) in pairs else None

    for obj, names in rows.items():
        dupes = len(names) - len(set(names))
        if dupes:
            sys.exit(f"{dupes} duplicate row names for object {obj}")

    return {
        "format_version": "1.0",
        "schema": {
            "objects": ["read_pages", "read_documents", "edition_date"],
            "arrows": [
                {"name": "document", "src": "read_pages", "tgt": "read_documents"},
                {"name": "edition_date", "src": "read_documents", "tgt": "edition_date"},
            ],
            # No equation survives the drop: both sides of the pilot's
            # equation collapse to the same stored column, and serving stores
            # no per-hit edition anchor to compare against.
            "equations": [],
        },
        "instance": {
            "rows": {obj: sorted(names) for obj, names in rows.items()},
            "tables": {arrow: dict(sorted(t.items())) for arrow, t in tables.items()},
        },
    }


def write(path: Path, model: dict) -> None:
    path.write_text(json.dumps(model, indent=2) + "\n")
    rows = model["instance"]["rows"]
    print(f"wrote {path}: "
          + ", ".join(f"{obj}={len(names)}" for obj, names in rows.items()))


def census() -> None:
    """Live selector ambiguity census under the pilot's selection semantics."""
    c = ro(BATCH)
    caps = []
    for cid, src, ed, meta in c.execute(
            "SELECT capture_id, source, edition_date, metadata_json FROM backfill_captures"):
        caps.append((cid, src, ed, frozenset(json.loads(meta).get("edition-dates", []))))
    selectors = set(c.execute("SELECT DISTINCT source, edition_date FROM backfill_captures"))
    selectors |= set(c.execute("SELECT DISTINCT source, edition_date FROM backfill_partitions"))
    c.close()
    hist = {0: 0, 1: 0, 2: 0}
    ambiguous = []
    for src, ed in sorted(selectors):
        matches = [cid for cid, s, e, aliases in caps
                   if s == src and (e == ed or ed in aliases)]
        hist[min(len(matches), 2)] = hist.get(min(len(matches), 2), 0) + 1
        if len(matches) > 1:
            ambiguous.append((src, ed, len(matches)))
    print(json.dumps({
        "captures": len(caps),
        "selectors": len(selectors),
        "histogram": {"0-match": hist[0], "1-match": hist[1], ">=2-match": hist[2]},
        "ambiguous": ambiguous,
        "storing_column_for_selector_match": None,
    }, indent=2))


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--slice", metavar="PATH")
    ap.add_argument("--full", metavar="PATH")
    ap.add_argument("--census", action="store_true")
    a = ap.parse_args()
    if not any((a.slice, a.full, a.census)):
        ap.error("one of --slice, --full, --census is required")
    if a.slice:
        write(Path(a.slice), build(SLICE_EDITION))
    if a.full:
        write(Path(a.full), build(None))
    if a.census:
        census()


if __name__ == "__main__":
    main()