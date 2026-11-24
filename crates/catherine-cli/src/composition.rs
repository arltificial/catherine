//! Purpose: the CLI's composition surface support (openspec change
//! `add-model-composition`) — loading several documents, gluing them, and
//! attaching per-finding source attribution.
//!
//! Responsibilities: [`load_glued`] reads and parses every input path,
//! maps a kernel composition failure onto the CLI's labeled error, and
//! returns the [`Glued`] model; [`sourced`] pairs each finding with the
//! documents that declared its violated constraint and referenced paths;
//! [`render_sourced`] renders those findings with a source line.
//!
//! Rationale: composition is one execution path over the kernel's checked
//! pushout (design Decision 1) — the CLI never re-implements the merge.
//! Source attribution is additive presentation data built from the glue's
//! side tables — equation, key, coproduct, and pullback declarations keep
//! their own tables, all aligned with their schema sections — so the
//! single-document envelope (which never runs this module) stays
//! byte-identical.

use crate::render::{self, LabeledError, SourcedFinding};
use catherine_core::checks::ExecutionScope;
use catherine_core::composition::{glue, parse_document, CompositionError, Document, Glued};
use catherine_core::findings::{DomainQuestion, Finding, FindingKind, ViolationWitness};
use catherine_core::model::Schema;
use genesis::envelope::Warning;
use std::io::Write;

/// Read, parse, and glue every input path once. Read failures are
/// `read_error`; document failures keep the kernel's code and position;
/// glue failures keep the composition code.
pub(crate) fn load_glued(model_paths: &[String]) -> Result<Glued, LabeledError> {
    let mut documents = Vec::with_capacity(model_paths.len());
    for path in model_paths {
        documents.push(load_document(path)?);
    }
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
    let mut message = error.message().to_string();
    if names_documents(error.code()) {
        message = format!("{message} (documents: {})", error.documents().join(", "));
    }
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
        FindingKind::Violation(ViolationWitness::Equation(witness)) => glued
            .equation_sources
            .get(witness.equation)
            .map(equation_docs),
        FindingKind::Violation(ViolationWitness::Key(witness)) => {
            glued.key_sources.get(witness.key).map(key_docs)
        }
        FindingKind::Violation(ViolationWitness::Naturality(witness)) => glued
            .arrow_sources
            .get(&witness.arrow)
            .map(|sources| dedup(sources)),
        FindingKind::Violation(ViolationWitness::Coproduct(witness)) => glued
            .coproduct_sources
            .get(witness.coproduct)
            .map(coproduct_docs),
        FindingKind::Violation(ViolationWitness::Pullback(witness)) => glued
            .pullback_sources
            .get(witness.pullback())
            .map(pullback_docs),
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
    };
    sources.filter(|documents| !documents.is_empty())
}

/// An equation's declaring document plus the documents behind each path.
fn equation_docs(source: &catherine_core::composition::EquationSource) -> Vec<String> {
    let mut documents = vec![source.declaring.clone()];
    extend_unique(&mut documents, &source.lhs);
    extend_unique(&mut documents, &source.rhs);
    documents
}

/// A key's declaring document plus the documents behind its arrows.
fn key_docs(source: &catherine_core::composition::KeySource) -> Vec<String> {
    let mut documents = vec![source.declaring.clone()];
    extend_unique(&mut documents, &source.arrows);
    documents
}

/// A coproduct's declaring document plus the documents behind its
/// injection arrows.
fn coproduct_docs(source: &catherine_core::composition::CoproductSource) -> Vec<String> {
    let mut documents = vec![source.declaring.clone()];
    extend_unique(&mut documents, &source.arrows);
    documents
}

/// A pullback's declaring document plus the documents behind its legs
/// and cospan arrows.
fn pullback_docs(source: &catherine_core::composition::PullbackSource) -> Vec<String> {
    let mut documents = vec![source.declaring.clone()];
    extend_unique(&mut documents, &source.arrows);
    documents
}

/// The documents behind two paths' arrows, lhs then rhs.
fn path_docs(
    lhs: &catherine_core::model::Path,
    rhs: &catherine_core::model::Path,
    glued: &Glued,
) -> Vec<String> {
    let mut documents = Vec::new();
    for path in [lhs, rhs] {
        for name in &path.0 {
            if let Some(sources) = glued.arrow_sources.get(name) {
                extend_unique(&mut documents, sources);
            }
        }
    }
    documents
}

