use vstd::prelude::*;

verus! {
fn model_step(x: usize) -> (r: usize)
    requires x < usize::MAX
    ensures r == x + 1
{
    x + 1
}
} // verus!

// Plain Rust: fixture IO + orchestration, unverified by design.
fn parse_count(s: &str) -> usize {
    serde_json::from_str::<usize>(s).unwrap_or(0)
}

fn main() {
    let doc = parse_count("41");
    let r = model_step(doc);
    assert!(r == 42);
    println!("spike: {} -> {}", doc, r);
}
