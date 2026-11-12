//! Purpose: the analysis layer (openspec change `add-bounded-analysis`,
//! beads `catherine-1ay`) — the deterministic question-answerer agents ask
//! mid-conversation (DESIGN §4.2): *can* these constraints be satisfied at
//! all (bounded search → witness, contradiction, or `UNKNOWN { scope }`),
//! is this declaration derivable (redundancy via normal-form rewriting),
//! which parallel paths were never reconciled (underconstraint,
//! hypothesized), where do two instances differ (distinguish over an
//! explicit observation set), the query traversals (reachability,
//! blast radius, fan-in/fan-out) ported from specodelic behind the parity
//! gate, and the rule lens (P8: effective regions and the four rule-table
//! bug classes over a declared finite domain, mapped onto the taxonomy).
//!
//! Responsibilities: expose the analysis operations over the kernel's
//! public surfaces (`Schema`, `Instance`, `TypedInstance`, `check`) —
//! dependency direction `analysis → core`, nothing flows back. Every op
//! is deterministic and bounded: bounds are inputs, `UNKNOWN { scope }`
//! is a first-class result (label `unknown`, exact scope explored), and
//! findings emit into the P2 taxonomy — the three analysis kinds with
//! their P2 witness shapes, never a new kind. `UNKNOWN` is never
//! collapsed into clean, a finding, or an error; a zero-size bound
//! yields `UNKNOWN` with the empty scope, never a contradiction claim.
//!
//! Rationale: kernel crates stay genesis-free and serde-free (the CLI
//! owns every envelope); this crate depends on `catherine-core` alone at
//! runtime — bounded search is plain hand-rolled enumeration over the
//! finite row spaces, no solver. Honesty is the contract: no unbounded
//! claims, witnesses on every finding, and `docs/DESIGN.md` §4.2's
//! "results are a witness or `UNKNOWN { scope }` — never a naked 'no
//! violation found'".

pub mod bounds;
pub mod distinguish;
pub mod parallel;
pub mod query;
pub mod rewrite;
pub mod rule_lens;
pub mod search;
