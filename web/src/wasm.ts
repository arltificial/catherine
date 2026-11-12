// Purpose: the wasm boundary's TS side (openspec change `add-wasm-web-view`,
// GREEN tasks 3 + 6) — the typed accessor over the wasm exports and the
// one place that turns a boundary failure back into a typed TS value.
//
// Responsibilities: [`CatherineWasm`] — the export surface the browser
// glue (the wasm-bindgen build) installs as `window.catherineWasm` — and
// [`runCheck`]/[`runAnalyze`], which parse the exports' JSON-string
// reports and decode the typed boundary error's `code`-tagged encoding.
//
// Rationale: the boundary speaks JSON strings; this module is the only
// code that parses them, so the views never see raw JSON and the error
// codes stay matchable (never message text). The bindings are injected,
// not imported: no framework, no bundler-time wasm coupling — the glue
// step owns the instantiation.

import type { FindingEnc } from "./findings";

/** The wasm export surface (installed by the browser glue as `window.catherineWasm`). */
export interface CatherineWasm {
  /** JSON document in → `{"findings":[...]}` out; a boundary rejection throws the typed error's JSON string. */
  check(document: string): string;
  /** JSON document + config in → `{"findings":[...], "unknown"?: {...}}` out; same error discipline. */
  analyze(document: string, config: string): string;
  /** The document cap the boundary enforces, so the UI can pre-flight. */
  max_document_bytes(): number;
}

declare global {
  interface Window {
    catherineWasm?: CatherineWasm;
  }
}

/** The injected bindings — a missing install is a glue/wiring bug, thrown not hidden. */
export function bindings(): CatherineWasm {
  const wasm = window.catherineWasm;
  if (wasm === undefined) {
    throw new Error(
      "catherine web: window.catherineWasm is not installed — the wasm glue step has not run",
    );
  }
  return wasm;
}

/** The typed boundary error, decoded — the `code` tag is the matchable contract. */
export type BoundaryErrorEnc =
  | { code: "document_too_large"; bytes: number; max_bytes: number }
  | { code: "parse"; line: number; column: number; message: string }
  | { code: "analysis_config"; message: string };

/** One export call's outcome: findings (plus `analyze`'s search-outcome discriminator), or the typed boundary error. */
export type BoundaryResult =
  | { ok: true; findings: FindingEnc[]; outcome?: string }
  | { ok: false; error: BoundaryErrorEnc };

/** A `check`/`analyze` report as decoded JSON. */
interface FindingsReport {
  findings: FindingEnc[];
  /** `analyze` only: `"witness" | "contradiction" | "unknown"`. */
  outcome?: string;
}

/** Runtime guard for the boundary report's shape (the one field the views read). */
function isFindingsReport(value: unknown): value is FindingsReport {
  return (
    typeof value === "object" &&
    value !== null &&
    "findings" in value &&
    Array.isArray(value.findings)
  );
}

/**
 * Call an export and decode its report: the JSON-string findings report
 * on success, the thrown typed-error JSON string as a typed
 * [`BoundaryErrorEnc`]. Any non-boundary throw propagates — that is a
 * bug, not a diagnostic.
 */
function run(call: () => string): BoundaryResult {
  let report: string;
  try {
    report = call();
  } catch (thrown) {
    const raw =
      typeof thrown === "string" ? thrown : thrown instanceof Error ? thrown.message : undefined;
    const decoded =
      raw !== undefined ? (safeJsonParse(raw) as BoundaryErrorEnc | undefined) : undefined;
    if (decoded !== undefined && typeof decoded === "object" && "code" in decoded) {
      return { ok: false, error: decoded };
    }
    throw thrown;
  }
  const parsed = safeJsonParse(report);
  if (!isFindingsReport(parsed)) {
    throw new Error("catherine web: boundary report is not a findings report");
  }
  return { ok: true, findings: parsed.findings, outcome: parsed.outcome };
}

/** `JSON.parse` that answers undefined instead of throwing. */
function safeJsonParse(raw: string): unknown {
  try {
    return JSON.parse(raw) as unknown;
  } catch {
    return undefined;
  }
}

/** The check export, decoded. */
export function runCheck(document: string): BoundaryResult {
  return run(() => bindings().check(document));
}

/** The analyze export, decoded; `config` is [`AnalysisConfigEnc`]'s JSON. */
export function runAnalyze(document: string, config: string): BoundaryResult {
  return run(() => bindings().analyze(document, config));
}

/** The analysis config surface — mirrors the boundary's `AnalysisConfig`. */
export interface AnalysisConfigEnc {
  max_candidates?: number;
  max_path_length?: number;
  max_rewrite_steps?: number;
  max_rows?: Record<string, number>;
}
