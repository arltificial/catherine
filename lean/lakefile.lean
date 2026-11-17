import Lake
open Lake DSL

/-!
Purpose: Lake package manifest for the Lean 4 assurance twin of
catherine-core — the external reference semantics that re-implements the
kernel (schema, typed instances, path evaluation, executed checks) so its
proofs and the lean-parity oracle can cross-examine the Rust
implementation. Standard-library-only by design decision 1: no Mathlib.

Responsibilities: declare the Catherine library (root module
`Catherine.lean`) as the default build target and the `lean-parity`
oracle executable (root `Catherine.ParityMain`, tasks.md item 2); keep
the build light and reproducible.

Rationale: the Lake DSL keeps provisioning to elan + one pinned toolchain
file (see lean-toolchain); anything the kernel needs beyond the std is a
scope violation, not a dependency to add.
-/

package Catherine

@[default_target]
lean_lib Catherine

/-- The parity oracle: root `Catherine.ParityMain`; the hyphenated
executable name is a quoted identifier in the Lake DSL. -/
lean_exe «lean-parity» where
  root := `Catherine.ParityMain
