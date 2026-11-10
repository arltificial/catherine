# Category-Theoretic Modeling Methodology

**Cold-read reference · 2026-10-27 · Version 1.0**

## Purpose
Use category theory to understand and improve a system **before implementing it**: clarify relationships, make constraints explicit, derive consequences, expose contradictions or under-specification, and compare designs. This is a practical synthesis inspired by Michael Johnson, C. N. G. Dampney, Robert Rosebrugh, David Spivak, and AlgebraicJulia, **not** a verbatim method of any author. No prior conversation, files, or installed tools are assumed.

## Distinguish the findings
- **Invalid instance:** concrete data violate declared constraints.
- **Contradictory specification:** no possible instance/model meets the constraints (subject to stated model assumptions).
- **Under-specification:** materially different interpretations satisfy the model.
- **Missing requirement:** a real-world rule was never stated; *formal consistency alone cannot generally detect its absence*. Use adversarial examples, domain questions, and independent requirements.
- **Redundancy:** some stated structures may be derivable; eliminate only when observable meaning remains intact.

Label evidence **PROVED** (assumptions and proof), **EXECUTED** (command and output), **ILLUSTRATED** (example), **HYPOTHESIZED**, or **UNKNOWN**.

## Mathematical vocabulary
- **Object:** a type of thing (Order, Assignment).
- **Arrow** `f: A → B`: a functional relationship or attribute.
- **Composition** `g ∘ f`: following relationships in succession.
- **Commutativity:** two parallel paths must agree, `g ∘ f = k ∘ h`.
- **Instance** `I: C → Set`: interprets objects as sets and arrows as functions, respecting equations.
- **Pullback** `A ×_C B`: pairs whose images in `C` agree.
- **Monomorphism:** injective function when interpreted in `Set` (useful for uniqueness).
- **Coproduct:** tagged alternatives; to model an exhaustive classification, explicitly declare that the whole is their coproduct.
- **Sketch:** graph with designated commuting diagrams and (co)limit cones, more expressive than path equations alone.
- **Functor:** structure-preserving translation between categorical schemas.
- **Natural transformation:** coherent mapping between instances.
- **Data migration:** for schema functor `F: C → D`, `Σ_F ⊣ Δ_F ⊣ Π_F`; `Δ` reindexes, while `Σ` and `Π` are left/right Kan extensions. Neither adjunction nor functoriality alone guarantees lossless, business-valid migration.

## Iterative modeling procedure

### 0. Freeze the baseline
Write the domain problem, actors, positive/negative examples, assumptions, and observable behavior. Describe the simplest conventional solution *before* choosing categorical constructs. Record ambiguities as UNKNOWN.

### 1. Draw the semantic graph
Identify entities, attributes, and functional arrows. Use a relationship object with arrows to participants for many-to-many relations when appropriate. Separate identities, events, states, and derived quantities. A graph alone is not a complete specification.

### 2. Inspect parallel paths
For every meaningful pair of paths with the same source and target, ask: **Must they agree, might they agree, or should they differ?** Assert an equation only when the domain supports it. A false equation rules out legitimate states.

### 3. Add only justified structural constraints
Use pullbacks for exact compatibility relationships; monomorphisms for uniqueness/injectivity; coproducts for disjoint, exhaustive alternatives; products/limits for jointly characterized structures. Accompany every construction with its plain-language requirement and a valid/invalid concrete example. Temporal ordering, transactions, concurrency, and rollback require additional behavioral models.

### 4. Derive consequences and attack the model
Look for: violating instances; unexpected implied constraints; impossibility; redundant arrows; multiple materially distinct satisfying models; missing relationships suggested by examples. Ask domain experts what the model cannot answer. **Do not equate satisfiability with completeness.**

### 5. Model evolution and views
Compare versions by their valid interpretations and required observations. Where justified, construct structure-preserving schema functors and migration operations. Check what data are lost, duplicated, invented, or ambiguously updated. A canonical mathematical migration is not necessarily an acceptable business operation.

### 6. Refactor and stop
Simplify only with evidence of preserved meaning. Compare against ordinary types, assertions, SQL constraints, state machines, model checking, and property-based tests. Stop introducing categorical machinery if it produces no additional discovery, clarity, or guarantee.

## Micro-example: equipment assignments
Objects: `Assignment`, `Staff`, `Equipment`, `Unit`. Arrows: `staff: Assignment → Staff`, `equipment: Assignment → Equipment`, `staffUnit: Staff → Unit`, `equipmentUnit: Equipment → Unit`.

A proposed rule is

`staffUnit ∘ staff = equipmentUnit ∘ equipment`.

An assignment involving staff in Unit A and equipment in Unit B violates this *declared* rule. But is cross-unit assignment always forbidden? Perhaps there is an `Authorization` entity or temporary delegation. Questions about shared equipment, authorization expiry, and historical organizational changes illustrate requirements discovery through concrete counterexamples, not magical inference from the graph.

## Tool boundaries
- **Johnson / Dampney / Rosebrugh:** categorical information models using commuting diagrams, EA sketches, queries, and view-update reasoning.
- **Spivak:** schemas as categories, instances as set-valued functors, and functorial migrations `Σ ⊣ Δ ⊣ Π`.
- **AlgebraicJulia Catlab / ACsets:** efficient attributed C-set data structures and useful categorical operations. **Do not assume ACset storage enforces every declared path equation or Johnson-style (co)limit constraint. Test actual semantics.**
- **Conventional methods:** essential comparison baseline, not a straw man.

When tools matter: use official documentation, pin installed versions/commits, record exact commands and outputs, distinguish unavailable from failed checks, and never claim execution without running it.

## Kata experiment protocol
Introduce one requirement at a time. Give all methods identical information. Record predictions *before* evaluating. Introduce independent defects and holdout cases; include deliberately unfavorable katas such as FizzBuzz, behavioral cases such as Bowling/Tennis, and structural cases such as Inventory/Orders. Compare correctness, early defect discovery, useful new questions, simplification, conceptual burden, runtime and reproducibility. Avoid artificial aggregate rankings when a method cannot express required behavior.

## Required answer in a new chat
1. Problem, sources, assumptions and UNKNOWNs.
2. Smallest conventional baseline.
3. Categorical objects, arrows, equations, designated (co)limits and concrete instances.
4. Findings ledger: claim, evidence type, derivation or command/output, counterexample and confidence.
5. Discovered contradictions, under-specification and missing-domain questions.
6. Comparison with simpler alternatives, including overhead and failures.
7. Smallest next falsifiable experiment or justified stopping decision.

## Reading anchors
- Johnson & Dampney, *On the Value of Commutative Diagrams in Information Modelling* — design methodology, constraints, industrial cases.
- Johnson & Rosebrugh, *Universal View Updatability* / *Fibrations and Universal View Updatability* — views and change propagation.
- Johnson & Rosebrugh, *Sketch Data Models, Relational Schema and Data Specifications* — formal relationships between modeling approaches.
- Spivak, *Functorial Data Migration*, arXiv:1009.1166v4 (2013).
- Patterson, Lynch & Fairbanks, *Categorical Data Structures for Technical Computing*, arXiv:2106.04703v5 (2022).

**Guiding rule:** Use the smallest mathematical structure that produces falsifiable design insights; elegance by itself is not evidence of value.
