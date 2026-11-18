/-!
# Purpose: Lean reference formalization of the catherine-core schema —
objects, arrows, paths, equations with constraint status
(`accepted`/`proposed`), declarative keys, declared coproducts, and
declared pullback squares, mirroring `openspec/specs/model-document` and
the Rust kernel types in `crates/catherine-core/src/model/{mod,relations}.rs`.

## Responsibilities: define the schema datatypes (dense per-object row id
space, paths, arrows, equations, keys, coproducts, pullbacks, constraint
status) and
the *total* lookups the checker and the harness need (`arrow?`,
`rowCount?` return `Option`, never panic and never assume the parsed
presentation's validation guarantees); nothing execution-side.

## Rationale: the twin must represent exactly what the Rust kernel
represents (tasks.md item 3), no more — extra structure would let proofs
drift from what fixtures exercise (design Decision 4 divergence
protocol). Ordered collections are `List`s kept in the canonical order
the Rust parser produces (`objects` sorted, `arrows` by name, equations
by `(lhs, rhs, rationale, status)`, keys/coproducts by `(object, arrows,
status)`); the mirror projection owns that ordering.
-/

namespace Catherine

/-- A declared identifier (`[A-Za-z_][A-Za-z0-9_]*` in the document; the
type carries no validity re-check, as in Rust). -/
abbrev Ident := String

/-- A dense per-object row id (`0..n-1`, assigned in sorted row-name
order by the trusted projection). -/
abbrev RowId := Nat

/-- A composable arrow sequence: the path `f ∘ g ∘ …` as its arrow names
in composition order. -/
abbrev Path := List Ident

/-- A generating morphism, typed with exactly one src and one tgt
(self-loops are legal). -/
structure Arrow where
  src : Ident
  tgt : Ident
  deriving Repr, DecidableEq, BEq, Inhabited

/-- Per-constraint status: accepted constraints are executed, proposed
ones are questions (the one label rule shared by equations, keys, and
coproducts). -/
inductive ConstraintStatus
  | accepted
  | proposed
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The document spelling of the status (also the canonical sort key). -/
def ConstraintStatus.asString : ConstraintStatus → String
  | .accepted => "accepted"
  | .proposed => "proposed"

/-- A path equation `lhs = rhs` with its constraint status. The
rationale is host presentation (it participates in the parser's canonical
sort) and is deliberately absent from the kernel shape, exactly as the
mirror's schema section carries only `lhs`, `rhs`, and `status`. -/
structure Equation where
  lhs : Path
  rhs : Path
  status : ConstraintStatus
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declarative key: injectivity of an object's arrow-value tuple over
that object's rows. -/
structure Key where
  object : Ident
  arrows : List Ident
  status : ConstraintStatus
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declared coproduct: coverage and disjointness of `object` over the
injection arrows `V_k → object`. -/
structure Coproduct where
  object : Ident
  arrows : List Ident
  status : ConstraintStatus
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declared pullback square (format 1.6): the apex object, its two
leg arrows, and the two cospan arrows whose common target the square
projects onto. -/
structure Pullback where
  apex : Ident
  legs : List Ident
  cospan : List Ident
  status : ConstraintStatus
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The schema presentation in canonical order: `objects` sorted,
`arrows` by name, equations by `(lhs, rhs, rationale, status)`,
keys/coproducts by `(object, arrows, status)`, pullbacks by
`(apex, legs, cospan, status)`. -/
structure Schema where
  objects : List Ident
  arrows : List (Ident × Arrow)
  equations : List Equation
  keys : List Key
  coproducts : List Coproduct
  pullbacks : List Pullback
  deriving Repr, DecidableEq, BEq, Inhabited

/-- Total arrow lookup: the arrow declared under `name`, if any. -/
def Schema.arrow? (schema : Schema) (name : Ident) : Option Arrow :=
  match schema.arrows.find? (fun entry => entry.1 == name) with
  | some entry => some entry.2
  | none => none

/-- Total row-space lookup: the dense row count of `object` in a row
table, if the object is declared there. Namespaced under `Schema` because
the schema owns the row-id discipline; the table itself comes from the
projected instance. -/
def Schema.rowCount? (rows : List (Ident × Nat)) (object : Ident) : Option Nat :=
  match rows.find? (fun entry => entry.1 == object) with
  | some entry => some entry.2
  | none => none

end Catherine