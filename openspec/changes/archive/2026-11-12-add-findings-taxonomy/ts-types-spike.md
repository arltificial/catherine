# TS types spike — the findings JSON (REFACTOR phase)

Hand-written TypeScript sketch of the v1 findings JSON encoding as
defined by serde in `crates/catherine-core/src/findings.rs`. This note is
documentation only — **no codegen pipeline, no build step, no `web/` or
npm touches** (the change's deliberate exclusions). P3 (`catherine
findings`) prints this JSON verbatim; P5's wasm boundary hands it to
consumers typed as below.

```ts
type EpistemicLabel = "executed" | "hypothesized" | "unknown";
// "proved" and "illustrated" are deliberately absent — reserved for
// later layers, never fabricated by the kernel.

interface EquationWitness {
  equation: number; // canonical index into schema.equations
  row: number;      // dense row id of the equation's domain object
  lhs: number;      // composed lhs path value (row id)
  rhs: number;      // composed rhs path value (row id)
}
interface DanglingWitness {
  arrow: string;
  src: number;
}
interface RedundantWitness {
  derivable: string[]; // arrow-name sequence
  reducing: string[];  // arrow-name sequence
}
interface ContradictionWitness {
  scope: number[]; // canonical equation indices
}
type DomainQuestion =
  | { case: "parallel_paths"; detail: { lhs: string[]; rhs: string[] } }
  | { case: "uncovered_inputs"; detail: { class: string; examples: number[] } };
type UnderconstraintWitness = DomainQuestion & { question: string };
// Serialized (field order = declaration order): case, detail, question —
// the structural witness first, the free-form question text last.

type Finding =
  | { label: EpistemicLabel; kind: "violation"; witness: EquationWitness }
  | { label: EpistemicLabel; kind: "dangling"; witness: DanglingWitness }
  | { label: EpistemicLabel; kind: "redundant"; witness: RedundantWitness }
  | { label: EpistemicLabel; kind: "contradiction"; witness: ContradictionWitness }
  | { label: EpistemicLabel; kind: "underconstraint"; witness: UnderconstraintWitness };

// Findings arrive sorted by (kind, witness ids, label) — the kernel's
// total order — so repeated checks of the same model are byte-identical.
type Findings = Finding[];
```

Shape provenance (matches the serde encoding field-for-field): `label`
first, then the adjacent tag pair `kind`/`witness` (serde `flatten` of
the adjacently tagged `FindingKind`); the underconstraint witness
flattens its tagged domain-question union as `case`/`detail` beside
`question`. Field order in the JSON is declaration order; all
collections are ordered arrays (no maps keyed at runtime).
