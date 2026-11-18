// Purpose: the findings panel (openspec change `add-wasm-web-view`,
// RED task 2 + GREEN task 6) — renders the kernel's findings encoding
// (the same JSON the CLI forwards in `data.findings`; the Rust shapes
// live in crates/catherine-core/src/findings.rs) as rows of kind +
// epistemic label + witness, the same taxonomy the terminal prints.
//
// Responsibilities: [`FindingEnc`] and its witnesses — the TS mirror of
// the kernel's serde encoding, pinned by the parity test (wasm task 7);
// the violation witness is an untagged union (equation, key, coproduct,
// or pullback) split on field presence, with the coproduct's optional
// `injections` field carried exactly when the kernel emits it and the
// pullback's `row`/`lhs`/`rhs`, `pair`, or `rows` carrying its verdict —
// and [`renderFindings`]: one `li.finding` per finding
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
 * The coproduct arm — the declaration's canonical index and the failed
 * row. `injections` is present exactly when the row lies in more than one
 * declared injection (the disjointness failure); the kernel omits the
 * field for a coverage failure, so it is optional here rather than an
 * empty array that would render as an undefined-valued fact.
 */
export interface CoproductViolationWitness {
  coproduct: number;
  row: number;
  injections?: string[];
}

/**
 * The pullback arm — the declaration's canonical index plus the
 * verdict-specific offender: `row`/`lhs`/`rhs` for a commutation
 * failure, `pair` for an uncovered fiber pair, or `rows` for two apex
 * rows projecting to one pair. Exactly one shape's fields are present,
 * so the arm is discriminated by field presence like the others.
 */
export interface PullbackViolationWitness {
  pullback: number;
  row?: number;
  lhs?: number;
  rhs?: number;
  pair?: [number, number];
  rows?: [number, number];
}

/**
 * The kernel's untagged violation-witness union: equation violations keep
 * `{equation, row, lhs, rhs}` byte-identically; key violations are
 * `{key, rows}`; coproduct violations are `{coproduct, row, injections?}`;
 * pullback violations carry `pullback` plus their verdict's offender.
 * The field sets are disjoint, so `"equation" in witness`,
 * `"key" in witness`, and so on discriminate.
 */
export type ViolationWitness =
  | EquationViolationWitness
  | KeyViolationWitness
  | CoproductViolationWitness
  | PullbackViolationWitness;

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

/** The equation arm's facts: which equation failed, on which row, with the two values. */
function equationViolationText(w: EquationViolationWitness): string {
  return `equation ${w.equation} on row ${w.row}: ${w.lhs} ≠ ${w.rhs}`;
}

/** The key arm's facts: the canonical key index and the colliding row pair. */
function keyViolationText(w: KeyViolationWitness): string {
  return `key ${w.key}: rows ${w.rows[0]} and ${w.rows[1]} collide`;
}

/** The coproduct arm's facts: uncovered row, or the colliding injections. */
function coproductViolationText(w: CoproductViolationWitness): string {
  const covering = w.injections ?? [];
  return covering.length === 0
    ? `coproduct ${w.coproduct}: row ${w.row} uncovered`
    : `coproduct ${w.coproduct}: row ${w.row} covered by ${covering.join(", ")} (disjointness)`;
}

/** The pullback arm's facts: the failed square, uncovered pair, or duplicate rows. */
function pullbackViolationText(w: PullbackViolationWitness): string {
  if (w.pair !== undefined) {
    return `pullback ${w.pullback}: pair ${w.pair[0]}, ${w.pair[1]} uncovered`;
  }
  if (w.rows !== undefined) {
    return `pullback ${w.pullback}: rows ${w.rows[0]} and ${w.rows[1]} project to one pair`;
  }
  return `pullback ${w.pullback}: row ${w.row} gives ${w.lhs} ≠ ${w.rhs}`;
}

/** The violation witness, split on the union's disjoint field sets. */
function violationText(w: ViolationWitness): string {
  if ("equation" in w) {
    return equationViolationText(w);
  }
  if ("key" in w) {
    return keyViolationText(w);
  }
  if ("coproduct" in w) {
    return coproductViolationText(w);
  }
  return pullbackViolationText(w);
}

/** The witness facts, as text — one rendering path per witness shape. */
export function witnessText(finding: FindingEnc): string {
  switch (finding.kind) {
    case "violation":
      return violationText(finding.witness);
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
