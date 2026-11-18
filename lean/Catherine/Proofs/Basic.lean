import Catherine.Schema
import Catherine.Findings
import Catherine.Instance
import Catherine.EvalPath
import Catherine.Check

/-!
# Purpose: the semantic notions the three kernel theorems are stated
against, plus the shared lemmas that both the statements and the proof
phase use — semantic equation violation and applicability, key-tuple
collisions, coproduct complete-image accounting, and unresolved-cell
(dangling) semantics. These are *spec* predicates: they say what the
finding witnesses mean, independently of how `Check.check` walks the
instance, so a theorem expressed over them proves the kernel contract
(`openspec/specs/kernel-checks/spec.md`,
`openspec/specs/findings/spec.md`), not a restatement of the walk.

## Responsibilities: define each predicate so it is the semantic reading
of one finding family — `EquationViolation`/`EquationApplies` for the
equation walk's applicability guard and disagreement test,
`KeyCollision`/`KeyTupleAt` for a resolved, equal key tuple on two
ascending rows, `CoproductRowFailure` for exactly the
`Check.coproductDeclaration` decision (complete-image accounting with
the partial-image mode), `PullbackCommutesFailure`/`PullbackCoversFailure`/
`PullbackUniqueFailure` for the square's three verdicts, and
`CellUnresolved`/`TypingViolatedAt` for a
`none` table cell no typing violation accounts for. Also expose the raw
projected cell (`RawCell`) the builder consumes and prove the easy
shared lemmas (label rule, empty-path composition, `check` membership,
the `equationFinding` iff, the pullback membership shape).

## Rationale: the frozen kernel modules stay untouched (no semantic
refactor, so parity evidence is unchanged); the proofs hang off these
definitions instead. `RawCell` repeats the `buildArrow` `take` lookup
because the discipline lemmas must talk about the *raw* cell and the
*built* table at the same row, which no kernel function exposes. The
`equationFinding_eq_some_iff` bridge is proved here (not left as a gap)
because it is definitional: it only case-splits the two composites.
-/

namespace Catherine
namespace Proofs

/-! ## The label rule (findings spec: "Every finding states how it was
established") -/

/-- `labelFor` maps accepted constraints to `executed`. -/
@[simp] theorem labelFor_accepted : labelFor .accepted = .executed := rfl

/-- `labelFor` maps proposed constraints to `hypothesized`. -/
@[simp] theorem labelFor_proposed : labelFor .proposed = .hypothesized := rfl

/-- The label rule never fabricates the scope-bounded `unknown` label. -/
theorem labelFor_never_unknown (status : ConstraintStatus) :
    labelFor status ≠ .unknown := by
  cases status <;> simp [labelFor]

/-- Accepted and proposed findings are never merged: the two labels differ.
This is the executable reading of the findings spec's "SHALL never emit
`proved`/`illustrated`" / "SHALL be distinguishable" requirements. -/
theorem labelFor_never_merged : labelFor .accepted ≠ labelFor .proposed := by
  simp

/-! ## Semantic equations (kernel-checks: "Every declared equation is
executed on every applicable row") -/

/-- A semantic equation violation on a row: both composites resolve to a
row and the rows disagree. Short-circuited composites are *not* semantic
violations — their cell carries a dangling or typing violation instead. -/
def EquationViolation (inst : Instance.TypedInstance) (equation : Equation)
    (row : RowId) : Prop :=
  ∃ lhs rhs : RowId,
    EvalPath.compose inst equation.lhs row = some lhs ∧
    EvalPath.compose inst equation.rhs row = some rhs ∧
    lhs ≠ rhs

/-- A row is applicable to an equation when the equation's `lhs` is
non-empty, its lead arrow is declared, the lead arrow's source object has
a dense row space in the instance, and `row` lies inside it. This is the
exact guard `Check.equationFindings` applies before running the two
composites. -/
def EquationApplies (schema : Schema) (inst : Instance.TypedInstance)
    (equation : Equation) (row : RowId) : Prop :=
  ∃ lead rest arrow n,
    equation.lhs = lead :: rest ∧
    schema.arrow? lead = some arrow ∧
    Schema.rowCount? inst.rows arrow.src = some n ∧
    row < n

