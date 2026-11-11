//! Purpose: the deduplicated genesis-vibes envelope construction — one
//! path per envelope shape for every subcommand.
//!
//! Responsibilities: build the findings envelope (the kernel's
//! `Vec<Finding>` under `data.findings`, no reshaping), the error
//! envelope (the kernel's labeled error data under the envelope's
//! `ErrorResult` form), and the config-loading warning.
//!
//! Rationale: one envelope shape — the genesis `Check`/`List` kinds
//! over the same envelope, no second shape; Invariant 3.2.5 keeps
//! remediation non-empty by construction.

use super::errors::LabeledError;
use super::CLI_VERSION;
use catherine_core::findings::Finding;
use genesis::envelope::{Envelope, EnvelopeKind, ErrorResult, RemediationEntry, Warning};
use serde::Serialize;

/// The findings payload: P2's serialization verbatim — the kernel's
/// `Vec<Finding>` under `data.findings`, no reshaping.
#[derive(Serialize)]
pub struct FindingsData<'a> {
    pub findings: &'a [Finding],
}

/// The one findings-envelope path both subcommands share: kind `Check`,
/// empty hints, the caller's warnings carried alongside the data.
pub fn findings_envelope(
    findings: &[Finding],
    warnings: Vec<Warning>,
) -> Envelope<FindingsData<'_>> {
    Envelope::success(
        CLI_VERSION,
        EnvelopeKind::Check,
        FindingsData { findings },
        warnings,
        Vec::new(),
    )
}

/// The one error-envelope path: the envelope's error form with the
/// kernel's labeled error data under it — stable code, message with
/// position, remediation hint (Invariant 3.2.5: remediation non-empty).
pub fn error_envelope(err: &LabeledError, warnings: Vec<Warning>) -> Envelope<ErrorResult> {
    let result = ErrorResult::new(
        &err.code,
        &err.positioned_message(),
        None,
        None,
        None,
        Vec::new(),
        vec![RemediationEntry {
            command: "catherine check <model.ct.json>".to_string(),
            description: err.hint.to_string(),
        }],
    )
    .expect("remediation is non-empty by construction");
    Envelope::error(CLI_VERSION, result, warnings)
}

/// A config-loading warning, ruled under `config`.
pub fn config_warning(message: &str) -> Warning {
    Warning {
        rule_name: "config".to_string(),
        entity_id: None,
        message: message.to_string(),
        suggested_remediation: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use catherine_core::findings::{EpistemicLabel, EquationWitness, Finding, FindingKind};
    use catherine_core::model::RowId;

    fn violation() -> Finding {
        Finding {
            label: EpistemicLabel::Executed,
            kind: FindingKind::Violation(EquationWitness {
                equation: 0,
                row: RowId(3),
                lhs: RowId(1),
                rhs: RowId(2),
            }),
        }
    }

    #[test]
    fn findings_envelope_carries_the_suite_shape() {
        let findings = vec![violation()];
        let envelope = findings_envelope(&findings, Vec::new());
        let v: serde_json::Value = serde_json::to_value(&envelope).expect("envelope serializes");
        for key in ["ok", "envelope_kind", "data", "warnings", "hints"] {
            assert!(v.get(key).is_some(), "envelope key `{key}` present");
        }
        assert_eq!(v["ok"], true);
        assert_eq!(v["data"]["findings"][0]["kind"], "violation");
        assert_eq!(v["data"]["findings"][0]["witness"]["equation"], 0);
    }

    #[test]
    fn error_envelope_carries_code_position_and_hint() {
        let err = LabeledError {
            code: "duplicate_ident".to_string(),
            line: Some(14),
            column: Some(16),
            message: "duplicate arrow ident `in_section`".to_string(),
            hint: "idents must be unique within their collection — rename one of the duplicates",
        };
        let v: serde_json::Value =
            serde_json::to_value(error_envelope(&err, Vec::new())).expect("envelope serializes");
        assert_eq!(v["ok"], false);
        assert_eq!(v["envelope_kind"], "error");
        assert_eq!(v["data"]["code"], "duplicate_ident");
        let message = v["data"]["message"].as_str().expect("message");
        assert!(
            message.contains("line 14") && message.contains("column 16"),
            "{message}"
        );
        assert!(!v["data"]["remediation"]
            .as_array()
            .expect("remediation")
            .is_empty());
    }
}
