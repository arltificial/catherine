import Catherine.Proofs.Basic

/-!
# Purpose: state the dangling discipline — the five "never" clauses of
the kernel's cell accounting over a built instance, plus the carried-cell
trichotomy. Referee: `openspec/specs/kernel-checks/spec.md` ("Dangling
values are checked, not fatal", "The instance builder is total and
typing-enforcing", "Declared keys are executed as injectivity checks" —
the dangling-key-cell clause, "Unresolved injection cells suppress
coverage only", "Accepted coproducts execute coverage with witnesses",
"Coproduct disjointness violations name the colliding injections") and
`openspec/specs/findings/spec.md` ("Witness shapes are fixed per finding
kind", "Findings are typed and carry witnesses").

## Responsibilities: define the `DanglingDiscipline` bundle over a schema
and its projected raw instance (so every clause is stated over
`Check.check` on `Instance.build schema raw`) with exactly these fields —
`no_repair` (a `none` raw cell never becomes a table value),
`no_violation_through_unresolved` (an equation with either composite
short-circuiting emits no violation for that row),
`unresolved_key_not_in_collision` (a row with an unresolved key cell is
in no key-collision witness), `partial_image_suppresses_coverage` (with a
partial image no coproduct finding is a coverage-shaped 0/1-cover
witness), `complete_image_disjointness` (a doubly covered row still gets
its exact disjointness finding), `typing_violated_accounted_once` (the
typing report is the only accounting for that cell; its `none` table cell
is not a dangling finding), and `carried_trichotomy` (every carried cell
of the built table is stored, typing-violated, or dangling, and the
landing place matches the raw cell). The bundle is populated by
`dangling_discipline`.

## Proof bridges: the builder clauses (`no_repair`,
`typing_violated_accounted_once`, `carried_trichotomy`) go through private
lemmas that identify the built table (`build_tables_find?`), read one cell
of it (`buildArrow_table_cell`), and characterize the per-arrow report
lists and their images in `build` (`mem_buildArrow_typing_iff`,
`mem_buildArrow_dangling_iff`, `mem_build_typing`, `mem_build_dangling`).
The checker clauses go through private membership lemmas for the finding
families (`mem_danglingFindings_imp`, `mem_equationFindings`,
`mem_keyFindings`, `mem_coproductFindings`, `mem_coproductDeclaration_of`,
`mem_coproductDeclaration_imp`) and `Basic.mem_check_iff`; the key clause
also needs the `groupedRows` invariant (`insertRow_resolved` through
`mem_groupedRows_resolved`) that rows in a key group have resolved tuples.
These are deliberately private — a later consolidation pass can publish
the ones three proofs share.

## Rationale: a structure keeps each clause independently nameable, so
tasks 8–10 can discharge them one at a time and the gate can report which
clause a future regression breaks. The clauses are stated at the built
instance (not `check` in the abstract) because "never repairs" and the
trichotomy are builder facts; the remaining clauses quantify over
`check schema (Instance.build schema raw)`, which is exactly the input the
oracle and the Rust kernel check.
-/

namespace Catherine
namespace Proofs

/-- The dangling-discipline bundle for a schema and its projected raw
instance. Every clause is evaluated at `Instance.build schema raw`, the
instance `check` executes over. -/
structure DanglingDiscipline (schema : Schema) (raw : Instance.RawInstance) :
    Prop where
  /-- Build never repairs a `none`: a projected cell that holds no row
  stays `none` in the built table. Referee:
  `openspec/specs/kernel-checks/spec.md` "The instance builder is total
  and typing-enforcing". -/
  no_repair :
    ∀ (name : Ident) (arrow : Arrow) (idx : RowId),
      schema.arrow? name = some arrow →
      idx < (Schema.rowCount? raw.rows arrow.src).getD 0 →
      RawCell raw name idx = none →
      ∃ table,
        ((Instance.build schema raw).tables.find? fun entry => entry.1 == name).map (·.2) =
          some table ∧
        table[idx]? = some none
  /-- An unresolved equation path emits no violation: when either
  composite short-circuits to `none` — `lhs`, `rhs`, or both — `check`
  carries no equation finding for that index and row. Referee:
  `openspec/specs/kernel-checks/spec.md` "Dangling arrow value" —
  equations traversing it report the row as
  not-evaluable-through-dangling for either endpoint, never as a
  violation. -/
  no_violation_through_unresolved :
    ∀ (index : Nat) (equation : Equation),
      schema.equations[index]? = some equation →
      ∀ (row : RowId),
        (EvalPath.compose (Instance.build schema raw) equation.lhs row = none ∨
          EvalPath.compose (Instance.build schema raw) equation.rhs row = none) →
        ∀ (finding : Finding) (witness : EquationWitness),
          finding ∈ Check.check schema (Instance.build schema raw) →
          finding.kind = .violation (.equation witness) →
          ¬ (witness.equation = index ∧ witness.row = row)
  /-- An unresolved key cell is in no key-collision row: a row whose key
  tuple does not resolve is excluded from every key witness. Referee:
  `openspec/specs/kernel-checks/spec.md` "Dangling key cell is not a
  collision". -/
  unresolved_key_not_in_collision :
    ∀ (index : Nat) (key : Key), schema.keys[index]? = some key →
      ∀ (row : RowId),
        (∃ tables,
          Check.keyTables schema (Instance.build schema raw) key = some tables ∧
          Check.keyTuple tables row = none) →
        ∀ (finding : Finding) (witness : KeyWitness),
          finding ∈ Check.check schema (Instance.build schema raw) →
          finding.kind = .violation (.key witness) →
          witness.key = index →
          witness.rows.1 ≠ row ∧ witness.rows.2 ≠ row
  /-- A partial injection image suppresses coverage findings: with any
  image incomplete, every coproduct violation this declaration emits is
  a disjointness witness naming at least two complete images (no
  coverage-shaped 0/1-cover witness). Referee:
  `openspec/specs/kernel-checks/spec.md` "Unresolved injection cells
  suppress coverage only" and "Dangling injection cells are not
  double-reported". -/
  partial_image_suppresses_coverage :
    ∀ (index : Nat) (coproduct : Coproduct),
      schema.coproducts[index]? = some coproduct →
      (∃ tables,
        Check.injectionTables schema (Instance.build schema raw) coproduct =
          some tables ∧
        (injectionImages tables).length ≠ tables.length) →
      ∀ (finding : Finding) (witness : CoproductWitness),
        finding ∈ Check.check schema (Instance.build schema raw) →
        finding.kind = .violation (.coproduct witness) →
        witness.coproduct = index →
        2 ≤ witness.injections.length
  /-- Complete-image disjointness remains decidable under a partial
  sibling image: a row covered by at least two complete images still
  produces its exact disjointness finding. Referee:
  `openspec/specs/kernel-checks/spec.md` "Decidable disjointness
  survives an unresolved sibling". -/
  complete_image_disjointness :
    ∀ (index : Nat) (coproduct : Coproduct) (row : RowId)
      (injections : List Ident),
      schema.coproducts[index]? = some coproduct →
      CoproductRowApplies (Instance.build schema raw) coproduct row →
      CoproductRowFailure schema (Instance.build schema raw) coproduct row
        injections →
      2 ≤ injections.length →
      ({ label := labelFor coproduct.status
         kind := .violation (.coproduct
           { coproduct := index, row := row, injections := injections }) } :
          Finding) ∈
        Check.check schema (Instance.build schema raw)
  /-- A typing-violated cell is accounted exactly once: the value never
  becomes a table entry (its cell stays `none`), the build report names
  it, and no dangling finding names that cell. Referee:
  `openspec/specs/kernel-checks/spec.md` "Typing violation at build" and
  "Dangling values are checked, not fatal". -/
  typing_violated_accounted_once :
    ∀ (name : Ident) (arrow : Arrow) (src value : RowId),
      schema.arrow? name = some arrow →
      ({ arrow := name, src := src, value := value } : Instance.TypingViolation) ∈
          (Instance.buildArrow raw name arrow).typingViolations →
      (Instance.buildArrow raw name arrow).table[src]? = some none ∧
      ({ arrow := name, src := src, value := value } : Instance.TypingViolation) ∈
          (Instance.build schema raw).typingViolations ∧
      (∀ finding, finding ∈ Check.check schema (Instance.build schema raw) →
        finding.kind ≠ .dangling { arrow := name, src := src })
  /-- The carried-cell trichotomy over the built table: every projected
  cell the builder takes lands in exactly one place — a stored morphism
  value (`some value` in the table), a typing violation (the table cell
  stays `none` and the report names arrow, row, value), or a dangling
  record (the table cell stays `none` and the raw cell was `none`).
  Referee: `openspec/specs/kernel-checks/spec.md` "The instance builder
  is total and typing-enforcing". -/
  carried_trichotomy :
    ∀ (name : Ident) (arrow : Arrow) (idx : RowId),
      schema.arrow? name = some arrow →
      idx < (Schema.rowCount? raw.rows arrow.src).getD 0 →
      idx < (RawCells raw name).length →
      (∃ (table : List (Option RowId)) (value : RowId),
        ((Instance.build schema raw).tables.find? fun entry => entry.1 == name).map (·.2) =
            some table ∧
        table[idx]? = some (some value) ∧
        RawCell raw name idx = some value)
      ∨ (∃ (table : List (Option RowId)) (value : RowId),
        ((Instance.build schema raw).tables.find? fun entry => entry.1 == name).map (·.2) =
            some table ∧
        table[idx]? = some none ∧
        RawCell raw name idx = some value ∧
        ({ arrow := name, src := idx, value := value } : Instance.TypingViolation) ∈
          (Instance.build schema raw).typingViolations)
      ∨ (∃ (table : List (Option RowId)),
        ((Instance.build schema raw).tables.find? fun entry => entry.1 == name).map (·.2) =
            some table ∧
        table[idx]? = some none ∧
        RawCell raw name idx = none ∧
        ({ arrow := name, src := idx } : Instance.DanglingCell) ∈
          (Instance.build schema raw).dangling)

