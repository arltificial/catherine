import Catherine.Schema

/-!
# Purpose: Lean reference formalization of the catherine finding taxonomy —
violation / dangling / redundant / contradiction / underconstraint, each
with its mandatory witness payload, the three constructible epistemic
labels, and the total order used to sort reported findings
(`openspec/specs/findings`).

## Responsibilities: define the witness structs, the untagged
violation-witness union (equation, key, naturality, coproduct — the
naturality shape is defined and never produced by `check`), the five
finding kinds in declaration order, `Finding {label, kind}`, the total
`Ord` instance (kind declaration order → witness ids → label last), and
`labelFor : ConstraintStatus → EpistemicLabel` (accepted → executed,
proposed → hypothesized). Only `executed`, `hypothesized`, and `unknown`
are constructible: `proved`/`illustrated` do not exist in the type, so
the kernel cannot fabricate them.

## Rationale: witnesses are the soundness story (tasks.md item 8): a finding
type without its witness would let the twin report what the Rust kernel
cannot back (design Decision 2). The order mirrors Rust's derived
`Ord`/custom `Finding::cmp` exactly, so the parity oracle compares
canonical multisets rather than walk order.
-/

namespace Catherine

/-- How a finding was established — its epistemic weight. Exactly three
variants are constructible; `proved`/`illustrated` are deliberately
absent. -/
inductive EpistemicLabel
  | executed
  | hypothesized
  | unknown
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The document spelling of the label (also the final sort tiebreak). -/
def EpistemicLabel.asString : EpistemicLabel → String
  | .executed => "executed"
  | .hypothesized => "hypothesized"
  | .unknown => "unknown"

/-- Declaration-order rank of the label: `executed < hypothesized < unknown`. -/
def EpistemicLabel.rank : EpistemicLabel → Nat
  | .executed => 0
  | .hypothesized => 1
  | .unknown => 2

instance : Ord EpistemicLabel :=
  ⟨fun a b => compare a.rank b.rank⟩

/-- An equation that failed on a row, with both composed path values. -/
structure EquationWitness where
  equation : Nat
  row : RowId
  lhs : RowId
  rhs : RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A key whose arrow tuple agreed on two rows of its object. -/
structure KeyWitness where
  key : Nat
  rows : RowId × RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declared instance map's commutation square fails for one arrow on
one source row (defined for the 1.4 shape; `check` never produces it). -/
structure NaturalityWitness where
  arrow : Ident
  sourceRow : RowId
  lhs : Option RowId
  rhs : Option RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declared coproduct's coverage or disjointness claim failed on one
row of its object. -/
structure CoproductWitness where
  coproduct : Nat
  row : RowId
  injections : List Ident
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The untagged violation witness union; variant order is the taxonomy's
kind-internal order (equations before keys before naturality before
coproducts). -/
inductive ViolationWitness
  | equation (w : EquationWitness)
  | key (w : KeyWitness)
  | naturality (w : NaturalityWitness)
  | coproduct (w : CoproductWitness)
  deriving Repr, DecidableEq, BEq, Inhabited

/-- An arrow cell that holds no row, naming the arrow and the source row
the dangling value sits on. -/
structure DanglingWitness where
  arrow : Ident
  src : RowId
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A declared path derivable from the others and the path it reduces to
(analysis shape; no kernel producer). -/
structure RedundantWitness where
  derivable : Path
  reducing : Path
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The equations whose joint enforcement the bounded analysis found to
admit no instance (analysis shape; no kernel producer). -/
structure ContradictionWitness where
  scope : List Nat
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The tagged domain question of an underconstraint finding: two
parallel paths the schema never forces to agree, or a class of inputs no
equation covers. -/
inductive DomainQuestion
  | parallelPaths (lhs rhs : Path)
  | uncoveredInputs (cls : String) (examples : List RowId)
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The underconstraint witness: the tagged domain question first, then
the question text (analysis shape; no kernel producer). -/
structure UnderconstraintWitness where
  case : DomainQuestion
  question : String
  deriving Repr, DecidableEq, BEq, Inhabited

/-- The finding kind with its mandatory witness payload. Variant
declaration order is the taxonomy's kind order. -/
inductive FindingKind
  | violation (w : ViolationWitness)
  | dangling (w : DanglingWitness)
  | redundant (w : RedundantWitness)
  | contradiction (w : ContradictionWitness)
  | underconstraint (w : UnderconstraintWitness)
  deriving Repr, DecidableEq, BEq, Inhabited

/-- A finding: the label rides on the finding as independent data; the
kind carries the witness. -/
structure Finding where
  label : EpistemicLabel
  kind : FindingKind
  deriving Repr, DecidableEq, BEq, Inhabited

/-! ## The total taxonomy order

Kind (declaration order) → witness ids (canonical id order inside each
witness; within a violation, the variant order) → label last. Mirrors
Rust's derived `Ord` on the witness structs plus the custom
`Finding::cmp`. -/

/-- Field-wise witness comparison, declaration order. -/
def compareEquationWitness (a b : EquationWitness) : Ordering :=
  (compare a.equation b.equation).then <|
    (compare a.row b.row).then <|
      (compare a.lhs b.lhs).then (compare a.rhs b.rhs)

