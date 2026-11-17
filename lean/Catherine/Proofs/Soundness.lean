import Catherine.Proofs.Basic

/-!
# Purpose: state kernel soundness — every finding `Check.check` emits
carries the semantic witness that demonstrates it, and no other witness
variant (naturality, redundant, contradiction, underconstraint) is ever
produced. Referee: `openspec/specs/kernel-checks/spec.md` ("Every
declared equation is executed on every applicable row", "Declared keys
are executed as injectivity checks", "Accepted coproducts execute
coverage with witnesses", "Coproduct disjointness violations name the
colliding injections", "Unresolved injection cells suppress coverage
only", "Dangling values are checked, not fatal") and
`openspec/specs/findings/spec.md` ("Findings are typed and carry
witnesses", "Witness shapes are fixed per finding kind").

## Responsibilities: state `soundness` as a four-way witness
disjunction over every member of `check` (equation with index, applicable
row, both composite values, and the `labelFor` status label; key with its
index, ascending colliding pair, equal resolved tuple, and label rule;
coproduct with its index, row, complete/incomplete-mode witness, and the
named injection list; dangling with the arrow, the unresolved cell, and
the proof that no typing violation accounts for it). State and prove the
four exclusion lemmas that justify omitting the unproducible variants
from that disjunction — they are theorems about `check`, not assumptions.

## Rationale: the four-way disjunction mirrors the witnesses the specs
mandate, and the exclusions are separate theorems because "the walk
cannot produce a naturality finding" is a fact to be proved from the
walk, not a conjecture baked into a weakened statement. The membership
bridges below are all stated against the frozen walk, so the proof reads
each family's filter as the semantic predicate `Basic` defines for it;
the duplicate-prone list plumbing (`groupedRows`/`ascendingPairs`
ascending-order invariants) is kept private to this file and consolidated
by a later refactor pass.
-/

namespace Catherine
namespace Proofs

/-! ## List plumbing shared by the four family bridges -/

/-- Membership in a `flatten`ed `mapIdx` run: some declaration of `l`
(indexed by its position) emits `b`. -/
private theorem mem_flatten_mapIdx {α β : Type} (f : Nat → α → List β)
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

/-- A `contains` test that has failed means the pair is absent. -/
private theorem contains_eq_false_iff_not_mem {α : Type} [BEq α] [LawfulBEq α]
    (l : List α) (a : α) : (l.contains a = false) ↔ a ∉ l := by
  rw [← List.contains_iff_mem]
  cases h : l.contains a <;> simp_all

/-! ## Dangling membership -/

/-- Membership in `Check.danglingFindings` is exactly a table entry with an
unresolved cell at an in-range source row that the violated-pair list does
not account for. -/
private theorem mem_danglingFindings_iff (inst : Instance.TypedInstance)
    (violated : List (Ident × RowId)) (finding : Finding) :
    finding ∈ Check.danglingFindings inst violated ↔
      ∃ (arrow : Ident) (table : List (Option RowId)) (src : RowId),
        (arrow, table) ∈ inst.tables ∧
        table[src]? = some none ∧
        violated.contains (arrow, src) = false ∧
        finding = { label := .executed,
                    kind := .dangling { arrow := arrow, src := src } } := by
  rw [Check.danglingFindings, List.mem_flatMap]
  constructor
  · rintro ⟨entry, hentry, hmem⟩
    obtain ⟨arrow, table⟩ := entry
    rw [List.mem_filterMap] at hmem
    obtain ⟨src, hsrc, hf⟩ := hmem
    rw [List.mem_range] at hsrc
    rcases hcell : table[src]? with _ | v
    · rw [hcell] at hf
      dsimp only at hf
      cases hf
    · rcases v with _ | v
      · rw [hcell] at hf
        by_cases hc : violated.contains (arrow, src) = true
        · rw [ite_eq_left hc] at hf
          cases hf
        · rw [ite_eq_right hc] at hf
          exact ⟨arrow, table, src, hentry, hcell, by simpa using hc, by simpa using hf.symm⟩
      · rw [hcell] at hf
        dsimp only at hf
        cases hf
  · rintro ⟨arrow, table, src, hentry, hcell, hnotviol, rfl⟩
    refine ⟨(arrow, table), hentry, ?_⟩
    rw [List.mem_filterMap]
    have hlen : src < table.length := (List.getElem?_eq_some_iff.mp hcell).choose
    refine ⟨src, List.mem_range.mpr hlen, ?_⟩
    rw [hcell]
    rw [ite_eq_right]
    simpa using hnotviol

/-! ## Equation membership -/

/-- Membership in `Check.equationFindings` is exactly the semantic
equation witness: a declaration with a non-empty `lhs`, a declared lead
arrow, a dense source row space, and an applicable row whose composites
both resolve and disagree. -/
private theorem mem_equationFindings_iff (schema : Schema)
    (inst : Instance.TypedInstance) (finding : Finding) :
    finding ∈ Check.equationFindings schema inst ↔
      ∃ (index : Nat) (equation : Equation) (row : RowId),
        schema.equations[index]? = some equation ∧
        EquationApplies schema inst equation row ∧
        ∃ lhs rhs : RowId,
          EvalPath.compose inst equation.lhs row = some lhs ∧
          EvalPath.compose inst equation.rhs row = some rhs ∧
          lhs ≠ rhs ∧
          finding =
            { label := labelFor equation.status,
              kind := .violation (.equation
                { equation := index, row := row, lhs := lhs, rhs := rhs }) } := by
  rw [Check.equationFindings, mem_flatten_mapIdx]
  constructor
  · rintro ⟨index, equation, hidx, hmem⟩
    rcases hlhs : equation.lhs with _ | ⟨lead, rest⟩
    · rw [hlhs] at hmem
      simp at hmem
    · rw [hlhs] at hmem
      dsimp only at hmem
      rcases harrow : schema.arrow? lead with _ | arrow
      · rw [harrow] at hmem
        simp at hmem
      · rw [harrow] at hmem
        dsimp only at hmem
        rcases hrows : Schema.rowCount? inst.rows arrow.src with _ | n
        · rw [hrows] at hmem
          simp at hmem
        · rw [hrows] at hmem
          dsimp only at hmem
          rw [List.mem_filterMap] at hmem
          obtain ⟨row, hrowmem, hf⟩ := hmem
          rw [List.mem_range] at hrowmem
          obtain ⟨lhs, rhs, hl, hr, hne, rfl⟩ :=
            (equationFinding_eq_some_iff inst index equation row finding).mp hf
          exact ⟨index, equation, row, hidx,
                 ⟨lead, rest, arrow, n, hlhs, harrow, hrows, hrowmem⟩,
                 lhs, rhs, hl, hr, hne, rfl⟩
  · rintro ⟨index, equation, row, hidx, happ, lhs, rhs, hl, hr, hne, rfl⟩
    obtain ⟨lead, rest, arrow, n, hlhs, harrow, hrows, hrowlt⟩ := happ
    refine ⟨index, equation, hidx, ?_⟩
    rw [hlhs]
    simp only []
    rw [harrow]
    simp only []
    rw [hrows]
    simp only []
    exact List.mem_filterMap.mpr
      ⟨row, List.mem_range.mpr hrowlt,
       (equationFinding_eq_some_iff inst index equation row _).mpr
         ⟨lhs, rhs, hl, hr, hne, rfl⟩⟩

/-! ## Key membership: the grouping invariants

`Check.groupedRows` folds over the key object's rows, `insertRow`-ing each
row that resolves its whole tuple. The invariant maintained by that fold
is what the membership bridge needs: every group's rows are strictly
ascending, all rows of one group share the same resolved tuple, and every
row visited is below the fold's bound. -/

/-- The invariant of the `insertRow` accumulator. -/
private structure GroupGood (tables : List (List (Option RowId)))
    (bound : Nat) (groups : List (List RowId × List RowId)) : Prop where
  /-- Every group's row list is strictly ascending. -/
  sorted : ∀ g ∈ groups, g.2.Pairwise (· < ·)
  /-- Every row of a group resolves to the group's tuple. -/
  keyOf : ∀ g ∈ groups, ∀ r ∈ g.2, Check.keyTuple tables r = some g.1
  /-- Every row of a group lies below the fold's current bound. -/
  ltBound : ∀ g ∈ groups, ∀ r ∈ g.2, r < bound

/-- `Check.insertRow` at a matching tuple appends the row to that group. -/
private theorem insertRow_cons_true (tuple : List RowId) (row : RowId)
    (existing rows : List RowId) (rest : List (List RowId × List RowId))
    (h : (existing == tuple) = true) :
    Check.insertRow tuple row ((existing, rows) :: rest)
      = (existing, rows ++ [row]) :: rest := by
  show (if existing == tuple then (existing, rows ++ [row]) :: rest
        else (existing, rows) :: Check.insertRow tuple row rest) = _
  rw [ite_eq_left h]

/-- `Check.insertRow` at a fresh tuple keeps the group list and recurses. -/
private theorem insertRow_cons_false (tuple : List RowId) (row : RowId)
    (existing rows : List RowId) (rest : List (List RowId × List RowId))
    (h : (existing == tuple) = false) :
    Check.insertRow tuple row ((existing, rows) :: rest)
      = (existing, rows) :: Check.insertRow tuple row rest := by
  show (if existing == tuple then (existing, rows ++ [row]) :: rest
        else (existing, rows) :: Check.insertRow tuple row rest) = _
  rw [ite_eq_right]
  simp [h]

/-- One `insertRow` step preserves the accumulator invariant, sliding the
bound from `row` to `row + 1`. -/
private theorem insertRow_good (tables : List (List (Option RowId)))
    (tuple : List RowId) (row : RowId)
    (groups : List (List RowId × List RowId))
    (hrow : Check.keyTuple tables row = some tuple)
    (hg : GroupGood tables row groups) :
    GroupGood tables (row + 1) (Check.insertRow tuple row groups) := by
  revert hg
  induction groups with
  | nil =>
    intro hg
    rw [show Check.insertRow tuple row ([] : List (List RowId × List RowId))
          = [(tuple, [row])] from rfl]
    refine ⟨?_, ?_, ?_⟩
    · intro g hg
      rw [List.mem_singleton] at hg
      subst hg
      simp
    · intro g hg r hr
      rw [List.mem_singleton] at hg
      subst hg
      rw [List.mem_singleton] at hr
      subst hr
      exact hrow
    · intro g hg r hr
      rw [List.mem_singleton] at hg
      subst hg
      rw [List.mem_singleton] at hr
      rw [hr]
      exact Nat.lt_succ_self row
  | cons head rest ih =>
    intro hg
    obtain ⟨hp, hk, hl⟩ := hg
    obtain ⟨existing, rows⟩ := head
    by_cases hbeq : (existing == tuple) = true
    · rw [insertRow_cons_true tuple row existing rows rest hbeq]
      refine ⟨?_, ?_, ?_⟩
      · intro g hg
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · rw [List.pairwise_append]
          refine ⟨hp (existing, rows) (by simp), List.pairwise_singleton _ row, ?_⟩
          intro a ha b hb
          rw [List.mem_singleton] at hb
          subst hb
          exact hl (existing, rows) (by simp) a ha
        · exact hp g (List.mem_cons_of_mem _ hg)
      · intro g hg r hr
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · rw [List.mem_append] at hr
          rcases hr with hr | hr
          · exact hk (existing, rows) (by simp) r hr
          · rw [List.mem_singleton] at hr
            rw [hr]
            rw [hrow]
            rw [← beq_iff_eq.mp hbeq]
        · exact hk g (List.mem_cons_of_mem _ hg) r hr
      · intro g hg r hr
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · rw [List.mem_append] at hr
          rcases hr with hr | hr
          · exact Nat.lt_succ_of_lt (hl (existing, rows) (by simp) r hr)
          · rw [List.mem_singleton] at hr
            rw [hr]
            exact Nat.lt_succ_self row
        · exact Nat.lt_succ_of_lt (hl g (List.mem_cons_of_mem _ hg) r hr)
    · rw [insertRow_cons_false tuple row existing rows rest (by simpa using hbeq)]
      have hrest : GroupGood tables row rest :=
        ⟨fun g hg => hp g (List.mem_cons_of_mem _ hg),
         fun g hg => hk g (List.mem_cons_of_mem _ hg),
         fun g hg => hl g (List.mem_cons_of_mem _ hg)⟩
      obtain ⟨ihp, ihk, ihl⟩ := ih hrest
      refine ⟨?_, ?_, ?_⟩
      · intro g hg
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · exact hp (existing, rows) (by simp)
        · exact ihp g hg
      · intro g hg r hr
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · exact hk (existing, rows) (by simp) r hr
        · exact ihk g hg r hr
      · intro g hg r hr
        rw [List.mem_cons] at hg
        rcases hg with rfl | hg
        · exact Nat.lt_succ_of_lt (hl (existing, rows) (by simp) r hr)
        · exact ihl g hg r hr

/-- The grouping fold's step: group the row by its resolved tuple, if any. -/
private def stepGroups (tables : List (List (Option RowId)))
    (groups : List (List RowId × List RowId)) (row : RowId) :
    List (List RowId × List RowId) :=
  match Check.keyTuple tables row with
  | none => groups
  | some tuple => Check.insertRow tuple row groups

/-- One fold step preserves the accumulator invariant. -/
private theorem stepGroups_good (tables : List (List (Option RowId))) (n : Nat)
    (groups : List (List RowId × List RowId))
    (hg : GroupGood tables n groups) :
    GroupGood tables (n + 1) (stepGroups tables groups n) := by
  obtain ⟨hp, hk, hl⟩ := hg
  rcases hkey : Check.keyTuple tables n with _ | tuple
  · rw [show stepGroups tables groups n = groups from by simp [stepGroups, hkey]]
    exact ⟨hp, hk, fun g hg r hr => Nat.lt_succ_of_lt (hl g hg r hr)⟩
  · rw [show stepGroups tables groups n = Check.insertRow tuple n groups from
          by simp [stepGroups, hkey]]
    exact insertRow_good tables tuple n groups hkey ⟨hp, hk, hl⟩

/-- The `groupedRows` fold maintains the accumulator invariant over the
whole dense row space. -/
private theorem foldl_range_good (tables : List (List (Option RowId))) (n : Nat) :
    GroupGood tables n ((List.range n).foldl (stepGroups tables) []) := by
  induction n with
  | zero => exact ⟨by simp, by simp, by simp⟩
  | succ k ih =>
    rw [List.range_succ, List.foldl_append]
    exact stepGroups_good tables k _ ih

/-- Every group emitted by `Check.groupedRows` is strictly ascending, its
rows all resolve to one shared tuple, and they all lie inside the row
space. -/
private theorem groupedRows_spec (tables : List (List (Option RowId))) (n : Nat)
    {rows : List RowId} (h : rows ∈ Check.groupedRows tables n) :
    rows.Pairwise (· < ·) ∧
      (∃ tuple, ∀ r ∈ rows, Check.keyTuple tables r = some tuple) ∧
      (∀ r ∈ rows, r < n) := by
  rw [Check.groupedRows, List.mem_map] at h
  obtain ⟨g, hg, rfl⟩ := h
  have hgood := foldl_range_good tables n
  exact ⟨hgood.sorted g hg,
         ⟨g.1, fun r hr => hgood.keyOf g hg r hr⟩,
         fun r hr => hgood.ltBound g hg r hr⟩

/-- The first component of an ascending pair is a row of the group. -/
private theorem mem_ascendingPairs_fst {l : List RowId} {a b : RowId}
    (h : (a, b) ∈ Check.ascendingPairs l) : a ∈ l := by
  induction l with
  | nil => simp [Check.ascendingPairs] at h
  | cons hd tl ih =>
    rw [show Check.ascendingPairs (hd :: tl)
          = tl.map (fun hi => (hd, hi)) ++ Check.ascendingPairs tl from rfl] at h
    rcases List.mem_append.mp h with h | h
    · rw [List.mem_map] at h
      obtain ⟨x, _, hx⟩ := h
      simp only [Prod.mk.injEq] at hx
      obtain ⟨rfl, _⟩ := hx
      exact List.mem_cons_self
    · exact List.mem_cons_of_mem _ (ih h)

/-- The second component of an ascending pair is a row of the group. -/
private theorem mem_ascendingPairs_snd {l : List RowId} {a b : RowId}
    (h : (a, b) ∈ Check.ascendingPairs l) : b ∈ l := by
  induction l with
  | nil => simp [Check.ascendingPairs] at h
  | cons hd tl ih =>
    rw [show Check.ascendingPairs (hd :: tl)
          = tl.map (fun hi => (hd, hi)) ++ Check.ascendingPairs tl from rfl] at h
    rcases List.mem_append.mp h with h | h
    · rw [List.mem_map] at h
      obtain ⟨x, hxmem, hx⟩ := h
      simp only [Prod.mk.injEq] at hx
      obtain ⟨_, rfl⟩ := hx
      exact List.mem_cons_of_mem _ hxmem
    · exact List.mem_cons_of_mem _ (ih h)

/-- An ascending pair of a strictly ascending group is itself ascending. -/
private theorem ascendingPairs_lt {l : List RowId} (hs : l.Pairwise (· < ·))
    {a b : RowId} (h : (a, b) ∈ Check.ascendingPairs l) : a < b := by
  induction l with
  | nil => simp [Check.ascendingPairs] at h
  | cons hd tl ih =>
    rw [show Check.ascendingPairs (hd :: tl)
          = tl.map (fun hi => (hd, hi)) ++ Check.ascendingPairs tl from rfl] at h
    rw [List.pairwise_cons] at hs
    rcases List.mem_append.mp h with h | h
    · rw [List.mem_map] at h
      obtain ⟨x, hxmem, hx⟩ := h
      simp only [Prod.mk.injEq] at hx
      obtain ⟨rfl, rfl⟩ := hx
      exact hs.1 x hxmem
    · exact ih hs.2 h

/-- Membership in `Check.keyFindings` yields the semantic key collision:
both rows of the ascending pair resolve the declaration's whole key tuple
to the same value. (The converse would additionally need both rows inside
the object's dense row space, which the walk supplies but `KeyCollision`
does not state; soundness only consumes this direction.) -/
private theorem mem_keyFindings_forward (schema : Schema)
    (inst : Instance.TypedInstance) {finding : Finding}
    (h : finding ∈ Check.keyFindings schema inst) :
    ∃ (index : Nat) (key : Key) (lo hi : RowId) (tuple : List RowId),
      schema.keys[index]? = some key ∧
      KeyCollision schema inst key lo hi tuple ∧
      finding =
        { label := labelFor key.status,
          kind := .violation (.key { key := index, rows := (lo, hi) }) } := by
  rw [Check.keyFindings, mem_flatten_mapIdx] at h
  obtain ⟨index, key, hidx, hmem⟩ := h
  rcases hrow : Schema.rowCount? inst.rows key.object with _ | n
  · rcases htab : Check.keyTables schema inst key with _ | tables
    · rw [hrow, htab] at hmem
      simp at hmem
    · rw [hrow, htab] at hmem
      simp at hmem
  · rcases htab : Check.keyTables schema inst key with _ | tables
    · rw [hrow, htab] at hmem
      simp at hmem
    · rw [hrow, htab] at hmem
      simp only [List.mem_flatMap] at hmem
      obtain ⟨rows, hrows, hpair⟩ := hmem
      rw [List.mem_map] at hpair
      obtain ⟨pair, hpairmem, hf⟩ := hpair
      obtain ⟨lo, hi⟩ := pair
      obtain ⟨hsorted, ⟨tuple, htuple⟩, _⟩ := groupedRows_spec tables n hrows
      refine ⟨index, key, lo, hi, tuple, hidx, ⟨?_, ?_, ?_⟩, ?_⟩
      · exact ascendingPairs_lt hsorted hpairmem
      · exact ⟨tables, htab, htuple lo (mem_ascendingPairs_fst hpairmem)⟩
      · exact ⟨tables, htab, htuple hi (mem_ascendingPairs_snd hpairmem)⟩
      · exact hf.symm

/-! ## Coproduct membership -/

/-- The per-row decision of one declaration's walk over its row space,
characterized: the emitted finding names the row, the covering injection
list, and the mode (complete images require exactly one cover; partial
images require at least two). -/
private theorem mem_coproduct_rows (n : Nat) (images : List Check.InjectionImage)
    (complete : Bool) (label : EpistemicLabel) (index : Nat) (finding : Finding) :
    finding ∈ (List.range n).filterMap (fun row =>
        let covering := Check.coveringInjections images row
        if complete then
          if covering.length == 1 then none
          else some (Check.coproductFinding label index row covering)
        else
          if 2 ≤ covering.length then some (Check.coproductFinding label index row covering)
          else none)
    ↔ ∃ (row : RowId) (injections : List Ident),
        row < n ∧
        injections = Check.coveringInjections images row ∧
        ((complete = true ∧ injections.length ≠ 1) ∨
          (complete = false ∧ 2 ≤ injections.length)) ∧
        finding = Check.coproductFinding label index row injections := by
  rw [List.mem_filterMap]
  constructor
  · rintro ⟨row, hrow, hf⟩
    rw [List.mem_range] at hrow
    dsimp only at hf
    by_cases hcomp : complete = true
    · rw [ite_eq_left hcomp] at hf
      by_cases hlen : (Check.coveringInjections images row).length = 1
      · rw [ite_eq_left (beq_iff_eq.mpr hlen)] at hf
        simp at hf
      · rw [ite_eq_right (by rw [beq_iff_eq]; exact hlen)] at hf
        exact ⟨row, _, hrow, rfl, Or.inl ⟨hcomp, hlen⟩, by simpa using hf.symm⟩
    · rw [ite_eq_right hcomp] at hf
      by_cases htwo : 2 ≤ (Check.coveringInjections images row).length
      · rw [ite_eq_left htwo] at hf
        exact ⟨row, _, hrow, rfl,
               Or.inr ⟨by simpa using hcomp, htwo⟩, by simpa using hf.symm⟩
      · rw [ite_eq_right htwo] at hf
        simp at hf
  · rintro ⟨row, injections, hrow, hinj, hdisj, rfl⟩
    refine ⟨row, List.mem_range.mpr hrow, ?_⟩
    rw [← hinj]
    rcases hdisj with ⟨hcomp, hne⟩ | ⟨hfalse, htwo⟩
    · rw [ite_eq_left hcomp]
      rw [ite_eq_right (by rw [beq_iff_eq]; exact hne)]
    · rw [ite_eq_right (by simpa using hfalse)]
      rw [ite_eq_left htwo]

/-- One declaration's findings, characterized semantically: it emits a row's
finding exactly when the row applies and the declaration's decision fails
there, with the covering injections named. The match reduction is
definitional: the RHS of the audit's `CoproductRowApplies`/`Failure`
predicates is what the walk computes once the two lookups are known. -/
private theorem mem_coproductDeclaration_iff (schema : Schema)
    (inst : Instance.TypedInstance) (index : Nat) (coproduct : Coproduct)
    (finding : Finding) :
    finding ∈ Check.coproductDeclaration schema inst index coproduct ↔
      ∃ (row : RowId) (injections : List Ident),
        CoproductRowApplies inst coproduct row ∧
        CoproductRowFailure schema inst coproduct row injections ∧
        finding = Check.coproductFinding (labelFor coproduct.status) index row injections := by
  rw [Check.coproductDeclaration]
  rcases htab : Check.injectionTables schema inst coproduct with _ | tables
  · constructor
    · intro h
      simp at h
    · rintro ⟨row, injections, _, ⟨tables', images, htab', _, _, _⟩, _⟩
      simp_all
  · rcases hrow : Schema.rowCount? inst.rows coproduct.object with _ | n
    · constructor
      · intro h
        simp at h
      · rintro ⟨row, injections, ⟨n', hrow', _⟩, _, _⟩
        simp_all
    · constructor
      · intro h
        obtain ⟨row, injections, hlt, hinj, hdisj, heq⟩ :=
          (mem_coproduct_rows n (injectionImages tables)
            ((injectionImages tables).length == tables.length)
            (labelFor coproduct.status) index finding).mp h
        refine ⟨row, injections, ⟨n, hrow, hlt⟩, ?_, heq⟩
        refine ⟨tables, injectionImages tables, htab, rfl, hinj, ?_⟩
        rcases hdisj with ⟨hcomp, hne⟩ | ⟨hfalse, htwo⟩
        · exact Or.inl ⟨beq_iff_eq.mp hcomp, hne⟩
        · refine Or.inr ⟨?_, htwo⟩
          intro hbeq
          have htrue : ((injectionImages tables).length == tables.length) = true :=
            beq_iff_eq.mpr hbeq
          rw [htrue] at hfalse
          cases hfalse
      · rintro ⟨row, injections, happ, hfail, heq⟩
        obtain ⟨n', hrow', hlt⟩ := happ
        obtain ⟨tables', images, htab', himg, hinj, hdisj⟩ := hfail
        have hn : n' = n := Option.some.inj (hrow'.symm.trans hrow)
        have hte : tables' = tables := Option.some.inj (htab'.symm.trans htab)
        rw [hn] at hlt
        rw [hte] at himg
        rw [himg] at hinj
        rw [hte, himg] at hdisj
        refine (mem_coproduct_rows n (injectionImages tables)
          ((injectionImages tables).length == tables.length)
          (labelFor coproduct.status) index finding).mpr
          ⟨row, injections, hlt, hinj, ?_, heq⟩
        rcases hdisj with ⟨hlen, hne⟩ | ⟨hneq, htwo⟩
        · exact Or.inl ⟨beq_iff_eq.mpr hlen, hne⟩
        · refine Or.inr ⟨?_, htwo⟩
          rw [Bool.eq_false_iff]
          intro hbeq
          exact hneq (beq_iff_eq.mp hbeq)