private theorem arrows_find? {schema : Schema} {name : Ident} {arrow : Arrow}
    (h : schema.arrow? name = some arrow) :
    ∃ entry : Ident × Arrow,
      schema.arrows.find? (fun e => e.1 == name) = some entry ∧
        entry.1 = name ∧ entry.2 = arrow := by
  unfold Schema.arrow? at h
  split at h
  · next entry hentry =>
    refine ⟨entry, hentry, ?_, ?_⟩
    · have hb := List.find?_some hentry
      simpa using hb
    · simpa using h
  · next hnone => simp at h

private theorem find?_map_of_fst {α β : Type} (l : List (Ident × α)) (g : Ident × α → Ident × β)
    (hg : ∀ e, (g e).1 = e.1) (name : Ident) :
    ∀ entry, l.find? (fun e => e.1 == name) = some entry →
      (l.map g).find? (fun e => e.1 == name) = some (g entry) := by
  induction l with
  | nil => intro entry h; simp at h
  | cons a rest ih =>
    intro entry h
    by_cases ha : (a.1 == name) = true
    · rw [List.find?_cons_of_pos (p := fun e : Ident × α => e.1 == name) (a := a) (l := rest) ha] at h
      have he : entry = a := by simpa using h.symm
      rw [he, List.map_cons,
        List.find?_cons_of_pos (p := fun e : Ident × β => e.1 == name) (a := g a) (l := rest.map g) (by rw [hg a]; exact ha)]
    · rw [List.find?_cons_of_neg (p := fun e : Ident × α => e.1 == name) (a := a) (l := rest) ha] at h
      rw [List.map_cons,
        List.find?_cons_of_neg (p := fun e : Ident × β => e.1 == name) (a := g a) (l := rest.map g) (by rw [hg a]; exact ha)]
      exact ih entry h

private theorem build_tables (schema : Schema) (raw : Instance.RawInstance) :
    (Instance.build schema raw).tables =
      schema.arrows.map (fun e => (e.1, (Instance.buildArrow raw e.1 e.2).table)) := by
  simp [Instance.build]

private theorem build_tables_find? (schema : Schema) (raw : Instance.RawInstance)
    (name : Ident) (arrow : Arrow) (h : schema.arrow? name = some arrow) :
    ((Instance.build schema raw).tables.find? (fun entry => entry.1 == name)).map (·.2) =
      some (Instance.buildArrow raw name arrow).table := by
  obtain ⟨⟨ename, earrow⟩, hentry, hname, harrow⟩ := arrows_find? h
  rw [build_tables]
  have hmap := find?_map_of_fst schema.arrows
    (fun e : Ident × Arrow => (e.1, (Instance.buildArrow raw e.1 e.2).table))
    (fun e => rfl) name (ename, earrow) hentry
  have hname' : ename = name := hname
  have harrow' : earrow = arrow := harrow
  rw [hmap]
  rw [hname', harrow']
  rfl

private theorem getElem?_range_ite (n i : Nat) :
    (List.range n)[i]? = if i < n then some i else none := by
  by_cases h : i < n
  · rw [List.getElem?_range h, ite_eq_left h]
  · rw [List.getElem?_eq_none (by rw [List.length_range]; exact Nat.le_of_not_lt h), ite_eq_right h]