/-- The empty path evaluates to the identity on its start row. -/
@[simp] theorem compose_nil (inst : Instance.TypedInstance) (start : RowId) :
    EvalPath.compose inst [] start = some start := rfl

/-- Membership in a `flatten`ed `mapIdx` run: some declaration of `l`
(indexed by its position) emits `b`. Shared by the family membership
bridges (`Soundness`) and the pullback shape lemma below. -/
theorem mem_flatten_mapIdx {α β : Type} (f : Nat → α → List β)
    (l : List α) (b : β) :
    b ∈ (l.mapIdx f).flatten ↔ ∃ i a, l[i]? = some a ∧ b ∈ f i a := by
  rw [List.mem_flatten]
  constructor
  · rintro ⟨body, hbody, hmem⟩
    rw [List.mem_mapIdx] at hbody
    obtain ⟨i, hi, rfl⟩ := hbody
    exact ⟨i, l[i], List.getElem?_eq_getElem hi, hmem⟩
  · rintro ⟨i, a, ha, hmem⟩
    rw [List.getElem?_eq_some_iff] at ha
    obtain ⟨hi, rfl⟩ := ha
    exact ⟨f i l[i], List.mem_mapIdx.mpr ⟨i, hi, rfl⟩, hmem⟩

/-- A `contains` test that has failed means the element is absent. -/
theorem contains_eq_false_iff_not_mem {α : Type} [BEq α] [LawfulBEq α]
    (l : List α) (a : α) : (l.contains a = false) ↔ a ∉ l := by
  rw [← List.contains_iff_mem]
  cases h : l.contains a <;> simp_all

/-- Membership in `check` unfolds to membership in one of the five
finding families; the taxonomy sort only permutes, it never invents. -/
@[simp] theorem mem_check_iff (schema : Schema) (inst : Instance.TypedInstance)
    (finding : Finding) :
    finding ∈ Check.check schema inst ↔
      finding ∈ Check.danglingFindings inst
          (inst.typingViolations.map fun violation =>
            (violation.arrow, violation.src)) ∨
      finding ∈ Check.equationFindings schema inst ∨
      finding ∈ Check.keyFindings schema inst ∨
      finding ∈ Check.coproductFindings schema inst ∨
      finding ∈ Check.pullbackFindings schema inst := by
  simp [Check.check, List.mem_append, or_assoc, or_left_comm, or_comm]

/-- One equation's finding on one row, characterized semantically: it
exists iff both composites resolve and disagree, and then it is exactly
the equation witness with `labelFor` the declaration's status. -/
theorem equationFinding_eq_some_iff (inst : Instance.TypedInstance)
    (index : Nat) (equation : Equation) (row : RowId) (finding : Finding) :
    Check.equationFinding inst index equation row = some finding ↔
      ∃ lhs rhs : RowId,
        EvalPath.compose inst equation.lhs row = some lhs ∧
        EvalPath.compose inst equation.rhs row = some rhs ∧
        lhs ≠ rhs ∧
        finding =
          { label := labelFor equation.status
            kind := .violation (.equation
              { equation := index, row := row, lhs := lhs, rhs := rhs }) } := by
  cases hleft : EvalPath.compose inst equation.lhs row with
  | none =>
    simp [Check.equationFinding, hleft]
  | some lhs =>
    cases hright : EvalPath.compose inst equation.rhs row with
    | none =>
      simp [Check.equationFinding, hleft, hright]
    | some rhs =>
      by_cases h : lhs = rhs
      · subst h
        simp [Check.equationFinding, hleft, hright]
      · have hbeq : (lhs == rhs) = false := by
          simp [h]
        simp [Check.equationFinding, hleft, hright, hbeq, h, eq_comm]

