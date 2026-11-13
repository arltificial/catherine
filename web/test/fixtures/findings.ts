// Purpose: the golden-DOM fixtures (openspec change `add-wasm-web-view`,
// RED task 2; the key witness added for `add-declarative-keys`) — one
// finding per witness shape, in the kernel's own serde encoding, so the
// panel test exercises every kind, both violation labels, and both arms
// of the untagged violation-witness union. Encodings, not snapshots:
// these mirror the Rust shapes in crates/catherine-core/src/findings.rs;
// the parity test pins the wasm side to the kernel.

import type { FindingEnc } from "../../src/findings";

/** A key violation — `{key, rows}` with ascending rows, the union's second arm. */
export const keyViolationFixture: { name: string; finding: FindingEnc } = {
  name: "violation-key",
  finding: {
    label: "executed",
    kind: "violation",
    witness: { key: 0, rows: [3, 7] },
  },
};

/** Every finding shape the taxonomy has, with a representative witness. */
export const findingClassFixtures: readonly { name: string; finding: FindingEnc }[] = [
  {
    name: "violation-executed",
    finding: {
      label: "executed",
      kind: "violation",
      witness: { equation: 0, row: 3, lhs: 1, rhs: 2 },
    },
  },
  {
    name: "violation-hypothesized",
    finding: {
      label: "hypothesized",
      kind: "violation",
      witness: { equation: 1, row: 0, lhs: 2, rhs: 1 },
    },
  },
  {
    name: "dangling",
    finding: {
      label: "executed",
      kind: "dangling",
      witness: { arrow: "from", src: 1 },
    },
  },
  {
    name: "contradiction",
    finding: {
      label: "executed",
      kind: "contradiction",
      witness: { scope: [0, 2] },
    },
  },
  {
    name: "underconstraint",
    finding: {
      label: "hypothesized",
      kind: "underconstraint",
      witness: {
        case: "parallel_paths",
        detail: { lhs: ["from"], rhs: ["guard", "to"] },
        question: "should these agree?",
      },
    },
  },
  {
    name: "redundant",
    finding: {
      label: "executed",
      kind: "redundant",
      witness: { derivable: ["f", "g"], reducing: ["h"] },
    },
  },
  keyViolationFixture,
];
