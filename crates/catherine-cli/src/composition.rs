//! Purpose: the CLI's composition surface support (openspec change
//! `add-model-composition`) — loading several documents, gluing them, and
//! attaching per-finding source attribution.
//!
//! Responsibilities: [`load_glued`] reads and parses every input path,
//! maps a kernel composition failure onto the CLI's labeled error, and
//! returns the [`Glued`] model; [`sourced`] pairs each finding with the
//! documents that declared its violated constraint and referenced paths;
//! [`render_sourced`] renders those findings with a source line; the
//! [`merge`] submodule merges two models along their shared sub-schema,
//! downgrading shared-entity identity conflicts to UNKNOWN-labeled merge
//! cells whose witnesses name both conflicting sides (anti-goal: never
//! silently place an unplaceable cell — the UNKNOWN witness is the
//! feature, and clean merges carry no spurious labels).
//!
//! Rationale: composition is one execution path over the kernel's checked
//! pushout (design Decision 1) — the CLI never re-implements the merge.
//! Source attribution is additive presentation data built from the glue's
//! side tables — equation, key, coproduct, and pullback declarations keep
//! their own tables, all aligned with their schema sections — so the
//! single-document envelope (which never runs this module) stays
//! byte-identical.

use crate::dispatch::Ctx;
use crate::render::{self, LabeledError, SourcedFinding};
use catherine_core::checks::ExecutionScope;
use catherine_core::composition::{glue, parse_document, CompositionError, Document, Glued};
use catherine_core::findings::{DomainQuestion, Finding, FindingKind, ViolationWitness};
use catherine_core::model::Schema;
use std::io::Write;

pub mod merge;

/// Read, parse, and glue every input path once. Read failures are
/// `read_error`; document failures keep the kernel's code and position;
/// glue failures keep the composition code.
pub(crate) fn load_glued(model_paths: &[String]) -> Result<Glued, LabeledError> {
    let documents = model_paths
        .iter()
        .map(|path| load_document(path))
        .collect::<Result<Vec<_>, _>>()?;
    glue(&documents).map_err(glue_error)
}

/// Read one document and defer its equation resolution to the glue.
fn load_document(path: &str) -> Result<Document, LabeledError> {
    let text = std::fs::read_to_string(path).map_err(|error| LabeledError {
        code: "read_error".to_string(),
        line: None,
        column: None,
        message: format!("cannot read model document `{path}`: {error}"),
        hint: "pass a readable .ct.json model document path",
    })?;
    parse_document(path, &text).map_err(|error| LabeledError::from_parse(&error))
}

/// Map a kernel composition error onto the CLI's labeled error: the
/// stable code, the offending node's position, and a code-specific
/// remediation hint. Conflict messages also name both documents (the
/// dangling-path message keeps the parse-time shape verbatim).
fn glue_error(error: CompositionError) -> LabeledError {
    let (line, column) = match error.position() {
        Some((line, column)) => (Some(line), Some(column)),
        None => (None, None),
    };
    let message = if names_documents(error.code()) {
        format!(
            "{} (documents: {})",
            error.message(),
            error.documents().join(", ")
        )
    } else {
        error.message().to_string()
    };
    LabeledError {
        code: error.code().to_string(),
        line,
        column,
        message,
        hint: glue_hint(error.code()),
    }
}

/// Whether the conflict code's message should name the documents too.
fn names_documents(code: &str) -> bool {
    matches!(code, "schema_conflict" | "row_conflict")
}

/// The remediation hint for the composition failures.
fn glue_hint(code: &str) -> &'static str {
    match code {
        "schema_conflict" => {
            "a shared arrow must have identical endpoints in every input — align the declarations or rename one"
        }
        "row_conflict" => {
            "a shared object's row set must agree across the documents, and a commonly-declared cell must carry one value — reconcile the declarations"
        }
        "unknown_arrow" => "declare the arrow in the declaring input's schema.arrows, or fix the path spelling",
        _ => "the equation the error names must be well-typed with a valid status in its declaring document",
    }
}

/// Pair every finding with its source attribution from the glue's side
/// tables. Ordering is the finding order; each source list keeps input
/// declaration order and is deduplicated.
pub(crate) fn sourced(findings: Vec<Finding>, glued: &Glued) -> Vec<SourcedFinding> {
    findings
        .into_iter()
        .map(|finding| {
            let source = sources_for(&finding, glued);
            SourcedFinding { finding, source }
        })
        .collect()
}

