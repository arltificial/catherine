//! Purpose: the row for capabilities the CLI has no surface for yet — the
//! joint multi-document check.
//! Responsibilities: observe that each document checks clean alone while a
//! joint invocation is refused at arity; once composition lands, require the
//! seam violation class the composition spec defines.
//! Rationale: grouped out of `cases.rs` at the file-line gate; the cleared
//! arm asserts the spec's violation shape instead of treating any non-arity
//! exit as a fix.

use super::probe::{self, envelope, findings, warnings};
use super::Observation;
use crate::common;
use serde_json::Value;

pub(super) fn composition_absence() -> Observation {
    for name in [
        "composition_billing.ct.json",
        "composition_shipping.ct.json",
        "composition_seam_claim.ct.json",
    ] {
        let path = probe::fixture(name);
        let out = common::run(&["check", &path, "--json"]);
        let value = envelope(&out, name);
        assert_eq!(
            common::code(&out),
            0,
            "{name}: each document checks clean alone: {}",
            common::stderr(&out)
        );
        assert!(
            findings(&value).is_empty(),
            "{name}: no single-document findings"
        );
        assert!(
            warnings(&value).is_empty(),
            "{name}: no single-document warnings"
        );
    }

    let billing = probe::fixture("composition_billing.ct.json");
    let shipping = probe::fixture("composition_shipping.ct.json");
    let seam = probe::fixture("composition_seam_claim.ct.json");
    let joint = common::run(&["check", &billing, &shipping, &seam, "--json"]);
    match common::code(&joint) {
        2 => {
            let message = common::stderr(&joint);
            assert!(
                message.contains("unexpected argument"),
                "the joint check is refused at arity: {message}"
            );
            Observation::Limitation
        }
        3 => joint_fix(&envelope(&joint, "joint check")),
        other => panic!(
            "joint check: unexpected exit {other}: {}",
            common::stderr(&joint)
        ),
    }
}

/// The fixed signature per the `composition` spec delta: the seam equation is
/// violated across documents, the finding is `violation`/`executed` with the
/// equation witness, and the attribution names the declaring document.
fn joint_fix(value: &Value) -> Observation {
    let found = findings(value);
    assert!(!found.is_empty(), "the glued seam equation fires a finding");
    for finding in found {
        assert_eq!(finding["kind"], "violation", "seam finding kind");
        assert_eq!(finding["label"], "executed", "seam finding label");
        let witness = &finding["witness"];
        assert!(witness["equation"].is_number(), "seam witness equation");
        assert!(witness["row"].is_number(), "seam witness row");
    }
    let text = found
        .iter()
        .map(Value::to_string)
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        text.contains("composition_seam_claim"),
        "the finding names the declaring document: {text}"
    );
    Observation::Cleared
}