/-- Membership in `Check.coproductFindings` yields the semantic coproduct
failure: an applicable row of a declared coproduct whose complete-image
accounting fails, naming the covering injections. -/
private theorem mem_coproductFindings_forward (schema : Schema)
    (inst : Instance.TypedInstance) {finding : Finding}
    (h : finding ∈ Check.coproductFindings schema inst) :
    ∃ (index : Nat) (coproduct : Coproduct) (row : RowId)
      (injections : List Ident),
      schema.coproducts[index]? = some coproduct ∧
      CoproductRowApplies inst coproduct row ∧
      CoproductRowFailure schema inst coproduct row injections ∧
      finding =
        { label := labelFor coproduct.status,
          kind := .violation (.coproduct
            { coproduct := index, row := row, injections := injections }) } := by
  rw [Check.coproductFindings, mem_flatten_mapIdx] at h
  obtain ⟨index, coproduct, hidx, hmem⟩ := h
  obtain ⟨row, injections, happ, hfail, rfl⟩ :=
    (mem_coproductDeclaration_iff schema inst index coproduct finding).mp hmem
  exact ⟨index, coproduct, row, injections, hidx, happ, hfail, rfl⟩

/-! ## Soundness -/

/-- The violated-pair list of `check` contains exactly the typing-violated
cells, so a failed `contains` test is the semantic absence of a typing
violation. -/
private theorem contains_false_iff_not_typingViolatedAt
    (inst : Instance.TypedInstance) (arrow : Ident) (src : RowId) :
    ((inst.typingViolations.map fun violation =>
        (violation.arrow, violation.src)).contains (arrow, src) = false) ↔
      ¬ TypingViolatedAt inst arrow src :=
  contains_eq_false_iff_not_mem _ _

