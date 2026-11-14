# Kata — template-method (gap study, pre-registration)

Status: **STILL GAP — no ledger entry is recorded, by design.** Beads
ticket `catherine-r5k.6` (parent epic `catherine-r5k`). The candidate law
was named *before* the fixtures were written, it was executed against the
kernel, and the close is a named-missing-law null result: the kernel can
evaluate a hand-authored call log but cannot reach the pattern's ground
truth (which subclass overrides which hook, and which hooks the inherited
skeleton actually invokes). `ledger/katas/template-method/` is deliberately
absent; the reasoning and the missing predicate are recorded here and in
the ticket report.

Everything in the sections *Fixtures* through *Kill condition* was frozen
before the run. *Observed results* is appended after it.

## Candidate law (named first, unvalidated)

> The hook-call sequence transcribed from the template method body equals
> the order the abstract protocol declares for those hooks.

On the kernel surface (`schema.objects`, `schema.arrows`,
`schema.equations`, `instance.rows`, `instance.tables`) this has exactly
one encoding — the commuting triangle

```
calls ; hook_rank = call_rank        (an accepted path equation)
```

The claim under test: whether that triangle is a *Template Method* law —
i.e. whether it decides the faults the pattern defines — or only a
self-consistency check over two tables the analyst typed by hand.

## Pattern-equivalence mapping

| Template Method element | `.ct.json` encoding | Status |
|---|---|---|
| abstract hook (a declared, overridable operation) | a `Hook` row | representable as data |
| position of a hook in the declared protocol | `hook_rank : Hook → Rank` | representable as a function |
| one message send in the template body | a `Call` row | representable as data |
| which hook that send invokes | `calls : Call → Hook` | representable as a function |
| the send's position in the body | `call_rank : Call → Rank` | representable as a function |
| "this hook is invoked at all" | `hook_call : Hook → Call` (`null` = never invoked) | representable as a partial function |
| "the body invokes each hook at its declared position" | `calls ; hook_rank = call_rank` | **the candidate law** |
| "this subclass overrides that hook" | — | **no encoding** |
| "the skeleton is inherited and the steps are deferred" | — | **no encoding** |
| "the body" (the message sends themselves) | — | **no encoding** (the tables above are a transcription *of* a body, not a body) |

## What the surface cannot say (named missing predicates)

1. **No invocation model.** The document grammar is objects (a set), arrows
   (total functions between row sets), equations (path equalities over
   arrows), keys (injectivity), rows, and tables
   (`crates/catherine-core/src/model.rs`: `Schema { objects, arrows,
   equations, keys }`). There is no term for a method, a body, or a message
   send, so nothing in a document *is* the template method; `Call` rows are
   an author's transcription of code the kernel never reads.
2. **No override/dispatch predicate.** There is no inheritance lattice, no
   abstract/concrete distinction, and no resolution rule. A document cannot
   say "`AuditedReport` overrides `audit`"; adding a `Subclass` object and
   an `overrides : Subclass → Hook` arrow would be another declared table
   with no semantics tying it to a call site, and no finding could rest on
   it. The pattern's characteristic fault — a hook a subclass overrides
   that the inherited skeleton never invokes — is therefore
   unwritable.
3. **No ordered domain.** Equations are pointwise per-row function
   equalities; `Rank` rows are labels, and the canonical model erases
   presentation names (the model hash is name-invariant —
   `crates/catherine-core/src/canonical/mod.rs`). The model therefore
   records *incidence* (`which rank row a call lands in`), not *order*: the
   sequence exists only in the analyst's naming convention (`r1 < r2 < r3`
   under sorted-name dense ids). A sequence-equality predicate as such is
   absent, and a chain encoding (`next ; at = at ; hook_next`) would still
   compare two authored chains, not a body.

The gap is not "the kernel lacks an order check"; it is that the kernel
has no access to the artifact the pattern is defined over. The two
transcription-visible faults below are parity, and the one
pattern-defining fault has no document form at all.

## Fixtures

| Fixture | What it is | Pre-registered kernel output |
|---|---|---|
| `template.ct.json` | clean: three hooks, three calls, the triangle holds | no findings (exit 0) |
| `template-fault-order-drift.ct.json` | the body invokes `finish` before `work`; `call_rank` records the drift | two `violation`s, equation 0, Call rows 1 and 2 |
| `template-fault-omitted-hook.ct.json` | the body never invokes `work`; the author declares a `hook_call` coverage cell, which is unresolved | one `dangling` on `hook_call`, Hook row of `work` |
| `near-floor.ct.json` | one hook, one call: ordering has no content at the floor | no findings (exit 0) |
| `procedural-lookalike-fault-order-drift.ct.json` | the same shape of drift in a straight-line procedure: no superclass, no override, no deferred step | two `violation`s, equation 0, Action rows 1 and 2 |

`faults.json` is the machine-readable pre-registration: per entry, the
expected transcription, the expected conventional detections, the
expected kernel elements, and (for the gap row) the missing surface.