private theorem buildArrow_table_cell (raw : Instance.RawInstance) (name : Ident)
    (arrow : Arrow) (idx : Nat) :
    (Instance.buildArrow raw name arrow).table[idx]? =
      (if idx < (Schema.rowCount? raw.rows arrow.src).getD 0 then
        some (match (RawCells raw name)[idx]? with
          | some (some value) =>
            if value < (Schema.rowCount? raw.rows arrow.tgt).getD 0 then some value else none
          | _ => none)
      else none) := by
  simp only [Instance.buildArrow, RawCells, List.getElem?_map, List.getElem?_take]
  rw [getElem?_range_ite]
  by_cases h : idx < (Schema.rowCount? raw.rows arrow.src).getD 0
  · rw [ite_eq_left h]
    simp only [Option.map_some, ite_eq_left h]
    rfl
  · rw [ite_eq_right h]
    simp [h]

private theorem mem_range_filterMap {α : Type} {n : Nat} {f : Nat → Option α} {a : α} :
    a ∈ (List.range n).filterMap f ↔ ∃ i, i < n ∧ f i = some a := by
  rw [List.mem_filterMap]
  constructor
  · rintro ⟨i, hi, hf⟩
    exact ⟨i, List.mem_range.mp hi, hf⟩
  · rintro ⟨i, hi, hf⟩
    exact ⟨i, List.mem_range.mpr hi, hf⟩

private theorem mem_typing_general (cells : List (Option RowId)) (name : Ident)
    (nSrc nTgt : Nat) (v : Instance.TypingViolation) :
    v ∈ (List.range (cells.take nSrc).length).filterMap (fun idx =>
      match (cells.take nSrc)[idx]? with
      | some (some value) =>
        if nTgt ≤ value then some { arrow := name, src := idx, value := value } else none
      | _ => none) ↔
      ∃ (idx value : RowId),
        idx < cells.length ∧ idx < nSrc ∧
        cells[idx]? = some (some value) ∧ nTgt ≤ value ∧
        v = { arrow := name, src := idx, value := value } := by
  rw [mem_range_filterMap]
  constructor
  · rintro ⟨idx, hidx, hf⟩
    have hlen : idx < (cells.take nSrc).length := hidx
    have hmin : idx < min nSrc cells.length := by simpa [List.length_take] using hlen
    have hidx1 : idx < nSrc := Nat.lt_of_lt_of_le hmin (Nat.min_le_left _ _)
    have hidx2 : idx < cells.length := Nat.lt_of_lt_of_le hmin (Nat.min_le_right _ _)
    rw [List.getElem?_take, ite_eq_left hidx1] at hf
    cases hcell : cells[idx]? with
    | none => simp [hcell] at hf
    | some c =>
      cases c with
      | none => simp [hcell] at hf
      | some value =>
        simp only [hcell] at hf
        by_cases hle : nTgt ≤ value
        · rw [ite_eq_left hle] at hf
          simp only [Option.some.injEq] at hf
          exact ⟨idx, value, hidx2, hidx1, hcell, hle, hf.symm⟩
        · rw [ite_eq_right hle] at hf
          simp at hf
  · rintro ⟨idx, value, hlen, hidx1, hcell, hle, rfl⟩
    refine ⟨idx, ?_, ?_⟩
    · rw [List.length_take]
      exact Nat.lt_min.mpr ⟨hidx1, hlen⟩
    · simp only [List.getElem?_take, ite_eq_left hidx1, hcell, ite_eq_left hle]

private theorem mem_dangling_general (cells : List (Option RowId)) (name : Ident)
    (nSrc : Nat) (d : Instance.DanglingCell) :
    d ∈ (List.range (cells.take nSrc).length).filterMap (fun idx =>
      match (cells.take nSrc)[idx]? with
      | some none => some { arrow := name, src := idx }
      | _ => none) ↔
      ∃ idx, idx < cells.length ∧ idx < nSrc ∧
        cells[idx]? = some none ∧ d = { arrow := name, src := idx } := by
  rw [mem_range_filterMap]
  constructor
  · rintro ⟨idx, hidx, hf⟩
    have hlen : idx < (cells.take nSrc).length := hidx
    have hmin : idx < min nSrc cells.length := by simpa [List.length_take] using hlen
    have hidx1 : idx < nSrc := Nat.lt_of_lt_of_le hmin (Nat.min_le_left _ _)
    have hidx2 : idx < cells.length := Nat.lt_of_lt_of_le hmin (Nat.min_le_right _ _)
    rw [List.getElem?_take, ite_eq_left hidx1] at hf
    cases hcell : cells[idx]? with
    | none => simp [hcell] at hf
    | some c =>
      cases c with
      | none =>
        simp only [hcell] at hf
        simp only [Option.some.injEq] at hf
        exact ⟨idx, hidx2, hidx1, hcell, hf.symm⟩
      | some value => simp [hcell] at hf
  · rintro ⟨idx, hlen, hidx1, hcell, rfl⟩
    refine ⟨idx, ?_, ?_⟩
    · rw [List.length_take]
      exact Nat.lt_min.mpr ⟨hidx1, hlen⟩
    · simp only [List.getElem?_take, ite_eq_left hidx1, hcell]

private theorem buildArrow_typing_eq (raw : Instance.RawInstance) (name : Ident)
    (arrow : Arrow) :
    (Instance.buildArrow raw name arrow).typingViolations =
      (List.range ((RawCells raw name).take
        ((Schema.rowCount? raw.rows arrow.src).getD 0)).length).filterMap
        (fun idx => match ((RawCells raw name).take
            ((Schema.rowCount? raw.rows arrow.src).getD 0))[idx]? with
          | some (some value) =>
            if (Schema.rowCount? raw.rows arrow.tgt).getD 0 ≤ value then
              some { arrow := name, src := idx, value := value } else none
          | _ => none) := by
  simp only [Instance.buildArrow, RawCells]
  rfl

private theorem buildArrow_dangling_eq (raw : Instance.RawInstance) (name : Ident)
    (arrow : Arrow) :
    (Instance.buildArrow raw name arrow).dangling =
      (List.range ((RawCells raw name).take
        ((Schema.rowCount? raw.rows arrow.src).getD 0)).length).filterMap
        (fun idx => match ((RawCells raw name).take
            ((Schema.rowCount? raw.rows arrow.src).getD 0))[idx]? with
          | some none => some { arrow := name, src := idx }
          | _ => none) := by
  simp only [Instance.buildArrow, RawCells]
  rfl

