# The AI-draft → catherine-attack loop recipe

Purpose: define a reproducible protocol in which an LLM drafts a `.ct.json`
data model from prose requirements and the catherine binary — never the
LLM — decides whether the draft survives.

Responsibilities: this document specifies the loop steps, the exact
commands, file naming, the stopping rule, and the mandatory negative
controls. It does not certify any model; certification happens only when
the binary exits clean on a real run.

Rationale: the wave conclusion (E5) this recipe implements is a division
of epistemic labor — the AI proposes, the tool disposes. An LLM cannot
self-certify a model it wrote, and catherine never authors anything; each
side stays inside what it can actually know. A demo of the loop without
both negative controls proves nothing: a battery that only ever says
"clean" is indistinguishable from a battery that never fires.

Status: v1 (2026-11-25) · audience: agents and maintainers running the
draft-attack loop. Executed demo: `.wai/projects/catherine/research/
2026-11-25-ai-draft-attack-demo.md`.

## The division of labor

- **LLM arm**: reads the prose requirements, produces a draft document
  (schema + instance), revises when findings come back. Never judges its
  own draft.
- **catherine arm**: runs the battery on the draft, emits findings with
  exit codes. Never authorizes, never drafts.

## The loop

### (a) Prose in

Write the prose requirements to `fixtures/attack-demo/<name>/prose.md`.
This is the LLM's *only* input for the draft step. Keep it short and
normative ("must", "exactly one") so obligations are checkable.

### (b) LLM drafts

The LLM writes `draft-v1.ct.json`: `schema` (objects, arrows, equations —
usually empty on the first pass), `instance` (rows + tables). Naming:
`draft-v<N>.ct.json`, one file per revision. Drafts live next to the
prose under `fixtures/attack-demo/<name>/`.

### (c) Battery

Run, from the repo root (`cargo run --quiet --bin catherine --` is the
invocation; `$C` below):

```sh
C="cargo run --quiet --bin catherine --"
$C check           fixtures/attack-demo/<name>/draft-v1.ct.json --human
$C underconstraint fixtures/attack-demo/<name>/draft-v1.ct.json --max-path-length 3 --human
$C search          fixtures/attack-demo/<name>/draft-v1.ct.json --rows Loan=1 ... --human
$C naturality      fixtures/attack-demo/<name>/draft-v1.ct.json fixtures/attack-demo/<name>/target.ct.json --human   # only once an instance_map exists
```

- `check` — typing, dangling, equation violations on the declared
  instance (exit 0 clean / 3 violation / 1 labeled error).
- `underconstraint --max-path-length 3` — parallel paths with identical
  endpoints and no declared equation; *hypothesized*, exit 6, never a
  violation. This is the workhorse against LLM drafts, which reliably
  underconstrain parallel structure.
- `search --rows OBJECT=COUNT` — bounded satisfiability: a witness, a
  contradiction, or UNKNOWN with the scope. A witness is a successful
  bounded run (exit 0), not a clean bill.
- `naturality` — only if the draft declares an `instance_map`; the target
  document must have a schema *identical to the source's, equations
  included* (a differing equation set is refused as `schema_mismatch`).

Record every command and its output verbatim, with exit codes.

### (d) Findings back to the LLM

Feed the finding text and exit codes back verbatim; the LLM writes
`draft-v2.ct.json` addressing them (e.g. adding the missing path
equation with a `rationale` quoting the prose rule). No other edits.

### (e) Iterate

Repeat (c)–(d). Each iteration is one new `draft-v<N>.ct.json` file;
earlier drafts are never edited in place.

### (f) Stopping rule

Stop only when, on the current draft:

1. every battery verb exits 0 (for `search`: witness or equivalent-in-
   scope — an UNKNOWN payload means keep iterating or shrink the bounds,
   never declare success), and
2. **both negative controls have been demonstrated on this demo** (below).

"Clean battery" without demonstrated controls does not stop the loop.

## Negative controls (mandatory in every demo)

- **Overconstraint control** — take the final draft, inject one false
  accepted equation or one instance cell that breaks a declared
  constraint, save as `control-overconstraint.ct.json`, and run `check`.
  The battery **must catch it** (violation finding, exit 3). If it
  passes, the draft is unconstrained enough that the battery is vacuous —
  go back to (d).
- **Clean control** — run the full battery on the unmodified final draft.
  It **must pass silent** (exit 0 on every verb; `search`'s witness line
  is a payload, not a finding).

Record both outcomes in the demo transcript. The pairing is the point:
the same battery that stayed silent on the clean draft must fire on the
injected fault, or its silence means nothing.

## Discipline precedent

Follows `.wai/projects/catherine/research/2026-11-13-granularity-rule-
calibration.md`: executed commands with verbatim outputs, honest
epistemic labels (a bounded run is not "clean"; a hypothesized
underconstraint is not a violation), and no claim the binary did not
make.
