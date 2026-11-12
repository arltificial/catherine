#!/usr/bin/env node
// Purpose: the TS type generator (openspec change `add-wasm-web-view`,
// task 5) — derives web/src/types/ct-schema-v1.ts from the kernel's
// schema artifact crates/catherine-core/schema/ct-schema-v1.json.
//
// The generated file is checked in but never hand-edited: rerun
// `npm run generate:types` after any schema change; CI-minded editors
// can diff the output. No dependencies — plain node.
//
// Supported JSON-Schema constructs (exactly what the artifact uses):
// type, properties, required, additionalProperties, propertyNames,
// items, $ref/$defs, enum, oneOf, minItems, pattern, uniqueItems.

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const schemaPath = resolve(here, "../../crates/catherine-core/schema/ct-schema-v1.json");
const outPath = join(here, "../src/types/ct-schema-v1.ts");

const schema = JSON.parse(readFileSync(schemaPath, "utf8"));

/** $defs name → exported TS type name ("ident" → "Ident"). */
const defTypeName = (name) =>
  name
    .split(/[_-]/)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join("");

/** Map a `$ref` node to its exported TS type name. */
const tsRef = (node) => defTypeName(node.$ref.split("/").pop());

/** Map an `enum` node to a union of its string literals. */
const tsEnum = (node) => node.enum.map((v) => JSON.stringify(v)).join(" | ");

/** Map a `oneOf` node to a union of parenthesized alternatives. */
const tsOneOf = (node, inDefs) =>
  node.oneOf.map((alt) => `(${tsType(alt, inDefs)})`).join(" | ");

/** Map an `object` node to an object type literal (or a `Record`). */
function tsObject(node, inDefs) {
  const required = new Set(node.required ?? []);
  const lines = [];
  for (const [key, prop] of Object.entries(node.properties ?? {})) {
    const opt = required.has(key) ? "" : "?";
    lines.push(`  ${JSON.stringify(key)}${opt}: ${tsType(prop, inDefs)};`);
  }
  if (node.additionalProperties) {
    const value = tsType(node.additionalProperties, inDefs);
    if (lines.length === 0) return `Record<string, ${value}>`;
    lines.push(`  [key: string]: ${value};`);
  } else if (lines.length === 0) {
    return "Record<string, unknown>";
  }
  return `{\n${lines.join("\n")}\n}`;
}

/** Map a typed (non-$ref/enum/oneOf) node by its `type` keyword. */
function tsByType(node, inDefs) {
  switch (node.type) {
    case "string":
      return "string";
    case "null":
      return "null";
    case "array":
      return `Array<${tsType(node.items ?? {}, inDefs)}>`;
    case "object":
      return tsObject(node, inDefs);
    default:
      return "unknown";
  }
}

/** Map one JSON-schema node to a TS type expression. */
function tsType(node, inDefs) {
  if (typeof node === "boolean") return "unknown";
  if (node.$ref) return tsRef(node);
  if (node.enum) return tsEnum(node);
  if (node.oneOf) return tsOneOf(node, inDefs);
  return tsByType(node, inDefs);
}

const defs = schema.$defs ?? {};
const defBlocks = Object.entries(defs).map(([name, node]) => {
  const exportName = defTypeName(name);
  return `export type ${exportName} = ${tsType(node, true)};`;
});

// Root document type: the schema's top object as an exported interface.
const rootName = "CtSchemaV1Document";
const rootBlock = `export type ${rootName} = ${tsType(
  { ...schema, $defs: undefined, $ref: undefined },
  false,
)};`;

const header = `// GENERATED from ${schemaPath.replace(resolve(here, "../..") + "/", "")}
// by scripts/generate-types.mjs (openspec change \`add-wasm-web-view\`,
// task 5) — do not hand-edit; rerun \`npm run generate:types\`.
`;

mkdirSync(dirname(outPath), { recursive: true });
writeFileSync(
  outPath,
  `${header}\n${[...defBlocks, rootBlock].join("\n\n")}\n`,
);

console.log(`generated ${outPath}`);
