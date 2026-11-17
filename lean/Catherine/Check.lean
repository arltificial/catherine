import Catherine.Schema
import Catherine.Instance
import Catherine.EvalPath
import Catherine.Findings

/-!
# Purpose: Lean reference implementation of the executed checker — one
dangling finding per unresolved cell (minus typing-violated cells), one
violation per equation per applicable row whose composites disagree, one
violation per unordered pair of rows whose declared key tuple agrees, one
violation per row of a declared coproduct whose coverage or disjointness
claim fails, and the final taxonomy sort — the function the lean-parity
oracle compares against `catherine check --json`
(`crates/catherine-core/src/checks.rs`, `checks/coproducts.rs`).

## Responsibilities: define `check : Schema → TypedInstance → List Finding`
with its frozen signature. Every lookup is total: a declaration outside
the parsed presentation's shape (undeclared arrow, wrong source/target
object, missing row space or table) skips that declaration, exactly as the
Rust walk's `continue`/`Option` guards do — it never invents a finding and
never crashes. An empty schema or instance yields `[]`.

## Rationale: the oracle (tasks.md item 6) and the theorems (items 7–10)
both take `check` as the object of study — one function, one place to keep
in parity, so the walk order mirrors the Rust walk even though the
returned list is sorted under the frozen `Ord` (kind, then witness ids,
label last). Each family carries its witness by construction: a violation
names the equation/key/coproduct index and the offending rows, a dangling
names the arrow and source row. Labels come from one rule (`labelFor`):
accepted constraints are executed, proposed ones are questions. The
per-family helpers are public (not `private`) so the proof phase can state
lemmas about one family without restating the whole walk.
-/

namespace Catherine
namespace Check

/-! ## Dangling values -/

/-- One dangling finding per unresolved table cell that no typing
violation accounts for. A violated value never became a table entry, so
its `none` cell is a build report and not a dangling reference. -/
def danglingFindings (inst : Instance.TypedInstance)
    (violated : List (Ident × RowId)) : List Finding :=
  inst.tables.flatMap fun entry =>
    (List.range entry.2.length).filterMap fun idx =>
      match entry.2[idx]? with
      | some none =>
        if violated.contains (entry.1, idx) then none
        else some { label := .executed, kind := .dangling { arrow := entry.1, src := idx } }
      | _ => none

/-! ## Equations -/

/-- The violation for one equation on one row, when both composites
resolve and disagree. A short-circuited composite (a dangling or
typing-violated cell on the path) emits nothing: the cell is already its
own finding, never double-reported as a violation. -/
def equationFinding (inst : Instance.TypedInstance) (index : Nat)
    (equation : Equation) (row : RowId) : Option Finding :=
  match EvalPath.compose inst equation.lhs row,
      EvalPath.compose inst equation.rhs row with
  | some lhs, some rhs =>
    if lhs == rhs then none
    else
      some
        { label := labelFor equation.status
          kind := .violation (.equation
            { equation := index, row := row, lhs := lhs, rhs := rhs }) }
  | _, _ => none

/-- Every declared equation on every row of its domain object — the rows
of the lead arrow's source object, which is also the equation's domain.
An equation with an empty `lhs`, an undeclared lead arrow, or a source
object without a row space is inapplicable and emits nothing. -/
def equationFindings (schema : Schema) (inst : Instance.TypedInstance) :
    List Finding :=
  (schema.equations.mapIdx fun index equation =>
    match equation.lhs with
    | [] => []
    | lead :: _ =>
      match schema.arrow? lead with
      | none => []
      | some arrow =>
        match Schema.rowCount? inst.rows arrow.src with
        | none => []
        | some n =>
          (List.range n).filterMap fun row => equationFinding inst index equation row).flatten

/-! ## Keys -/

/-- The key arrows' value tables, in key order. A key naming an
undeclared arrow, an arrow not sourced at its object, or an arrow without
a table is outside the parsed presentation's shape and skips the key
rather than mis-grouping rows. -/
def keyTables (schema : Schema) (inst : Instance.TypedInstance) (key : Key) :
    Option (List (List (Option RowId))) :=
  key.arrows.mapM fun name =>
    match schema.arrow? name with
    | none => none
    | some arrow =>
      if arrow.src == key.object then
        (inst.tables.find? (fun entry => entry.1 == name)).map fun entry => entry.2
      else none

/-- One row's key tuple; `none` when any key cell is unresolved (dangling,
typing-violated, or outside the table's row space). -/
def keyTuple (tables : List (List (Option RowId))) (row : RowId) :
    Option (List RowId) :=
  tables.mapM fun cells => cells[row]?.join

/-- Insert `row` into the group keyed by `tuple`, keeping each group's rows
ascending (the walk visits rows ascending). -/
def insertRow (tuple : List RowId) (row : RowId) :
    List (List RowId × List RowId) → List (List RowId × List RowId)
  | [] => [(tuple, [row])]
  | (existing, rows) :: rest =>
    if existing == tuple then (existing, rows ++ [row]) :: rest
    else (existing, rows) :: insertRow tuple row rest

