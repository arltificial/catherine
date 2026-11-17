import Catherine.Schema
import Catherine.Findings
import Catherine.Instance
import Catherine.EvalPath
import Catherine.Check
import Catherine.Proofs

/-!
Purpose: root module of the Lean 4 assurance twin for catherine-core.
Re-exports the kernel reference semantics (schema, findings taxonomy,
typed instances, path evaluation, executed checks) so the lean-parity
oracle and the theorems (Soundness, Completeness, DanglingDiscipline) can
consume one coherent surface.

Responsibilities: import the five kernel modules in dependency order,
then the proof modules (`Catherine.Proofs`); stay a pure aggregation
point — no definitions belong here.

Rationale: a single root keeps `lake build` green on stubs (tasks.md
item 1) and gives later phases one import target instead of five.
-/
