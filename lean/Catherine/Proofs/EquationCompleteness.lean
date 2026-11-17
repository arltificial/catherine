import Catherine.Proofs.Basic

/-!
# Purpose: state equation completeness and the label-agreement lemma —
for every declared equation on an applicable row whose composites resolve
and disagree, `Check.check` contains the exact equation finding, labeled
`labelFor` the declaration's status. Referee:
`openspec/specs/kernel-checks/spec.md` ("Every declared equation is
executed on every applicable row", "Equation status decides the epistemic
weight of outcomes") and `openspec/specs/findings/spec.md` ("Every
finding states how it was established", "Label matches producer").

## Responsibilities: state `equation_completeness` (declared equation →
applicable row → semantic violation → the finding is in `check`, with the
composite values and the status label) and prove the label-agreement
lemma `labelFor_agrees` together with its corollaries (accepted is
`executed`, proposed is `hypothesized`, the two are never merged, the
kernel never fabricates `unknown`). Both are proved: the completeness
argument names the declared equation's index in the `mapIdx` walk and
reads the finding off `Check.equationFinding`.

## Rationale: completeness is the other half of soundness — soundness
says every finding is real, completeness says every semantic failure on an
applicable row is reported. Keys/coproduct completeness is deliberately
out of scope (design Decision 3), so no such theorem is stated. The label
lemma is separate so the `#print axioms` audit can show it is proved
without the completeness gap.
-/

namespace Catherine
namespace Proofs

/-- **Label agreement** — `labelFor` sends accepted constraints to
`executed` and proposed ones to `hypothesized`; the two labels are
distinguishable, never merged, and `unknown` is never fabricated by the
rule. Referee: `openspec/specs/kernel-checks/spec.md` ("Equation status
decides the epistemic weight of outcomes") and
`openspec/specs/findings/spec.md` ("Every finding states how it was
established", "Label matches producer"). -/
theorem labelFor_agrees (status : ConstraintStatus) :
    labelFor status =
      match status with
      | .accepted => .executed
      | .proposed => .hypothesized := rfl

/-- Accepted declarations are executed: `labelFor .accepted = .executed`. -/
theorem labelFor_agrees_accepted : labelFor .accepted = .executed :=
  labelFor_accepted

/-- Proposed declarations are hypotheses:
`labelFor .proposed = .hypothesized`. -/
theorem labelFor_agrees_proposed : labelFor .proposed = .hypothesized :=
  labelFor_proposed

/-- The two status labels are never merged into one severity. -/
theorem labelFor_agrees_never_merged :
    labelFor .accepted ≠ labelFor .proposed :=
  labelFor_never_merged

/-- The per-declaration equation family: the `mapIdx` body of
`Check.equationFindings` spelled out, so the completeness proof can name
the family belonging to a declared index. This is a local restatement (the
kernel is untouched); `equationFindings_eq` records that it agrees with
the walk by definition. -/
private def equationFamily (schema : Schema) (inst : Instance.TypedInstance)
    (index : Nat) (equation : Equation) : List Finding :=
  match equation.lhs with
  | [] => []
  | lead :: _ =>
    match schema.arrow? lead with
    | none => []
    | some arrow =>
      match Schema.rowCount? inst.rows arrow.src with
      | none => []
      | some n =>
        (List.range n).filterMap fun row =>
          Check.equationFinding inst index equation row

/-- The equation family list is exactly the declarations' images: the walk
never adds or drops a declaration. -/
private theorem equationFindings_eq (schema : Schema) (inst : Instance.TypedInstance) :
    Check.equationFindings schema inst =
      (schema.equations.mapIdx (equationFamily schema inst)).flatten := rfl

/-- **Equation completeness** — for every canonical equation index whose
declaration is `schema.equations[index]`, every applicable row
(`EquationApplies`: non-empty `lhs`, declared lead arrow, source row
space, row inside it), and every semantic violation on that row, the
exact equation finding — both composed values, that row, that index, and
the `labelFor` status label — belongs to `Check.check schema inst`.
Referee: `openspec/specs/kernel-checks/spec.md` ("Every declared
equation is executed on every applicable row"; "Equation status decides
the epistemic weight of outcomes") and `openspec/specs/findings/spec.md`
("Violation finding carries row and values"; "Label matches producer"). -/
theorem equation_completeness (schema : Schema) (inst : Instance.TypedInstance)
    (index : Nat) (equation : Equation)
    (declared : schema.equations[index]? = some equation)
    (row : RowId)
    (applies : EquationApplies schema inst equation row)
    (violation : EquationViolation inst equation row) :
    ∃ lhs rhs : RowId,
      EvalPath.compose inst equation.lhs row = some lhs ∧
      EvalPath.compose inst equation.rhs row = some rhs ∧
      lhs ≠ rhs ∧
      ({ label := labelFor equation.status
         kind := .violation (.equation
           { equation := index, row := row, lhs := lhs, rhs := rhs }) } : Finding) ∈
        Check.check schema inst := by
  obtain ⟨lead, rest, arrow, n, hlhs, harrow, hrows, hrowlt⟩ := applies
  obtain ⟨lhs, rhs, hleft, hright, hne⟩ := violation
  refine ⟨lhs, rhs, hleft, hright, hne, ?_⟩
  let finding : Finding :=
    { label := labelFor equation.status
      kind := .violation (.equation
        { equation := index, row := row, lhs := lhs, rhs := rhs }) }
  -- the walk emits exactly this finding for the declared index and row
  have hsome : Check.equationFinding inst index equation row = some finding := by
    rw [equationFinding_eq_some_iff]
    exact ⟨lhs, rhs, hleft, hright, hne, rfl⟩
  -- the declared index is a position of the equation list
  obtain ⟨hidx, hget⟩ := List.getElem?_eq_some_iff.mp declared
  -- so the finding is in the declaration's family, at the row's position
  have hfamily : finding ∈ equationFamily schema inst index equation := by
    unfold equationFamily
    rw [hlhs]
    dsimp only
    rw [harrow]
    dsimp only
    rw [hrows]
    dsimp only
    rw [List.mem_filterMap]
    exact ⟨row, List.mem_range.mpr hrowlt, hsome⟩
  -- and that family is one image of the declaration walk
  have hfindings : finding ∈ Check.equationFindings schema inst := by
    rw [equationFindings_eq, List.mem_flatten]
    refine ⟨equationFamily schema inst index equation, ?_, hfamily⟩
    rw [List.mem_mapIdx]
    exact ⟨index, hidx, by rw [hget]⟩
  -- `check` contains every family; the taxonomy sort only permutes
  rw [mem_check_iff]
  exact Or.inr (Or.inl hfindings)

end Proofs
end Catherine