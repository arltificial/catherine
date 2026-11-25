//! The proposed-status coproduct: coverage failure is a question, not a
//! rejection — the same witness, labeled `hypothesized`.

/// A proposed coproduct's coverage failure is a question: the same
/// witness, labeled `hypothesized`.
#[test]
fn proposed_coproduct_is_hypothesized() {
    use super::*;

    let findings = findings((
        PROPOSED,
        r#""C": ["c1", "c2", "c3"], "V1": ["v1"], "V2": ["v2"]"#,
        r#""v1": "c1""#,
        r#""v2": "c2""#,
    ));
    assert_eq!(
        findings,
        vec![coproduct_finding((EpistemicLabel::Hypothesized, 2), &[])]
    );
    assert_eq!(findings.len(), 1, "one hypothesized violation");
    assert!(
        findings[0].label == EpistemicLabel::Hypothesized,
        "a proposed coproduct is a question, not a failure"
    );
}