/-! ## Semantic keys (kernel-checks: "Declared keys are executed as
injectivity checks") -/

/-- A row's resolved key tuple for a key: the complete tuple of resolved
cell values under `Check.keyTables`/`Check.keyTuple`. -/
def KeyTupleAt (schema : Schema) (inst : Instance.TypedInstance)
    (key : Key) (row : RowId) (tuple : List RowId) : Prop :=
  ∃ tables, Check.keyTables schema inst key = some tables ∧
    Check.keyTuple tables row = some tuple

/-- A key collision: two distinct ascending rows of the key's object
resolve every key cell and agree on the tuple. -/
def KeyCollision (schema : Schema) (inst : Instance.TypedInstance)
    (key : Key) (lo hi : RowId) (tuple : List RowId) : Prop :=
  lo < hi ∧
    KeyTupleAt schema inst key lo tuple ∧
    KeyTupleAt schema inst key hi tuple

/-! ## Semantic coproducts (kernel-checks: "Accepted coproducts execute
coverage with witnesses" / "Coproduct disjointness violations name the
colliding injections" / "Unresolved injection cells suppress coverage
only") -/

/-- The complete injection images of a declaration's tables, in declared
order — exactly the images `Check.coproductDeclaration` accounts for. -/
def injectionImages (tables : List (Ident × List (Option RowId))) :
    List Check.InjectionImage :=
  tables.filterMap fun entry => Check.completeImage entry.1 entry.2

/-- The semantic coproduct failure on one row: exactly the decision
`Check.coproductDeclaration` performs. With every image complete a row
fails iff it is not covered exactly once (coverage/disjointness); with a
partial image only a row covered by at least two *complete* images is
decidable (disjointness survives, coverage is suppressed). The
`injections` argument is the witness list `coveringInjections` names. -/
def CoproductRowFailure (schema : Schema) (inst : Instance.TypedInstance)
    (coproduct : Coproduct) (row : RowId) (injections : List Ident) : Prop :=
  ∃ tables images,
    Check.injectionTables schema inst coproduct = some tables ∧
    images = injectionImages tables ∧
    injections = Check.coveringInjections images row ∧
    ((images.length = tables.length ∧ injections.length ≠ 1) ∨
      (images.length ≠ tables.length ∧ 2 ≤ injections.length))

/-- A coproduct row is applicable when its object has a dense row space
and the row lies inside it — the guard before the per-row decision. -/
def CoproductRowApplies (inst : Instance.TypedInstance)
    (coproduct : Coproduct) (row : RowId) : Prop :=
  ∃ n, Schema.rowCount? inst.rows coproduct.object = some n ∧ row < n

/-! ## Semantic pullbacks (kernel-checks: "Declared pullbacks are executed
as commutes, covers, and unique") -/

/-- A declared pullback's commutes failure on one apex row: the declaration
resolves, the row's legs resolve to a projection, and the two composed
cospan values exist and disagree. -/
def PullbackCommutesFailure (schema : Schema) (inst : Instance.TypedInstance)
    (pullback : Pullback) (row lhs rhs : RowId) : Prop :=
  ∃ square projection,
    Check.squareOf schema inst pullback = some square ∧
    Check.legValues square row = some projection ∧
    Check.commutes square projection = some (lhs, rhs)

/-- A declared pullback's covers failure: a pair of cospan-source rows
with a common resolved target that no resolved apex row projects to. -/
def PullbackCoversFailure (schema : Schema) (inst : Instance.TypedInstance)
    (pullback : Pullback) (a b : RowId) : Prop :=
  ∃ square n ta tb,
    Check.squareOf schema inst pullback = some square ∧
    Schema.rowCount? inst.rows pullback.apex = some n ∧
    a ∈ square.aRows ∧ b ∈ square.bRows ∧
    square.f[a]? = some (some ta) ∧
    square.g[b]? = some (some tb) ∧
    ta = tb ∧
    (a, b) ∉ Check.projectionPairs square (List.range n)