/// The documents behind one finding's violated constraint and its
/// referenced paths.
fn sources_for(finding: &Finding, glued: &Glued) -> Option<Vec<String>> {
    let sources = match &finding.kind {
        FindingKind::Violation(witness) => violation_sources(witness, glued),
        FindingKind::Dangling(witness) => glued
            .arrow_sources
            .get(&witness.arrow)
            .map(|sources| dedup(sources)),
        FindingKind::Underconstraint(witness) => match &witness.case {
            DomainQuestion::ParallelPaths { lhs, rhs } => Some(path_docs(lhs, rhs, glued)),
            DomainQuestion::UncoveredInputs { .. } => None,
        },
        FindingKind::Redundant(witness) => {
            Some(path_docs(&witness.derivable, &witness.reducing, glued))
        }
        FindingKind::Contradiction(witness) => Some(contradiction_docs(&witness.scope, glued)),
        // The granularity lint questions the whole glued model's shape
        // (ticket `catherine-kw9.3`), not one declaring document.
        FindingKind::Granularity(_) => None,
    };
    sources.filter(|documents| !documents.is_empty())
}

/// The violation witness's sources, one arm per witness shape; the
/// lookup is by the violated declaration's canonical index.
fn violation_sources(witness: &ViolationWitness, glued: &Glued) -> Option<Vec<String>> {
    match witness {
        ViolationWitness::Equation(witness) => glued
            .equation_sources
            .get(witness.equation)
            .map(equation_docs),
        ViolationWitness::Key(witness) => glued.key_sources.get(witness.key).map(key_docs),
        ViolationWitness::Naturality(witness) => glued
            .arrow_sources
            .get(&witness.arrow)
            .map(|sources| dedup(sources)),
        ViolationWitness::Coproduct(witness) => glued
            .coproduct_sources
            .get(witness.coproduct)
            .map(coproduct_docs),
        ViolationWitness::Pullback(witness) => glued
            .pullback_sources
            .get(witness.pullback())
            .map(pullback_docs),
        // The migration-naturality witness has no glue side table yet; its
        // attribution belongs to the migration-naturality surface, so no
        // source line is fabricated here.
        ViolationWitness::MigrationNaturality(_) => None,
    }
}

/// An equation's declaring document plus the documents behind each path.
fn equation_docs(source: &catherine_core::composition::EquationSource) -> Vec<String> {
    let combined: Vec<&String> = std::iter::once(&source.declaring)
        .chain(source.lhs.iter())
        .chain(source.rhs.iter())
        .collect();
    dedup_refs(&combined)
}

/// A key's declaring document plus the documents behind its arrows.
fn key_docs(source: &catherine_core::composition::KeySource) -> Vec<String> {
    declaration_docs(&source.declaring, &source.arrows)
}

/// A coproduct's declaring document plus the documents behind its
/// injection arrows.
fn coproduct_docs(source: &catherine_core::composition::CoproductSource) -> Vec<String> {
    declaration_docs(&source.declaring, &source.arrows)
}

/// A pullback's declaring document plus the documents behind its legs
/// and cospan arrows.
fn pullback_docs(source: &catherine_core::composition::PullbackSource) -> Vec<String> {
    declaration_docs(&source.declaring, &source.arrows)
}

/// A declaration's document plus the documents behind its arrows —
/// the declaring/arrows shape the key, coproduct, and pullback side
/// tables share.
fn declaration_docs(declaring: &String, arrows: &[String]) -> Vec<String> {
    let combined: Vec<&String> = std::iter::once(declaring).chain(arrows.iter()).collect();
    dedup_refs(&combined)
}

/// The documents behind two paths' arrows, lhs then rhs.
fn path_docs(
    lhs: &catherine_core::model::Path,
    rhs: &catherine_core::model::Path,
    glued: &Glued,
) -> Vec<String> {
    let sources: Vec<&String> = lhs
        .0
        .iter()
        .chain(rhs.0.iter())
        .filter_map(|name| glued.arrow_sources.get(name))
        .flatten()
        .collect();
    dedup_refs(&sources)
}

/// The documents behind every equation in a contradiction scope.
fn contradiction_docs(scope: &[usize], glued: &Glued) -> Vec<String> {
    let per_equation: Vec<Vec<String>> = scope
        .iter()
        .filter_map(|index| glued.equation_sources.get(*index))
        .map(equation_docs)
        .collect();
    let sources: Vec<&String> = per_equation.iter().flatten().collect();
    dedup_refs(&sources)
}

