#+TITLE: Catherine — general capability evaluation (what can be modeled, what cannot)
#+DATE: 2026-11-24
#+OPTIONS: toc:2

* Status
Evidence for the decision memo (~notes/decision-memo-2026-11-24.org~). Where
this note and the memo differ, the memo governs.

* Scope and method
Evaluated: catherine 0.4.1 (repo = ~catherine, HEAD 4801b4b, /target/release/catherine).

Method: live runs of the shipped CLI against (a) the repository's own fixtures,
(b) five fresh probe documents written for this evaluation (/tmp/probe), and
(c) the repository's executable limitation suite. Every row below cites the
invocation that produced it.

Supporting integrity checks run clean:
- ~just check-gap-claims~ → "every limitation claim names a disposition row id"
- ~cargo test -p catherine-cli --test expressibility_gaps~ → 6 passed
- ~ah check~ → ok, 0 findings
- determinism: two JSON runs of the same 2000-row model are byte-identical
- speed: 8 ms for a 2000-row / 2000-cell model check (offline, local)

* Verdict in one paragraph
Catherine is a *constraint execution kernel over finite relational instances*,
not a general modeling system. It is excellent at one thing: turning a declared
diagram into executed obligations and reporting each failure with a witness
row. It is weak — by design, mostly — at everything that is not a path
agreement over stored references: cardinality, existence, arithmetic, time,
partiality, and value semantics. The interesting result is not the list of
gaps; it is that *the gaps are themselves executable*, registered in
~docs/ref/expressibility-dispositions.md~ with a live case per row. A model of
the tool's limits is therefore maintained by the same machinery as the rest of
the tool. That is the single strongest design decision in the repository.

* What can be modeled (with live evidence)

** Path agreement / commutative diagrams — the core
| Surface | Command | Observed |
|---|---|---|
| Equation violation | ~catherine check fixtures/bowling.ct.json~ | exit 3, "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item at row Assignment#3: lhs=Unit#1 rhs=Unit#2" |
| Reference integrity | ~check fixtures/dangling.ct.json~ | exit 4, "dangling: arrow in_section at row Block#0" |
| Parallel-path discovery | ~underconstraint fixtures/analysis_underconstraint.ct.json~ | exit 6, "underconstraint: parallel paths f∘g / h (hypothesized) — should these agree?" |
| Derivability / redundancy | ~redundant fixtures/analysis_redundant.ct.json~ | exit 7, "redundant: f∘g∘k derivable as n" |
| Behavioral equivalence in a declared scope | ~distinguish ... --observe 'f@a#0'~ | exit 0, "distinguishable: f @ a#0 left=b#0 right=b#1" |
| Reachability | ~query --seed/--follow~ | typed closure; dangling values never followed |

Interpretation: the schema-as-category + instance-as-functor-to-Set framing is
operationally sound for *referential* obligations. Anything expressible as
"these two stored paths must land on the same row" is checked exactly, fast,
and with a witness.

** Closed variants (coproducts), pullback squares, naturality, composition
| Claim | Command | Observed |
|---|---|---|
| Variant coverage/disjointness | ~check fixtures/coproduct/rogue-voucher.ct.json~ | exit 3, "violation: coproduct-0 Method(card_inj∘transfer_inj∘cash_inj) at row Method#3: uncovered" |
| Pullback square | ~check fixtures/pullback_violation.ct.json~ | exit 3, "violation: pullback-0 P(p1, p2 → f, g) at row P#0: lhs=C#0 rhs=C#1" (three verdicts: commutes / covers / unique) |
| Naturality of a row map | ~naturality fixtures/naturality/crm.ct.json fixtures/naturality/warehouse_break.ct.json~ | exit 3, "violation: naturality of account_owner at row account#2: lhs=owner#1 rhs=owner#0" |
| Cross-document seam | ~check composition_billing.ct.json composition_shipping.ct.json composition_seam_claim.ct.json~ | exit 3, "violation: eq-0 invoice_region ≠ dispatch_region at row Order#2" + ~source~ attribution naming all three documents |

These four are the "unexpressible → expressible" wins of the last cycle, and
they are the tool's most distinctive contribution: a seam between two documents
is a first-class, executable, attributed claim.

