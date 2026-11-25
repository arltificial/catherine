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
use catherine_analysis::bounds::ScopeCaps;
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
pub(super) fn composition(
    model_paths: &[String],
    max_path_length: usize,
    ctx: &mut crate::dispatch::Ctx<'_, impl Write, impl Write>,
) -> i32 {
    composition::warn_duplicate_inputs(model_paths, ctx);
    let glued = match composition::load_glued(model_paths) {
        Ok(glued) => glued,
        Err(labeled) => return ctx.emit_error(&labeled),
    };
    composition::warn_empty_interface(&glued, ctx);
    let findings =
        catherine_analysis::parallel::underconstraint(&glued.schema, &path_caps(max_path_length));
    composition::emit_findings(&findings, None, &glued, ctx)
}
