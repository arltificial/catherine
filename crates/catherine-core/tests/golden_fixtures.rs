//! Purpose: golden fixtures pinning malformed-presentation rejection —
//! per fixture, the exact error code and the line/column of the offending
//! JSON node (design Decision 5: assertions are structural, never
//! snapshots; only the message fragments that name the offending element
//! are asserted). One table, one loop — adding a fixture is adding a row.

use catherine_core::error::{ErrorCode, ParseError};
use catherine_core::model::Model;
use catherine_core::parse::parse;

const GOLDEN_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/golden");

/// One pinned rejection: the file, the stable code, the line/column of
/// the offending node (the fixtures' layouts pin these by hand), and the
/// message fragments naming the offending element.
struct Case {
    file: &'static str,
    code: ErrorCode,
    line: usize,
    column: usize,
    message_contains: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        file: "duplicate_object.json",
        code: ErrorCode::DuplicateIdent,
        line: 4,
        column: 27,
        message_contains: &["duplicate object ident `A`"],
    },
    Case {
        file: "duplicate_arrow.json",
        code: ErrorCode::DuplicateIdent,
        line: 7,
        column: 16,
        message_contains: &["duplicate arrow ident `f`"],
    },
    Case {
        file: "unknown_arrow.json",
        code: ErrorCode::UnknownArrow,
        line: 10,
        column: 21,
        message_contains: &["path [f, q]", "references undeclared arrow `q`"],
    },
    Case {
        file: "ill_typed_equation.json",
        code: ErrorCode::IllTypedEquation,
        line: 11,
        column: 7,
        message_contains: &["[f, g]", "[h]", "A -> C", "A -> B"],
    },
    Case {
        file: "invalid_ident_unicode.json",
        code: ErrorCode::InvalidIdent,
        line: 4,
        column: 17,
        message_contains: &["invalid ident `álpha`"],
    },
    Case {
        file: "invalid_ident_empty.json",
        code: ErrorCode::InvalidIdent,
        line: 4,
        column: 17,
        message_contains: &["invalid ident ``"],
    },
    Case {
        file: "invalid_ident_digit.json",
        code: ErrorCode::InvalidIdent,
        line: 4,
        column: 17,
        message_contains: &["invalid ident `0a`"],
    },
    Case {
        file: "invalid_status.json",
        code: ErrorCode::InvalidStatus,
        line: 7,
        column: 64,
        message_contains: &[
            "invalid equation status `rejected`",
            "`proposed` or `accepted`",
        ],
    },
    Case {
        file: "absent_version.json",
        code: ErrorCode::UnsupportedVersion,
        line: 1,
        column: 1,
        message_contains: &["missing `format_version`"],
    },
    Case {
        file: "unknown_major.json",
        code: ErrorCode::UnsupportedVersion,
        line: 2,
        column: 21,
        message_contains: &["unsupported `format_version` `2.0`", "known major is 1"],
    },
    Case {
        file: "unknown_minor.json",
        code: ErrorCode::UnsupportedVersion,
        line: 2,
        column: 21,
        message_contains: &["unsupported `format_version` `1.9`", "known version is 1.1"],
    },
];

fn read_fixture(file: &str) -> String {
    std::fs::read_to_string(format!("{GOLDEN_DIR}/{file}"))
        .unwrap_or_else(|e| panic!("fixture {file} must be readable: {e}"))
}

#[test]
fn malformed_presentations_are_rejected_at_the_pinned_nodes() {
    for case in CASES {
        let (file, code, line, column) = (case.file, case.code, case.line, case.column);
        let code_str = code.as_str();
        let text = read_fixture(file);
        match parse(&text) {
            Ok(_) => panic!(
                "{file}: expected {code_str} at {line}:{column} but the document was accepted \
                 (missing behavior: malformed presentation not rejected)"
            ),
            Err(ParseError::Semantic(e)) => {
                assert_eq!(e.code, code, "{file}: wrong code (want {code_str}, got {e:?})");
                assert_eq!(
                    (e.line, e.column),
                    (line, column),
                    "{file}: wrong position (want {code_str} at {line}:{column}, got {e:?})"
                );
                for fragment in case.message_contains {
                    assert!(
                        e.message.contains(fragment),
                        "{file}: message {:?} must contain {fragment:?}",
                        e.message
                    );
                }
            }
            Err(ParseError::Syntax {
                line: l,
                column: c,
                message,
            }) => panic!(
                "{file}: expected {code_str} at {line}:{column}, got a syntax error at {l}:{c}: {message}"
            ),
        }
    }
}

#[test]
fn empty_schema_and_empty_instance_parse_cleanly() {
    let text = read_fixture("empty_ok.json");
    let model = parse(&text).expect("empty schema + empty instance parse cleanly");
    assert_eq!(
        model,
        Model::empty(),
        "the empty document parses to the empty model"
    );
}
