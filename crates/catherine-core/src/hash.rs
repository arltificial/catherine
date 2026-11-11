// Ported from the wild project (repository `charly/wild`, file
// `src/formats.rs` — canonical-JSON sha256 identity, byte-for-byte
// normative), licensed under Apache-2.0. The project ships no NOTICE
// file; this header records the attribution required by Apache-2.0 §4(c)
// and preserved by hard constraint 4 of openspec change
// `add-model-document-kernel`.
//
//! Purpose: the model identity — sha256 over the canonical serialization
//! bytes — for later content-addressed ledger use (P6).
//!
//! Responsibilities: compute the sha256 digest (`sha256:` plus 64
//! lowercase hex digits, wild's exact format) over canonical bytes, and
//! expose `model_hash` as the one-liner P1+ consumers call.
//!
//! Rationale: wild's discipline is canonical-bytes hashing — identity is
//! the digest of the canonical form, so equal models hash equal
//! regardless of input key order or presentation row names (design
//! Decision 4). The digest itself is implemented in-crate per FIPS 180-4
//! because the brief locks dependencies to serde/serde_json/proptest (§6)
//! — the algorithm is fully specified, so there is no design freedom to
//! exercise; the normative test vectors below pin it byte-for-byte.

use crate::canonical::canonical_bytes;
use crate::model::Model;
use std::fmt::Write as _;

/// sha256 content digest: `sha256:` plus 64 lowercase hex digits (wild
/// `formats.rs`, verbatim format).
pub fn sha256_digest(bytes: &[u8]) -> String {
    let digest = sha256(bytes);
    let mut out = String::with_capacity(7 + 64);
    out.push_str("sha256:");
    for byte in &digest {
        let _ = write!(out, "{byte:02x}");
    }
    out
}

/// The model identity: sha256 over the canonical bytes — the model
/// re-labeled with structure-derived row ids. Equal models hash equal
/// regardless of input key order or presentation row names; any
/// structural difference changes the hash.
pub fn model_hash(model: &Model) -> String {
    sha256_digest(canonical_bytes(model).as_bytes())
}

/// FIPS 180-4 initial hash state.
const H0: [u32; 8] = [
    0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
];

/// FIPS 180-4 §4.2.2 round constants.
const K: [u32; 64] = [
    0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5,
    0xd807aa98, 0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174,
    0xe49b69c1, 0xefbe4786, 0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da,
    0x983e5152, 0xa831c66d, 0xb00327c8, 0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967,
    0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13, 0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85,
    0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819, 0xd6990624, 0xf40e3585, 0x106aa070,
    0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a, 0x5b9cca4f, 0x682e6ff3,
    0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7, 0xc67178f2,
];

/// sha256 over `bytes` (FIPS 180-4). Streams full blocks straight from
/// the input; only the 1–2 padding tail blocks are staged on the stack —
/// no input copy, no heap.
fn sha256(bytes: &[u8]) -> [u8; 32] {
    let mut h = H0;
    let whole = bytes.len() / 64;
    for block in bytes[..whole * 64].as_chunks::<64>().0 {
        compress(&mut h, block);
    }
    let rest = &bytes[whole * 64..];
    let bit_len = (bytes.len() as u64) * 8;
    let mut tail = [0u8; 128];
    tail[..rest.len()].copy_from_slice(rest);
    tail[rest.len()] = 0x80;
    let tail_len = if rest.len() + 1 + 8 <= 64 { 64 } else { 128 };
    tail[tail_len - 8..tail_len].copy_from_slice(&bit_len.to_be_bytes());
    compress(&mut h, &tail[..64]);
    if tail_len == 128 {
        compress(&mut h, &tail[64..]);
    }
    let mut out = [0u8; 32];
    for (i, word) in h.iter().enumerate() {
        out[i * 4..i * 4 + 4].copy_from_slice(&word.to_be_bytes());
    }
    out
}

/// FIPS 180-4 §6.2.2 message schedule: expand the 16 block words to 64.
fn message_schedule(block: &[u8]) -> [u32; 64] {
    let mut w = [0u32; 64];
    for (i, word) in block.as_chunks::<4>().0.iter().enumerate() {
        w[i] = u32::from_be_bytes(*word);
    }
    for i in 16..64 {
        let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
        let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
        w[i] = w[i - 16]
            .wrapping_add(s0)
            .wrapping_add(w[i - 7])
            .wrapping_add(s1);
    }
    w
}

