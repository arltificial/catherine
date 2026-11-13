// Purpose: the findings panel (openspec change `add-wasm-web-view`,
// RED task 2 + GREEN task 6) — renders the kernel's findings encoding
// (the same JSON the CLI forwards in `data.findings`; the Rust shapes
// live in crates/catherine-core/src/findings.rs) as rows of kind +
// epistemic label + witness, the same taxonomy the terminal prints.
//
// Responsibilities: [`FindingEnc`] and its witnesses — the TS mirror of
// the kernel's serde encoding, pinned by the parity test (wasm task 7);
// the violation witness is an untagged union (equation or key) split on
// field presence — and [`renderFindings`]: one `li.finding` per finding
// carrying `data-kind` and `data-label`, with `.kind`, `.label`, and `.witness`
// spans inside. Structural, not snapshot: every fact a finding carries
// is present and queryable in the DOM.
//
// Rationale: the browser and the terminal must agree on the taxonomy —
// kind names the shape, the label names the weight — so the panel never
// reshapes, re-labels, or drops a finding; it renders the boundary's
// encoding verbatim. The panel is never the source of truth: the JSON
// textarea is the document, this is only its view.

import { clear, el, text } from "./dom";

/** The epistemic label — exactly the three spellings the kernel type allows. */
export type EpistemicLabel = "executed" | "hypothesized" | "unknown";

/** The equation arm of the violation witness — which equation failed, on which row, with the two values. */
export interface EquationViolationWitness {
  equation: number;
  row: number;
  lhs: number;
  rhs: number;
}

/** The key arm — the canonical key index and the colliding row pair, ascending. */
export interface KeyViolationWitness {
  key: number;
  rows: [number, number];
}

/**
 * The kernel's untagged violation-witness union: equation violations keep
 * `{equation, row, lhs, rhs}` byte-identically; key violations are
 * `{key, rows}`. The field sets are disjoint, so `"equation" in witness`
 * discriminates.
 */
export type ViolationWitness = EquationViolationWitness | KeyViolationWitness;

export interface DanglingWitness {
  arrow: string;
  src: number;
}

export interface ContradictionWitness {
  scope: number[];
}

export interface RedundantWitness {
  derivable: string[];
  reducing: string[];
}

/** The underconstraint witness, serde-flattened: `case`/`detail` first, the question text last. */
export interface UnderconstraintWitness {
  case: string;
  detail: Record<string, unknown>;
  question: string;
}

/** One finding, exactly as the kernel encodes it: `{label, kind, witness}`. */
export type FindingEnc =
  | { label: EpistemicLabel; kind: "violation"; witness: ViolationWitness }
  | { label: EpistemicLabel; kind: "dangling"; witness: DanglingWitness }
  | { label: EpistemicLabel; kind: "contradiction"; witness: ContradictionWitness }
  | { label: EpistemicLabel; kind: "redundant"; witness: RedundantWitness }
  | { label: EpistemicLabel; kind: "underconstraint"; witness: UnderconstraintWitness };

/** `a ∘ b` — the composition spelling the CLI's human renderer uses. */
function pathText(path: readonly string[]): string {
  return path.join(" ∘ ");
}

/** The witness facts, as text — one rendering path per witness shape. */
export function witnessText(finding: FindingEnc): string {
  switch (finding.kind) {
    case "violation": {
      const w = finding.witness;
      return "equation" in w
        ? `equation ${w.equation} on row ${w.row}: ${w.lhs} ≠ ${w.rhs}`
        : `key ${w.key}: rows ${w.rows[0]} and ${w.rows[1]} collide`;
    }
    case "dangling":
      return `${finding.witness.arrow} on row ${finding.witness.src} unresolved`;
    case "contradiction":
      return `accepted equations [${finding.witness.scope.join(", ")}] admit no instance within the bounds`;
    case "redundant":
      return `${pathText(finding.witness.derivable)} derivable; reduces to ${pathText(finding.witness.reducing)}`;
    case "underconstraint": {
      const w = finding.witness;
      const pair =
        Array.isArray(w.detail.lhs) && Array.isArray(w.detail.rhs)
          ? `${pathText(w.detail.lhs)} / ${pathText(w.detail.rhs)}`
          : JSON.stringify(w.detail);
      return `${pair} — ${w.question}`;
    }
  }
}

/**
 * Render the findings into `container` (which the panel owns: it is
 * cleared first). One `li.finding` per finding — kind label, epistemic
 * label, witness facts, in the kernel's order.
 */
export function renderFindings(findings: readonly FindingEnc[], container: Element): void {
  clear(container);
  const list = el("ul", { class: "findings" });
  for (const finding of findings) {
    list.append(
      el(
        "li",
        { class: "finding", "data-kind": finding.kind, "data-label": finding.label },
        [
          el("span", { class: "kind" }, [text(finding.kind)]),
          el("span", { class: "label" }, [text(finding.label)]),
          el("span", { class: "witness" }, [text(witnessText(finding))]),
        ],
      ),
    );
  }
  container.append(list);
}