/// A deduplicated copy of a document list, preserving first-occurrence
/// order.
fn dedup(source: &[String]) -> Vec<String> {
    source
        .iter()
        .enumerate()
        .filter(|(index, document)| !source[..*index].contains(*document))
        .map(|(_, document)| document.clone())
        .collect()
}

/// [`dedup`] over borrowed entries collected from several side tables.
fn dedup_refs(source: &[&String]) -> Vec<String> {
    source
        .iter()
        .enumerate()
        .filter(|(index, document)| !source[..*index].contains(document))
        .map(|(_, document)| (*document).clone())
        .collect()
}

/// Render the attributed findings in the human format, adding the source
/// line after each finding's witness lines.
pub(crate) fn render_sourced(findings: &[SourcedFinding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .flat_map(|sourced| {
            let lines = render::render_findings(std::slice::from_ref(&sourced.finding), schema);
            let source_line = sourced
                .source
                .as_ref()
                .map(|source| format!("  source: {}", source.join(", ")));
            lines.into_iter().chain(source_line)
        })
        .collect()
}

/// Push the disjoint-union warning onto the run's warnings and, in human
/// mode, print it to stderr (catherine-uh0: run-level warnings reach stderr
/// in every non-JSON mode, the JSON envelope carries them otherwise). Only
/// this run-level warning is printed here: config warnings are already on
/// stderr in human mode (the settings resolver owns them).
pub(crate) fn warn_empty_interface(glued: &Glued, ctx: &mut Ctx<'_, impl Write, impl Write>) {
    if !glued.empty_interface {
        return;
    }
    let warning = render::empty_composition_warning(&glued.documents);
    crate::runners::print_run_warnings(ctx.json, std::slice::from_ref(&warning), ctx.stderr);
    ctx.warnings.push(warning);
}

/// Warn once per document path the invocation passed more than once, in
/// human mode printing each to stderr (the JSON envelope carries them).
/// Detection is by canonicalized path (byte-identical when the filesystem
/// cannot resolve it); the repetition itself is preserved — the documented
/// multiset rule still counts every occurrence, so nothing is deduplicated.
pub(crate) fn warn_duplicate_inputs<S: AsRef<str>>(
    model_paths: &[S],
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) {
    for path in duplicate_paths(model_paths) {
        let warning = render::duplicate_document_warning(&path);
        crate::runners::print_run_warnings(ctx.json, std::slice::from_ref(&warning), ctx.stderr);
        ctx.warnings.push(warning);
    }
}

/// The repeated input paths, deduplicated and in first-repetition order.
/// The canonicalized key decides identity; the reported spelling is the
/// repeated occurrence as written, so a warning names what the user typed.
fn duplicate_paths<S: AsRef<str>>(paths: &[S]) -> Vec<String> {
    let keys: Vec<String> = paths.iter().map(|path| input_key(path.as_ref())).collect();
    let duplicates: Vec<String> = keys
        .iter()
        .enumerate()
        .filter(|(index, key)| keys[..*index].contains(*key))
        .map(|(index, _)| paths[index].as_ref().to_string())
        .collect();
    dedup(&duplicates)
}

/// The identity used for duplicate detection: the canonicalized path when
/// the filesystem can resolve it, else the path string as written.
fn input_key(path: &str) -> String {
    std::fs::canonicalize(path)
        .map(|canonical| canonical.to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_string())
}

/// Emit findings with glue source attribution: the sourced `--json`
/// envelope, or the human lines (quiet suppresses the human output only).
/// Returns the exit code from the findings.
pub(crate) fn emit_findings(
    findings: &[Finding],
    scope: Option<ExecutionScope>,
    glued: &Glued,
    ctx: &mut Ctx<'_, impl Write, impl Write>,
) -> i32 {
    let attributed = sourced(findings.to_vec(), glued);
    if ctx.json {
        let envelope = render::sourced_findings_envelope(attributed, scope, ctx.take_warnings());
        crate::run::write_envelope(&envelope, ctx.stdout, ctx.stderr, "findings");
    } else if !ctx.quiet {
        for line in render_sourced(&attributed, &glued.schema) {
            let _ = writeln!(ctx.stdout, "{line}");
        }
    }
    crate::run::exit_code(findings)
}