** Evolution / governance surfaces
- Δ migration: ~migrate fixtures/probes/migration/floor-visible-loss.ct.json~ →
  exit 8, 6 loud ~lost-row~ findings (one per dropped row). This is the axis
  with no conventional counterpart.
- Schema diff: ~diff fixtures/schema_diff/*~ → ~cosmetic~ (0 findings),
  ~accreted~ ("added: key A(f)"), and ~weakened~ (exit 3, "a v1-enforced
  constraint is not enforced by v2" with a separating witness). Note the
  bound sensitivity: with no ~--rows~ bound, ~weakened~ returns ~unknown~
  (budget expired after 0 candidates) — the honest default, but a sharp edge.
- Decision-table rule lens: ~rules fixtures/rules_clean.ct.json --values n=1,10,20~ →
  "clean — no findings over the declared domain".
- Pattern lenses: ~pattern <model> --list~ → 12 registered lenses, each
  printing its abstraction, law kind, and *scope* (e.g. composite:
  "identity/totality equations; associativity unexpressible"; visitor:
  "no evaluation semantics").

* What cannot be modeled (with live evidence)
Each row is a real, registered limitation (~docs/ref/expressibility-dispositions.md~)
or a probe I ran during this evaluation. Status: gap = limitation live at HEAD.

** Logical / algebraic structure
| Limitation | Evidence | Missing capability |
|---|---|---|
| Identity / inverse laws unstatable | ~check gap_suite/identity_involution.ct.json~ → exit 1, "error[syntax] equation path must be non-empty" | an identity path form (empty side or named identity arrow) |
| General limit cones absent | ~schema.cones~ rejected as ~unknown_field~ | cone construction (pullback-square form only) |
| Associativity needs hand-built carrier rows | ~associativity_omitted_triples.ct.json~ → exit 0 clean; ~associativity_scaffolded.ct.json~ → exit 3, 18 violations | finite-carrier operation declaration with executable associativity |
| Injectivity / disequality absent (v1 language) | ~crates/catherine-analysis/src/search.rs~ module docs: "pure path equalities are collapse-satisfiable (one-row-collapse satisfies every equation)" | non-collapse-satisfiable constraint forms |

** Cardinality, existence, aggregates
| Limitation | Evidence | Missing capability |
|---|---|---|
| Uniqueness ("at most one order per customer") is invisible | probe ~uniqueness.ct.json~ (two orders → same customer) → exit 0, zero findings, no warning | cardinality/existence predicates over rows |
| Existence of a relation cannot be required | same probe family: no surface states "every Customer has an Order" | existential constraints |
| Arithmetic / aggregate invariants unstatable | cells hold *row references only*; a value must be modeled as an object row (probe ~attr.ct.json~ with ~Amount/a100~ is accepted, but ~"100"~ as a literal is not a value) | a value domain with semantics |
| Set-valued cells / many-to-many unstatable | probe ~m2m.ct.json~ (cell value ~["t1","t2"]~) → exit 1, "error[syntax] invalid type: sequence, expected a string" | multi-valued cells (today: two arrows + an agreement equation) |
| Identifier vocabulary constrained | probe with row id ~TYPO-99~ → exit 1, "error[invalid_ident] … must match [A-Za-z_][A-Za-z0-9_]*" | a data-value type distinct from identifiers |

** Time, history, partiality, evolution of the schema map
| Limitation | Evidence | Missing capability |
|---|---|---|
| History-dependent behavior (accumulation/fold) unstatable | probe ~history.ct.json~ (frames chained by ~frame_prev~) → only a dangling and parallel-path questions; no fold semantics | temporal/accumulator structure (recorded limit, kata-1/5) |
| Optional references reported as errors, not modeled | ~check gap_suite/optional_reference.ct.json~ → exit 4, "dangling: arrow address at row Person#1" | optional-arrow declaration with defined semantics |
| Vacuous clean instances | ~check gap_suite/zero_rows_instance.ct.json~ → exit 0, no findings, *no warning* | a vacuity diagnostic for equations over zero rows |
| Schema-map functoriality unchecked | ~migrate gap_suite/nonfunctorial_schema_map.ct.json~ → ~data.lost: []~ although f:A→B maps to h:Y→Y | endpoint + equation-preservation checking of mapped arrows |
| Dropped equations/pullbacks migrate silently | ~migrate gap_suite/dropped_equation_migration.ct.json~ and ~dropped_pullback_migration.ct.json~ → ~lost: []~ | lost-constraint findings on migrate |
| Observation scope for ~distinguish~ is manual | ~distinguish~ without ~--observe~ → exit 2 usage error; with one observation → "equivalent-in-scope" even though another arrow differs | automatic/reported observation scope (enumeration to a declared depth) |

** Deliberate non-goals (not gaps — the tool says "model this conventionally")
- Rule-based logic at or below the triviality floor: ~check fixtures/katas/fizzbuzz.ct.json~ → exit 0, zero findings, zero warnings. The silence *is* the product feature (release-gated negative control).
- No theorem proving, no rewriting, no wiring diagrams, no runtime monitor, no persistence/server, no view-update checking, no Σ/Π migrations (Δ only).

* Meta-observations (things the evaluation surfaced that are not in the docs)
1. **Two surfaces default to ~unknown~.** ~search~ on the repository's own
   contradiction fixture returns "unknown — explored 256 candidates";
   ~diff~ on the ~weakened~ fixture returns "unknown — budget expired after 0
   candidates" unless an explicit ~--rows~ bound is supplied. Honest, but the
   default is uninformative: the first-time user reads "unknown" as a broken
   tool. A default row bound derived from the document's instance would make
   the default path productive.
2. **The contradiction producer is nearly vacuous in this language.** Because
   path equalities are collapse-satisfiable, a proven contradiction arises
   "exactly when the bounds make full resolution impossible" (the crate's own
   words). So ~search~ mostly returns a witness or an UNKNOWN; its contradiction
   finding is a scope artifact rather than a modeling result. This is honestly
   documented, but it means the exit-code-5 class has little real traffic.
3. **The discovery surface can be noisy.** On the history probe, ~underconstraint~
   asked "should ~frame_prev~ and ~frame_prev∘frame_prev~ agree?" — a question
   generated by path length, not by domain ambiguity. Discovery questions need
   a confidence/scoping signal to stay credible.
4. **The modeling economy is subtler than "objects and arrows".** Since values
   are row references only, every attribute becomes an object with rows; a
   model of a domain with 30 columns of scalar data becomes 30 objects. The
   methodology's "one arrow per stored, independently writable reference" rule
   (USING Recipe A) is the mitigation, but the tool itself gives no signal that
   you have crossed from "modeling the system" to "modeling the storage plan".
5. **The gap registry is the product's real differentiator.** A competitor can
   copy the checker; nobody else ships a live, executable table of their own
   limits with a case per row and a CI gate that fails if a row goes stale
   (~just check-gap-claims~, ~cargo test --test expressibility_gaps~).

* Practical guidance
Use catherine when the claim is *referential*: two stored paths must agree, a
variant set must be closed, a seam must agree, a schema edit must not lose
data, a decision table must be gap/overlap free. Do not reach for it when the
claim is about cardinality, arithmetic, time, optionality, or the *absence* of
a requirement — those need conventional tests, and the tool will not tell you
that it cannot help unless the limitation is registered (today it is, and the
registry is the thing to consult).

* Ranked next capabilities (if the limit registry is to keep shrinking)
1. Identity path form (unlocks every inverse/identity law; currently a syntax error).
2. Cardinality/existence predicates (uniqueness, at-least-one, at-most-one) — the largest class of unmodelable-but-real claims.
3. Vacuity diagnostic on zero-row instances (cheap; turns a silent clean into an honest one).
4. Functoriality + lost-constraint checks on ~migrate~ (two registered gaps, one mechanism).
5. Default row bounds for ~search~/~diff~ (turns the default path from ~unknown~ into a result).
6. Multi-valued cells / a distinct value type (scalars, sets, decimals without inventing objects for them).

* Reproduction
All rows are reproducible from the repository root:
  target/release/catherine <command> <fixture> [--json|--human]
  cargo test -p catherine-cli --test expressibility_gaps
  just check-gap-claims
Probe documents used for the fresh rows are described inline above; they are
throwaway /tmp/probe/*.ct.json files, not repository fixtures.

