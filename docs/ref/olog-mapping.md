# Modeling ologs in catherine — the Spivak–Kent mapping

Status: v1 (2026-11-17) · audience: anyone who wants an olog whose facts
*hold*, not merely decorate — or an agent asked to encode one. Companion:
`USING.md` (the loop), `THEORY.md` (the mathematics),
`category_theoretic_modeling_methodology.md` (the procedure).

> **Epistemic status.** The runs below are **EXECUTED** (2026-11-17, debug
> build at `target/debug/catherine`), each re-runnable from the cited
> fixture. Section citations to the paper were checked against the fetched
> text of [arXiv:1102.1889](https://arxiv.org/abs/1102.1889) (v2) — not from
> memory (a prior draft miscited §2; corrected here).

## The claim

An olog (Spivak–Kent, "ontology log") is a category presentation — types,
aspects, facts — together with an instance. A catherine model document is
the same shape: schema (objects, arrows, path equations) plus instance
(rows, tables). The mapping below is native at the **semantic layer**;
the open surface is presentation and value-typed aspects (§ Open surface).

## The mapping

| Olog (arXiv:1102.1889) | catherine `.ct.json` |
|---|---|
| Type, "a ___" (§2.1) | schema object |
| Aspect, functional (§2, rules of good practice) | schema arrow |
| Fact / G-specification element (§4.2.1) | path equation, `status: accepted`, English in `rationale` |
| Candidate fact (not yet asserted) | equation with `status: proposed` |
| Instance `I: Olog → Set` — "clicking a box" (§3; §3.2 Ex. 3.2.1 as a database) | instance `rows` + per-arrow `tables` |
| Facts must hold in the intended instance | `catherine check`: a false fact is a `violation` with a witness row |
| Aspects must be functional | `check`: a missing or non-existent target is a `dangling` finding |
| "Should this diagram commute?" (fact discovery) | `catherine underconstraint` |
| Map between instances | `catherine naturality` over a declared `instance_map` (`map ∘ arrow` vs `arrow ∘ map`, per arrow and mapped row) |

The paper's rules of good practice (§2) become executed surfaces, not prose:

1. *Aspects are functional* — structural: arrows are total on rows; a
   `None` cell or unknown target is never silently tolerated.
2. *Facts hold in the intended instance* — executed: violations carry the
   row and both path values.
3. *Facts not overly specific* — `proposed` status and `underconstraint`
   surface fact-declaration as a question instead of forcing it.

## Worked example — the paper's employee olog

`fixtures/katas/olog-employee.ct.json` encodes Example 3.2.1's core: types
`Employee`/`Manager`/`Department`; aspects `has_manager`,
`is_employee`, `works_in`; the §4.2.1 fact *"the manager has the same
department as any employee"* as `has_manager∘is_employee∘works_in =
works_in`.

```
$ catherine check fixtures/katas/olog-employee.ct.json ; echo exit=$?   # exit=0
$ catherine check fixtures/katas/olog-employee-fault-cross-dept.ct.json # carol in sales, her manager eve in research
violation: eq-0 has_manager∘is_employee∘works_in ≠ works_in
  at row Employee#2: lhs=Department#0 rhs=Department#1                    # exit=3
$ catherine underconstraint <same schema, fact undeclared>
underconstraint: parallel paths has_manager∘is_employee∘works_in / works_in (hypothesized)
  should these agree?                                                     # exit=6
```

The fault fixture demonstrates rule 2 executed: the paper *asserts* the
fact; catherine names the row that breaks it. The undeclared variant
demonstrates fact *discovery*: the paper narrates "should this commute?";
the tool raises it as a finding, with source attribution.

## What catherine adds beyond the paper

- **Keys** (1.3) and **coproducts** (1.5): uniqueness and covering
  discipline olog prose leaves informal.
- **Bounded search / distinguish** (`search`, `distinguish`): smallest
  breaking instance, honest `UNKNOWN {scope}`.
- **Composition**: two documents glue by declared name, so a question that
  exists only at the seam is raised — the natural multi-author olog move.
- **Δ migration** (P7) and **schema diff** (P10): reindexing along a
  declared schema map, with loud lost-data findings; v1→v2 edits
  classified with witnesses.

## Open surface (the honest boundary)

- **Labels (the "log" itself).** Decided (`catherine-c65`, 2026-11-18):
  v1 carries no label field on objects or arrows. Olog English stays in
  equation `rationale` and in prose beside the model; `diff` already
  classifies English-only edits `cosmetic` (executed), and no executed
  check consumes a label, so a label surface earns kernel space only
  when one does. Since catherine-bak an arrow `label` key is an
  `unknown_field` rejection, so no shadow spelling exists either.
  Reopening is one additive minor (an absent section is hash-neutral)
  and is triggered by any of:
  1. a kata or workflow must render the model back with its English
     (olog/ER round-trip);
  2. an agent consumer needs per-object/arrow prose to answer an
     `underconstraint` question;
  3. diff or the ledger must detect orphaned annotation — annotation
     has to live inside the hashed artifact;
  4. a corpus E-license demands labels.
  Shape if reopened: presentation-only and hash-excluded — structural
  identity (objects/arrows/equations in `canonical_bytes`), labels
  carried by `serialize`, `diff` cosmetic, round-trip property. Not
  hash-included: rewording English must not churn sealed ledger
  `model_hashes`.
- **Unknown fields are rejected** (catherine-bak, executed: an extra
  `label` on an arrow is `error[unknown_field]`, exit 1). The parser no
  longer contradicts the spec's no-silent-drop principle; a field the
  accepted version does not define is refused by name at its line.
- **Value-typed aspects.** The paper's `string`-valued aspects (§3.2: first
  name, last name, name) map to objects-with-rows (the `Pins` convention in
  `bowling.ct.json`); typed columns (`ColumnType`) are rule-lens-only
  (`crates/catherine-core/src/model/mod.rs`).
- **Migration is Δ-only** — Σ/Π functorial migration is not shipped.

For an olog whose only need is asserting facts in labeled boxes, model it
conventionally. Catherine earns its keep the moment the facts must *hold*
in data — rule 2 execution, counterexample search, migration safety.
