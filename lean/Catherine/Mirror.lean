import Lean
import Catherine.Schema
import Catherine.Instance

/-!
# Purpose: the trusted mirror projection and decoder for the lean-parity
oracle — turns a canonical `.ct.json` document into the mirror JSON of
the brief (§3.1): schema in canonical order and a dense, partial
instance projection that the kernel modules consume.

## Responsibilities: project a document with Rust `parse`'s normalization —
per-object dense row ids in sorted row-name order, equations sorted by
`(lhs, rhs, rationale, status string)`, keys/coproducts sorted by
`(object, arrows, status string)`, table cells resolved target-first then
by the first declared object (sorted) whose row-name map contains the
name; missing `keys`/`coproducts`/`equations`/`instance.tables` default
to empty; an unresolved row name (or a table row outside its arrow's
source space) is a harness error naming fixture, arrow, and row.
`decodeMirror` reads the same JSON back into `Schema`/`RawInstance`, so
`--emit-mirror` output is the exact format the oracle consumes. No
validation, spans, or error recovery: `parse` stays Rust-side (design
Decision 2).

## Rationale: the mirror is the only interface between the Rust kernel and
the Lean twin, so both directions live in one module and share the
canonical order. Treating an unresolvable name as a harness error keeps
it out of the divergence channel — a fixture defect must never be
attributed to a semantic mismatch.
-/

namespace Catherine
namespace Mirror

open Lean

/-- One projected document: the fixture's display name, its schema, and
its instance presentation. -/
structure MirrorDoc where
  fixture : String
  schema : Schema
  raw : Instance.RawInstance
  deriving Repr

/-! ## Small JSON helpers -/

private def fieldStr (node : Json) (key : String) : Except String String :=
  node.getObjVal? key >>= Json.getStr?

private def arrayField (node : Json) (key : String) : Except String (Array Json) :=
  node.getObjVal? key >>= Json.getArr?

/-- An optional array section: absent (or explicit null) is empty. -/
private def optionalArrayField (node : Json) (key : String) : Except String (Array Json) :=
  match node.getObjValD key with
  | .null => pure #[]
  | value => value.getArr?

private def stringsOf (json : Json) : Except String (List String) := do
  let array ← json.getArr?
  array.toList.mapM Json.getStr?

/-- A string section value, defaulting to empty exactly as the Rust
loader's `get(..).and_then(as_str).unwrap_or_default()` does. -/
private def optionalString (node : Json) (key : String) : String :=
  match node.getObjValD key with
  | .str value => value
  | _ => ""

private def statusOf (node : Json) : Except String ConstraintStatus :=
  match node.getObjValD "status" with
  | .null => pure .accepted
  | .str "accepted" => pure .accepted
  | .str "proposed" => pure .proposed
  | .str other => throw s!"unknown status `{other}`"
  | _ => throw "status is not a string"

/-! ## Canonical ordering -/

private def lessString (a b : String) : Bool :=
  compare a b != .gt

/-- Sorted, duplicate-free string list (the Rust `BTreeSet` shape). -/
def sortedNames (names : List String) : List String :=
  let sorted := names.mergeSort lessString
  sorted.foldl
    (fun acc name =>
      match acc with
      | [] => [name]
      | previous :: _ => if name == previous then acc else name :: acc)
    []
  |>.reverse

private structure EquationEntry where
  lhs : Path
  rhs : Path
  rationale : String
  status : ConstraintStatus

private def equationEntryCompare (a b : EquationEntry) : Ordering :=
  (compare a.lhs b.lhs).then <|
    (compare a.rhs b.rhs).then <|
      (compare a.rationale b.rationale).then
        (compare a.status.asString b.status.asString)

private def keyCompare (a b : Key) : Ordering :=
  (compare a.object b.object).then <|
    (compare a.arrows b.arrows).then (compare a.status.asString b.status.asString)

