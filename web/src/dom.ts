// Purpose: the deduplicated DOM helpers every panel renderer shares
// (openspec change `add-wasm-web-view`, REFACTOR task 8) — one
// construction path for elements, attributes, text, and clearing, so
// the findings panel, the schema graph, and the app shell never grow
// three copies of the same createElement boilerplate. No framework:
// these are plain wrappers over the DOM the platform already provides.

/** An element with attributes and children, built once. */
export function el<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  attrs: Record<string, string> = {},
  children: readonly (Node | string)[] = [],
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  for (const [name, value] of Object.entries(attrs)) {
    node.setAttribute(name, value);
  }
  for (const child of children) {
    node.append(child);
  }
  return node;
}

/** One text node's worth of content — the only text path renderers use. */
export function text(value: string): Text {
  return document.createTextNode(value);
}

/** Remove every child — the re-render path (renderers own their root). */
export function clear(node: Element): void {
  node.replaceChildren();
}

/** The app's element by id — a missing id is a wiring bug, thrown not hidden. */
export function byId<T extends Element>(id: string): T {
  const node = document.getElementById(id);
  if (node === null) {
    throw new Error(`catherine web: missing #${id} — index.html and app.ts disagree`);
  }
  return node as unknown as T;
}