/-- A declared pullback's unique failure: two distinct apex rows whose
resolved projections agree — the legs are not jointly injective. -/
def PullbackUniqueFailure (schema : Schema) (inst : Instance.TypedInstance)
    (pullback : Pullback) (lo hi : RowId) : Prop :=
  ∃ square pair,
    Check.squareOf schema inst pullback = some square ∧
    lo < hi ∧
    Check.legValues square lo = some pair ∧
    Check.legValues square hi = some pair

/-- The semantic reading of one pullback witness arm. -/
def PullbackWitnessSemantic (schema : Schema) (inst : Instance.TypedInstance)
    (pullback : Pullback) : PullbackWitness → Prop
  | .commutes w => PullbackCommutesFailure schema inst pullback w.row w.lhs w.rhs
  | .covers w => PullbackCoversFailure schema inst pullback w.pair.1 w.pair.2
  | .unique w => PullbackUniqueFailure schema inst pullback w.rows.1 w.rows.2

/-- Membership in `Check.pullbackFindings` yields the emitted shape: a
declared pullback (canonical index), its status label, and one of the
three verdict witnesses. Used by the soundness disjunction and by the
dangling-discipline refutations. -/
theorem mem_pullbackFindings_shape (schema : Schema) (inst : Instance.TypedInstance)
    {finding : Finding} :
    finding ∈ Check.pullbackFindings schema inst →
      ∃ (index : Nat) (pullback : Pullback) (w : PullbackWitness),
        schema.pullbacks[index]? = some pullback ∧
        finding.label = labelFor pullback.status ∧
        finding.kind = .violation (.pullback w) := by
  intro h
  rw [Check.pullbackFindings, mem_flatten_mapIdx] at h
  obtain ⟨index, pullback, hidx, hmem⟩ := h
  unfold Check.pullbackDeclaration at hmem
  cases hsq : Check.squareOf schema inst pullback with
  | none => simp [hsq] at hmem
  | some sq =>
    cases hn : Schema.rowCount? inst.rows pullback.apex with
    | none => simp [hsq, hn] at hmem
    | some n =>
      rw [hsq, hn] at hmem
      rw [List.mem_append, List.mem_append] at hmem
      rcases hmem with (hc | hu) | hcov
      · rw [List.mem_filterMap] at hc
        obtain ⟨row, _hrow, hfind⟩ := hc
        unfold Check.commutesFinding at hfind
        cases hleg : Check.legValues sq row with
        | none => simp [hleg] at hfind
        | some projection =>
          cases hcom : Check.commutes sq projection with
          | none => simp [hleg, hcom] at hfind
          | some pair =>
            obtain ⟨lhs, rhs⟩ := pair
            simp only [hleg, hcom, Option.some.injEq] at hfind
            refine ⟨index, pullback, .commutes
              { pullback := index, row := row, lhs := lhs, rhs := rhs }, hidx, ?_, ?_⟩
            · rw [← hfind]
            · rw [← hfind]
      · unfold Check.uniqueFindings at hu
        rw [List.mem_flatMap] at hu
        obtain ⟨lo, _hlo, hu⟩ := hu
        rw [List.mem_filterMap] at hu
        obtain ⟨hi, _hhi, hu⟩ := hu
        unfold Check.uniqueFinding at hu
        cases hl : Check.legValues sq lo with
        | none => simp [hl] at hu
        | some left =>
          cases hr : Check.legValues sq hi with
          | none => simp [hl, hr] at hu
          | some right =>
            by_cases hb : (left == right) = true
            · simp only [hl, hr, hb, ite_true] at hu
              simp only [Option.some.injEq] at hu
              refine ⟨index, pullback, .unique
                { pullback := index, rows := (lo, hi) }, hidx, ?_, ?_⟩
              · rw [← hu]
              · rw [← hu]
            · simp [hl, hr, hb] at hu
      · cases hpart : ((List.range n).any fun row =>
            (Check.legValues sq row).isNone) with
        | true => simp [hpart] at hcov
        | false =>
          simp only [hpart, Bool.false_eq_true, ite_false] at hcov
          unfold Check.coverageFindings at hcov
          rw [List.mem_flatMap] at hcov
          obtain ⟨a, _ha, hcov⟩ := hcov
          rw [List.mem_filterMap] at hcov
          obtain ⟨b, _hb, hfind⟩ := hcov
          unfold Check.fiberPairFinding at hfind
          cases hfa : sq.f[a]? with
          | none => simp [hfa] at hfind
          | some fa =>
            cases hgb : sq.g[b]? with
            | none => simp [hfa, hgb] at hfind
            | some gb =>
              cases fa with
              | none => simp [hfa, hgb] at hfind
              | some ta =>
                cases gb with
                | none => simp [hfa, hgb] at hfind
                | some tb =>
                  by_cases htarget : (ta == tb) = true
                  · simp only [hfa, hgb, htarget, ite_true] at hfind
                    unfold Check.coversFinding at hfind
                    cases hcont : decide ((a, b) ∈ Check.projectionPairs sq (List.range n)) with
                    | true => simp [hcont] at hfind
                    | false =>
                      simp only [hcont, Option.some.injEq] at hfind
                      refine ⟨index, pullback, .covers
                        { pullback := index, pair := (a, b) }, hidx, ?_, ?_⟩
                      · rw [← hfind]
                      · rw [← hfind]
                  · simp [hfa, hgb, htarget] at hfind