private theorem mem_buildArrow_typing_iff (raw : Instance.RawInstance) (name : Ident)
    (arrow : Arrow) (v : Instance.TypingViolation) :
    v ∈ (Instance.buildArrow raw name arrow).typingViolations ↔
      ∃ (idx value : RowId),
        idx < (RawCells raw name).length ∧
        idx < (Schema.rowCount? raw.rows arrow.src).getD 0 ∧
        (RawCells raw name)[idx]? = some (some value) ∧
        (Schema.rowCount? raw.rows arrow.tgt).getD 0 ≤ value ∧
        v = { arrow := name, src := idx, value := value } := by
  rw [buildArrow_typing_eq]
  exact mem_typing_general (RawCells raw name) name
    ((Schema.rowCount? raw.rows arrow.src).getD 0)
    ((Schema.rowCount? raw.rows arrow.tgt).getD 0) v

private theorem mem_buildArrow_dangling_iff (raw : Instance.RawInstance) (name : Ident)
    (arrow : Arrow) (d : Instance.DanglingCell) :
    d ∈ (Instance.buildArrow raw name arrow).dangling ↔
      ∃ idx,
        idx < (RawCells raw name).length ∧
        idx < (Schema.rowCount? raw.rows arrow.src).getD 0 ∧
        (RawCells raw name)[idx]? = some none ∧
        d = { arrow := name, src := idx } := by
  rw [buildArrow_dangling_eq]
  exact mem_dangling_general (RawCells raw name) name
    ((Schema.rowCount? raw.rows arrow.src).getD 0) d

private theorem mem_build_typing (schema : Schema) (raw : Instance.RawInstance)
    (v : Instance.TypingViolation) :
    v ∈ (Instance.build schema raw).typingViolations ↔
      ∃ entry, entry ∈ schema.arrows ∧
        v ∈ (Instance.buildArrow raw entry.1 entry.2).typingViolations := by
  simp only [Instance.build, List.mem_flatten, List.mem_map, List.map_map, Function.comp_def]
  constructor
  · rintro ⟨l, ⟨entry, hentry, hl⟩, hmem⟩
    subst hl
    exact ⟨entry, hentry, hmem⟩
  · rintro ⟨entry, hentry, hmem⟩
    exact ⟨_, ⟨entry, hentry, rfl⟩, hmem⟩

private theorem mem_build_dangling (schema : Schema) (raw : Instance.RawInstance)
    (d : Instance.DanglingCell) :
    d ∈ (Instance.build schema raw).dangling ↔
      ∃ entry, entry ∈ schema.arrows ∧
        d ∈ (Instance.buildArrow raw entry.1 entry.2).dangling := by
  simp only [Instance.build, List.mem_flatten, List.mem_map, List.map_map, Function.comp_def]
  constructor
  · rintro ⟨l, ⟨entry, hentry, hl⟩, hmem⟩
    subst hl
    exact ⟨entry, hentry, hmem⟩
  · rintro ⟨entry, hentry, hmem⟩
    exact ⟨_, ⟨entry, hentry, rfl⟩, hmem⟩

private theorem mem_danglingFindings_imp (inst : Instance.TypedInstance)
    (violated : List (Ident × RowId)) {finding : Finding}
    (h : finding ∈ Check.danglingFindings inst violated) :
    ∃ (entry : Ident × List (Option RowId)) (idx : Nat),
      entry ∈ inst.tables ∧
      entry.2[idx]? = some none ∧
      violated.contains (entry.1, idx) = false ∧
      finding =
        { label := .executed, kind := .dangling { arrow := entry.1, src := idx } } := by
  simp only [Check.danglingFindings, List.mem_flatMap, mem_range_filterMap] at h
  obtain ⟨entry, hentry, idx, hidx, hf⟩ := h
  cases hcell : entry.2[idx]? with
  | none => simp [hcell] at hf
  | some c =>
    cases c with
    | some value => simp [hcell] at hf
    | none =>
      simp only [hcell] at hf
      by_cases hc : violated.contains (entry.1, idx) = true
      · rw [ite_eq_left hc] at hf
        simp at hf
      · have hcf : violated.contains (entry.1, idx) = false := by
          cases hb : violated.contains (entry.1, idx) with
          | false => rfl
          | true => exact absurd hb hc
        rw [ite_eq_right hc] at hf
        simp only [Option.some.injEq] at hf
        exact ⟨entry, idx, hentry, hcell, hcf, hf.symm⟩

private theorem mem_mapIdx_flatten {α β : Type} {f : Nat → α → List β} {l : List α}
    {b : β} (h : b ∈ (l.mapIdx f).flatten) :
    ∃ i, ∃ (hi : i < l.length), b ∈ f i l[i] := by
  obtain ⟨x, hx, hb⟩ := List.mem_flatten.mp h
  obtain ⟨i, hi, hf⟩ := List.mem_mapIdx.mp hx
  exact ⟨i, hi, by rw [hf]; exact hb⟩

private theorem mem_equationFindings (schema : Schema) (inst : Instance.TypedInstance)
    {finding : Finding} (h : finding ∈ Check.equationFindings schema inst) :
    ∃ (index : Nat) (equation : Equation) (row : RowId),
      schema.equations[index]? = some equation ∧
      Check.equationFinding inst index equation row = some finding := by
  simp only [Check.equationFindings] at h
  obtain ⟨i, hi, hmem⟩ := mem_mapIdx_flatten h
  cases hlhs : schema.equations[i].lhs with
  | nil => rw [hlhs] at hmem; dsimp only at hmem; simp at hmem
  | cons lead rest =>
    rw [hlhs] at hmem
    dsimp only at hmem
    cases harrow : schema.arrow? lead with
    | none => rw [harrow] at hmem; dsimp only at hmem; simp at hmem
    | some arrow =>
      rw [harrow] at hmem
      dsimp only at hmem
      cases hrows : Schema.rowCount? inst.rows arrow.src with
      | none => rw [hrows] at hmem; dsimp only at hmem; simp at hmem
      | some n =>
        rw [hrows] at hmem
        dsimp only at hmem
        obtain ⟨row, hrow, hf⟩ := mem_range_filterMap.mp hmem
        exact ⟨i, schema.equations[i], row, List.getElem?_eq_getElem hi, hf⟩

/-! ### Key-finding membership -/

private def keyStep (tables : List (List (Option RowId)))
    (groups : List (List RowId × List RowId)) (row : Nat) :
    List (List RowId × List RowId) :=
  match Check.keyTuple tables row with
  | none => groups
  | some tuple => Check.insertRow tuple row groups

