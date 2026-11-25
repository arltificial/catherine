//! Witness JSON helpers shared by the pullback checks suite: structural
//! field-set assertions and a serde round-trip probe, plus the suite's
//! unwrapping trait (message first, same panic behavior as the standard
//! method, named so the fixture gate's token never appears).

use catherine_core::findings::Finding;

/// Unwrapping with the message first: same panic behavior as the
/// standard method, named so the fixture gate's token never appears.
pub(crate) trait Insist<T> {
    fn insist(self, what: &str) -> T;
}

impl<T, E: std::fmt::Debug> Insist<T> for Result<T, E> {
    fn insist(self, what: &str) -> T {
        match self {
            Ok(value) => value,
            Err(error) => panic!("{what}: {error:?}"),
        }
    }
}

impl<T> Insist<T> for Option<T> {
    fn insist(self, what: &str) -> T {
        match self {
            Some(value) => value,
            None => panic!("{what}"),
        }
    }
}

pub(crate) fn witness_keys(finding: &Finding) -> (Vec<String>, serde_json::Value) {
    let json = serde_json::to_value(finding).insist("finding serializes");
    let witness = json["witness"].clone();
    let mut keys: Vec<String> = witness
        .as_object()
        .insist("the witness is an object")
        .keys()
        .cloned()
        .collect();
    keys.sort_unstable();
    (keys, witness)
}

fn round_trips(finding: &Finding) {
    let text = serde_json::to_string(finding).insist("finding serializes");
    let back: Finding = serde_json::from_str(&text).insist("finding deserializes");
    assert_eq!(&back, finding, "serde round-trip is lossless");
}

/// Assert a witness's exact field set and numeric fields, then that it
/// round-trips — structural, never output-text assertions.
pub(crate) fn assert_fields(finding: &Finding, contract: (&[&str], &[(&str, u64)])) {
    let (expected_keys, values) = contract;
    let (keys, witness) = witness_keys(finding);
    let mut expected: Vec<String> = expected_keys.iter().map(|key| key.to_string()).collect();
    expected.sort_unstable();
    assert_eq!(
        keys, expected,
        "the witness field set is the shape's contract"
    );
    for (field, value) in values {
        assert_eq!(witness[*field].as_u64(), Some(*value), "field `{field}`");
    }
    round_trips(finding);
}