/-! ## Semantic cells (kernel-checks: "Dangling values are checked, not
fatal" and "The instance builder is total and typing-enforcing") -/

/-- A table cell's value: the instance carries a table under `arrow`
(an entry of `tables`) and that table holds `value` at `src`. Phrased by
membership, not `find?`, because `Check.danglingFindings` iterates every
table entry. -/
def CellValue (inst : Instance.TypedInstance) (arrow : Ident) (src : RowId)
    (value : Option RowId) : Prop :=
  ∃ table : List (Option RowId),
    (arrow, table) ∈ inst.tables ∧ table[src]? = some value

/-- An unresolved table cell: the cell holds no row. -/
def CellUnresolved (inst : Instance.TypedInstance) (arrow : Ident)
    (src : RowId) : Prop :=
  CellValue inst arrow src none

/-- A cell a build-time typing violation accounts for (the exact pair
list `Check.danglingFindings` subtracts). -/
def TypingViolatedAt (inst : Instance.TypedInstance) (arrow : Ident)
    (src : RowId) : Prop :=
  (arrow, src) ∈ inst.typingViolations.map fun violation =>
    (violation.arrow, violation.src)

/-- A semantic dangling cell: the table cell resolves to no row and no
typing violation names it — so it is a dangling finding, not a
double-report of a failed cell. -/
def DanglingCellSemantic (inst : Instance.TypedInstance) (arrow : Ident)
    (src : RowId) : Prop :=
  CellUnresolved inst arrow src ∧ ¬ TypingViolatedAt inst arrow src

/-! ## The projected raw cell (kernel-checks: builder totality) -/

/-- The projected value vector a declared arrow carries in the raw
presentation (empty when the presentation carries no table). -/
def RawCells (raw : Instance.RawInstance) (name : Ident) : List (Option RowId) :=
  match raw.values.find? (fun entry => entry.1 == name) with
  | some entry => entry.2
  | none => []

/-- The projected raw cell at `idx`, with absent cells read as `none`
(the builder's `take`-then-index reading). -/
def RawCell (raw : Instance.RawInstance) (name : Ident)
    (idx : Nat) : Option RowId :=
  (RawCells raw name)[idx]?.join

end Proofs
end Catherine