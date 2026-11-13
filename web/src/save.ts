// Purpose: the one save path (openspec spec `wasm-ui`, scenario
// "The view is never the source of truth") — the document text the
// textarea holds is written verbatim as a downloaded `.ct.json`, with no
// re-serialization, no added keys, and no side-car file. Extracted from
// app.ts so the guarantee is testable without booting the whole page.
//
// Responsibilities: name the artifact, wrap the exact bytes in a JSON
// blob, and trigger the download. Rationale: a save path that rewrote
// the document (pretty-printing, metadata injection) would make the view
// a second source of truth; keeping it a byte copy keeps the kernel the
// only parser.

import { el } from "./dom";

/** The downloaded artifact's name — a plain model document. */
export const SAVED_FILENAME = "model.ct.json";

/** Offer `source` as a downloaded model document — bytes verbatim. */
export function saveArtifact(source: string): void {
  const blob = new Blob([source], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = el("a", { href: url, download: SAVED_FILENAME });
  link.click();
  URL.revokeObjectURL(url);
}