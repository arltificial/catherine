// Purpose: the app shell (openspec change `add-wasm-web-view`, GREEN
// task 6) — wires the one editing surface (the JSON textarea), file
// load/save, the schema graph, and the findings panel into a single
// static page. No server, no framework, no persisted state: the
// textarea is the document, every panel is a view of it.

import { byId, clear, el, text } from "./dom";
import { renderFindings } from "./findings";
import { renderSchemaGraph } from "./schemaGraph";
import { runCheck, type BoundaryErrorEnc } from "./wasm";
import type { CtSchemaV1Document } from "./types/ct-schema-v1";

/** The boundary error, as a user-facing line — the `code` is the stable part. */
function errorText(error: BoundaryErrorEnc): string {
  switch (error.code) {
    case "document_too_large":
      return `document too large: ${error.bytes} bytes (cap ${error.max_bytes}) — the kernel was not invoked`;
    case "parse":
      return `parse error at line ${error.line}, column ${error.column}: ${error.message}`;
    case "analysis_config":
      return `analysis config error: ${error.message}`;
  }
}

/** Parse the textarea's JSON into a document — a syntax error becomes a reported message. */
function readDocument(source: string): CtSchemaV1Document | undefined {
  try {
    return JSON.parse(source) as CtSchemaV1Document;
  } catch (error) {
    showError(`not valid JSON: ${error instanceof Error ? error.message : String(error)}`);
    return undefined;
  }
}

/** The one error surface — clears, then reports. */
function showError(message: string): void {
  const box = byId<HTMLElement>("error");
  clear(box);
  if (message.length > 0) {
    box.append(el("p", { class: "error-line" }, [text(message)]));
  }
}

/** Re-render every panel from the textarea's current document. */
function renderAll(): void {
  showError("");
  const document = readDocument(byId<HTMLTextAreaElement>("doc").value);
  if (document === undefined) {
    return;
  }
  renderSchemaGraph(document.schema, byId<SVGSVGElement>("schema-graph"));
  const result = runCheck(byId<HTMLTextAreaElement>("doc").value);
  if (result.ok) {
    renderFindings(result.findings, byId<HTMLElement>("findings-panel"));
  } else {
    showError(errorText(result.error));
    renderFindings([], byId<HTMLElement>("findings-panel"));
  }
}

/** Load a `.ct.json` file into the textarea (file input change), then re-render. */
function loadFile(file: File): void {
  const reader = new FileReader();
  reader.addEventListener("load", () => {
    byId<HTMLTextAreaElement>("doc").value = String(reader.result ?? "");
    renderAll();
  });
  reader.readAsText(file);
}

/** Save the textarea's current document as a downloaded `.ct.json` file. */
function saveFile(): void {
  const blob = new Blob([byId<HTMLTextAreaElement>("doc").value], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const link = el("a", { href: url, download: "model.ct.json" });
  link.click();
  URL.revokeObjectURL(url);
}

/** Wire the static page: every control, once. */
export function start(): void {
  byId<HTMLButtonElement>("check-btn").addEventListener("click", renderAll);
  byId<HTMLButtonElement>("save-btn").addEventListener("click", saveFile);
  byId<HTMLInputElement>("file").addEventListener("change", (event) => {
    const file = (event.target as HTMLInputElement).files?.[0];
    if (file !== undefined) {
      loadFile(file);
    }
  });
  renderAll();
}

// The static page's only entry: wire once, on module load.
start();
