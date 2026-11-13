// Purpose: the save-path guarantee (openspec spec `wasm-ui`, scenario
// "The view is never the source of truth") — saving the textarea's
// document downloads a plain `.ct.json` whose bytes are the document the
// kernel parses, with no side-car state added by the UI.
//
// Responsibilities: assert the downloaded filename, media type, and that
// the blob bytes equal the edited text and re-parse to the same document
// (no added or renamed keys). Rationale: jsdom can observe the real
// anchor/blob path, so the test exercises the shipped save surface
// rather than a helper that merely returns its argument.

import { afterEach, describe, expect, it, vi } from "vitest";
import { SAVED_FILENAME, saveArtifact } from "../src/save";

const documentText = JSON.stringify(
  {
    schema: { version: "1.0", objects: [], arrows: [], equations: [] },
    instance: { rows: {}, values: {} },
  },
  null,
  2,
);

/** What the browser received: the blob handed to the download URL plus the link's name. */
function captureDownload(): { blob: Blob; filename: string } {
  const captured: { blob: Blob; filename: string } = {
    blob: new Blob(),
    filename: "",
  };
  // jsdom ships no object-URL implementation; the save path's only use of it
  // is handing the blob to the download, so capture that blob directly.
  Object.assign(URL, {
    createObjectURL: vi.fn((blob: Blob) => {
      captured.blob = blob;
      return "blob:test";
    }),
    revokeObjectURL: vi.fn(() => undefined),
  });
  vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(function (
    this: HTMLAnchorElement,
  ) {
    captured.filename = this.download;
  });
  return captured;
}

/** jsdom's Blob has no `text()`; FileReader is the platform's read path. */
function blobText(blob: Blob): Promise<string> {
  const { promise, resolve, reject } = Promise.withResolvers<string>();
  const reader = new FileReader();
  reader.addEventListener("load", () => resolve(String(reader.result ?? "")));
  reader.addEventListener("error", () => reject(reader.error));
  reader.readAsText(blob);
  return promise;
}

afterEach(() => {
  vi.restoreAllMocks();
  delete (URL as unknown as Record<string, unknown>).createObjectURL;
  delete (URL as unknown as Record<string, unknown>).revokeObjectURL;
});

describe("save path (the view is never the source of truth)", () => {
  it("downloads the edited document verbatim as a .ct.json with no side-car state", async () => {
    const captured = captureDownload();

    saveArtifact(documentText);

    expect(captured.filename).toBe(SAVED_FILENAME);
    expect(captured.filename).toBe("model.ct.json");
    expect(captured.blob.type).toBe("application/json");
    const saved = await blobText(captured.blob);
    expect(saved, "saved bytes must be the edited document, not a re-render").toBe(documentText);
    expect(JSON.parse(saved), "the UI adds no keys the kernel would parse as model state").toEqual(
      JSON.parse(documentText),
    );
    expect(Object.keys(JSON.parse(saved)).sort()).toEqual(["instance", "schema"]);
  });
});