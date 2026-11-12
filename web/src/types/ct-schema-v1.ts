// GENERATED from crates/catherine-core/schema/ct-schema-v1.json
// by scripts/generate-types.mjs (openspec change `add-wasm-web-view`,
// task 5) — do not hand-edit; rerun `npm run generate:types`.

export type Ident = string;

export type Path = Array<Ident>;

export type CtSchemaV1Document = {
  "format_version": string;
  "schema": {
  "objects": Array<Ident>;
  "arrows": Array<{
  "name": Ident;
  "src": Ident;
  "tgt": Ident;
}>;
  "equations": Array<{
  "lhs": Path;
  "rhs": Path;
  "rationale"?: string;
  "status"?: "proposed" | "accepted";
}>;
};
  "instance": {
  "rows": Record<string, Array<Ident>>;
  "tables": Record<string, Record<string, (null) | (Ident)>>;
};
};