/// One FIPS 180-4 compression round over a 64-byte block.
fn compress(h: &mut [u32; 8], block: &[u8]) {
    let w = message_schedule(block);
    let [mut a, mut b, mut c, mut d, mut e, mut f, mut g, mut hh] = *h;
    for i in 0..64 {
        let s1 = e.rotate_right(6) ^ e.rotate_right(11) ^ e.rotate_right(25);
        let ch = (e & f) ^ (!e & g);
        let t1 = hh
            .wrapping_add(s1)
            .wrapping_add(ch)
            .wrapping_add(K[i])
            .wrapping_add(w[i]);
        let s0 = a.rotate_right(2) ^ a.rotate_right(13) ^ a.rotate_right(22);
        let maj = (a & b) ^ (a & c) ^ (b & c);
        let t2 = s0.wrapping_add(maj);
        hh = g;
        g = f;
        f = e;
        e = d.wrapping_add(t1);
        d = c;
        c = b;
        b = a;
        a = t1.wrapping_add(t2);
    }
    let round = [a, b, c, d, e, f, g, hh];
    for (hi, ri) in h.iter_mut().zip(round) {
        *hi = hi.wrapping_add(ri);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse::parse;

    /// The NIST/FIPS 180-4 test vectors — bytes ARE the contract (the
    /// brief's normative-byte discipline for the wild port; the digest is
    /// in-crate, so the vectors are its normative suite).
    #[test]
    fn sha256_matches_the_normative_vectors() {
        assert_eq!(
            sha256_digest(b""),
            "sha256:e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert_eq!(
            sha256_digest(b"abc"),
            "sha256:ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            sha256_digest(b"abcdbcdecdefdefgefghfghighijhijkijkljklmklmnlmnomnopnopq"),
            "sha256:248d6a61d20638b8e5c026930c3e6039a33ce45964ff2167f6ecedd419db06c1"
        );
        assert_eq!(
            sha256_digest(&vec![b'a'; 1_000_000]),
            "sha256:cdc76e5c9914fb9281a1c7e284d73e67f1809a48a497200e046d39ccc7112cd0"
        );
    }

    /// The digest spelling matches wild's `sha256:` + 64-lowercase-hex
    /// shape (its `is_digest` accepts exactly this).
    #[test]
    fn digest_format_is_wilds_sha256_spelling() {
        let digest = sha256_digest(b"wild");
        assert!(digest.starts_with("sha256:"));
        let hex = &digest["sha256:".len()..];
        assert_eq!(hex.len(), 64);
        assert!(
            hex.bytes()
                .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "lowercase hex only"
        );
    }

    /// Equal models hash equal regardless of input order; a structural
    /// difference (here: the self-loop retargeted) changes the hash.
    #[test]
    fn model_hash_is_structure_only() {
        let ordered = r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A", "B"],
            "arrows": [{"name": "f", "src": "A", "tgt": "B"}, {"name": "s", "src": "B", "tgt": "B"}],
            "equations": [{"lhs": ["s"], "rhs": ["s", "s"], "rationale": "idempotent"}]
          },
          "instance": {"rows": {"A": ["_0"], "B": ["_0"]}, "tables": {"f": {"_0": "_0"}, "s": {"_0": null}}}
        }"#;
        let shuffled = r#"{
          "instance": {"tables": {"s": {}, "f": {"_0": "_0"}}, "rows": {"B": ["_0"], "A": ["_0"]}},
          "schema": {
            "equations": [{"rhs": ["s", "s"], "rationale": "idempotent", "lhs": ["s"]}],
            "arrows": [{"tgt": "B", "src": "B", "name": "s"}, {"tgt": "B", "src": "A", "name": "f"}],
            "objects": ["B", "A"]
          },
          "format_version": "1.0"
        }"#;
        let a = parse(ordered).expect("parses");
        let b = parse(shuffled).expect("parses");
        assert_eq!(model_hash(&a), model_hash(&b), "equal models, equal hash");
        let retargeted = r#"{
          "format_version": "1.0",
          "schema": {
            "objects": ["A", "B"],
            "arrows": [{"name": "f", "src": "A", "tgt": "B"}, {"name": "s", "src": "B", "tgt": "A"}],
            "equations": [{"lhs": ["s"], "rhs": ["s", "s"], "rationale": "idempotent"}]
          },
          "instance": {"rows": {"A": ["_0"], "B": ["_0"]}, "tables": {"f": {"_0": "_0"}, "s": {}}}
        }"#;
        let c = parse(retargeted).expect("parses");
        assert_ne!(
            model_hash(&a),
            model_hash(&c),
            "a structural difference changes the hash"
        );
    }
}