private def coproductCompare (a b : Coproduct) : Ordering :=
  (compare a.object b.object).then <|
    (compare a.arrows b.arrows).then (compare a.status.asString b.status.asString)

/-! ## Projection of a document -/

/-- One object's dense row-name → row-id map, in sorted-name order. -/
private def rowIds (names : List Ident) : List (Ident × RowId) :=
  names.mapIdx fun index name => (name, index)

private def equationsOf (schemaNode : Json) : Except String (List Equation) := do
  let nodes ← optionalArrayField schemaNode "equations"
  let entries ← nodes.toList.mapM fun node => do
    let lhs ← stringsOf (node.getObjValD "lhs")
    let rhs ← stringsOf (node.getObjValD "rhs")
    let status ← statusOf node
    pure { lhs := lhs, rhs := rhs, rationale := optionalString node "rationale", status := status }
  let sorted := entries.mergeSort (fun a b => equationEntryCompare a b != .gt)
  pure (sorted.map fun entry => { lhs := entry.lhs, rhs := entry.rhs, status := entry.status })

private def keysOf (schemaNode : Json) : Except String (List Key) := do
  let nodes ← optionalArrayField schemaNode "keys"
  let keys ← nodes.toList.mapM fun node => do
    let object ← fieldStr node "object"
    let arrows ← stringsOf (node.getObjValD "arrows")
    let status ← statusOf node
    pure { object := object, arrows := arrows, status := status }
  pure (keys.mergeSort (fun a b => keyCompare a b != .gt))

private def coproductsOf (schemaNode : Json) : Except String (List Coproduct) := do
  let nodes ← optionalArrayField schemaNode "coproducts"
  let coproducts ← nodes.toList.mapM fun node => do
    let object ← fieldStr node "object"
    let arrows ← stringsOf (node.getObjValD "arrows")
    let status ← statusOf node
    pure { object := object, arrows := arrows, status := status }
  pure (coproducts.mergeSort (fun a b => coproductCompare a b != .gt))

/-- Per-object dense row counts and row-name indexes, in sorted object
order. -/
private def rowsOf (instNode : Json) (objects : List Ident) :
    Except String (List (Ident × Nat) × List (Ident × List (Ident × RowId))) := do
  let rowsNode := instNode.getObjValD "rows"
  let mut counts : List (Ident × Nat) := []
  let mut index : List (Ident × List (Ident × RowId)) := []
  for object in objects do
    let names ←
      match rowsNode.getObjValD object with
      | .null => pure []
      | value => stringsOf value
    let names := sortedNames names
    counts := counts ++ [(object, names.length)]
    index := index ++ [(object, rowIds names)]
  pure (counts, index)

/-- Resolve one table value: `null` is dangling; a row name resolves into
the arrow target's id space first (the well-typed case), then into the
first declared object's id space that contains it. -/
private def resolveCell (name : Ident) (arrow : Arrow)
    (index : List (Ident × List (Ident × RowId))) (cell : Json) : Except String (Option RowId) :=
  match cell with
  | .null => pure none
  | .str target =>
    let targetMap :=
      match index.find? (fun entry => entry.1 == arrow.tgt) with
      | some entry => entry.2
      | none => []
    match targetMap.find? (fun entry => entry.1 == target) with
    | some entry => pure (some entry.2)
    | none =>
      match index.findSome? (fun entry => entry.2.find? (fun row => row.1 == target)) with
      | some entry => pure (some entry.2)
      | none => throw s!"table `{name}`: value `{target}` names no declared row"
  | other => throw s!"table `{name}`: value {other.compress} is not a row name or null"