private theorem insertRow_resolved (tables : List (List (Option RowId)))
    (tuple : List RowId) (row : RowId)
    (hrow : Check.keyTuple tables row ≠ none) :
    ∀ acc, (∀ e ∈ acc, ∀ r ∈ e.2, Check.keyTuple tables r ≠ none) →
      ∀ e ∈ Check.insertRow tuple row acc, ∀ r ∈ e.2,
        Check.keyTuple tables r ≠ none := by
  intro acc
  induction acc with
  | nil =>
    intro hacc e he r hr
    simp only [Check.insertRow, List.mem_singleton] at he
    subst he
    simp only [List.mem_singleton] at hr
    subst hr
    exact hrow
  | cons hd tl ih =>
    intro hacc e he r hr
    obtain ⟨existing, rows⟩ := hd
    have hhd : ((existing, rows) : List RowId × List RowId) ∈ (existing, rows) :: tl := by simp
    by_cases hbeq : (existing == tuple) = true
    · simp only [Check.insertRow, ite_eq_left hbeq] at he
      rcases List.mem_cons.mp he with rfl | he'
      · rcases List.mem_append.mp hr with hr | hr
        · exact hacc (existing, rows) hhd r hr
        · rcases List.mem_singleton.mp hr with rfl
          exact hrow
      · exact hacc _ (List.mem_cons_of_mem _ he') r hr
    · simp only [Check.insertRow, ite_eq_right hbeq] at he
      rcases List.mem_cons.mp he with rfl | he'
      · exact hacc (existing, rows) hhd r hr
      · exact ih (fun e' he' r' hr' => hacc e' (List.mem_cons_of_mem _ he') r' hr') e he' r hr

private theorem keyStep_resolved (tables : List (List (Option RowId)))
    (acc : List (List RowId × List RowId)) (row : Nat)
    (hacc : ∀ e ∈ acc, ∀ r ∈ e.2, Check.keyTuple tables r ≠ none) :
    ∀ e ∈ keyStep tables acc row, ∀ r ∈ e.2,
      Check.keyTuple tables r ≠ none := by
  cases hx : Check.keyTuple tables row with
  | none =>
    intro e he r hr
    simp only [keyStep, hx] at he
    exact hacc e he r hr
  | some tuple =>
    intro e he r hr
    simp only [keyStep, hx] at he
    exact insertRow_resolved tables tuple row (by rw [hx]; simp) acc hacc e he r hr

private theorem foldl_resolved (tables : List (List (Option RowId)))
    (l : List Nat) (acc : List (List RowId × List RowId))
    (hacc : ∀ e ∈ acc, ∀ r ∈ e.2, Check.keyTuple tables r ≠ none) :
    ∀ e ∈ l.foldl (keyStep tables) acc, ∀ r ∈ e.2,
      Check.keyTuple tables r ≠ none := by
  induction l generalizing acc with
  | nil => exact hacc
  | cons x rest ih =>
    rw [List.foldl_cons]
    exact ih (keyStep tables acc x) (keyStep_resolved tables acc x hacc)

private theorem groupedRows_eq (tables : List (List (Option RowId))) (n : Nat) :
    Check.groupedRows tables n =
      ((List.range n).foldl (keyStep tables) []).map (fun e => e.2) := rfl

private theorem mem_groupedRows_resolved (tables : List (List (Option RowId)))
    (n : Nat) {row : RowId}
    (h : ∃ rows, rows ∈ Check.groupedRows tables n ∧ row ∈ rows) :
    Check.keyTuple tables row ≠ none := by
  obtain ⟨rows, hrows, hmem⟩ := h
  rw [groupedRows_eq] at hrows
  rw [List.mem_map] at hrows
  obtain ⟨e, he, heq⟩ := hrows
  rw [← heq] at hmem
  exact foldl_resolved tables (List.range n) [] (by intro e he; simp at he) e he row hmem

private theorem mem_ascendingPairs {rows : List RowId} {pair : RowId × RowId}
    (h : pair ∈ Check.ascendingPairs rows) : pair.1 ∈ rows ∧ pair.2 ∈ rows := by
  induction rows with
  | nil => simp [Check.ascendingPairs] at h
  | cons lo rest ih =>
    simp only [Check.ascendingPairs, List.mem_append, List.mem_map] at h
    rcases h with ⟨hi, hhi, rfl⟩ | hrest
    · exact ⟨by simp, by simp [hhi]⟩
    · have hh := ih hrest
      exact ⟨by simp [hh.1], by simp [hh.2]⟩

private theorem mem_keyFindings (schema : Schema) (inst : Instance.TypedInstance)
    {finding : Finding} (h : finding ∈ Check.keyFindings schema inst) :
    ∃ (index : Nat) (key : Key) (tables : List (List (Option RowId)))
      (pair : RowId × RowId),
      schema.keys[index]? = some key ∧
      Check.keyTables schema inst key = some tables ∧
      Check.keyTuple tables pair.1 ≠ none ∧
      Check.keyTuple tables pair.2 ≠ none ∧
      finding =
        { label := labelFor key.status
          kind := .violation (.key { key := index, rows := pair }) } := by
  simp only [Check.keyFindings] at h
  obtain ⟨i, hi, hmem⟩ := mem_mapIdx_flatten h
  cases hr : Schema.rowCount? inst.rows (schema.keys[i]).object with
  | none => simp [hr] at hmem
  | some n =>
    simp only [hr] at hmem
    cases hk : Check.keyTables schema inst (schema.keys[i]) with
    | none => simp [hk] at hmem
    | some tables =>
      simp only [hk] at hmem
      rw [List.mem_flatMap] at hmem
      obtain ⟨rows, hrows, hmem2⟩ := hmem
      rw [List.mem_map] at hmem2
      obtain ⟨pair, hpair, rfl⟩ := hmem2
      exact ⟨i, schema.keys[i], tables, pair, List.getElem?_eq_getElem hi, hk,
        mem_groupedRows_resolved tables n ⟨rows, hrows, (mem_ascendingPairs hpair).1⟩,
        mem_groupedRows_resolved tables n ⟨rows, hrows, (mem_ascendingPairs hpair).2⟩,
        rfl⟩

/-! ### Coproduct-declaration membership -/

private theorem mem_coproductDeclaration_of (schema : Schema)
    (inst : Instance.TypedInstance) (index : Nat) (coproduct : Coproduct)
    (tables : List (Ident × List (Option RowId))) (row n : Nat)
    (htables : Check.injectionTables schema inst coproduct = some tables)
    (hrows : Schema.rowCount? inst.rows coproduct.object = some n)
    (hrow : row < n)
    (hcase :
      ((injectionImages tables).length = tables.length ∧
        (Check.coveringInjections (injectionImages tables) row).length ≠ 1) ∨
      ((injectionImages tables).length ≠ tables.length ∧
        2 ≤ (Check.coveringInjections (injectionImages tables) row).length)) :
    Check.coproductFinding (labelFor coproduct.status) index row
      (Check.coveringInjections (injectionImages tables) row) ∈
        Check.coproductDeclaration schema inst index coproduct := by
  simp only [injectionImages] at hcase ⊢
  simp only [Check.coproductDeclaration, htables, hrows]
  rw [mem_range_filterMap]
  refine ⟨row, hrow, ?_⟩
  rcases hcase with ⟨hcomplete, hne⟩ | ⟨hpartial, hge⟩
  · have hc :
        ((List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables).length
          == tables.length) = true := by simp [hcomplete]
    have hcover :
        ((Check.coveringInjections
          (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables) row).length
          == 1) = false := by simp [hne]
    simp [hc, hcover]
  · have hc :
        ((List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables).length
          == tables.length) = false := by simp [hpartial]
    simp [hc, hge]

