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
the partial-image mode), and `CellUnresolved`/`TypingViolatedAt` for a
`none` table cell no typing violation accounts for. Also expose the raw
projected cell (`RawCell`) the builder consumes and prove the easy
shared lemmas (label rule, empty-path composition, `check` membership,
the `equationFinding` iff).

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

/-- Membership in `check` unfolds to membership in one of the four
finding families; the taxonomy sort only permutes, it never invents. -/
@[simp] theorem mem_check_iff (schema : Schema) (inst : Instance.TypedInstance)
    (finding : Finding) :
    finding ∈ Check.check schema inst ↔
      finding ∈ Check.danglingFindings inst
          (inst.typingViolations.map fun violation =>
            (violation.arrow, violation.src)) ∨
      finding ∈ Check.equationFindings schema inst ∨
      finding ∈ Check.keyFindings schema inst ∨
      finding ∈ Check.coproductFindings schema inst := by
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