/-- One arrow's total table over its source object's row space. -/
private def tableOf (instNode : Json) (name : Ident) (arrow : Arrow)
    (index : List (Ident × List (Ident × RowId))) : Except String (List (Option RowId)) := do
  let srcRows :=
    match index.find? (fun entry => entry.1 == arrow.src) with
    | some entry => entry.2
    | none => []
  let entries ←
    match (instNode.getObjValD "tables").getObjValD name with
    | .null => pure []
    | value => do
      let cells ← value.getObj?
      pure cells.toList
  let mut cells : Array (Option RowId) := Array.replicate srcRows.length none
  for (rowName, cell) in entries do
    let srcId ←
      match srcRows.find? (fun entry => entry.1 == rowName) with
      | some entry => pure entry.2
      | none => throw s!"table `{name}`: `{rowName}` is not a row of `{arrow.src}`"
    let value ← resolveCell name arrow index cell
    if srcId < cells.size then
      cells := cells.set! srcId value
  pure cells.toList

private def valuesOf (instNode : Json) (arrows : List (Ident × Arrow))
    (index : List (Ident × List (Ident × RowId))) :
    Except String (List (Ident × List (Option RowId))) :=
  arrows.mapM fun (name, arrow) => do
    let cells ← tableOf instNode name arrow index
    pure (name, cells)

/-- Project a canonical document into the mirror shape. Every failure is
a harness error (prefixed with the fixture name), never a divergence. -/
def projectDocument (fixture : String) (doc : Json) : Except String MirrorDoc :=
  let projected : Except String MirrorDoc := do
    let schemaNode ← doc.getObjVal? "schema"
    let objects ← stringsOf (← schemaNode.getObjVal? "objects")
    let objects := sortedNames objects
    let arrowNodes ← arrayField schemaNode "arrows"
    let arrows ← arrowNodes.toList.mapM fun node => do
      let name ← fieldStr node "name"
      let src ← fieldStr node "src"
      let tgt ← fieldStr node "tgt"
      pure (name, (⟨src, tgt⟩ : Arrow))
    let arrows := arrows.mergeSort (fun a b => lessString a.1 b.1)
    let equations ← equationsOf schemaNode
    let keys ← keysOf schemaNode
    let coproducts ← coproductsOf schemaNode
    let instNode := doc.getObjValD "instance"
    let (counts, index) ← rowsOf instNode objects
    let values ← valuesOf instNode arrows index
    pure
      { fixture := fixture
        schema :=
          { objects := objects
            arrows := arrows
            equations := equations
            keys := keys
            coproducts := coproducts }
        raw := { rows := counts, values := values } }
  projected.mapError fun error => s!"{fixture}: {error}"

/-! ## Encoding -/

private def pathJson (path : Path) : Json :=
  .arr (path.map Json.str).toArray

private def cellsJson (cells : List (Option RowId)) : Json :=
  .arr <| (cells.map fun cell =>
    match cell with
    | none => Json.null
    | some id => Json.num id).toArray

private def arrowsJson (arrows : List (Ident × Arrow)) : Json :=
  .arr <| (arrows.map fun (name, arrow) =>
    Json.mkObj
      [ ("name", Json.str name)
      , ("src", Json.str arrow.src)
      , ("tgt", Json.str arrow.tgt) ]).toArray

private def equationsJson (equations : List Equation) : Json :=
  .arr <| (equations.map fun equation =>
    Json.mkObj
      [ ("lhs", pathJson equation.lhs)
      , ("rhs", pathJson equation.rhs)
      , ("status", Json.str equation.status.asString) ]).toArray

private def keysJson (keys : List Key) : Json :=
  .arr <| (keys.map fun key =>
    Json.mkObj
      [ ("object", Json.str key.object)
      , ("arrows", pathJson key.arrows)
      , ("status", Json.str key.status.asString) ]).toArray

private def coproductsJson (coproducts : List Coproduct) : Json :=
  .arr <| (coproducts.map fun coproduct =>
    Json.mkObj
      [ ("object", Json.str coproduct.object)
      , ("arrows", pathJson coproduct.arrows)
      , ("status", Json.str coproduct.status.asString) ]).toArray

