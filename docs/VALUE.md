# Catherine — the value surface: three axes and the gate

Status: normative · 2026-11-24 · distills the three-axis-gate decision
(`.wai/projects/catherine/designs/2026-11-24-design-the-three-axis-gate-
what-catherine-wil.md`). This file is the home of the "state-shape domain"
definition; every other document that says "state-shape" means this.

We do not widen what catherine can express. We make the three axes it already
serves honest and enforceable, and we leave the constraint language where it
is. The differentiator that is real and hard to copy is not a feature list —
it is that catherine's *limits* are executable.

## The three axes

| Axis | Serves | Done when |
|------|--------|-----------|
| **A — referential agreement** | Path equations, keys, coproducts, pullbacks, naturality, joint composition | Every stated referential claim is executed with a witness; what the language leaves unstated is named in a registry row |
| **B — schema evolution** | `migrate` (Δ), `diff`, the schema-map parser | No change can silently produce a wrong instance or a lost constraint |
| **C — executable limits** | The expressibility registry, the gap suite, the negative-control bundles | Every limitation claim names a registry row and a live case; a stale row fails CI |

## The gate

A proposed capability that serves none of A, B or C is refused by default.
Beyond that the backlog does not grow by default: a new item must name the
axis it serves, or be refused.

## "State-shape domain", defined once

A domain whose claims are about the *shape and agreement of stored
references* — which rows exist, which references they carry, and which
composed paths must agree — rather than about computation over values
(arithmetic, time, decisions over data).

Rule of thumb:

- "these two stored paths must land on the same row" → in scope (axis A);
- evolution of the schema or its instances → axis B;
- values, time or cardinality → out of scope, named in the registry as a
  gap (axis C).

## The executable half of the constraint set

A capability with no registry row has not been stated. The registry and its
dispositions — what is closed, what is a gap, and the live case behind each —
live in [`docs/ref/expressibility-dispositions.md`](ref/expressibility-dispositions.md).
