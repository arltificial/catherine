#![allow(dead_code)]
//! Purpose: the checked-in replay expectation table (openspec change
//! `add-pattern-lenses`, design Decision 7; beads `catherine-mx1.2`) — one
//! row per fault the sealed kata ledger recorded, so every lens ticket has
//! a Meter that never reads `ledger/` at test time.
//!
//! Responsibilities: for the twelve in-scope patterns, the fixture the
//! ledger ran, the `--values` the recorded command supplied, the fault id,
//! the recorded `catherine_class`, the `(kind, label)` multiset that
//! entry's `run.findings` records for the fixture, and the parity flag
//! (`true` when `comparison.unique_detections` does not count the fault,
//! i.e. the conventional baseline caught it). Derived from
//! `ledger/katas/*/` at commit `e818263`; never edited by a lens ticket.
//!
//! Rationale: a `&'static` Rust literal, not JSON — a typo in a pattern id
//! or finding kind fails to compile. The ledger is sealed evidence, so a
//! live read would let an edit move the gate silently. `clean` rows are the
//! recorded zero-finding arms; `gap`/`construction` rows assert silence or
//! a parse refusal; the migration-kind rows (`lost_row`, `lost_cell`) are
//! recorded verbatim and are matched by the lens's `migrations` channel —
//! `LensFindings` carries the migrate surface's `MigrationFinding`s beside
//! the taxonomy's checks, so a Δ row is never satisfied by a check finding.

/// One recorded `(finding kind, epistemic label)` pair.
pub type FindingKey = (&'static str, &'static str);

/// `violation`/`executed`.
pub const V: FindingKey = ("violation", "executed");
/// `dangling`/`executed`.
pub const D: FindingKey = ("dangling", "executed");
/// `underconstraint`/`hypothesized`.
pub const U: FindingKey = ("underconstraint", "hypothesized");
/// `redundant`/`executed`.
pub const R: FindingKey = ("redundant", "executed");
/// The migration class's `lost_row`/`executed`.
pub const LR: FindingKey = ("lost_row", "executed");
/// The migration class's `lost_cell`/`executed`.
pub const LC: FindingKey = ("lost_cell", "executed");

/// One expectation: the fixture, the bound values, the fault, the recorded
/// class, the fixture's recorded `(kind, label)` multiset, and parity.
pub struct Expectation {
    /// The pattern id whose lens the row gates.
    pub pattern: &'static str,
    /// The fixture path, relative to the repository root.
    pub fixture: &'static str,
    /// The `--values` spellings the recorded command passed (empty when the
    /// recorded op supplied no finite domain).
    pub values: &'static [&'static str],
    /// The ledger's fault id (`clean` for the zero-finding arms).
    pub fault: &'static str,
    /// The ledger's `catherine_class` (`finding`, `gap`, `construction`).
    pub class: &'static str,
    /// The `(kind, label)` multiset the entry records for this fixture.
    pub findings: &'static [FindingKey],
    /// The baseline caught it: reported as parity only, no unique claim.
    pub parity: bool,
}

