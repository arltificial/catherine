// Verus lane parity driver (openspec change `add-verus-verification-lane`).
//
// Purpose: run the equation-family walk on real .ct.json documents and
// print the model's findings as normalized lines, so the lane script
// can diff them against the kernel's `catherine check --json` findings
// on the same documents. This is the runtime half of the parity gate:
// the spec-level half lives in crates/catherine-verus/model/lib.rs,
// where Verus proves the verified walk equals its functional spec; this
// driver carries the same exec logic as plain Rust over parsed fixture
// documents (design Decision 4 — the binary consumes the same JSON the
// kernel does).
//
// Responsibilities: (1) project the .ct.json document into the model's
// typed values exactly the way the kernel compiles them
// (crates/catherine-engine/src/engine/compile.rs: objects and arrows in
// declaration order, dense row ids per object's `instance.rows` list,
// equation paths resolved arrow-name → nid with undeclared names as
// dead `None` steps, equation domain = the lhs lead arrow's source
// object); (2) run the walk copied from model/lib.rs (`engine_compose`,
// the `equation_findings` row loop, `push_equation_finding`) minus its
// spec/proof annotations; (3) print one line per emitted finding, in
// walk order — which the model's Lemma A proves is the sorted key order
// the kernel envelope reports; (4) refuse out-of-fragment documents
// with exit 3 (per-object row counts above ROWS_CAP=8 exceed the
// verified model's preconditions — a boundary, never a silent drop).
//
// Trust boundary: the projection and the walk copy here are TRUSTED
// plain Rust, outside any proof. Their drift is exactly what the lane's
// kernel diff exists to catch: if this copy or the projection deviates
// from the kernel on any corpus document, the diff fails loudly.
//
// Exit codes: 0 findings printed; 2 usage; 3 the document is outside
// the verified fragment or malformed (named on stderr).

use serde_json::Value;
use std::collections::BTreeMap;
use std::process::ExitCode;

/// The verified model's row cap (model/lib.rs `Bounded<RowId, ROWS_CAP>`):
/// every object's row count must fit, or the document is outside the
/// verified fragment.
const ROWS_CAP: usize = 8;

/// One compiled equation, mirroring `CompiledEquation`
/// (engine/compile.rs): lhs/rhs paths as nid steps (`None` = a dead
/// step, from an undeclared arrow name), domain = lhs lead arrow's
/// source object nid.
struct Equation {
    lhs: Vec<Option<usize>>,
    rhs: Vec<Option<usize>>,
    domain: Option<usize>,
    status: String,
    rationale: String,
}

/// The typed values the walk reads, mirroring `EngineTyped` /
/// `EngineSchema`: dense row counts per object nid, one cell vector per
/// arrow nid, equations in declaration order.
struct Model {
    row_counts: Vec<Option<usize>>,
    tables: Vec<Vec<Option<usize>>>,
    equations: Vec<Equation>,
}

struct DocError(String);

fn doc_err(msg: impl Into<String>) -> DocError {
    DocError(msg.into())
}