## Fault matrix (pre-registered)

Element vocabulary: one element is one hook, spelled `hook:<Name>`. The
conventional arm identifies a hook when its executed trace violates the
declared protocol at that position, when a declared step is absent from
the trace, or when a concrete class defines a hook the inherited skeleton
never invokes. The kernel arm identifies a hook only through a witness: a
`violation` row is a `Call`/`Action` row, resolved through `calls[row]`;
a `dangling` `hook_call` cell names its `Hook` row.

| Entry | Code fault (conventional arm) | Fixture | Kernel element(s) predicted | Conventional element(s) predicted | Unique |
|---|---|---|---|---|---|
| `near-floor` (control) | none — one step, no ordering to violate | `near-floor.ct.json` | ∅ | ∅ | ∅ |
| `order-drift` | `DriftedReport.run` calls `finish` before `work` | `template-fault-order-drift.ct.json` | `hook:work`, `hook:finish` | `hook:work`, `hook:finish` | ∅ |
| `omitted-hook` | `OmittingReport.run` never calls `work` | `template-fault-omitted-hook.ct.json` | `hook:work` | `hook:work` | ∅ |
| `dead-override` | `AuditedReport` defines `audit`, which the inherited skeleton never invokes; the base body is byte-identical to the clean program's | `template.ct.json` (the same document as clean) | ∅ | `hook:audit` | ∅ |
| `procedural-lookalike` (probe) | none — this is not a template method | `procedural-lookalike-fault-order-drift.ct.json` | `hook:transform`, `hook:emit` | not run | not counted |

The `dead-override` row is the whole gap in one line: **the clean program
and the defective program share one body, therefore one faithful
transcription, therefore one identical document.** The kernel must and
does stay silent on both; the conventional arm separates them because it
runs the subclass. A "law" that cannot separate them is not a law of this
pattern.

## Near-floor control

`near-floor.ct.json` has one hook and one call. Ordering has no content
at the floor: a one-element sequence is trivially ordered, so the
conventional test has no ordering assertion to make and the equation is
trivially satisfied. Both arms are silent, and the control is vacuously
killed. Any detection here would have been a false positive.

## Conventional baseline protocol (frozen before the run)

`prototype/template_method_baseline.py` is the conventional arm and the
comparison harness in one executable:

1. **Conventional arm (executed code).** `Report.run` is the fixed
   skeleton `prepare(); work(); finish()`; `PlainReport` overrides
   nothing; `AuditedReport` defines `audit`; `DriftedReport` and
   `OmittingReport` override `run`. A `Spy` records every hook that
   actually runs. Detections are computed from the live trace only — no
   fixture is consulted for this arm.
2. **Transcription arm.** For each fixture, the hook order is read from
   the document's own `calls` and `call_rank` tables, sorted by the
   dense rank ids, plus any `hook_call` cell that is unresolved. This is
   exactly what a human reads off the body to fill the document in.
3. **Kernel arm.** `cargo run -q -p catherine-cli -- check <fixture>
   --json`; every witness is resolved mechanically to a `hook:<Name>`
   element. No finding is copied by hand.
4. **Comparison.** `unique = kernel elements − conventional elements`,
   except for entries marked `counted: false` in `faults.json` (the
   lookalike probe), whose elements are recorded and never counted.
5. `--assert` compares all three arms to `faults.json` and exits 1 on
   any drift; `--hash` prints the script's sha256.

## Pre-registered predictions (hypotheses, not results)

| Entry | Predicted kernel output | Predicted conventional output | Predicted verdict if it were a kata |
|---|---|---|---|
| `near-floor` | ∅ | ∅ | `killed` (vacuous control) |
| `order-drift` | 2 `violation`s | the same two hooks | `killed` (parity) |
| `omitted-hook` | 1 `dangling` | the same hook | `killed` (parity) |
| `dead-override` | ∅ (no surface) | `hook:audit` | **no verdict: unexpressible** |
| `procedural-lookalike` | 2 `violation`s, same shape | not run | not counted (probe) |

Predicted close: **still gap**, no ledger entry. The two expressible
faults are parity against a conventional order test, and the
pattern-defining fault cannot be written down; recording a `killed` entry
for the expressible pair would file a generic commuting-triangle check in
the ledger under a pattern name, and that is not a named law of Template
Method (the lookalike proves the check has no inheritance content).

## Kill condition / falsifiers

The gap close is refuted if any of these is observed:

- the kernel emits a finding on `template.ct.json` (the shared document of
  the clean and dead-override programs), or any witness that resolves to a
  subclass/override;
- the kernel separates the clean program from the dead-override program —
  they share one body, so they must share one document and one output;
- the lookalike's witness set differs from the template drift's (which
  would mean the firing carries template-specific content after all);
- the conventional arm fails to detect any pre-registered fault, or
  detects one outside the pre-registered element set;