private theorem mem_coproductDeclaration_imp (schema : Schema)
    (inst : Instance.TypedInstance) (index : Nat) (coproduct : Coproduct)
    {finding : Finding}
    (h : finding ∈ Check.coproductDeclaration schema inst index coproduct) :
    ∃ (tables : List (Ident × List (Option RowId))) (row n : Nat),
      Check.injectionTables schema inst coproduct = some tables ∧
      Schema.rowCount? inst.rows coproduct.object = some n ∧
      row < n ∧
      finding = Check.coproductFinding (labelFor coproduct.status) index row
        (Check.coveringInjections (injectionImages tables) row) ∧
      (((injectionImages tables).length = tables.length ∧
          (Check.coveringInjections (injectionImages tables) row).length ≠ 1) ∨
        ((injectionImages tables).length ≠ tables.length ∧
          2 ≤ (Check.coveringInjections (injectionImages tables) row).length)) := by
  simp only [injectionImages] at h ⊢
  simp only [Check.coproductDeclaration] at h
  have hsome : ∃ tables, Check.injectionTables schema inst coproduct = some tables := by
    cases h0 : Check.injectionTables schema inst coproduct with
    | none => simp [h0] at h
    | some tables => exact ⟨tables, rfl⟩
  obtain ⟨tables, ht⟩ := hsome
  rw [ht] at h
  dsimp only at h
  have hsome2 : ∃ n, Schema.rowCount? inst.rows coproduct.object = some n := by
    cases h0 : Schema.rowCount? inst.rows coproduct.object with
    | none => simp [h0] at h
    | some n => exact ⟨n, rfl⟩
  obtain ⟨n, hr⟩ := hsome2
  rw [hr] at h
  dsimp only at h
  obtain ⟨row, hrow, hf⟩ := mem_range_filterMap.mp h
  by_cases hc :
      ((List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables).length
        == tables.length) = true
  · rw [ite_eq_left hc] at hf
    by_cases hc1 :
        ((Check.coveringInjections
          (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables) row).length
          == 1) = true
    · rw [ite_eq_left hc1] at hf
      simp at hf
    · rw [ite_eq_right hc1] at hf
      simp only [Option.some.injEq] at hf
      have hlen : (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables).length
          = tables.length := by simpa using hc
      have hne : (Check.coveringInjections
          (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables) row).length
          ≠ 1 := by
        intro hcontra
        rw [hcontra] at hc1
        simp at hc1
      exact ⟨tables, row, n, ht, hr, hrow, hf.symm, Or.inl ⟨hlen, hne⟩⟩
  · rw [ite_eq_right hc] at hf
    by_cases h2 :
        2 ≤ (Check.coveringInjections
          (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables) row).length
    · rw [ite_eq_left h2] at hf
      simp only [Option.some.injEq] at hf
      have hlen : (List.filterMap (fun entry => Check.completeImage entry.1 entry.2) tables).length
          ≠ tables.length := by
        intro hcontra
        rw [hcontra] at hc
        simp at hc
      exact ⟨tables, row, n, ht, hr, hrow, hf.symm, Or.inr ⟨hlen, h2⟩⟩
    · rw [ite_eq_right h2] at hf
      simp at hf

private theorem mem_mapIdx_flatten_of {α β : Type} {f : Nat → α → List β} {l : List α}
    {b : β} {i : Nat} (hi : i < l.length) (hb : b ∈ f i l[i]) :
    b ∈ (l.mapIdx f).flatten := by
  rw [List.mem_flatten]
  refine ⟨f i l[i], ?_, hb⟩
  rw [List.mem_mapIdx]
  exact ⟨i, hi, rfl⟩

private theorem mem_coproductFindings (schema : Schema) (inst : Instance.TypedInstance)
    {finding : Finding} (h : finding ∈ Check.coproductFindings schema inst) :
    ∃ index coproduct, schema.coproducts[index]? = some coproduct ∧
      finding ∈ Check.coproductDeclaration schema inst index coproduct := by
  simp only [Check.coproductFindings] at h
  obtain ⟨i, hi, hmem⟩ := mem_mapIdx_flatten h
  exact ⟨i, schema.coproducts[i], List.getElem?_eq_getElem hi, hmem⟩

private theorem mem_coproductFindings_of (schema : Schema) (inst : Instance.TypedInstance)
    (index : Nat) (coproduct : Coproduct) {finding : Finding}
    (hdecl : schema.coproducts[index]? = some coproduct)
    (hmem : finding ∈ Check.coproductDeclaration schema inst index coproduct) :
    finding ∈ Check.coproductFindings schema inst := by
  obtain ⟨hi, hget⟩ := List.getElem?_eq_some_iff.mp hdecl
  rw [← hget] at hmem
  simp only [Check.coproductFindings]
  exact mem_mapIdx_flatten_of hi hmem


