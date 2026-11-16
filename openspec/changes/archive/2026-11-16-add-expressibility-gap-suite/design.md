# Design: expressibility gap suite

One feature cycle (RED → GREEN) plus one refactor cycle. Test and docs
infrastructure only — no kernel, CLI, or format behavior change.

## Decision 1 — dispositions are executable both ways

Each row pairs a fixture with a structural assertion of the *claimed*
behavior. The suite fails when observation and disposition disagree in
either direction: a `gap` row whose case now behaves differently (the
limitation was fixed) fails until the row flips to `closed` citing the
fixing commit; a `closed` row whose case regresses fails until the row
flips back with the regression recorded. Rationale: a one-way
(never-flip) suite would ossify limitations into folklore — the exact
failure mode the corpus review found (0.1.0-era conclusions presented
as current). The pattern-kata disposition table is the in-repo
precedent for the shape; the GAP/CLOSED runner from the chat is the
precedent for the flip mechanic.

## Decision 2 — registration time is truth time

Rows are registered against HEAD behavior at RED, not against the
0.1.0-era chat claims. Each row cites its source conversation for
*provenance* and the verified HEAD observation for *truth*. Where a
chat claim and HEAD disagree (e.g. unknown-section handling was reworked
into named additive-section passes; the pattern lens shipped), the row
registers `closed` with the fixing evidence — the suite starts honest.
Rationale: importing stale claims would manufacture defects; the corpus
review's CORR-002 finding is the cautionary case.

## Decision 3 — integration-test placement, not unit

Cases live as CLI-level integration tests (fixture document in,
structured observation out) because expressibility is a surface
property — "cannot be declared" is about what the CLI/format accepts,
not about a function's internals. Rejected: Rust unit tests inside
kernel crates (would test parser internals, not the user-visible
limitation) and shell-script runners (not cross-platform, not under
`cargo test`).

## Decision 4 — the grep gate is new work that keeps the table complete

The gate has **no in-repo precedent** — the pattern-kata disposition
table ships without a gate (verified: no docs gate in `justfile` /
`lefthook.yml` / `scripts/`) — so this change builds one from scratch:
a textual grep over known doc locations (README, CHANGELOG,
`docs/**`, openspec close notes) requiring any limitation-claim sentence
to name a disposition row id. Machine files (`.beads/issues.jsonl`) are
explicitly out of scope. Rationale: the suite's value is completeness
under drift; without the gate the table decays back to chat-only
knowledge. The gate is deliberately textual, matching the mechanical
character of the existing gates (pretender, bd-export freshness) — no
NLP, no cleverness.

## Decision 5 — scope: expressibility, not correctness

A row asserts "the tool cannot state/see X". Correctness bugs found
while registering cases (e.g. if a case exposes a crash or a wrong
verdict) are filed as ordinary defect-loop issues and the case waits —
or registers as a defect row in the negative-input matrix, which is
that suite's jurisdiction. Rationale: two suites with overlapping
jurisdictions decay into double bookkeeping; the negative-input matrix
(catherine-348) already owns malformed-input behavior.