/// Project the parsed .ct.json document into the model's typed values,
/// following kernel compile.rs exactly.
fn project(doc: &Value) -> Result<Model, DocError> {
    let schema = doc
        .get("schema")
        .ok_or_else(|| doc_err("missing `schema`"))?;
    let instance = doc
        .get("instance")
        .ok_or_else(|| doc_err("missing `instance`"))?;

    // Objects and arrows: the kernel interns names through BTreeMaps, so
    // dense nids are NAME order, not document order (compile.rs: "id
    // order is name order"). Equations arrive already sorted by the
    // parser (canonical order: file order is presentation); the driver
    // reproduces that canonical order before the walk.
    let objects_json: Vec<&str> = schema
        .get("objects")
        .and_then(Value::as_array)
        .ok_or_else(|| doc_err("schema.objects must be an array"))?
        .iter()
        .map(|v| v.as_str().ok_or_else(|| doc_err("object names must be strings")))
        .collect::<Result<_, _>>()?;
    let objects: Vec<String> = {
        let mut sorted = objects_json.clone();
        sorted.sort();
        sorted.into_iter().map(String::from).collect()
    };
    let arrows_in: Vec<(&str, &str, &str)> = {
        let mut raw: Vec<(&str, &str, &str)> = schema
            .get("arrows")
            .and_then(Value::as_array)
            .ok_or_else(|| doc_err("schema.arrows must be an array"))?
            .iter()
            .map(|a| {
                let name = a.get("name").and_then(Value::as_str).ok_or_else(|| doc_err("arrow missing name"))?;
                let src = a.get("src").and_then(Value::as_str).ok_or_else(|| doc_err("arrow missing src"))?;
                let tgt = a.get("tgt").and_then(Value::as_str).ok_or_else(|| doc_err("arrow missing tgt"))?;
                Ok((name, src, tgt))
            })
            .collect::<Result<_, _>>()?;
        raw.sort_by_key(|(name, _, _)| *name);
        raw
    };

    // Dense row ids: each object's `instance.rows` list, in order; keep
    // the name→id map for table-cell resolution.
    let rows_obj = instance
        .get("rows")
        .and_then(Value::as_object)
        .ok_or_else(|| doc_err("instance.rows must be an object"))?;
    let mut row_name_ids: BTreeMap<&str, BTreeMap<&str, usize>> = BTreeMap::new();
    let mut row_counts: Vec<Option<usize>> = Vec::with_capacity(objects.len());
    for object in &objects {
        let names = rows_obj
            .get(object.as_str())
            .and_then(Value::as_array)
            .ok_or_else(|| doc_err(format!("instance.rows missing declared object `{object}`")))?;
        if names.len() > ROWS_CAP {
            return Err(doc_err(format!(
                "object `{object}` has {} rows; the verified fragment caps objects at {ROWS_CAP} rows",
                names.len()
            )));
        }
        let mut ids = BTreeMap::new();
        for (i, name) in names.iter().enumerate() {
            let name = name
                .as_str()
                .ok_or_else(|| doc_err("row names must be strings"))?;
            ids.insert(name, i);
        }
        row_name_ids.insert(object.as_str(), ids);
        row_counts.push(Some(names.len()));
    }

    // Arrow nids in declaration order; undeclared names compile to dead
    // steps (compile_path: `arrows.get(name).copied()`).
    let arrow_nid: BTreeMap<&str, usize> = arrows_in
        .iter()
        .enumerate()
        .map(|(i, (name, _, _))| (*name, i))
        .collect();
    let object_nid: BTreeMap<&str, usize> = objects
        .iter()
        .enumerate()
        .map(|(i, name)| (name.as_str(), i))
        .collect();

    // One cell vector per arrow nid: cells[src row id] = Some(tgt row id)
    // where the parsed table defines the pair, None where undefined
    // (compile_instance + Bounded::from_option_vec; the cell vector's
    // length is the source object's row count).
    let tables_json = instance
        .get("tables")
        .and_then(Value::as_object)
        .ok_or_else(|| doc_err("instance.tables must be an object"))?;
    let mut tables: Vec<Vec<Option<usize>>> = Vec::with_capacity(arrows_in.len());
    for (name, src, tgt) in &arrows_in {
        let tgt_ids = row_name_ids
            .get(tgt)
            .ok_or_else(|| doc_err(format!("arrow `{name}` targets undeclared object `{tgt}`")))?;
        let src_ids = row_name_ids
            .get(src)
            .ok_or_else(|| doc_err(format!("arrow `{name}` sources undeclared object `{src}`")))?;
        let mut cells = vec![None; src_ids.len()];
        if let Some(cells_json) = tables_json.get(*name).and_then(Value::as_object) {
            for (src_row, tgt_row) in cells_json {
                let s = src_ids
                    .get(src_row.as_str())
                    .copied()
                    .ok_or_else(|| doc_err(format!(
                        "arrow `{name}` maps undeclared source row `{src_row}`"
                    )))?;
                let t = tgt_row
                    .as_str()
                    .and_then(|n| tgt_ids.get(n).copied())
                    .ok_or_else(|| doc_err(format!(
                        "arrow `{name}` maps `{src_row}` to undeclared target row"
                    )))?;
                cells[s] = Some(t);
            }
        }
        tables.push(cells);
    }

    // Equations in declaration order; paths resolve arrow names, the
    // domain is the lhs lead arrow's source object.
    let mut equations = Vec::new();
    for eq in schema
        .get("equations")
        .and_then(Value::as_array)
        .ok_or_else(|| doc_err("schema.equations must be an array"))?
    {
        let compile = |side: &Value| -> Result<Vec<Option<usize>>, DocError> {
            let names = side
                .as_array()
                .ok_or_else(|| doc_err("equation paths must be arrays of arrow names"))?;
            if names.is_empty() {
                return Err(doc_err("equation path must be non-empty"));
            }
            Ok(names
                .iter()
                .map(|n| {
                    let name = n.as_str().ok_or_else(|| doc_err("equation path entries must be strings"))?;
                    Ok(arrow_nid.get(name).copied())
                })
                .collect::<Result<Vec<_>, DocError>>()?)
        };
        let lhs = compile(eq.get("lhs").ok_or_else(|| doc_err("equation missing lhs"))?)?;
        let rhs = compile(eq.get("rhs").ok_or_else(|| doc_err("equation missing rhs"))?)?;
        let status = eq
            .get("status")
            .and_then(Value::as_str)
            .ok_or_else(|| doc_err("equation missing status"))?
            .to_string();
        let rationale = eq
            .get("rationale")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_string();
        equations.push(Equation { lhs, rhs, domain: None, status, rationale });
    }

    // Canonical equation order (parse/schema.rs: sorted by
    // (lhs, rhs, rationale, status); file order is presentation).
    equations.sort_by(|a, b| {
        (&a.lhs, &a.rhs, &a.rationale, a.status.as_str()).cmp(&(
            &b.lhs,
            &b.rhs,
            &b.rationale,
            b.status.as_str(),
        ))
    });
    for equation in &mut equations {
        // The domain follows the equation (the lhs lead arrow's source
        // object), recomputed here so it survives the reorder.
        equation.domain = equation
            .lhs
            .first()
            .copied()
            .flatten()
            .map(|lead| object_nid.get(arrows_in[lead].1).copied())
            .flatten();
    }

    Ok(Model { row_counts, tables, equations })
}

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: verus-parity <model.ct.json>");
        return ExitCode::from(2);
    };
    if args.next().is_some() {
        eprintln!("usage: verus-parity <model.ct.json>");
        return ExitCode::from(2);
    }
    let raw = match std::fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(e) => {
            eprintln!("verus-parity: cannot read {path}: {e}");
            return ExitCode::from(3);
        }
    };
    let doc: Value = match serde_json::from_str(&raw) {
        Ok(doc) => doc,
        Err(e) => {
            eprintln!("verus-parity: {path} is not valid JSON: {e}");
            return ExitCode::from(3);
        }
    };
    let model = match project(&doc) {
        Ok(model) => model,
        Err(DocError(msg)) => {
            eprintln!("verus-parity: {path} is outside the verified fragment: {msg}");
            return ExitCode::from(3);
        }
    };

    // The walk, copied from model/lib.rs (engine_compose +
    // equation_findings + push_equation_finding, minus spec/proof).
    for (index, equation) in model.equations.iter().enumerate() {
        let Some(domain) = equation.domain.and_then(|d| model.row_counts[d]) else {
            continue;
        };
        for row in 0..domain {
            let Some(lhs) = engine_compose(&model, &equation.lhs, row) else {
                continue;
            };
            let Some(rhs) = engine_compose(&model, &equation.rhs, row) else {
                continue;
            };
            if lhs != rhs {
                push_equation_finding(index, row, lhs, rhs, &equation.status);
            }
        }
    }
    ExitCode::SUCCESS
}

/// `engine_compose` (model/lib.rs, copied verbatim minus annotations):
/// fold a path through the arrow tables; any dead step or undefined
/// cell fails the composition.
fn engine_compose(model: &Model, path: &[Option<usize>], start: usize) -> Option<usize> {
    let mut current = start;
    for step in path {
        let step = (*step)?;
        let cell = model.tables.get(step)?.get(current)?;
        current = (*cell)?;
    }
    Some(current)
}

/// `push_equation_finding` (model/lib.rs): the walk emits one line per
/// disagreement; `label_for` (engine/findings.rs) maps the status.
fn push_equation_finding(index: usize, row: usize, lhs: usize, rhs: usize, status: &str) {
    let label = match status {
        "accepted" => "executed",
        "proposed" => "hypothesized",
        other => {
            eprintln!("verus-parity: unknown equation status `{other}`");
            std::process::exit(3);
        }
    };
    println!("EQ {index} {row} {lhs} {rhs} {label}");
}
