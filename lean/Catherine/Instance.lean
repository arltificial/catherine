import Catherine.Schema

/-!
# Purpose: Lean reference formalization of the typed instance — dense row
spaces over schema objects and partial morphism tables over schema
arrows, plus the total typing-enforcing builder whose three landing
places (stored value / dangling record / typing violation) mirror
`crates/catherine-core/src/instance.rs`.

## Responsibilities: define the instance representation (`RawInstance` as
projected from a mirror document, `TypedInstance` as the checker's input),
the build-time report shapes (`TypingViolation`, `DanglingCell`), and
`build`, the total typing-enforcing builder: for every declared arrow, a
table over the arrow's source row space where each raw cell lands in
exactly one of the three documented places — a stored morphism value, a
dangling record, or a typing-violation report.

## Rationale: partiality is where the Rust kernel hangs its dangling and
typing-violation accounting (tasks.md item 4), so the twin keeps the same
shape to make cell-accounting proofs (item 10) line up. The builder is
total on every input (a missing row space is zero rows, a missing raw
table is all-`none`) because the kernel never rejects a presentation at
build; it records. Report order is the Rust build order — declared arrow
order (the mirror keeps arrows sorted by name), then source row — so
`--check-build` compares list order, not just membership. `buildArrow` and
`ArrowBuild` are public so the proof phase can state one arrow's
cell-accounting lemma without unfolding the whole builder.
-/

namespace Catherine
namespace Instance

/-- A build-time typing violation: an arrow value resolved outside the
arrow's target object's dense id space — never stored as a morphism
value. -/
structure TypingViolation where
  arrow : Ident
  src : RowId
  value : RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A dangling arrow value: the cell holds no row — a representable
value, never dropped or repaired. -/
structure DanglingCell where
  arrow : Ident
  src : RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The projected presentation: per-object dense row counts and one
partial value vector per arrow (parallel to the arrow's source object's
rows, `none` = dangling). `rows` is sorted by object name, `values` by
arrow name. -/
structure RawInstance where
  rows : List (Ident × Nat)
  values : List (Ident × List (Option RowId))
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The typed instance over a presentation: the per-object dense row
counts, one partial morphism table per declared arrow, and the build-time
reports (typing violations, dangling cells) in build order (arrow name,
then source row). -/
structure TypedInstance where
  rows : List (Ident × Nat)
  tables : List (Ident × List (Option RowId))
  typingViolations : List TypingViolation
  dangling : List DanglingCell
  deriving Repr, DecidableEq, BEq, Inhabited

/-- One declared arrow's build result: its partial table over the source
row space plus the two reports, both in ascending source-row order. -/
structure ArrowBuild where
  table : List (Option RowId)
  typingViolations : List TypingViolation
  dangling : List DanglingCell

/-- Build one declared arrow's table and reports. `nSrc`/`nTgt` are the
source and target row counts (a missing row space is zero). The table has
exactly `nSrc` cells: a raw cell is stored iff it resolves inside the
target's dense id space, otherwise the cell stays `none` and a typing
violation names it; a raw `none` cell stays `none` and is reported
dangling. Raw cells beyond the source row space have no row to sit on and
are ignored (they are outside the parsed presentation's shape), and raw
cells the vector does not carry stay `none` without a dangling report —
exactly the Rust builder's `take`. -/
def buildArrow (raw : RawInstance) (name : Ident) (arrow : Arrow) :
    ArrowBuild :=
  let nSrc := (Schema.rowCount? raw.rows arrow.src).getD 0
  let nTgt := (Schema.rowCount? raw.rows arrow.tgt).getD 0
  let cells :=
    match raw.values.find? (fun entry => entry.1 == name) with
    | some entry => entry.2
    | none => []
  let cells := cells.take nSrc
  let table := (List.range nSrc).map fun idx =>
    match cells[idx]? with
    | some (some value) => if value < nTgt then some value else none
    | _ => none
  let dangling := (List.range cells.length).filterMap fun idx =>
    match cells[idx]? with
    | some none => some ({ arrow := name, src := idx } : DanglingCell)
    | _ => none
  let typingViolations := (List.range cells.length).filterMap fun idx =>
    match cells[idx]? with
    | some (some value) =>
      if nTgt ≤ value then
        some ({ arrow := name, src := idx, value := value } : TypingViolation)
      else none
    | _ => none
  { table := table, typingViolations := typingViolations, dangling := dangling }

/-- Build the typed instance for a projected presentation. Every declared
arrow gets a table (a presentation without a table for it is all `none`,
i.e. every source row dangling), and the two report lists are
concatenated in declared arrow order then source row order. -/
def build (schema : Schema) (raw : RawInstance) : TypedInstance :=
  let built := schema.arrows.map fun entry => (entry.1, buildArrow raw entry.1 entry.2)
  { rows := raw.rows
    tables := built.map fun entry => (entry.1, entry.2.table)
    typingViolations := (built.map fun entry => entry.2.typingViolations).flatten
    dangling := (built.map fun entry => entry.2.dangling).flatten }

end Instance
end Catherine