/-- The mirror JSON of one projected document (also `--emit-mirror`'s
file body). -/
def toJson (doc : MirrorDoc) : Json :=
  Json.mkObj
    [ ("fixture", Json.str doc.fixture)
    , ("schema",
        Json.mkObj
          [ ("objects", .arr (doc.schema.objects.map Json.str).toArray)
          , ("arrows", arrowsJson doc.schema.arrows)
          , ("equations", equationsJson doc.schema.equations)
          , ("keys", keysJson doc.schema.keys)
          , ("coproducts", coproductsJson doc.schema.coproducts) ])
    , ("instance",
        Json.mkObj
          [ ("rows", Json.mkObj (doc.raw.rows.map fun (object, count) => (object, Json.num count)))
          , ("values",
              Json.mkObj
                (doc.raw.values.map fun (arrow, cells) => (arrow, cellsJson cells))) ]) ]

/-! ## Decoding -/

private def arrowsOf (schemaNode : Json) : Except String (List (Ident × Arrow)) := do
  let nodes ← optionalArrayField schemaNode "arrows"
  nodes.toList.mapM fun node => do
    let name ← fieldStr node "name"
    let src ← fieldStr node "src"
    let tgt ← fieldStr node "tgt"
    pure (name, (⟨src, tgt⟩ : Arrow))

private def decodeEquations (schemaNode : Json) : Except String (List Equation) := do
  let nodes ← optionalArrayField schemaNode "equations"
  nodes.toList.mapM fun node => do
    let lhs ← stringsOf (node.getObjValD "lhs")
    let rhs ← stringsOf (node.getObjValD "rhs")
    let status ← statusOf node
    pure { lhs := lhs, rhs := rhs, status := status }

private def decodeKeys (schemaNode : Json) : Except String (List Key) := do
  let nodes ← optionalArrayField schemaNode "keys"
  nodes.toList.mapM fun node => do
    let object ← fieldStr node "object"
    let arrows ← stringsOf (node.getObjValD "arrows")
    let status ← statusOf node
    pure { object := object, arrows := arrows, status := status }

private def decodeCoproducts (schemaNode : Json) : Except String (List Coproduct) := do
  let nodes ← optionalArrayField schemaNode "coproducts"
  nodes.toList.mapM fun node => do
    let object ← fieldStr node "object"
    let arrows ← stringsOf (node.getObjValD "arrows")
    let status ← statusOf node
    pure { object := object, arrows := arrows, status := status }

private def decodeCells (json : Json) : Except String (List (Option RowId)) := do
  let cells ← json.getArr?
  cells.toList.mapM fun cell =>
    match cell with
    | .null => pure none
    | value => do
      let id ← value.getNat?
      pure (some id)

/-- Decode the mirror JSON of brief §3.1 into a `MirrorDoc`. -/
def decodeMirror (json : Json) : Except String MirrorDoc := do
  let fixture ← fieldStr json "fixture"
  let schemaNode ← json.getObjVal? "schema"
  let objects ← stringsOf (← schemaNode.getObjVal? "objects")
  let arrows ← arrowsOf schemaNode
  let equations ← decodeEquations schemaNode
  let keys ← decodeKeys schemaNode
  let coproducts ← decodeCoproducts schemaNode
  let instNode ← json.getObjVal? "instance"
  let rowsNode ← (← instNode.getObjVal? "rows").getObj?
  let rows ← rowsNode.toList.mapM fun entry => do
    let count ← entry.2.getNat?
    pure (entry.1, count)
  let valuesNode ← (← instNode.getObjVal? "values").getObj?
  let values ← valuesNode.toList.mapM fun entry => do
    let cells ← decodeCells entry.2
    pure (entry.1, cells)
  pure
    { fixture := fixture
      schema :=
        { objects := objects
          arrows := arrows
          equations := equations
          keys := keys
          coproducts := coproducts }
      raw := { rows := rows, values := values } }

end Mirror
end Catherine