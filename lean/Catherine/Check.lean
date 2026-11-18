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
claim fails, one violation per failed pullback verdict (commutes, covers,
unique), and the final taxonomy sort — the function the lean-parity
oracle compares against `catherine check --json`
(`crates/catherine-core/src/checks.rs`, `checks/coproducts.rs`,
`checks/pullbacks.rs`).

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

/-! ## Pullbacks -/

/-- The declaration's exactly-two arrow names, in declared order. -/
def twoOf : List Ident → Option (Ident × Ident)
  | [first, second] => some (first, second)
  | _ => none

/-- An arrow's value table in the typed instance, if it carries one. -/
def tableOf (inst : Instance.TypedInstance) (name : Ident) :
    Option (List (Option RowId)) :=
  (inst.tables.find? (fun entry => entry.1 == name)).map fun entry => entry.2

/-- One declaration's resolved square: the four arrows' tables plus the
cospan sources' row spaces. -/
structure Square where
  p1 : List (Option RowId)
  p2 : List (Option RowId)
  f : List (Option RowId)
  g : List (Option RowId)
  aRows : List RowId
  bRows : List RowId

/-- Resolve a declaration's square; `none` when the declaration is outside
the parsed presentation's shape (not exactly two legs/cospan arrows, a leg
not sourced at the apex, cospan targets disagreeing, a missing table). -/
def squareOf (schema : Schema) (inst : Instance.TypedInstance)
    (pullback : Pullback) : Option Square :=
  match twoOf pullback.legs, twoOf pullback.cospan with
  | some (p1, p2), some (f, g) =>
    match schema.arrow? p1, schema.arrow? p2, schema.arrow? f, schema.arrow? g with
    | some leg1, some leg2, some cospan1, some cospan2 =>
      if leg1.src == pullback.apex && leg2.src == pullback.apex &&
          cospan1.tgt == cospan2.tgt then
        match tableOf inst p1, tableOf inst p2, tableOf inst f, tableOf inst g with
        | some p1Table, some p2Table, some fTable, some gTable =>
          some
            { p1 := p1Table, p2 := p2Table, f := fTable, g := gTable
              aRows := (Schema.rowCount? inst.rows cospan1.src).map List.range |>.getD []
              bRows := (Schema.rowCount? inst.rows cospan2.src).map List.range |>.getD [] }
        | _, _, _, _ => none
      else none
    | _, _, _, _ => none
  | _, _ => none

/-- One apex row's projection `(p1[row], p2[row])`; `none` when either leg
cell is unresolved (dangling or typing-violated), which leaves the row's
verdict undefined rather than violated. -/
def legValues (square : Square) (row : RowId) : Option (RowId × RowId) :=
  match square.p1[row]?, square.p2[row]? with
  | some (some left), some (some right) => some (left, right)
  | _, _ => none

/-- Both composed cospan values for one projection, when they exist and
disagree. A composition through a dangling cospan cell is undefined, never
a violation. -/
def commutes (square : Square) (projection : RowId × RowId) :
    Option (RowId × RowId) :=
  match square.f[projection.1]?, square.g[projection.2]? with
  | some (some lhs), some (some rhs) =>
    if lhs == rhs then none else some (lhs, rhs)
  | _, _ => none

/-- One commutes finding for an apex row whose projection resolves to
disagreeing cospan composites. -/
def commutesFinding (square : Square) (index : Nat) (label : EpistemicLabel)
    (row : RowId) : Option Finding :=
  match legValues square row with
  | none => none
  | some projection =>
    match commutes square projection with
    | none => none
    | some (lhs, rhs) =>
      some
        { label := label
          kind := .violation (.pullback (.commutes
            { pullback := index, row := row, lhs := lhs, rhs := rhs })) }

/-- Every resolved apex row's projection, rows ascending (the walk visits
rows ascending). -/
def projectionPairs (square : Square) (rows : List RowId) :
    List (RowId × RowId) :=
  rows.filterMap fun row => legValues square row

/-- One uniqueness finding for the pair `(lo, hi)` when both projections
resolve and agree — the witnesses' rows are enumerated `lo < hi`, so they
arrive ascending exactly as the Rust pass emits them. -/
def uniqueFinding (square : Square) (index : Nat) (label : EpistemicLabel)
    (lo hi : RowId) : Option Finding :=
  match legValues square lo, legValues square hi with
  | some left, some right =>
    if left == right then
      some
        { label := label
          kind := .violation (.pullback (.unique
            { pullback := index, rows := (lo, hi) })) }
    else none
  | _, _ => none

/-- One uniqueness finding per unordered pair of apex rows sharing a
projection (a 3-way duplication is three findings — each pair is a
demonstrated failure, not a summary count). -/
def uniqueFindings (square : Square) (rows : List RowId) (index : Nat)
    (label : EpistemicLabel) : List Finding :=
  rows.flatMap fun lo =>
    (rows.filter fun hi => lo < hi).filterMap fun hi =>
      uniqueFinding square index label lo hi

/-- One covers finding for the fiber pair `(a, b)`, unless an apex row
already projects to it. -/
def coversFinding (_square : Square) (index : Nat) (label : EpistemicLabel)
    (projections : List (RowId × RowId)) (a b : RowId) : Option Finding :=
  match decide ((a, b) ∈ projections) with
  | true => none
  | false =>
    some
      { label := label
        kind := .violation (.pullback (.covers
          { pullback := index, pair := (a, b) })) }

/-- One covers finding when the pair shares a resolved cospan target: the
fiber is enumerated by target on both sides, so only equal targets enter
the coverage ledger. -/
def fiberPairFinding (square : Square) (index : Nat) (label : EpistemicLabel)
    (projections : List (RowId × RowId)) (a b : RowId) : Option Finding :=
  match square.f[a]?, square.g[b]? with
  | some (some ta), some (some tb) =>
    if ta == tb then coversFinding square index label projections a b else none
  | _, _ => none

/-- Every uncovered pair of the resolved fiber: an `A` row and a `B` row
sharing a cospan target that no apex row projects to. -/
def coverageFindings (square : Square) (index : Nat) (label : EpistemicLabel)
    (projections : List (RowId × RowId)) : List Finding :=
  square.aRows.flatMap fun a =>
    square.bRows.filterMap fun b =>
      fiberPairFinding square index label projections a b

/-- One declaration's three verdicts: commutes per resolved apex row,
uniqueness per duplicated projection pair, and coverage only when every
apex leg cell resolves (an unknown projection could occupy any fiber
pair). -/
def pullbackDeclaration (schema : Schema) (inst : Instance.TypedInstance)
    (index : Nat) (pullback : Pullback) : List Finding :=
  match squareOf schema inst pullback, Schema.rowCount? inst.rows pullback.apex with
  | some square, some n =>
    let rows := List.range n
    let label := labelFor pullback.status
    let projections := projectionPairs square rows
    let legsPartial := rows.any fun row => (legValues square row).isNone
    let commutesFindings := rows.filterMap fun row =>
      commutesFinding square index label row
    commutesFindings ++ uniqueFindings square rows index label ++
      (if legsPartial then [] else coverageFindings square index label projections)
  | _, _ => []

/-- Every declared pullback, in canonical declaration order. -/
def pullbackFindings (schema : Schema) (inst : Instance.TypedInstance) :
    List Finding :=
  (schema.pullbacks.mapIdx fun index pullback =>
    pullbackDeclaration schema inst index pullback).flatten

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
      ++ pullbackFindings schema inst
  findings.mergeSort (fun a b => compare a b != .gt)

end Check
end Catherine