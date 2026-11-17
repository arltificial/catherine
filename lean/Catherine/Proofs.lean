import Catherine.Proofs.Basic
import Catherine.Proofs.Soundness
import Catherine.Proofs.EquationCompleteness
import Catherine.Proofs.DanglingDiscipline

/-!
# Purpose: aggregate the proof-phase modules — the semantic predicates and
shared lemmas (`Catherine.Proofs.Basic`), soundness, equation
completeness with the label-agreement lemma, and the dangling discipline
bundle — behind one import for the axiom audit and for `Catherine.lean`.

## Responsibilities: import the four proof modules in dependency order;
stay a pure aggregation point — no definitions or theorems belong here.

## Rationale: `Catherine.Axioms` and any later proof-phase consumer need a
single import target, exactly as the kernel root `Catherine.lean` keeps
the semantics behind one surface.
-/