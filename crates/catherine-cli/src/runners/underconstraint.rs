//! Purpose: the joint (composition) underconstraint run — glue several
//! documents, raise parallel-path questions over the merged schema, and
//! attribute each finding to the documents whose union created it.
//!
//! Responsibilities: the underconstraint scope caps and the multi-document
//! execution path; the single-document path stays in `runners.rs`.
//!
//! Rationale: the seam-discovery signal is the reason the composition
//! surface exists; a question no single input raises must name the
//! documents that created it, so the same glue and source-attribution
//! machinery serves here as `check` (design Decision 1's one path).

use crate::composition;
use crate::run::emit_error;
use catherine_analysis::bounds::ScopeCaps;
use genesis::envelope::Warning;
use std::io::Write;

/// The underconstraint scope caps: the caller's path cap over the shared
/// defaults.
pub(super) fn path_caps(max_path_length: usize) -> ScopeCaps {
    ScopeCaps {
        max_path_length,
        ..ScopeCaps::default()
    }
}

/// The joint underconstraint run: glue the inputs, raise the parallel-path
/// questions over the merged schema, and render them with source
/// attribution.
#[allow(clippy::too_many_arguments)]
pub(super) fn composition(
    model_paths: &[String],
    max_path_length: usize,
    json: bool,
    quiet: bool,
    mut warnings: Vec<Warning>,
    stdout: &mut impl Write,
    stderr: &mut impl Write,
) -> i32 {
    let glued = match composition::load_glued(model_paths) {
        Ok(glued) => glued,
        Err(labeled) => return emit_error(&labeled, json, warnings, stdout, stderr),
    };
    composition::warn_empty_interface(&glued, json, &mut warnings, stderr);
    let findings =
        catherine_analysis::parallel::underconstraint(&glued.schema, &path_caps(max_path_length));
    composition::emit_findings(&findings, &glued, json, quiet, warnings, stdout, stderr)
}
