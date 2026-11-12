// Purpose: the SVG schema-graph panel (openspec change `add-wasm-web-view`,
// GREEN task 6) — objects as nodes, arrows as directed edges, over the
// document's schema section. Presentation only: the graph is derived
// from the textarea's JSON on every render, never held as state.

import type { CtSchemaV1Document } from "./types/ct-schema-v1";
import { clear } from "./dom";

type SchemaSection = CtSchemaV1Document["schema"];
type ArrowDecl = CtSchemaV1Document["schema"]["arrows"][number];

/** SVG namespace for createElementNS. */
const SVG_NS = "http://www.w3.org/2000/svg";

/** Geometry of the circular layout (viewBox units). */
const WIDTH = 640;
const HEIGHT = 320;
const NODE_RADIUS = 28;
const MARGIN = 48;

/** A node position on the layout circle (sorted idents, first at the top). */
function nodePosition(index: number, total: number): { cx: number; cy: number } {
  const angle = (2 * Math.PI * index) / total - Math.PI / 2;
  return {
    cx: WIDTH / 2 + (WIDTH / 2 - MARGIN - NODE_RADIUS) * Math.cos(angle),
    cy: HEIGHT / 2 + (HEIGHT / 2 - MARGIN - NODE_RADIUS) * Math.sin(angle),
  };
}

/** An SVG element with attributes and children — the graph's `el`. */
function svg<K extends keyof SVGElementTagNameMap>(
  tag: K,
  attrs: Record<string, string | number>,
  children: readonly Node[] = [],
): SVGElementTagNameMap[K] {
  const node = document.createElementNS(SVG_NS, tag);
  for (const [name, value] of Object.entries(attrs)) {
    node.setAttribute(name, String(value));
  }
  for (const child of children) {
    node.append(child);
  }
  return node;
}

/** One arrow as an edge between its endpoint nodes, labeled with its name. */
function arrowEdge(arrow: ArrowDecl, positions: ReadonlyMap<string, { cx: number; cy: number }>): SVGElementTagNameMap["g"] {
  const src = positions.get(arrow.src);
  const tgt = positions.get(arrow.tgt);
  if (src === undefined || tgt === undefined) {
    // The kernel rejects undeclared endpoints; the graph simply has no
    // edge to draw for a shape the parser would refuse.
    return svg("g", {});
  }
  const dx = tgt.cx - src.cx;
  const dy = tgt.cy - src.cy;
  const len = Math.hypot(dx, dy);
  const x1 = src.cx + (dx / len) * NODE_RADIUS;
  const y1 = src.cy + (dy / len) * NODE_RADIUS;
  const x2 = tgt.cx - (dx / len) * NODE_RADIUS;
  const y2 = tgt.cy - (dy / len) * NODE_RADIUS;
  const mx = (x1 + x2) / 2;
  const my = (y1 + y2) / 2;
  return svg("g", { class: "edge" }, [
    svg("line", { x1, y1, x2, y2, "marker-end": "url(#arrowhead)" }),
    svg("text", { x: mx, y: my - 6, class: "edge-label", "text-anchor": "middle" }, [
      document.createTextNode(arrow.name),
    ]),
  ]);
}

/**
 * Render the schema's graph into `svgElement` (cleared first): sorted
 * object idents on a circle, one labeled directed edge per arrow.
 * Deterministic — same schema, same picture.
 */
export function renderSchemaGraph(schema: SchemaSection, svgElement: SVGSVGElement): void {
  clear(svgElement);
  svgElement.setAttribute("viewBox", `0 0 ${WIDTH} ${HEIGHT}`);
  svgElement.append(
    svg("defs", {}, [
      svg("marker", {
        id: "arrowhead",
        markerWidth: 8,
        markerHeight: 8,
        refX: 7,
        refY: 3,
        orient: "auto",
      }, [svg("path", { d: "M0,0 L0,6 L7,3 z" })]),
    ]),
  );
  const objects = [...schema.objects].sort();
  const positions = new Map(
    objects.map((name, index) => [name, nodePosition(index, objects.length)] as const),
  );
  for (const arrow of schema.arrows) {
    svgElement.append(arrowEdge(arrow, positions));
  }
  for (const [name, { cx, cy }] of positions) {
    svgElement.append(
      svg("g", { class: "node" }, [
        svg("circle", { cx, cy, r: NODE_RADIUS, class: "node-circle" }),
        svg("text", { x: cx, y: cy + 4, class: "node-label", "text-anchor": "middle" }, [
          document.createTextNode(name),
        ]),
      ]),
    );
  }
}
