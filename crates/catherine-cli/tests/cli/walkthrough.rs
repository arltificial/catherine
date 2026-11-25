//! The DESIGN §5 agent-loop walkthrough (task 7, criterion 10): phase 1
//! replays the violating document's §5 violation block byte-for-byte and
//! its exit code; phase 2 replays the agent's proposed fix and its clean
//! re-check. Split out of `cli.rs` to keep both files inside the
//! file-size gate; the transcript pins stay with the acceptance suite.

use super::{BOWLING, BOWLING_FIXED};
use crate::common::{code, stderr, stdout, try_run};
use std::error::Error;

#[test]
fn design_s5_walkthrough_violating_then_fixed() -> Result<(), Box<dyn Error>> {
    // Phase 1: the violating document reproduces the §5 violation block
    // byte-for-byte and exits 3. The transcript's `underconstraint`
    // line is P4-gated (no producer exists; findings.rs emits nothing
    // for it) — its absence here is the honest state, never fabricated.
    let violating = try_run(&["check", BOWLING])?;
    assert_eq!(code(&violating), 3, "violation exit code");
    assert_eq!(stderr(&violating), "", "findings go to stdout");
    // Byte-for-byte: the `\` continuations strip leading indentation, so
    // `\x20 ` supplies exactly the transcript's two leading spaces on the
    // witness line. (A plain-literal twin of this block is pinned at the
    // unit level: `render::tests::violation_renders_the_s5_block`.)
    assert_eq!(
        stdout(&violating),
        "violation: eq-0 staff_unit∘assigned_staff ≠ equipment_unit∘assigned_item\n\
         \x20 at row Assignment#3: lhs=Unit#1 rhs=Unit#2\n",
        "the §5 violation block, byte-for-byte"
    );

    // Phase 2: the loop — the agent's proposed fix lands, the re-check
    // is clean: exit 0, zero findings.
    let fixed = try_run(&["check", BOWLING_FIXED])?;
    assert_eq!(code(&fixed), 0, "clean re-check exit code");
    assert_eq!(stdout(&fixed), "", "zero findings print nothing");
    assert_eq!(stderr(&fixed), "", "no errors");
    Ok(())
}
