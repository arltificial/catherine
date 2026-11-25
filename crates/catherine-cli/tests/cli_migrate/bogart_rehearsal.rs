//! The bogart rehearsal fixture, end to end (migration task 8): the
//! documented ten findings — four dropped RawCapture rows, six valued
//! dropped cells — and the exit-8 contract. Split out of `cli_migrate.rs`
//! to keep both files inside the file-size gate; the fixture pins stay
//! verbatim.

use super::BOGART;
use crate::common::{code, envelope, lost, stdout, try_run};
use serde_json::Value;
use std::error::Error;

#[test]
fn bogart_rehearsal_emits_the_documented_ten_findings_and_exits_8() -> Result<(), Box<dyn Error>> {
    let out = try_run(&["migrate", BOGART, "--json"])?;
    assert_eq!(code(&out), 8, "the rehearsal drops data by design");

    let v = envelope("stdout", &stdout(&out))?;
    assert_eq!(
        v["data"]["rows"],
        serde_json::json!({"ServingDocument": 2, "ServingPage": 3}),
        "raw corpus → serving projection"
    );

    let lost = lost(&v);
    assert_eq!(lost.len(), 10, "4 dropped rows + 6 valued dropped cells");
    assert_raw_capture_rows_are_named(lost);
    assert_lost_cell_order(lost);
    assert_serving_page_rows_transfer_verbatim(&v);
    Ok(())
}

/// The four dropped RawCapture rows, named in dense order.
fn assert_raw_capture_rows_are_named(lost: &[Value]) {
    let lost_rows: Vec<_> = lost.iter().filter(|f| f["kind"] == "lost_row").collect();
    assert_eq!(lost_rows.len(), 4, "every RawCapture row is named");
    for (i, finding) in lost_rows.iter().enumerate() {
        assert_eq!(finding["witness"]["object"], "RawCapture");
        assert_eq!(finding["witness"]["row"], i, "rows named in dense order");
    }
}

/// Sorted order: captured_page (RawCapture rows 0–3), then prev_page
/// (RawPage rows 1–2 — `_0` held no value, so it is not a lost cell).
fn assert_lost_cell_order(lost: &[Value]) {
    assert_eq!(lost[4]["kind"], "lost_cell");
    assert_eq!(lost[4]["witness"]["arrow"], "captured_page");
    assert_eq!(lost[4]["witness"]["row"], 0);
    assert_eq!(lost[7]["witness"]["arrow"], "captured_page");
    assert_eq!(lost[7]["witness"]["row"], 3);
    assert_eq!(lost[8]["witness"]["arrow"], "prev_page");
    assert_eq!(lost[8]["witness"]["row"], 1);
    assert_eq!(lost[9]["witness"]["arrow"], "prev_page");
    assert_eq!(lost[9]["witness"]["row"], 2);
}

/// Kept RawPage rows transfer verbatim; dropped objects are absent from
/// the correspondence — their loss is in `lost`.
fn assert_serving_page_rows_transfer_verbatim(v: &Value) {
    assert_eq!(
        v["data"]["row_map"]["ServingPage"],
        serde_json::json!({"0": 0, "1": 1, "2": 2}),
        "kept RawPage rows transfer verbatim"
    );
    assert!(
        v["data"]["row_map"].get("RawCapture").is_none(),
        "dropped objects are absent from the correspondence — their loss is in `lost`"
    );
}
