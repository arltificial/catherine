// Purpose: the golden-DOM test (openspec change `add-wasm-web-view`,
// RED task 2) — the findings panel renders kind + epistemic label +
// witness for every finding class fixture, structurally: the exact
// element tree per class is asserted in the test, never snapshotted.

import { describe, expect, it } from "vitest";
import { renderFindings, witnessText, type FindingEnc } from "../src/findings";
import { findingClassFixtures, keyViolationFixture } from "./fixtures/findings";

/** The golden tree for one finding row — built here, compared to the panel's output. */
function goldenRow(finding: FindingEnc): string {
  const li = document.createElement("li");
  li.setAttribute("class", "finding");
  li.setAttribute("data-kind", finding.kind);
  li.setAttribute("data-label", finding.label);
  for (const [cls, value] of [
    ["kind", finding.kind],
    ["label", finding.label],
    ["witness", witnessText(finding)],
  ] as const) {
    const span = document.createElement("span");
    span.setAttribute("class", cls);
    span.textContent = value;
    li.append(span);
  }
  return li.outerHTML;
}

describe("findings panel (golden DOM)", () => {
  it("renders kind + label + witness for every finding class fixture", () => {
    const panel = document.createElement("div");
    const fixtures = findingClassFixtures.map((f) => f.finding);
    renderFindings(fixtures, panel);

    const list = panel.querySelector("ul.findings");
    expect(list).not.toBeNull();
    const rows = [...list!.querySelectorAll("li.finding")];
    expect(rows).toHaveLength(findingClassFixtures.length);
    for (const [index, fixture] of findingClassFixtures.entries()) {
      expect(rows[index]!.outerHTML, `fixture ${fixture.name}`).toBe(goldenRow(fixture.finding));
    }
  });

  it("exposes each class queryably by kind and label", () => {
    const panel = document.createElement("div");
    renderFindings(
      findingClassFixtures.map((f) => f.finding),
      panel,
    );
    // Every kind in the taxonomy is present and matchable by data-kind.
    const kinds = new Set(
      [...panel.querySelectorAll("li.finding")].map((row) => row.getAttribute("data-kind")),
    );
    expect(kinds).toEqual(
      new Set(["violation", "dangling", "contradiction", "underconstraint", "redundant"]),
    );
    // Both violation labels survive as distinct rows — never merged, and the
    // key witness adds a second `executed` violation beside the equation one.
    expect(panel.querySelectorAll('li.finding[data-kind="violation"][data-label="executed"]')).toHaveLength(2);
    expect(panel.querySelectorAll('li.finding[data-kind="violation"][data-label="hypothesized"]')).toHaveLength(1);
    // Witness facts are present in every row's witness span.
    for (const row of panel.querySelectorAll("li.finding")) {
      expect(row.querySelector(".witness")?.textContent?.length ?? 0).toBeGreaterThan(0);
    }
  });

  it("names the key and both rows of a key violation — no undefined fields", () => {
    // The key arm of the untagged union renders both row ids.
    expect(witnessText(keyViolationFixture.finding)).toBe("key 0: rows 3 and 7 collide");

    const panel = document.createElement("div");
    renderFindings([keyViolationFixture.finding], panel);
    const row = panel.querySelector('li.finding[data-kind="violation"]');
    expect(row).not.toBeNull();
    const witness = row!.querySelector(".witness")?.textContent ?? "";
    expect(witness).toBe("key 0: rows 3 and 7 collide");
    expect(witness).toContain("3");
    expect(witness).toContain("7");
    expect(witness).not.toContain("undefined");
  });

  it("clears the panel on re-render — the panel is never stateful", () => {
    const panel = document.createElement("div");
    renderFindings(findingClassFixtures.map((f) => f.finding), panel);
    renderFindings([], panel);
    expect(panel.querySelectorAll("li.finding")).toHaveLength(0);
    expect(panel.querySelector("ul.findings")).not.toBeNull();
  });
});
