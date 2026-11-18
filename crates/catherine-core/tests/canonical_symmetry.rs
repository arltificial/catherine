//! Purpose: guard the canonical relabeling against the symmetric-row search
//! blow-up — six objects of four indistinguishable rows used to multiply the
//! individualization branches into a hang (catherine-2f6).
//!
//! Responsibilities: assert that the two presentations of one symmetric model
//! hash and canonicalize identically, and that the call returns at all.
//!
//! Rationale: the collapse of interchangeable branches is only acceptable if
//! it preserves rename invariance, so the test compares the canonical bytes of
//! the same structure under reversed row order rather than counting nodes. A
//! regression either hangs this test or breaks the equality.

use catherine_core::canonical::canonical_bytes;
use catherine_core::hash::model_hash;
use catherine_core::parse::parse;

/// Six objects of four rows each, no arrows: every row is indistinguishable
/// from its siblings, so a search that branches on each tied member multiplies
/// 4! across six objects.
fn symmetric_doc(reversed: bool) -> String {
    let objects: Vec<String> = (0..6).map(|i| format!("\"O{i}\"")).collect();
    let rows: Vec<String> = (0..6)
        .map(|i| {
            let names: Vec<String> = if reversed {
                (0..4).rev().map(|r| format!("\"r{r}\"")).collect()
            } else {
                (0..4).map(|r| format!("\"r{r}\"")).collect()
            };
            format!("\"O{i}\": [{}]", names.join(", "))
        })
        .collect();
    format!(
        "{{\"format_version\":\"1.0\",\"schema\":{{\"objects\":[{}],\"arrows\":[],\"equations\":[]}},\"instance\":{{\"rows\":{{{}}},\"tables\":{{}}}}}}",
        objects.join(","),
        rows.join(",")
    )
}

#[test]
fn interchangeable_rows_do_not_multiply_the_search() {
    let forward = parse(&symmetric_doc(false)).expect("forward presentation parses");
    let reversed = parse(&symmetric_doc(true)).expect("reversed presentation parses");

    assert_eq!(
        model_hash(&forward),
        model_hash(&reversed),
        "row order is presentation, not structure"
    );
    assert_eq!(
        canonical_bytes(&forward),
        canonical_bytes(&reversed),
        "dense canonical ids must not depend on the input's row order"
    );
}
