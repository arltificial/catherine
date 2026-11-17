import Catherine.Schema
import Catherine.Instance

/-!
# Purpose: Lean reference formalization of path evaluation — `compose`
over the morphism tables with `Option` short-circuit on any unresolved
cell, mirroring `TypedInstance::compose` in
`crates/catherine-core/src/instance.rs`.

## Responsibilities: define `compose : TypedInstance → Path → RowId →
Option RowId`: walk the path's arrow names in composition order, index the
arrow's table at the current row, and continue from the stored row; a
missing table, an index outside the table, or a `none` cell yields `none`
(the path cannot compose through a dangling or typing-violated value). An
empty path is the identity on its start row.

## Rationale: equation completeness (tasks.md item 9) is stated against
this evaluator, so it must be the kernel's evaluator verbatim — the
short-circuit is the semantics, not an optimisation: a row whose path
touches an unresolved cell is reported by its dangling finding and emits
no violation (design Decision 4 keeps any drift visible in the oracle).
-/

namespace Catherine
namespace EvalPath

/-- Compose `path` starting at `start` through the instance's morphism
tables. Each arrow's cell gives the next row; `none` anywhere
short-circuits the whole composition. -/
def compose (inst : Instance.TypedInstance) (path : Path) (start : RowId) :
    Option RowId :=
  match path with
  | [] => some start
  | name :: rest =>
    match inst.tables.find? (fun entry => entry.1 == name) with
    | none => none
    | some entry =>
      match entry.2[start]? with
      | some (some next) => compose inst rest next
      | _ => none

end EvalPath
end Catherine