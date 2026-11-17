import Catherine.Parity

/-!
# Purpose: executable entry point of the `lean-parity` oracle.

## Responsibilities: forward the process arguments to
`Catherine.Parity.main` (whose `IO UInt32` result becomes the process
exit code) and keep this module a one-line shim so the Lake manifest's
`lean_exe «lean-parity»` target has a stable root.

## Rationale: Lake requires the executable's root module to declare `main`;
keeping the harness logic in `Catherine.Parity` lets the library build
without pulling `Lean` into the kernel modules.
-/

def main (args : List String) : IO UInt32 :=
  Catherine.Parity.main args