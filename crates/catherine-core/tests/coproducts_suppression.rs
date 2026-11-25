//! Purpose: the decidable-subset rule for coproduct findings (arc-review
//! CORR-001): an unresolved injection cell leaves *coverage* undecidable
//! for every row, but a row lying in two complete injection images is a
//! decidable disjointness violation even when a sibling cell is
//! unresolved.
//!
//! Responsibilities: pin the three-way declaration over `C` — two
//! complete images plus one partial image — as one disjointness violation
//! plus the partial image's dangling findings; pin that an uncovered row
//! with an unresolved sibling stays suppressed; pin that a fully complete
//! clean table remains silent; and pin that a proposed declaration keeps
//! the `hypothesized` label on the decidable violation.
//!
//! Rationale: the partial image's unknown target could be any row, so a
//! coverage claim cannot be proved while it exists; disjointness of two
//! complete images is a demonstrated fact about the data and does not
//! depend on the missing cell. One finding per row, witness shape
//! unchanged (`{coproduct, row, injections}`), and the unresolved cell
//! still carries only its own dangling finding — never double-reported.
//! The zero-row object stays vacuously covered (EDGE-002), recorded in the
//! deployed requirement, not re-tested here.

use catherine_core::checks::check;
use catherine_core::findings::{
    CoproductWitness, DanglingWitness, EpistemicLabel, Finding, FindingKind, ViolationWitness,
};
use catherine_core::instance::TypedInstance;
use catherine_core::model::{Ident, RowId};
use catherine_core::parse::parse;

/// Two complete injections (`i1`, `i2` both cover `shared`) plus one
/// partial injection (`i3` resolves only `x1`). `rogue` is in no complete
/// image, so only its coverage claim is undecidable.
const DOC: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2", "V3"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"},
      {"name": "i3", "src": "V3", "tgt": "C"}
    ],
    "equations": [],
    "coproducts": [
      {"object": "C", "arrows": ["i1", "i2", "i3"], "status": "__STATUS__"}
    ]
  },
  "instance": {
    "rows": {
      "C": ["c1", "c2", "shared", "rogue"],
      "V1": ["v1", "v2"],
      "V2": ["w1", "w2"],
      "V3": ["x1", "x2", "x3"]
    },
    "tables": {
      "i1": {"v1": "c1", "v2": "shared"},
      "i2": {"w1": "c2", "w2": "shared"},
      "i3": {"x1": "c1"}
    }
  }
}"#;

/// One complete image (`i1` covers `c1`) and one partial image (`i2`
/// resolves `w1` only); `c2` is in no complete image.
const PARTIAL_ONLY: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"}
    ],
    "equations": [],
    "coproducts": [
      {"object": "C", "arrows": ["i1", "i2"], "status": "accepted"}
    ]
  },
  "instance": {
    "rows": {
      "C": ["c1", "c2"],
      "V1": ["v1"],
      "V2": ["w1", "w2"]
    },
    "tables": {
      "i1": {"v1": "c1"},
      "i2": {"w1": "c1"}
    }
  }
}"#;

/// Both images complete and each row covered exactly once.
const CLEAN: &str = r#"{
  "format_version": "1.5",
  "schema": {
    "objects": ["C", "V1", "V2"],
    "arrows": [
      {"name": "i1", "src": "V1", "tgt": "C"},
      {"name": "i2", "src": "V2", "tgt": "C"}
    ],
    "equations": [],
    "coproducts": [
      {"object": "C", "arrows": ["i1", "i2"], "status": "accepted"}
    ]
  },
  "instance": {
    "rows": {
      "C": ["c1", "c2"],
      "V1": ["v1"],
      "V2": ["w1"]
    },
    "tables": {
      "i1": {"v1": "c1"},
      "i2": {"w1": "c2"}
    }
  }
}"#;

/// Parse, build, and check one document.
fn findings(text: &str) -> Vec<Finding> {
    let model = parse(text).expect("the document parses");
    let built = TypedInstance::build(&model.schema, &model.instance);
    check(&model.schema, &built)
}

fn coproduct_finding(drawn: (EpistemicLabel, usize), injections: &[&str]) -> Finding {
    let (label, row) = drawn;
    Finding {
        label,
        kind: FindingKind::Violation(ViolationWitness::Coproduct(CoproductWitness {
            coproduct: 0,
            row: RowId(row),
            injections: injections
                .iter()
                .map(|name| Ident(name.to_string()))
                .collect(),
        })),
    }
}

fn dangling(arrow: &str, src: usize) -> Finding {
    Finding {
        label: EpistemicLabel::Executed,
        kind: FindingKind::Dangling(DanglingWitness {
            arrow: Ident(arrow.to_string()),
            src: RowId(src),
        }),
    }
}

/// Scenario "Decidable disjointness survives an unresolved sibling": the
/// row in both complete images is one executed violation naming both
/// injections; the partial image's two unresolved cells keep their own
/// dangling findings, and the uncovered `rogue` row stays suppressed.
#[test]
fn decidable_disjointness_survives_an_unresolved_sibling() {
    let found = findings(&DOC.replace("__STATUS__", "accepted"));
    assert_eq!(
        found,
        vec![
            coproduct_finding((EpistemicLabel::Executed, 3), &["i1", "i2"]),
            dangling("i3", 1),
            dangling("i3", 2),
        ],
        "one decidable violation plus the partial image's own dangling cells"
    );
}

/// An uncovered row is not reported while any image is partial: the
/// unknown target of `w2` could be `c2`, so coverage is undecidable for
/// every row and only the dangling cell is reported.
#[test]
fn uncovered_row_with_an_unresolved_sibling_stays_suppressed() {
    let found = findings(PARTIAL_ONLY);
    assert_eq!(
        found,
        vec![dangling("i2", 1)],
        "the uncovered row is withheld, never double-reported"
    );
}

/// A fully complete table keeps the original coverage semantics: exact
/// coverage is silent.
#[test]
fn clean_table_with_complete_images_is_still_silent() {
    assert_eq!(
        findings(CLEAN),
        Vec::new(),
        "every row in exactly one complete image emits nothing"
    );
}

/// A proposed declaration's decidable disjointness is a question: the
/// same violation witness, labeled `hypothesized`; dangling findings keep
/// their executed label.
#[test]
fn proposed_disjointness_survives_an_unresolved_sibling() {
    let found = findings(&DOC.replace("__STATUS__", "proposed"));
    assert_eq!(
        found,
        vec![
            coproduct_finding((EpistemicLabel::Hypothesized, 3), &["i1", "i2"]),
            dangling("i3", 1),
            dangling("i3", 2),
        ],
        "the rule label follows the declaration status"
    );
}
