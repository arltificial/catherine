# Prose requirements — backfill capture queue (the LLM's only input)

A backfill queue archives snapshots of web sources. Each capture row
stores which source it archived, the edition date it was captured for,
a content hash of the archived artifact, and optionally the set of
edition dates (aliases) it counts for. A selector names one source at
one edition date. A capture counts for a selector when the sources
match and the selector's edition date is the capture's own edition
date or one of its recorded aliases. Each partition row plans one
future capture for a selector and stores the same source and edition
date. On the serving side, each published document row stores its
edition date, and each page row stores which document — by document id
and version — it belongs to.

A selector is collided when two or more stored capture rows count for
it and those rows carry the same content hash: two stored rows for one
artifact. Model which rows store which references, and make every
stored obligation you can state checkable.

The instance rows you must use are given verbatim in
`instance-slice.json` next to this file; use them exactly, and add no
row of your own.