/// The complete table: twelve in-scope patterns, one row per recorded
/// fault. Shared kata directories repeat their rows per consuming pattern
/// (composite/null-object, the three wrapper laws, visitor/interpreter,
/// chain-of-responsibility/strategy) because the ledger ran one fixture per
/// abstraction.
#[rustfmt::skip]
pub static EXPECTATIONS: &[Expectation] = &[
    // composite — ledger/katas/composite-null-object/
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "broken-child-parent-link", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "nil-treated-as-real-leaf", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "floor-near-trivial", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/fault-broken-parent-link.ct.json", values: &[], fault: "broken-child-parent-link", class: "finding", findings: &[D], parity: false },
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/fault-nil-as-real-leaf.ct.json", values: &[], fault: "nil-treated-as-real-leaf", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "composite", fixture: "fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json", values: &[], fault: "lost-child-on-flatten", class: "finding", findings: &[LC, LC, LC, LC, LC, LC, LC], parity: false },

    // null-object — the same evidence, read through the nil element
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "broken-child-parent-link", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "nil-treated-as-real-leaf", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", values: &[], fault: "floor-near-trivial", class: "finding", findings: &[V, D], parity: true },
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/fault-broken-parent-link.ct.json", values: &[], fault: "broken-child-parent-link", class: "finding", findings: &[D], parity: false },
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/fault-nil-as-real-leaf.ct.json", values: &[], fault: "nil-treated-as-real-leaf", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "null-object", fixture: "fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json", values: &[], fault: "lost-child-on-flatten", class: "finding", findings: &[LC, LC, LC, LC, LC, LC, LC], parity: false },

    // builder — ledger/katas/builder-monoid/
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/floor/single-step-builder.ct.json", values: &[], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/floor/faults/01-dropped-step.ct.json", values: &[], fault: "floor-dropped-step", class: "finding", findings: &[D], parity: true },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/floor/faults/02-duplicated-step.ct.json", values: &[], fault: "floor-duplicated-step", class: "gap", findings: &[], parity: true },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/faults/01-dropped-step.ct.json", values: &[], fault: "dropped-step", class: "finding", findings: &[LR, LR, LR, LR, LR, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC], parity: false },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/faults/02-duplicated-step.ct.json", values: &[], fault: "duplicated-step", class: "finding", findings: &[R, R], parity: true },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/faults/03-out-of-order-step.ct.json", values: &[], fault: "out-of-order-step", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "builder", fixture: "fixtures/katas/builder-monoid/faults/04-optional-part-dropped.ct.json", values: &[], fault: "optional-part-dropped", class: "finding", findings: &[LC], parity: false },

    // adapter — ledger/katas/wrapper-laws/
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json", values: &[], fault: "C1", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json", values: &[], fault: "C2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json", values: &[], fault: "C3", class: "finding", findings: &[D], parity: true },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", values: &[], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", values: &[], fault: "W1", class: "finding", findings: &[D, D, D], parity: false },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", values: &[], fault: "W2", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", values: &[], fault: "W3", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "adapter", fixture: "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", values: &[], fault: "W4", class: "finding", findings: &[D], parity: false },

    // decorator — the same evidence, read through the stacking equation
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json", values: &[], fault: "C1", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json", values: &[], fault: "C2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json", values: &[], fault: "C3", class: "finding", findings: &[D], parity: true },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", values: &[], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", values: &[], fault: "W1", class: "finding", findings: &[D, D, D], parity: false },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", values: &[], fault: "W2", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", values: &[], fault: "W3", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "decorator", fixture: "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", values: &[], fault: "W4", class: "finding", findings: &[D], parity: false },

    // proxy — the same evidence, read through the identity-preservation law
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/floor/faults/01-identity-wrapper-swaps-result.ct.json", values: &[], fault: "C1", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/floor/faults/02-identity-wrapper-rewires-key.ct.json", values: &[], fault: "C2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/floor/faults/03-identity-wrapper-loses-note.ct.json", values: &[], fault: "C3", class: "finding", findings: &[D], parity: true },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", values: &[], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", values: &[], fault: "W1", class: "finding", findings: &[D, D, D], parity: false },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", values: &[], fault: "W2", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", values: &[], fault: "W3", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "proxy", fixture: "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", values: &[], fault: "W4", class: "finding", findings: &[D], parity: false },

    // visitor — ledger/katas/visitor-sum-types/
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/floor/two-variant.ct.json", values: &["kind=lit,neg"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/floor/faults/01-gap-remove-lit.ct.json", values: &["kind=lit,neg"], fault: "NF1", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/floor/faults/02-overlap-duplicate-lit.ct.json", values: &["kind=lit,neg"], fault: "NF2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/floor/faults/03-shadow-widen-lit.ct.json", values: &["kind=lit,neg"], fault: "NF3", class: "finding", findings: &[R], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/visitor.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/faults/01-gap-missing-num.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F1", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/faults/02-overlap-duplicate-add.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/faults/03-shadow-literal-covers-str.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F3", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/faults/04-derivable-num.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F4", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "visitor", fixture: "fixtures/katas/visitor-sum-types/faults/05-missing-variant-div.ct.json", values: &["kind=num,str,add,mul,neg,div"], fault: "F5", class: "finding", findings: &[U], parity: false },

    // interpreter — the same sum-type evidence, read as recursive dispatch
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/floor/two-variant.ct.json", values: &["kind=lit,neg"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/floor/faults/01-gap-remove-lit.ct.json", values: &["kind=lit,neg"], fault: "NF1", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/floor/faults/02-overlap-duplicate-lit.ct.json", values: &["kind=lit,neg"], fault: "NF2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/floor/faults/03-shadow-widen-lit.ct.json", values: &["kind=lit,neg"], fault: "NF3", class: "finding", findings: &[R], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/visitor.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/faults/01-gap-missing-num.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F1", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/faults/02-overlap-duplicate-add.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F2", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/faults/03-shadow-literal-covers-str.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F3", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/faults/04-derivable-num.ct.json", values: &["kind=num,str,add,mul,neg"], fault: "F4", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "interpreter", fixture: "fixtures/katas/visitor-sum-types/faults/05-missing-variant-div.ct.json", values: &["kind=num,str,add,mul,neg,div"], fault: "F5", class: "finding", findings: &[U], parity: false },

    // memento — ledger/katas/memento-roundtrip/
    Expectation { pattern: "memento", fixture: "fixtures/katas/memento-roundtrip/restore-identity.ct.json", values: &[], fault: "memento-identity", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "memento", fixture: "fixtures/katas/memento-roundtrip/restore-dropped-field.ct.json", values: &[], fault: "dropped-field", class: "finding", findings: &[LC, LC], parity: false },
    Expectation { pattern: "memento", fixture: "fixtures/katas/memento-roundtrip/restore-lossy-optional.ct.json", values: &[], fault: "lossy-optional", class: "finding", findings: &[LC, LC], parity: false },
    Expectation { pattern: "memento", fixture: "fixtures/katas/memento-roundtrip/restore-version-skew.ct.json", values: &[], fault: "version-skew", class: "finding", findings: &[LC, LC, LC, LC, LC], parity: false },
    Expectation { pattern: "memento", fixture: "fixtures/katas/memento-roundtrip/restore-version-skew.ct.json", values: &[], fault: "non-invertibility", class: "finding", findings: &[LC, LC, LC, LC, LC], parity: false },

    // state — ledger/katas/state-discovery/
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/near-floor.ct.json", values: &[], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/near-floor-fault-retarget.ct.json", values: &[], fault: "NF1", class: "finding", findings: &[V], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/near-floor-fault-missing-exit.ct.json", values: &[], fault: "NF2", class: "finding", findings: &[D], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/dangling-target.ct.json", values: &[], fault: "dangling-target", class: "finding", findings: &[D], parity: false },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/missing-exit.ct.json", values: &[], fault: "missing-exit", class: "finding", findings: &[D, U, U], parity: false },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/missing-exit.ct.json", values: &[], fault: "underconstraint-question", class: "finding", findings: &[D, U, U], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/drift-target.ct.json", values: &[], fault: "drift-target", class: "finding", findings: &[V], parity: false },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/unknown-target.ct.json", values: &[], fault: "undeclared-state", class: "construction", findings: &[], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/unreachable.ct.json", values: &[], fault: "unreachable-state", class: "gap", findings: &[U, U], parity: true },
    Expectation { pattern: "state", fixture: "fixtures/katas/state-discovery/unreachable.ct.json", values: &[], fault: "underconstraint-question", class: "finding", findings: &[U, U], parity: true },

    // chain-of-responsibility — ledger/katas/cor-strategy-rules/
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "floor-near-trivial", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/control/faults/01-gap-remove-fallback.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "gap", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/control/faults/02-overlap-total-fallback.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "overlap", class: "finding", findings: &[V, V], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/control/faults/03-strategy-selection-mismatch.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "strategy-selection", class: "gap", findings: &[], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/dispatch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/01-chain-gap-remove-fallback.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "gap", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/02-chain-gap-fallback-narrow.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "gap", class: "finding", findings: &[U], parity: false },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/03-shadow-duplicate-payout.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "shadow", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/04-shadow-widen-payout.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "shadow", class: "finding", findings: &[R, R], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/05-derivable-strategy-branch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "derivable", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/06-overlap-declared-disjoint.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "overlap", class: "finding", findings: &[V, V, V, V, V, V, V, V, V, V], parity: true },
    Expectation { pattern: "chain-of-responsibility", fixture: "fixtures/katas/cor-strategy-rules/faults/07-strategy-selection-mismatch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "strategy-selection", class: "gap", findings: &[], parity: true },

    // strategy — the same rule-table evidence, read as branch selection
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "floor-near-trivial", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/control/faults/01-gap-remove-fallback.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "gap", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/control/faults/02-overlap-total-fallback.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "overlap", class: "finding", findings: &[V, V], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/control/faults/03-strategy-selection-mismatch.ct.json", values: &["kind=card,cash", "amount=0,100"], fault: "strategy-selection", class: "gap", findings: &[], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/dispatch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "clean", class: "finding", findings: &[], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/01-chain-gap-remove-fallback.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "gap", class: "finding", findings: &[U], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/02-chain-gap-fallback-narrow.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "gap", class: "finding", findings: &[U], parity: false },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/03-shadow-duplicate-payout.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "shadow", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/04-shadow-widen-payout.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "shadow", class: "finding", findings: &[R, R], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/05-derivable-strategy-branch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "derivable", class: "finding", findings: &[R], parity: false },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/06-overlap-declared-disjoint.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "overlap", class: "finding", findings: &[V, V, V, V, V, V, V, V, V, V], parity: true },
    Expectation { pattern: "strategy", fixture: "fixtures/katas/cor-strategy-rules/faults/07-strategy-selection-mismatch.ct.json", values: &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], fault: "strategy-selection", class: "gap", findings: &[], parity: true },
];
