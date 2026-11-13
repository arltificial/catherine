// GENERATED from crates/catherine-core/schema/ct-schema-v1.json
// by scripts/generate-types.mjs (openspec change `add-wasm-web-view`,
// task 5) — do not hand-edit; rerun `npm run generate:types`.

export type Ident = string;

export type Path = Array<Ident>;

export type Key = {
  "object": Ident;
  "arrows": Array<Ident>;
  "status"?: "proposed" | "accepted";
};

export type Column = {
  "name": Ident;
  "type": "int" | "bool" | "string";
};

export type Rule = {
  "name": Ident;
  "predicate": Predicate;
  "action": string;
};

export type Predicate = ({
  "kind": "cmp";
  "column": Ident;
  "op": "eq" | "ne" | "lt" | "le" | "gt" | "ge";
  "value": unknown;
}) | ({
  "kind": "mod";
  "column": Ident;
  "modulus": unknown;
  "equals": unknown;
}) | ({
  "kind": "and";
  "args": Array<Predicate>;
}) | ({
  "kind": "or";
  "args": Array<Predicate>;
}) | ({
  "kind": "not";
  "arg": Predicate;
});

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
  "keys"?: Array<Key>;
};
  "instance": {
  "rows": Record<string, Array<Ident>>;
  "tables": Record<string, Record<string, (null) | (Ident)>>;
};
  "schema_map"?: {
  "target_schema": {
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
  "keys"?: Array<Key>;
};
  "objects": Record<string, (null) | (Ident)>;
  "arrows": Record<string, (null) | (Array<Ident>)>;
};
  "rule_presentation"?: {
  "domain": Ident;
  "columns": Array<Column>;
  "rules": Array<Rule>;
  "policy"?: "PriorityOrder" | "FirstMatch" | "DeclaredDisjoint";
};
};