/// The documents behind every equation in a contradiction scope.
fn contradiction_docs(scope: &[usize], glued: &Glued) -> Vec<String> {
    let mut documents = Vec::new();
    for index in scope {
        if let Some(source) = glued.equation_sources.get(*index) {
            extend_unique(&mut documents, &equation_docs(source));
        }
    }
    documents
}

/// Append the elements of `source` not already present, preserving order.
fn extend_unique(documents: &mut Vec<String>, source: &[String]) {
    for document in source {
        if !documents.contains(document) {
            documents.push(document.clone());
        }
    }
}

/// A deduplicated copy of a document list, preserving first-occurrence
/// order.
fn dedup(source: &[String]) -> Vec<String> {
    let mut documents = Vec::new();
    extend_unique(&mut documents, source);
    documents
}

/// Render the attributed findings in the human format, adding the source
/// line after each finding's witness lines.
pub(crate) fn render_sourced(findings: &[SourcedFinding], schema: &Schema) -> Vec<String> {
    findings
        .iter()
        .flat_map(|sourced| {
            let mut lines = render::render_findings(std::slice::from_ref(&sourced.finding), schema);
            if let Some(source) = &sourced.source {
                lines.push(format!("  source: {}", source.join(", ")));
            }
            lines
        })
        .collect()
}

/// Push the disjoint-union warning onto the run's warnings and, in human
/// mode, print it to stderr (catherine-uh0: run-level warnings reach stderr
/// in every non-JSON mode, the JSON envelope carries them otherwise). Only
/// this run-level warning is printed here: config warnings are already on
/// stderr in human mode (the settings resolver owns them).
pub(crate) fn warn_empty_interface(
    glued: &Glued,
    json: bool,
    warnings: &mut Vec<Warning>,
    stderr: &mut impl Write,
) {
    if !glued.empty_interface {
        return;
    }
    let warning = render::empty_composition_warning(&glued.documents);
    crate::runners::print_run_warnings(json, std::slice::from_ref(&warning), stderr);
    warnings.push(warning);
}

/// Warn once per document path the invocation passed more than once, in
/// human mode printing each to stderr (the JSON envelope carries them).
/// Detection is by canonicalized path (byte-identical when the filesystem
/// cannot resolve it); the repetition itself is preserved — the documented
/// multiset rule still counts every occurrence, so nothing is deduplicated.
pub(crate) fn warn_duplicate_inputs<S: AsRef<str>>(
    model_paths: &[S],
    json: bool,
    warnings: &mut Vec<Warning>,
    stderr: &mut impl Write,
) {
    for path in duplicate_paths(model_paths) {
        let warning = render::duplicate_document_warning(&path);
        crate::runners::print_run_warnings(json, std::slice::from_ref(&warning), stderr);
        warnings.push(warning);
    }
}

/// The repeated input paths, deduplicated and in first-repetition order.
/// The canonicalized key decides identity; the reported spelling is the
/// repeated occurrence as written, so a warning names what the user typed.
fn duplicate_paths<S: AsRef<str>>(paths: &[S]) -> Vec<String> {
    let keys: Vec<String> = paths.iter().map(|path| input_key(path.as_ref())).collect();
    let mut duplicates = Vec::new();
    for (index, key) in keys.iter().enumerate() {
        if keys[..index].contains(key) {
            duplicates.push(paths[index].as_ref().to_string());
        }
    }
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
///
/// The eight arguments are the emitter's actual arity: findings, the
/// execution scope, the glue source, the two output-mode switches, the
/// envelope warnings, and the two sinks — each independently meaningful
/// at the check-side and under-constraint runners. The sinks thread
/// through every render path here (`write_envelope` takes them the same
/// way), so bundling them would introduce a second plumbing convention
/// mid-tree for one lint count.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_findings(
    findings: &[Finding],
    scope: Option<ExecutionScope>,
    glued: &Glued,
    json: bool,
    quiet: bool,
    warnings: Vec<genesis::envelope::Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let attributed = sourced(findings.to_vec(), glued);
    if json {
        let envelope = render::sourced_findings_envelope(attributed, scope, warnings);
        crate::run::write_envelope(&envelope, stdout, stderr, "findings");
    } else if !quiet {
        for line in render_sourced(&attributed, &glued.schema) {
            let _ = writeln!(stdout, "{line}");
        }
    }
    crate::run::exit_code(findings)
}
