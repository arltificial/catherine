// GENERATED from crates/catherine-core/schema/ct-schema-v1.json
// by scripts/generate-types.mjs (openspec change `add-wasm-web-view`,
// task 5) — do not hand-edit; rerun `npm run generate:types`.

export type Ident = string;

/**
 * The check walk's execution scope (openspec change `add-execution-scope`,
 * catherine-kpn.5) — what the wasm boundary forwards as `data.scope` on the
 * check/findings report. Hand-added mirror of the kernel payload: the
 * generator derives only the input-document surface, so this block is
 * maintained beside the generated output, not by scripts/generate-types.mjs.
 * Present on a report only when checks actually executed (the field is
 * omitted, never null, for scope-free payloads).
 */
export type ExecutionScope = {
  "constraints_checked": number;
  "rows_evaluated": number;
  "equations_over_zero_rows": number;
};

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

export type Coproduct = {
  "object": Ident;
  "arrows": Array<Ident>;
  "status"?: "proposed" | "accepted";
};

export type Pullback = {
  "apex": Ident;
  "legs": Array<Ident>;
  "cospan": Array<Ident>;
  "status"?: "proposed" | "accepted";
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
  "coproducts"?: Array<Coproduct>;
  "pullbacks"?: Array<Pullback>;
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
  "coproducts"?: Array<Coproduct>;
  "pullbacks"?: Array<Pullback>;
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
  "instance_map"?: {
  "maps": Array<{
  "object": Ident;
  "rows": Record<string, Ident>;
}>;
};
};