instance : Ord EquationWitness :=
  ⟨compareEquationWitness⟩

/-- Key witness comparison: the key index, then both rows ascending. -/
def compareKeyWitness (a b : KeyWitness) : Ordering :=
  (compare a.key b.key).then <|
    (compare a.rows.1 b.rows.1).then (compare a.rows.2 b.rows.2)

instance : Ord KeyWitness :=
  ⟨compareKeyWitness⟩

/-- Naturality witness comparison, declaration order. -/
def compareNaturalityWitness (a b : NaturalityWitness) : Ordering :=
  (compare a.arrow b.arrow).then <|
    (compare a.sourceRow b.sourceRow).then <|
      (compare a.lhs b.lhs).then (compare a.rhs b.rhs)

instance : Ord NaturalityWitness :=
  ⟨compareNaturalityWitness⟩

/-- Coproduct witness comparison, declaration order. -/
def compareCoproductWitness (a b : CoproductWitness) : Ordering :=
  (compare a.coproduct b.coproduct).then <|
    (compare a.row b.row).then (compare a.injections b.injections)

instance : Ord CoproductWitness :=
  ⟨compareCoproductWitness⟩

/-- The violation variant's rank (declaration order). -/
def ViolationWitness.rank : ViolationWitness → Nat
  | .equation _ => 0
  | .key _ => 1
  | .naturality _ => 2
  | .coproduct _ => 3

/-- Violation witness comparison: variant order, then the witness ids. -/
def compareViolationWitness (a b : ViolationWitness) : Ordering :=
  (compare a.rank b.rank).then <|
    match a, b with
    | .equation x, .equation y => compare x y
    | .key x, .key y => compare x y
    | .naturality x, .naturality y => compare x y
    | .coproduct x, .coproduct y => compare x y
    | _, _ => .eq

instance : Ord ViolationWitness :=
  ⟨compareViolationWitness⟩

/-- Dangling witness comparison: arrow, then source row. -/
def compareDanglingWitness (a b : DanglingWitness) : Ordering :=
  (compare a.arrow b.arrow).then (compare a.src b.src)

instance : Ord DanglingWitness :=
  ⟨compareDanglingWitness⟩

/-- Redundant witness comparison: derivable path, then reducing path. -/
def compareRedundantWitness (a b : RedundantWitness) : Ordering :=
  (compare a.derivable b.derivable).then (compare a.reducing b.reducing)

instance : Ord RedundantWitness :=
  ⟨compareRedundantWitness⟩

/-- Contradiction witness comparison: the equation-index scope. -/
def compareContradictionWitness (a b : ContradictionWitness) : Ordering :=
  compare a.scope b.scope

instance : Ord ContradictionWitness :=
  ⟨compareContradictionWitness⟩

/-- Domain-question rank (declaration order). -/
def DomainQuestion.rank : DomainQuestion → Nat
  | .parallelPaths _ _ => 0
  | .uncoveredInputs _ _ => 1

/-- Domain-question comparison: variant order, then the payload. -/
def compareDomainQuestion (a b : DomainQuestion) : Ordering :=
  (compare a.rank b.rank).then <|
    match a, b with
    | .parallelPaths x y, .parallelPaths x' y' =>
      (compare x x').then (compare y y')
    | .uncoveredInputs c e, .uncoveredInputs c' e' =>
      (compare c c').then (compare e e')
    | _, _ => .eq

instance : Ord DomainQuestion :=
  ⟨compareDomainQuestion⟩

/-- Underconstraint witness comparison: structural witness, then the
question text. -/
def compareUnderconstraintWitness (a b : UnderconstraintWitness) : Ordering :=
  (compare a.case b.case).then (compare a.question b.question)

instance : Ord UnderconstraintWitness :=
  ⟨compareUnderconstraintWitness⟩

/-- Finding-kind rank (declaration order: violation, dangling, redundant,
contradiction, underconstraint). -/
def FindingKind.rank : FindingKind → Nat
  | .violation _ => 0
  | .dangling _ => 1
  | .redundant _ => 2
  | .contradiction _ => 3
  | .underconstraint _ => 4

/-- Finding-kind comparison: kind order, then the witness ids. -/
def compareFindingKind (a b : FindingKind) : Ordering :=
  (compare a.rank b.rank).then <|
    match a, b with
    | .violation x, .violation y => compare x y
    | .dangling x, .dangling y => compare x y
    | .redundant x, .redundant y => compare x y
    | .contradiction x, .contradiction y => compare x y
    | .underconstraint x, .underconstraint y => compare x y
    | _, _ => .eq

instance : Ord FindingKind :=
  ⟨compareFindingKind⟩

/-- The total finding order: kind first, label last. -/
def compareFinding (a b : Finding) : Ordering :=
  (compare a.kind b.kind).then (compare a.label b.label)

instance : Ord Finding :=
  ⟨compareFinding⟩

/-- The one label rule shared by equations, keys, and coproducts:
accepted constraints are executed, proposed ones are questions. -/
def labelFor : ConstraintStatus → EpistemicLabel
  | .accepted => .executed
  | .proposed => .hypothesized

end Catherine