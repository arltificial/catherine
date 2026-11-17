import Catherine.Proofs

/-!
# Purpose: the axiom audit for the proof phase — print the axiom
dependencies of every public theorem in `Catherine/Proofs/*.lean`, so an
unfinished proof (`sorryAx`) or a user-declared `axiom` is visible in
elaboration output (`scripts/check-lean-axioms.sh`, add-lean-kernel-model
task 11; CORR-002).

## Responsibilities: import `Catherine.Proofs`, issue one `#print axioms`
line per public theorem of the four proof modules, and declare nothing
else. The audited surface is every public theorem, not a selected subset:
the label rule and composition/membership lemmas (`Basic`), the soundness
theorem and the four exclusion theorems (`Soundness`), the label-agreement
lemmas and equation completeness (`EquationCompleteness`), and the
dangling discipline bundle (`DanglingDiscipline`). `private` helper lemmas
are deliberately excluded: they are not part of the proof contract and
cannot be cited by a consumer.

With all proofs landed the command must print no `sorryAx` and no
declared axiom for any of these names; the only permitted axiom set is
`[propext, Classical.choice, Quot.sound]` (or no axioms at all).

## Rationale: the parity oracle checks semantics against Rust but says
nothing about whether the theorems are proved or assumed. This file makes
the proof claim machine-checkable, so CI fails on a `sorry` rather than
accepting decoration. Auditing only the three frozen kernel theorems
(the pre-CORR-002 state) would leave a `sorry` in any of the exclusion
theorems or shared lemmas invisible to the gate, even though the ledger
cites those theorems as landed proofs; the full list closes that gap.
-/

/-! ## `Catherine.Proofs.Basic` — the label rule and shared lemmas -/

#print axioms Catherine.Proofs.labelFor_accepted
#print axioms Catherine.Proofs.labelFor_proposed
#print axioms Catherine.Proofs.labelFor_never_unknown
#print axioms Catherine.Proofs.labelFor_never_merged
#print axioms Catherine.Proofs.compose_nil
#print axioms Catherine.Proofs.mem_check_iff
#print axioms Catherine.Proofs.equationFinding_eq_some_iff

/-! ## `Catherine.Proofs.Soundness` — the soundness and exclusion theorems -/

#print axioms Catherine.Proofs.soundness
#print axioms Catherine.Proofs.check_no_naturality
#print axioms Catherine.Proofs.check_no_redundant
#print axioms Catherine.Proofs.check_no_contradiction
#print axioms Catherine.Proofs.check_no_underconstraint

/-! ## `Catherine.Proofs.EquationCompleteness` — completeness and labels -/

#print axioms Catherine.Proofs.labelFor_agrees
#print axioms Catherine.Proofs.labelFor_agrees_accepted
#print axioms Catherine.Proofs.labelFor_agrees_proposed
#print axioms Catherine.Proofs.labelFor_agrees_never_merged
#print axioms Catherine.Proofs.equation_completeness

/-! ## `Catherine.Proofs.DanglingDiscipline` — the dangling bundle -/

#print axioms Catherine.Proofs.dangling_discipline