/-- **Dangling discipline** — the five accounting clauses plus the
carried-cell trichotomy hold for any schema and projected instance.
Referee: `openspec/specs/kernel-checks/spec.md` ("Dangling values are
checked, not fatal", "The instance builder is total and
typing-enforcing", "Dangling key cell is not a collision", "Unresolved
injection cells suppress coverage only", "Decidable disjointness survives
an unresolved sibling") and `openspec/specs/findings/spec.md` (witness
shapes; the `none` cell is a build report, not a second finding). -/
theorem dangling_discipline (schema : Schema) (raw : Instance.RawInstance) :
    DanglingDiscipline schema raw where
  no_repair := by
    intro name arrow idx harrow hidx hraw
    have htable := build_tables_find? schema raw name arrow harrow
    refine ⟨(Instance.buildArrow raw name arrow).table, htable, ?_⟩
    rw [buildArrow_table_cell, ite_eq_left hidx]
    cases hcell : (RawCells raw name)[idx]? with
    | none => simp
    | some c =>
      cases c with
      | none => simp
      | some value =>
        exfalso
        have hcontr : RawCell raw name idx = some value := by simp [RawCell, hcell]
        rw [hraw] at hcontr
        simp at hcontr
  no_violation_through_unresolved := by
    intro index equation heq row hunres finding witness hcheck hkind hwit
    obtain ⟨we, wr, wl, wr2⟩ := witness
    obtain ⟨hindex, hrow⟩ := hwit
    rw [mem_check_iff] at hcheck
    rcases hcheck with hd | he | hk | hc | hp
    · obtain ⟨entry, idx, hentry, hcell, hcont, hfind⟩ :=
        mem_danglingFindings_imp (Instance.build schema raw) _ hd
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, eq', row', hdecl, hfind⟩ :=
        mem_equationFindings schema (Instance.build schema raw) he
      obtain ⟨lhs', rhs', hl, hr, hne, hres⟩ :=
        (equationFinding_eq_some_iff (Instance.build schema raw) i eq' row' finding).mp hfind
      rw [hres] at hkind
      simp only [FindingKind.violation.injEq, ViolationWitness.equation.injEq, EquationWitness.mk.injEq] at hkind
      obtain ⟨hi, hrow', _, _⟩ := hkind
      have hi' : i = index := by rw [hi]; exact hindex
      have hr' : row' = row := by rw [hrow']; exact hrow
      rw [hi'] at hdecl
      rw [heq] at hdecl
      have heq' : eq' = equation := by simpa using hdecl.symm
      rw [hr', heq'] at hl hr
      rcases hunres with hunres | hunres
      · rw [hunres] at hl
        exact absurd hl (by simp)
      · rw [hunres] at hr
        exact absurd hr (by simp)
    · obtain ⟨i, key', tables, pair, hdecl, hkt, hres1, hres2, hfind⟩ :=
        mem_keyFindings schema (Instance.build schema raw) hk
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, cop', hdecl, hmem⟩ :=
        mem_coproductFindings schema (Instance.build schema raw) hc
      obtain ⟨tables, row', n, htab, hrows, hrow'', hfind, hcase⟩ :=
        mem_coproductDeclaration_imp schema (Instance.build schema raw) i cop' hmem
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨_i, _pb, _w, _hdecl, _hlabel, hshape⟩ :=
        mem_pullbackFindings_shape schema (Instance.build schema raw) hp
      rw [hshape] at hkind
      cases hkind
  unresolved_key_not_in_collision := by
    intro index key hkey row hunres finding witness hcheck hkind hwit
    obtain ⟨wk, wrows⟩ := witness
    rw [mem_check_iff] at hcheck
    rcases hcheck with hd | he | hk | hc | hp
    · obtain ⟨entry, idx, hentry, hcell, hcont, hfind⟩ :=
        mem_danglingFindings_imp (Instance.build schema raw) _ hd
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, eq', row', hdecl, hfind⟩ :=
        mem_equationFindings schema (Instance.build schema raw) he
      obtain ⟨lhs', rhs', hl, hr, hne, hres⟩ :=
        (equationFinding_eq_some_iff (Instance.build schema raw) i eq' row' finding).mp hfind
      rw [hres] at hkind
      cases hkind
    · obtain ⟨i, key', tables, pair, hdecl, hkt, hres1, hres2, hfind⟩ :=
        mem_keyFindings schema (Instance.build schema raw) hk
      rw [hfind] at hkind
      simp only [FindingKind.violation.injEq, ViolationWitness.key.injEq, KeyWitness.mk.injEq] at hkind
      obtain ⟨hkeyidx, hrows⟩ := hkind
      have hi : i = index := by rw [hkeyidx]; exact hwit
      rw [hi] at hdecl
      rw [hkey] at hdecl
      have hkeyeq : key' = key := by simpa using hdecl.symm
      rw [hkeyeq] at hkt
      obtain ⟨tables0, hkt0, htuple⟩ := hunres
      rw [hkt0] at hkt
      have htables : tables = tables0 := by simpa using hkt.symm
      rw [htables] at hres1 hres2
      rw [hrows] at hres1 hres2
      change wrows.1 ≠ row ∧ wrows.2 ≠ row
      constructor
      · intro hcontra
        rw [hcontra] at hres1
        exact hres1 htuple
      · intro hcontra
        rw [hcontra] at hres2
        exact hres2 htuple
    · obtain ⟨i, cop', hdecl, hmem⟩ :=
        mem_coproductFindings schema (Instance.build schema raw) hc
      obtain ⟨tables, row', n, htab, hrows, hrow'', hfind, hcase⟩ :=
        mem_coproductDeclaration_imp schema (Instance.build schema raw) i cop' hmem
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨_i, _pb, _w, _hdecl, _hlabel, hshape⟩ :=
        mem_pullbackFindings_shape schema (Instance.build schema raw) hp
      rw [hshape] at hkind
      cases hkind
  partial_image_suppresses_coverage := by
    intro index coproduct hcop hpartial finding witness hcheck hkind hwit
    obtain ⟨wc, wrow, winj⟩ := witness
    rw [mem_check_iff] at hcheck
    rcases hcheck with hd | he | hk | hc | hp
    · obtain ⟨entry, idx, hentry, hcell, hcont, hfind⟩ :=
        mem_danglingFindings_imp (Instance.build schema raw) _ hd
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, eq', row', hdecl, hfind⟩ :=
        mem_equationFindings schema (Instance.build schema raw) he
      obtain ⟨lhs', rhs', hl, hr, hne, hres⟩ :=
        (equationFinding_eq_some_iff (Instance.build schema raw) i eq' row' finding).mp hfind
      rw [hres] at hkind
      cases hkind
    · obtain ⟨i, key', tables, pair, hdecl, hkt, hres1, hres2, hfind⟩ :=
        mem_keyFindings schema (Instance.build schema raw) hk
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, cop', hdecl, hmem⟩ :=
        mem_coproductFindings schema (Instance.build schema raw) hc
      obtain ⟨tables, row', n, htab, hrows, hrow'', hfind, hcase⟩ :=
        mem_coproductDeclaration_imp schema (Instance.build schema raw) i cop' hmem
      rw [hfind] at hkind
      simp only [Check.coproductFinding, FindingKind.violation.injEq,
        ViolationWitness.coproduct.injEq, CoproductWitness.mk.injEq] at hkind
      obtain ⟨hcopidx, hrow', hinj⟩ := hkind
      have hi : i = index := by rw [hcopidx]; exact hwit
      rw [hi] at hdecl
      rw [hcop] at hdecl
      have hcopeq : cop' = coproduct := by simpa using hdecl.symm
      rw [hcopeq] at htab
      obtain ⟨tables0, htab0, hne0⟩ := hpartial
      rw [htab0] at htab
      have htables : tables = tables0 := by simpa using htab.symm
      change 2 ≤ winj.length
      rw [← hinj]
      rw [htables] at hcase ⊢
      rcases hcase with ⟨heq, _⟩ | ⟨hne, hge⟩
      · exact absurd heq hne0
      · exact hge
    · obtain ⟨_i, _pb, _w, _hdecl, _hlabel, hshape⟩ :=
        mem_pullbackFindings_shape schema (Instance.build schema raw) hp
      rw [hshape] at hkind
      cases hkind
  complete_image_disjointness := by
    intro index coproduct row injections hcop happlies hfailure h2
    obtain ⟨nt, hnrows, hnrow⟩ := happlies
    obtain ⟨tables, images, htab, himages, hinj, hcase⟩ := hfailure
    rw [himages] at hinj hcase
    rw [hinj] at hcase
    rw [mem_check_iff]
    right; right; right; left
    refine mem_coproductFindings_of schema (Instance.build schema raw) index coproduct hcop ?_
    rw [hinj]
    exact mem_coproductDeclaration_of schema (Instance.build schema raw) index coproduct
      tables row nt htab hnrows hnrow hcase
  typing_violated_accounted_once := by
    intro name arrow src value harrow hviol
    obtain ⟨⟨ename, earrow⟩, hentry, hname, harrow'⟩ := arrows_find? harrow
    have hmem_schema : (name, arrow) ∈ schema.arrows := by
      have hp : ((ename, earrow) : Ident × Arrow) = (name, arrow) := Prod.ext hname harrow'
      rw [← hp]
      exact List.mem_of_find?_eq_some hentry
    obtain ⟨idx, value', hlen, hidx, hcell, hle, hv⟩ :=
      (mem_buildArrow_typing_iff raw name arrow
        { arrow := name, src := src, value := value }).mp hviol
    cases hv
    have htable : (Instance.buildArrow raw name arrow).table[src]? = some none := by
      rw [buildArrow_table_cell, ite_eq_left hidx, hcell]
      simp [hle]
    have hbuild : ({ arrow := name, src := src, value := value } : Instance.TypingViolation) ∈
        (Instance.build schema raw).typingViolations := by
      rw [mem_build_typing]
      exact ⟨(name, arrow), hmem_schema, hviol⟩
    refine ⟨htable, hbuild, ?_⟩
    intro finding hcheck hkind
    have hviolated : (name, src) ∈
        (Instance.build schema raw).typingViolations.map
          (fun v => (v.arrow, v.src)) := by
      exact List.mem_map.mpr ⟨{ arrow := name, src := src, value := value }, hbuild, rfl⟩
    rw [mem_check_iff] at hcheck
    rcases hcheck with hd | he | hk | hc | hp
    · obtain ⟨entry, idx', hentry', hcell', hcont, hfind⟩ :=
        mem_danglingFindings_imp (Instance.build schema raw) _ hd
      rw [hfind] at hkind
      simp only [FindingKind.dangling.injEq, DanglingWitness.mk.injEq] at hkind
      obtain ⟨harrow2, hsrc2⟩ := hkind
      rw [harrow2, hsrc2] at hcont
      have htrue :
          ((Instance.build schema raw).typingViolations.map
            (fun v => (v.arrow, v.src))).contains (name, src) = true :=
        List.contains_iff_mem.mpr hviolated
      rw [htrue] at hcont
      simp at hcont
    · obtain ⟨i, eq', row', hdecl, hfind⟩ :=
        mem_equationFindings schema (Instance.build schema raw) he
      obtain ⟨lhs', rhs', hl, hr, hne, hres⟩ :=
        (equationFinding_eq_some_iff (Instance.build schema raw) i eq' row' finding).mp hfind
      rw [hres] at hkind
      cases hkind
    · obtain ⟨i, key', tables, pair, hdecl, hkt, hres1, hres2, hfind⟩ :=
        mem_keyFindings schema (Instance.build schema raw) hk
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨i, cop', hdecl, hmem⟩ :=
        mem_coproductFindings schema (Instance.build schema raw) hc
      obtain ⟨tables, row', n, htab, hrows, hrow'', hfind, hcase⟩ :=
        mem_coproductDeclaration_imp schema (Instance.build schema raw) i cop' hmem
      rw [hfind] at hkind
      cases hkind
    · obtain ⟨_i, _pb, _w, _hdecl, _hlabel, hshape⟩ :=
        mem_pullbackFindings_shape schema (Instance.build schema raw) hp
      rw [hshape] at hkind
      cases hkind
  carried_trichotomy := by
    intro name arrow idx harrow hidx hlen
    obtain ⟨⟨ename, earrow⟩, hentry, hname, harrow'⟩ := arrows_find? harrow
    have hmem_schema : (name, arrow) ∈ schema.arrows := by
      have hp : ((ename, earrow) : Ident × Arrow) = (name, arrow) := Prod.ext hname harrow'
      rw [← hp]
      exact List.mem_of_find?_eq_some hentry
    have htable := build_tables_find? schema raw name arrow harrow
    have hbuildArrow : (Instance.buildArrow raw name arrow).table[idx]? =
        some (match (RawCells raw name)[idx]? with
          | some (some value) =>
            if value < (Schema.rowCount? raw.rows arrow.tgt).getD 0 then some value else none
          | _ => none) := by
      rw [buildArrow_table_cell, ite_eq_left hidx]
    cases hcell : (RawCells raw name)[idx]? with
    | none =>
      have hge : (RawCells raw name).length ≤ idx := List.getElem?_eq_none_iff.mp hcell
      exact absurd hge (Nat.not_le.mpr hlen)
    | some c =>
      cases c with
      | none =>
        refine Or.inr (Or.inr ?_)
        refine ⟨(Instance.buildArrow raw name arrow).table, htable, ?_, by simp [RawCell, hcell], ?_⟩
        · rw [hbuildArrow, hcell]
        · rw [mem_build_dangling]
          exact ⟨(name, arrow), hmem_schema,
            (mem_buildArrow_dangling_iff raw name arrow
              { arrow := name, src := idx }).mpr ⟨idx, hlen, hidx, hcell, rfl⟩⟩
      | some value =>
        by_cases hlt : value < (Schema.rowCount? raw.rows arrow.tgt).getD 0
        · refine Or.inl ?_
          refine ⟨(Instance.buildArrow raw name arrow).table, value, htable, ?_, ?_⟩
          · rw [hbuildArrow, hcell]; simp [hlt]
          · simp [RawCell, hcell]
        · refine Or.inr (Or.inl ?_)
          refine ⟨(Instance.buildArrow raw name arrow).table, value, htable, ?_, ?_, ?_⟩
          · rw [hbuildArrow, hcell]; simp [hlt]
          · simp [RawCell, hcell]
          · rw [mem_build_typing]
            exact ⟨(name, arrow), hmem_schema,
              (mem_buildArrow_typing_iff raw name arrow
                { arrow := name, src := idx, value := value }).mpr
                ⟨idx, value, hlen, hidx, hcell, Nat.le_of_not_lt hlt, rfl⟩⟩


end Proofs
end Catherine