/-- **Soundness** — every finding produced by `Check.check schema inst`
carries its semantic witness.

Equation disjunct: a declared equation (canonical index) on an
applicable row whose both composites resolve to unequal rows, labeled by
`labelFor` the declaration's status (`openspec/specs/kernel-checks/spec.md`
"Every declared equation is executed on every applicable row"; witnesses
per `openspec/specs/findings/spec.md` "Violation finding carries row and
values").

Key disjunct: a declared key (canonical index) whose two distinct
ascending rows resolve equal tuples, with the status label
(`kernel-checks`: "Declared keys are executed as injectivity checks";
`findings`: "Key violation carries its rows").

Coproduct disjunct: a declared coproduct (canonical index) on an
applicable row failing the exact complete-image accounting of
`Check.coproductDeclaration`, naming the covering injection list and the
status label (`kernel-checks`: coverage, disjointness, unresolved-image
suppression; `findings`: "Coproduct violation carries its row and
injections").

Dangling disjunct: a table cell that resolves to no row and that no
typing violation accounts for, labeled `executed` (`kernel-checks`:
"Dangling values are checked, not fatal", "The instance builder is total
and typing-enforcing"; `findings`: arrow/source-row witness shape).

The unproducible variants (naturality, redundant, contradiction,
underconstraint) are absent from the disjunction because
`check_no_naturality`, `check_no_redundant`,
`check_no_contradiction`, and `check_no_underconstraint` prove that
`check` never emits them — not because the statement assumes it. -/
theorem soundness (schema : Schema) (inst : Instance.TypedInstance) :
    ∀ finding ∈ Check.check schema inst,
      (∃ (index : Nat) (equation : Equation) (row : RowId),
        schema.equations[index]? = some equation ∧
        finding.label = labelFor equation.status ∧
        EquationApplies schema inst equation row ∧
        ∃ lhs rhs : RowId,
          EvalPath.compose inst equation.lhs row = some lhs ∧
          EvalPath.compose inst equation.rhs row = some rhs ∧
          lhs ≠ rhs ∧
          finding.kind =
            .violation (.equation
              { equation := index, row := row, lhs := lhs, rhs := rhs }))
      ∨ (∃ (index : Nat) (key : Key) (lo hi : RowId) (tuple : List RowId),
        schema.keys[index]? = some key ∧
        finding.label = labelFor key.status ∧
        KeyCollision schema inst key lo hi tuple ∧
        finding.kind = .violation (.key { key := index, rows := (lo, hi) }))
      ∨ (∃ (index : Nat) (coproduct : Coproduct) (row : RowId)
          (injections : List Ident),
        schema.coproducts[index]? = some coproduct ∧
        finding.label = labelFor coproduct.status ∧
        CoproductRowApplies inst coproduct row ∧
        CoproductRowFailure schema inst coproduct row injections ∧
        finding.kind =
          .violation (.coproduct
            { coproduct := index, row := row, injections := injections }))
      ∨ (∃ (arrow : Ident) (src : RowId),
        finding.label = .executed ∧
        DanglingCellSemantic inst arrow src ∧
        finding.kind = .dangling { arrow := arrow, src := src }) := by
  intro finding hmem
  rw [mem_check_iff] at hmem
  rcases hmem with h | h | h | h
  · obtain ⟨arrow, table, src, htab, hcell, hnotviol, rfl⟩ :=
      (mem_danglingFindings_iff inst
        (inst.typingViolations.map fun violation =>
          (violation.arrow, violation.src)) finding).mp h
    have hnt : ¬ TypingViolatedAt inst arrow src :=
      (contains_false_iff_not_typingViolatedAt inst arrow src).mp hnotviol
    exact Or.inr (Or.inr (Or.inr
      ⟨arrow, src, rfl, ⟨⟨table, htab, hcell⟩, hnt⟩, rfl⟩))
  · obtain ⟨index, equation, row, hidx, happ, lhs, rhs, hl, hr, hne, rfl⟩ :=
      (mem_equationFindings_iff schema inst finding).mp h
    exact Or.inl ⟨index, equation, row, hidx, rfl, happ,
                  lhs, rhs, hl, hr, hne, rfl⟩
  · obtain ⟨index, key, lo, hi, tuple, hidx, hcoll, rfl⟩ :=
      mem_keyFindings_forward schema inst h
    exact Or.inr (Or.inl ⟨index, key, lo, hi, tuple, hidx, rfl, hcoll, rfl⟩)
  · obtain ⟨index, coproduct, row, injections, hidx, happ, hfail, rfl⟩ :=
      mem_coproductFindings_forward schema inst h
    exact Or.inr (Or.inr (Or.inl
      ⟨index, coproduct, row, injections, hidx, rfl, happ, hfail, rfl⟩))

/-- `check` never produces a naturality violation: the variant shape is
defined in the taxonomy (`Catherine.Findings.NaturalityWitness`) but no
kernel family constructs it. Referee:
`openspec/specs/findings/spec.md` ("Witness shapes are fixed per finding
kind" — naturality is not a kernel producer) and
`openspec/specs/kernel-checks/spec.md` (the executed families are
equations, keys, coproducts, dangling). -/
theorem check_no_naturality (schema : Schema) (inst : Instance.TypedInstance) :
    ∀ finding ∈ Check.check schema inst,
      ∀ witness : NaturalityWitness,
        finding.kind ≠ .violation (.naturality witness) := by
  intro finding hmem witness heq
  have hs := soundness schema inst finding hmem
  rcases hs with ⟨_, _, _, _, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, hkind⟩
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq

/-- `check` never produces a redundant finding: derivability analysis is
out of kernel scope. Referee: `openspec/specs/findings/spec.md` (the
kernel emits typed executed findings; redundant is an analysis shape). -/
theorem check_no_redundant (schema : Schema) (inst : Instance.TypedInstance) :
    ∀ finding ∈ Check.check schema inst,
      ∀ witness : RedundantWitness, finding.kind ≠ .redundant witness := by
  intro finding hmem witness heq
  have hs := soundness schema inst finding hmem
  rcases hs with ⟨_, _, _, _, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, hkind⟩
  · rw [hkind] at heq
    cases heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq

/-- `check` never produces a contradiction finding: bounded joint
satisfiability is out of kernel scope. Referee:
`openspec/specs/findings/spec.md`. -/
theorem check_no_contradiction (schema : Schema)
    (inst : Instance.TypedInstance) :
    ∀ finding ∈ Check.check schema inst,
      ∀ witness : ContradictionWitness,
        finding.kind ≠ .contradiction witness := by
  intro finding hmem witness heq
  have hs := soundness schema inst finding hmem
  rcases hs with ⟨_, _, _, _, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, hkind⟩
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq

/-- `check` never produces an underconstraint finding: domain-question
analysis is out of kernel scope. Referee: `openspec/specs/findings/spec.md`. -/
theorem check_no_underconstraint (schema : Schema)
    (inst : Instance.TypedInstance) :
    ∀ finding ∈ Check.check schema inst,
      ∀ witness : UnderconstraintWitness,
        finding.kind ≠ .underconstraint witness := by
  intro finding hmem witness heq
  have hs := soundness schema inst finding hmem
  rcases hs with ⟨_, _, _, _, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, _, _, _, _, hkind⟩
    | ⟨_, _, _, _, hkind⟩
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq
  · rw [hkind] at heq
    simp at heq

end Proofs
end Catherine