- a transcription in `faults.json` disagrees with the document's own
  tables.

A fabricated finding, or a fixture edited after the run to make one arm
agree with the other, is a failure of the study.

## Observed results (recorded after the run)

**Baseline gate.** `python3 prototype/template_method_baseline.py
--assert` → exit 0, final line `ok: conventional arm, transcription arm
and kernel arm match the pre-registration; every unique set is empty`.
Script sha256 `187b885fa3f17613b9cdcc284de9ed9be1c63e188d5fe21ee8ee3522c5a1d628`.

Observed conventional detections and transcriptions:

| Variant | Live trace | Conventional detections |
|---|---|---|
| `clean` (`PlainReport`) | `prepare, work, finish` | — |
| `dead-override` (`AuditedReport`) | `prepare, work, finish` | `hook:audit` |
| `order-drift` (`DriftedReport`) | `prepare, finish, work` | `hook:work`, `hook:finish` |
| `omitted-stage` (`OmittingReport`) | `prepare, finish` | `hook:work` |
| `near-floor` (`OneStepTemplate`) | `only` | — |

| Fixture | Document transcription | Unresolved coverage cells |
|---|---|---|
| `template.ct.json` | `prepare, work, finish` | — |
| `template-fault-order-drift.ct.json` | `prepare, finish, work` | — |
| `template-fault-omitted-hook.ct.json` | `prepare, finish` | `work` |
| `near-floor.ct.json` | `only` | — |
| `procedural-lookalike-fault-order-drift.ct.json` | `fetch, emit, transform` | — |

**The collapse, executed.** `clean` and `dead-override` share the
inherited skeleton body (`PlainReport.run is AuditedReport.run` →
`True`), trace identically `prepare, work, finish`, and transcribe to the
*same* document `template.ct.json`. The conventional arm detects
`hook:audit` on the defective program and nothing on the clean one; the
kernel emits `[]` (exit 0) for the shared document. No document
distinguishes them.

**Kernel arm, verbatim** — each block is the `data.findings` payload of
the CLI's JSON envelope, byte for byte; the envelope's `ok`, `warnings`,
`meta` fields are elided.

```bash
cargo run -q -p catherine-cli -- check fixtures/katas/template-method/template.ct.json --json
# exit 0: {"data":{"findings":[]}}

cargo run -q -p catherine-cli -- check fixtures/katas/template-method/template-fault-order-drift.ct.json --json
# exit 3: {"data":{"findings":[
#   {"label":"executed","kind":"violation","witness":{"equation":0,"row":1,"lhs":1,"rhs":2}},
#   {"label":"executed","kind":"violation","witness":{"equation":0,"row":2,"lhs":2,"rhs":1}}]}}

cargo run -q -p catherine-cli -- check fixtures/katas/template-method/template-fault-omitted-hook.ct.json --json
# exit 4: {"data":{"findings":[
#   {"label":"executed","kind":"dangling","witness":{"arrow":"hook_call","src":2}}]}}
# (Hook dense ids are sorted names — finish=0, prepare=1, work=2 — so src 2 is `work`.)

cargo run -q -p catherine-cli -- check fixtures/katas/template-method/near-floor.ct.json --json
# exit 0: {"data":{"findings":[]}}

cargo run -q -p catherine-cli -- check fixtures/katas/template-method/procedural-lookalike-fault-order-drift.ct.json --json
# exit 3: {"data":{"findings":[
#   {"label":"executed","kind":"violation","witness":{"equation":0,"row":1,"lhs":1,"rhs":2}},
#   {"label":"executed","kind":"violation","witness":{"equation":0,"row":2,"lhs":2,"rhs":1}}]}}
```

The lookalike's `data.findings` array is byte-identical to the template
drift's (`equation:0`, rows `1`/`2`, `lhs/rhs` `1/2` and `2/1`). The two
documents are the same structure under different presentation names:
three objects, four arrows, one commuting triangle, the same tables. The
kernel's firing is a property of that triangle, not of inheritance.

**Verdict: still gap.** Every pre-registered prediction matched. The
candidate law is expressible only as an authored call-log agreement; it is
parity on its two visible faults (`killed` if it were a kata) and silent on
the pattern-defining fault (a dead override). The named-missing-law record
is therefore: **law** = "the hook-call sequence transcribed from the
template body equals the declared protocol order" (kernel form:
`calls ; hook_rank = call_rank`); **missing surface predicate** = an
override/dispatch predicate over a method-body model — a relation from a
concrete class's override to the inherited skeleton's invocation site —
which `.ct.json` cannot state because the grammar has objects, total
arrows, equations, and keys only, with no method, body, message send,
inheritance, or dispatch term, and because the canonical model is
name-blind so an authored rank label carries no order beyond the
analyst's naming convention. No `ledger/katas/template-method/` directory
is created: a re-open needs a new law — one that names that predicate or
a body model — not a fixture. A correction to this record is a
superseding document, never an edit of the observed section.