/-- Group the rows of a key's object whose every key cell resolves, by
their tuple. -/
def groupedRows (tables : List (List (Option RowId))) (n : Nat) :
    List (List RowId) :=
  ((List.range n).foldl (fun groups row =>
    match keyTuple tables row with
    | none => groups
    | some tuple => insertRow tuple row groups) []).map fun entry => entry.2

/-- Every unordered pair of a group, ascending rows first. -/
def ascendingPairs : List RowId → List (RowId × RowId)
  | [] => []
  | lo :: rest => rest.map (fun hi => (lo, hi)) ++ ascendingPairs rest

/-- Every declared key as an injectivity claim: one violation per
unordered pair of rows agreeing on the key tuple (a three-way collision is
three findings). Rows with an unresolved key cell are excluded — their
cell carries its own dangling finding. -/
def keyFindings (schema : Schema) (inst : Instance.TypedInstance) :
    List Finding :=
  (schema.keys.mapIdx fun index key =>
    match Schema.rowCount? inst.rows key.object, keyTables schema inst key with
    | some n, some tables =>
      let label := labelFor key.status
      (groupedRows tables n).flatMap fun rows =>
        (ascendingPairs rows).map fun pair =>
          { label := label, kind := .violation (.key { key := index, rows := pair }) }
    | _, _ => []).flatten

/-! ## Coproducts -/

/-- One injection's complete image: the coproduct-object rows it covers,
named by the injection. -/
structure InjectionImage where
  arrow : Ident
  rows : List RowId

/-- Every declared injection's table, in declared order. `none` when an
arrow is undeclared, targets away from the coproduct object, or has no
table — outside the parsed presentation's shape. -/
def injectionTables (schema : Schema) (inst : Instance.TypedInstance)
    (coproduct : Coproduct) : Option (List (Ident × List (Option RowId))) :=
  coproduct.arrows.mapM fun name =>
    match schema.arrow? name with
    | none => none
    | some arrow =>
      if arrow.tgt == coproduct.object then
        (inst.tables.find? (fun entry => entry.1 == name)).map fun entry => (name, entry.2)
      else none

/-- A table's rows when every cell resolves; `none` when the image is
partial (its unknown target could be any row). -/
def completeRows : List (Option RowId) → Option (List RowId)
  | [] => some []
  | some row :: rest => (completeRows rest).map fun rows => row :: rows
  | none :: _ => none

/-- One injection's complete image, if it has one. -/
def completeImage (arrow : Ident) (cells : List (Option RowId)) :
    Option InjectionImage :=
  match completeRows cells with
  | some rows => some { arrow := arrow, rows := rows }
  | none => none

/-- The complete images containing `row`, in declared arrow order. -/
def coveringInjections (images : List InjectionImage) (row : RowId) :
    List Ident :=
  images.filterMap fun image =>
    if image.rows.contains row then some image.arrow else none

/-- The coproduct violation witness: the declaration's index, the row, and
every injection the verdict holds responsible. -/
def coproductFinding (label : EpistemicLabel) (index : Nat) (row : RowId)
    (injections : List Ident) : Finding :=
  { label := label
    kind := .violation (.coproduct
      { coproduct := index, row := row, injections := injections }) }

/-- One declaration's findings over its object's rows. With every image
complete a row must be covered exactly once (no cover is the empty
witness, several name the covering injections); with any partial image
only rows in at least two complete images are decidable. -/
def coproductDeclaration (schema : Schema) (inst : Instance.TypedInstance)
    (index : Nat) (coproduct : Coproduct) : List Finding :=
  match injectionTables schema inst coproduct with
  | none => []
  | some tables =>
    let images := tables.filterMap fun entry => completeImage entry.1 entry.2
    let complete := images.length == tables.length
    let label := labelFor coproduct.status
    match Schema.rowCount? inst.rows coproduct.object with
    | none => []
    | some n =>
      (List.range n).filterMap fun row =>
        let covering := coveringInjections images row
        if complete then
          if covering.length == 1 then none
          else some (coproductFinding label index row covering)
        else
          if 2 ≤ covering.length then some (coproductFinding label index row covering)
          else none

/-- Every declared coproduct: one finding per decidably failing row. -/
def coproductFindings (schema : Schema) (inst : Instance.TypedInstance) :
    List Finding :=
  (schema.coproducts.mapIdx fun index coproduct =>
    coproductDeclaration schema inst index coproduct).flatten

/-! ## The walk -/

/-- Execute every declared constraint over the typed instance and emit
witness-backed findings in taxonomy order. -/
def check (schema : Schema) (inst : Instance.TypedInstance) : List Finding :=
  let violated := inst.typingViolations.map fun violation => (violation.arrow, violation.src)
  let findings :=
    danglingFindings inst violated
      ++ equationFindings schema inst
      ++ keyFindings schema inst
      ++ coproductFindings schema inst
  findings.mergeSort (fun a b => compare a b != .gt)

end Check
end Catherine