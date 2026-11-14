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

const fn row(
    pattern: &'static str,
    fixture: &'static str,
    values: &'static [&'static str],
    fault: &'static str,
    class: &'static str,
    findings: &'static [FindingKey],
    parity: bool,
) -> Expectation {
    Expectation {
        pattern,
        fixture,
        values,
        fault,
        class,
        findings,
        parity,
    }
}

/// The complete table: twelve in-scope patterns, one row per recorded
/// fault. Shared kata directories repeat their rows per consuming pattern
/// (composite/null-object, the three wrapper laws, visitor/interpreter,
/// chain-of-responsibility/strategy) because the ledger ran one fixture per
/// abstraction.
#[rustfmt::skip]
pub static EXPECTATIONS: &[Expectation] = &[
    // composite — ledger/katas/composite-null-object/
    row("composite", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "broken-child-parent-link", "finding", &[V, D], true),
    row("composite", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "nil-treated-as-real-leaf", "finding", &[V, D], true),
    row("composite", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "floor-near-trivial", "finding", &[V, D], true),
    row("composite", "fixtures/katas/composite-null-object/fault-broken-parent-link.ct.json", &[], "broken-child-parent-link", "finding", &[D], false),
    row("composite", "fixtures/katas/composite-null-object/fault-nil-as-real-leaf.ct.json", &[], "nil-treated-as-real-leaf", "finding", &[V], false),
    row("composite", "fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json", &[], "lost-child-on-flatten", "finding", &[LC, LC, LC, LC, LC, LC, LC], false),

    // null-object — the same evidence, read through the nil element
    row("null-object", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "broken-child-parent-link", "finding", &[V, D], true),
    row("null-object", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "nil-treated-as-real-leaf", "finding", &[V, D], true),
    row("null-object", "fixtures/katas/composite-null-object/floor/fault-floor-control.ct.json", &[], "floor-near-trivial", "finding", &[V, D], true),
    row("null-object", "fixtures/katas/composite-null-object/fault-broken-parent-link.ct.json", &[], "broken-child-parent-link", "finding", &[D], false),
    row("null-object", "fixtures/katas/composite-null-object/fault-nil-as-real-leaf.ct.json", &[], "nil-treated-as-real-leaf", "finding", &[V], false),
    row("null-object", "fixtures/katas/composite-null-object/fault-lost-child-migrate.ct.json", &[], "lost-child-on-flatten", "finding", &[LC, LC, LC, LC, LC, LC, LC], false),

    // builder — ledger/katas/builder-monoid/
    row("builder", "fixtures/katas/builder-monoid/floor/single-step-builder.ct.json", &[], "clean", "finding", &[], true),
    row("builder", "fixtures/katas/builder-monoid/floor/faults/01-dropped-step.ct.json", &[], "floor-dropped-step", "finding", &[D], true),
    row("builder", "fixtures/katas/builder-monoid/floor/faults/02-duplicated-step.ct.json", &[], "floor-duplicated-step", "gap", &[], true),
    row("builder", "fixtures/katas/builder-monoid/faults/01-dropped-step.ct.json", &[], "dropped-step", "finding", &[LR, LR, LR, LR, LR, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC, LC], false),
    row("builder", "fixtures/katas/builder-monoid/faults/02-duplicated-step.ct.json", &[], "duplicated-step", "finding", &[R, R], true),
    row("builder", "fixtures/katas/builder-monoid/faults/03-out-of-order-step.ct.json", &[], "out-of-order-step", "finding", &[V], false),
    row("builder", "fixtures/katas/builder-monoid/faults/04-optional-part-dropped.ct.json", &[], "optional-part-dropped", "finding", &[LC], false),

    // adapter — ledger/katas/wrapper-laws/
    row("adapter", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C1", "finding", &[V, V, D], true),
    row("adapter", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C2", "finding", &[V, V, D], true),
    row("adapter", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C3", "finding", &[V, V, D], true),
    row("adapter", "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", &[], "clean", "finding", &[], true),
    row("adapter", "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", &[], "W1", "finding", &[D, D, D], false),
    row("adapter", "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", &[], "W2", "finding", &[V], false),
    row("adapter", "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", &[], "W3", "finding", &[V], false),
    row("adapter", "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", &[], "W4", "finding", &[D], false),

    // decorator — the same evidence, read through the stacking equation
    row("decorator", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C1", "finding", &[V, V, D], true),
    row("decorator", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C2", "finding", &[V, V, D], true),
    row("decorator", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C3", "finding", &[V, V, D], true),
    row("decorator", "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", &[], "clean", "finding", &[], true),
    row("decorator", "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", &[], "W1", "finding", &[D, D, D], false),
    row("decorator", "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", &[], "W2", "finding", &[V], false),
    row("decorator", "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", &[], "W3", "finding", &[V], false),
    row("decorator", "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", &[], "W4", "finding", &[D], false),

    // proxy — the same evidence, read through the identity-preservation law
    row("proxy", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C1", "finding", &[V, V, D], true),
    row("proxy", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C2", "finding", &[V, V, D], true),
    row("proxy", "fixtures/katas/wrapper-laws/floor/identity-wrapper.ct.json", &[], "C3", "finding", &[V, V, D], true),
    row("proxy", "fixtures/katas/wrapper-laws/wrapper-laws.ct.json", &[], "clean", "finding", &[], true),
    row("proxy", "fixtures/katas/wrapper-laws/faults/01-adapter-drops-field.ct.json", &[], "W1", "finding", &[D, D, D], false),
    row("proxy", "fixtures/katas/wrapper-laws/faults/02-proxy-rewires-identity.ct.json", &[], "W2", "finding", &[V], false),
    row("proxy", "fixtures/katas/wrapper-laws/faults/03-decorator-order-swaps-result.ct.json", &[], "W3", "finding", &[V], false),
    row("proxy", "fixtures/katas/wrapper-laws/faults/04-wrapper-loses-optional.ct.json", &[], "W4", "finding", &[D], false),

    // visitor — ledger/katas/visitor-sum-types/
    row("visitor", "fixtures/katas/visitor-sum-types/floor/two-variant.ct.json", &["kind=lit,neg"], "clean", "finding", &[], true),
    row("visitor", "fixtures/katas/visitor-sum-types/floor/faults/01-gap-remove-lit.ct.json", &["kind=lit,neg"], "NF1", "finding", &[U], true),
    row("visitor", "fixtures/katas/visitor-sum-types/floor/faults/02-overlap-duplicate-lit.ct.json", &["kind=lit,neg"], "NF2", "finding", &[V], true),
    row("visitor", "fixtures/katas/visitor-sum-types/floor/faults/03-shadow-widen-lit.ct.json", &["kind=lit,neg"], "NF3", "finding", &[R], true),
    row("visitor", "fixtures/katas/visitor-sum-types/visitor.ct.json", &["kind=num,str,add,mul,neg"], "clean", "finding", &[], true),
    row("visitor", "fixtures/katas/visitor-sum-types/faults/01-gap-missing-num.ct.json", &["kind=num,str,add,mul,neg"], "F1", "finding", &[U], true),
    row("visitor", "fixtures/katas/visitor-sum-types/faults/02-overlap-duplicate-add.ct.json", &["kind=num,str,add,mul,neg"], "F2", "finding", &[V], true),
    row("visitor", "fixtures/katas/visitor-sum-types/faults/03-shadow-literal-covers-str.ct.json", &["kind=num,str,add,mul,neg"], "F3", "finding", &[R], false),
    row("visitor", "fixtures/katas/visitor-sum-types/faults/04-derivable-num.ct.json", &["kind=num,str,add,mul,neg"], "F4", "finding", &[R], false),
    row("visitor", "fixtures/katas/visitor-sum-types/faults/05-missing-variant-div.ct.json", &["kind=num,str,add,mul,neg,div"], "F5", "finding", &[U], false),

    // interpreter — the same sum-type evidence, read as recursive dispatch
    row("interpreter", "fixtures/katas/visitor-sum-types/floor/two-variant.ct.json", &["kind=lit,neg"], "clean", "finding", &[], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/floor/faults/01-gap-remove-lit.ct.json", &["kind=lit,neg"], "NF1", "finding", &[U], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/floor/faults/02-overlap-duplicate-lit.ct.json", &["kind=lit,neg"], "NF2", "finding", &[V], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/floor/faults/03-shadow-widen-lit.ct.json", &["kind=lit,neg"], "NF3", "finding", &[R], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/visitor.ct.json", &["kind=num,str,add,mul,neg"], "clean", "finding", &[], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/faults/01-gap-missing-num.ct.json", &["kind=num,str,add,mul,neg"], "F1", "finding", &[U], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/faults/02-overlap-duplicate-add.ct.json", &["kind=num,str,add,mul,neg"], "F2", "finding", &[V], true),
    row("interpreter", "fixtures/katas/visitor-sum-types/faults/03-shadow-literal-covers-str.ct.json", &["kind=num,str,add,mul,neg"], "F3", "finding", &[R], false),
    row("interpreter", "fixtures/katas/visitor-sum-types/faults/04-derivable-num.ct.json", &["kind=num,str,add,mul,neg"], "F4", "finding", &[R], false),
    row("interpreter", "fixtures/katas/visitor-sum-types/faults/05-missing-variant-div.ct.json", &["kind=num,str,add,mul,neg,div"], "F5", "finding", &[U], false),

    // memento — ledger/katas/memento-roundtrip/
    row("memento", "fixtures/katas/memento-roundtrip/restore-identity.ct.json", &[], "memento-identity", "finding", &[], true),
    row("memento", "fixtures/katas/memento-roundtrip/restore-dropped-field.ct.json", &[], "dropped-field", "finding", &[LC, LC], false),
    row("memento", "fixtures/katas/memento-roundtrip/restore-lossy-optional.ct.json", &[], "lossy-optional", "finding", &[LC, LC], false),
    row("memento", "fixtures/katas/memento-roundtrip/restore-version-skew.ct.json", &[], "version-skew", "finding", &[LC, LC, LC, LC, LC], false),
    row("memento", "fixtures/katas/memento-roundtrip/restore-version-skew.ct.json", &[], "non-invertibility", "finding", &[LC, LC, LC, LC, LC], false),

    // state — ledger/katas/state-discovery/
    row("state", "fixtures/katas/state-discovery/near-floor.ct.json", &[], "clean", "finding", &[], true),
    row("state", "fixtures/katas/state-discovery/near-floor-fault-retarget.ct.json", &[], "NF1", "finding", &[V], true),
    row("state", "fixtures/katas/state-discovery/near-floor-fault-missing-exit.ct.json", &[], "NF2", "finding", &[D], true),
    row("state", "fixtures/katas/state-discovery/dangling-target.ct.json", &[], "dangling-target", "finding", &[D], false),
    row("state", "fixtures/katas/state-discovery/missing-exit.ct.json", &[], "missing-exit", "finding", &[D, U, U], false),
    row("state", "fixtures/katas/state-discovery/missing-exit.ct.json", &[], "underconstraint-question", "finding", &[D, U, U], true),
    row("state", "fixtures/katas/state-discovery/drift-target.ct.json", &[], "drift-target", "finding", &[V], false),
    row("state", "fixtures/katas/state-discovery/unknown-target.ct.json", &[], "undeclared-state", "construction", &[], true),
    row("state", "fixtures/katas/state-discovery/unreachable.ct.json", &[], "unreachable-state", "gap", &[U, U], true),
    row("state", "fixtures/katas/state-discovery/unreachable.ct.json", &[], "underconstraint-question", "finding", &[U, U], true),

    // chain-of-responsibility — ledger/katas/cor-strategy-rules/
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", &["kind=card,cash", "amount=0,100"], "clean", "finding", &[], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", &["kind=card,cash", "amount=0,100"], "floor-near-trivial", "finding", &[], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/control/faults/01-gap-remove-fallback.ct.json", &["kind=card,cash", "amount=0,100"], "gap", "finding", &[U], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/control/faults/02-overlap-total-fallback.ct.json", &["kind=card,cash", "amount=0,100"], "overlap", "finding", &[V, V], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/control/faults/03-strategy-selection-mismatch.ct.json", &["kind=card,cash", "amount=0,100"], "strategy-selection", "gap", &[], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/dispatch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "clean", "finding", &[], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/01-chain-gap-remove-fallback.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "gap", "finding", &[U], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/02-chain-gap-fallback-narrow.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "gap", "finding", &[U], false),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/03-shadow-duplicate-payout.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "shadow", "finding", &[R], false),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/04-shadow-widen-payout.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "shadow", "finding", &[R, R], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/05-derivable-strategy-branch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "derivable", "finding", &[R], false),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/06-overlap-declared-disjoint.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "overlap", "finding", &[V, V, V, V, V, V, V, V, V, V], true),
    row("chain-of-responsibility", "fixtures/katas/cor-strategy-rules/faults/07-strategy-selection-mismatch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "strategy-selection", "gap", &[], true),

    // strategy — the same rule-table evidence, read as branch selection
    row("strategy", "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", &["kind=card,cash", "amount=0,100"], "clean", "finding", &[], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/control/two-handler.ct.json", &["kind=card,cash", "amount=0,100"], "floor-near-trivial", "finding", &[], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/control/faults/01-gap-remove-fallback.ct.json", &["kind=card,cash", "amount=0,100"], "gap", "finding", &[U], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/control/faults/02-overlap-total-fallback.ct.json", &["kind=card,cash", "amount=0,100"], "overlap", "finding", &[V, V], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/control/faults/03-strategy-selection-mismatch.ct.json", &["kind=card,cash", "amount=0,100"], "strategy-selection", "gap", &[], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/dispatch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "clean", "finding", &[], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/01-chain-gap-remove-fallback.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "gap", "finding", &[U], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/02-chain-gap-fallback-narrow.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "gap", "finding", &[U], false),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/03-shadow-duplicate-payout.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "shadow", "finding", &[R], false),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/04-shadow-widen-payout.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "shadow", "finding", &[R, R], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/05-derivable-strategy-branch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "derivable", "finding", &[R], false),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/06-overlap-declared-disjoint.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "overlap", "finding", &[V, V, V, V, V, V, V, V, V, V], true),
    row("strategy", "fixtures/katas/cor-strategy-rules/faults/07-strategy-selection-mismatch.ct.json", &["kind=fraud,refund,payout,payment,admin", "amount=0,50,5000", "channel=web,batch"], "strategy-selection", "gap", &[], true),
];
