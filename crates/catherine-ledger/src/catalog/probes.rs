//! The pre-registered domain-probe expectation rows, one builder per
//! probe: migration, rule-lens, naturality, composition, coproduct.
//! Split out of the catalog module verbatim (minus the `let mut` row
//! builders, which are now iterator chains) to keep every file under
//! the size and mutability caps; row contents are unchanged.

use crate::entry::Expectation;

/// probe-migration: Δ migration at real scale — the pre-registered
/// lost-data and non-invertibility expectations, plus the floor control
/// where the conventional row-count/checksum dry run sees the whole
/// loss.
/// probe-migration lost-data arm: the pre-registered named-row finding.
pub(super) fn probe_migration_lost_data() -> Expectation {
    Expectation {
        fault: "lost-data".into(),
        series_class: "behavioral".into(),
        catherine_class: "finding".into(),
        detail: "a Δ migration drops a source row with no image witness; the \
                 pre-registered expectation is a named-row lost-data finding (no \
                 incumbent emits named-row lost-data witnesses); the baseline \
                 row-count/checksum dry run catches only losses visible in counts"
            .into(),
    }
}

pub(super) fn probe_migration() -> Vec<Expectation> {
    vec![
        probe_migration_lost_data(),
        Expectation {
            fault: "non-invertibility".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "Δ∘Δ′ is not the identity on the declared schema map; the \
                     pre-registered expectation is a counterexample naming the failing \
                     element; a dry run cannot express it"
                .into(),
        },
        Expectation {
            fault: "floor-visible-loss".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — a loss fully visible to the row-count/checksum \
                     dry run; the probe must show zero unique detections here (design \
                     Decision 2)"
                .into(),
        },
    ]
}

/// probe-rule-lens-extracted: the rule lens restricted to extracted
/// tables no DMN analyzer can ingest — the four witness classes, plus
/// the near-floor table where the scripted conventional review catches
/// every injected fault.
pub(super) fn probe_rule_lens_extracted() -> Vec<Expectation> {
    probe_rule_lens_witnesses()
        .into_iter()
        .chain([Expectation {
            fault: "floor-near-trivial".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — at the near-floor table the conventional \
                 scripted review catches every injected fault; the probe must \
                 show zero unique detections here (design Decision 2 / rule-lens \
                 precedent)"
                .into(),
        }])
        .collect()
}

/// The coverage-gap and overlap witness rows.
pub(super) fn probe_rule_lens_gaps() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "gap".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "a coverage gap in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named gap finding under \
                     the extracted-table scope"
                .into(),
        },
        Expectation {
            fault: "overlap".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "overlapping rules in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named overlap finding \
                     under the extracted-table scope"
                .into(),
        },
    ]
}

/// The four witness classes the extracted-table lens must name.
pub(super) fn probe_rule_lens_witnesses() -> Vec<Expectation> {
    probe_rule_lens_gaps()
        .into_iter()
        .chain([
            Expectation {
                fault: "shadow".into(),
                series_class: "behavioral".into(),
                catherine_class: "finding".into(),
                detail: "a shadowed rule in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named shadow finding \
                     under the extracted-table scope"
                    .into(),
            },
            Expectation {
                fault: "derivable".into(),
                series_class: "behavioral".into(),
                catherine_class: "finding".into(),
                detail: "a derivable rule in an extracted table outside DMN-analyzer \
                     ingest; the rule-lens witness class is a named derivable finding \
                     under the extracted-table scope"
                    .into(),
            },
        ])
        .collect()
}

/// probe-naturality: the declared instance map against the crm/warehouse
/// sync defect — the floor control where `distinguish` already catches
/// the fault, plus the wrong-question arm where the same fault is
/// detected but the mapping-cell incoherence is not named. The uniqueness
/// axis cannot credit a better answer: with both arms firing, parity
/// derives `killed`, and the named gap is the recorded contribution.
pub(super) fn probe_naturality() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "floor-sync-defect".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — the crm/warehouse `a3→w3` sync defect is already \
                     detected by the executed `distinguish` baseline (it reports the pair \
                     distinguishable on `account#2`, left=1, right=0); the naturality probe \
                     must show zero unique detections here (design Decision 5)"
                .into(),
        },
        Expectation {
            fault: "wrong-question".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "the same `a3→w3` fault: `distinguish` answers \"instances differ\" on \
                     `account#2` but never says which claimed mapping cell is incoherent; \
                     naturality names arrow `account_owner` and source row `account#2`. \
                     Detection is parity (both arms fire), so the uniqueness axis derives \
                     `killed`; the wrong-question gap is the recorded contribution and no \
                     unique detection is claimed"
                .into(),
        },
    ]
}

/// probe-composition: the billing/shipping seam — the floor control on a
/// dangling cell the single-document check already reports, plus the
/// seam-violation arm whose hand-merged baseline detects the same row.
/// Detection parity is the pinned expectation: the value claim under probe
/// is eliminating the error-prone merge step, which is a named gap on the
/// entry, never a detection claim (design Decision 5).
pub(super) fn probe_composition() -> Vec<Expectation> {
    vec![
        Expectation {
            fault: "floor-dangling-cell".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "floor control — a dangling `invoice_region` cell is already \
                     reported by the single-document check (exit 4); gluing the \
                     document with itself must show zero unique detections here \
                     (design Decision 5)"
                .into(),
        },
        Expectation {
            fault: "seam-violation".into(),
            series_class: "behavioral".into(),
            catherine_class: "finding".into(),
            detail: "the billing/shipping seam: each document alone is clean, the \
                     joint run reports order `o3` invoiced north and dispatched \
                     south with the seam equation's declaring document attributed. \
                     The committed hand-merged baseline detects the same row, so the \
                     uniqueness axis derives parity (killed); the eliminated \
                     hand-merge step is recorded as a named gap, never a detection"
                .into(),
        },
    ]
}

/// probe-coproduct: the G3 claim — a rogue variant row and a dual-covered
/// row under a declared closed coproduct are accepted silently today, so
/// the coproduct check's detections are unique against the executed
/// conventional arm; rows are the pre-registered catalog verbatim and in
/// order (control row first), from `fixtures/probes/coproduct/README.md`.
pub(super) fn probe_coproduct() -> Vec<Expectation> {
    vec![coproduct_floor_control()]
        .into_iter()
        .chain(coproduct_payment_faults())
        .collect()
}

pub(super) fn coproduct_floor_control() -> Expectation {
    Expectation {
        fault: "floor-keys-caught".into(),
        series_class: "behavioral".into(),
        catherine_class: "finding".into(),
        detail: "floor control — two variant rows (`Card` c1, c2) collapse onto one \
                 coproduct row (`Method` card), a defect the incumbent `schema.keys` \
                 injectivity check names in full (violation, key 0, rows [0, 1]); the \
                 coproduct surface (coverage/disjointness of `Method`'s rows) cannot \
                 express injection injectivity, so the probe must show zero unique \
                 detections here (design Decision 5)"
            .into(),
    }
}

/// The rogue-variant arm: a fourth variant row outside the declared
/// injections' image (gap G3).
pub(super) fn coproduct_rogue_variant() -> Expectation {
    Expectation {
        fault: "rogue-variant".into(),
        series_class: "behavioral".into(),
        catherine_class: "finding".into(),
        detail: "a rogue fourth variant — the `voucher` row of `Method` lies in the \
                 image of no declared injection of the accepted `Method` coproduct \
                 (`card_inj`, `transfer_inj`, `cash_inj`); the pre-registered \
                 expectation is one violation/executed naming the uncovered recipient \
                 row (gap G3: today the row is silently accepted, exit 0, findings [])"
            .into(),
    }
}

/// The payments documents: rogue and dual-covered fault arms, clean table.
pub(super) fn coproduct_payment_faults() -> Vec<Expectation> {
    vec![coproduct_rogue_variant()]
        .into_iter()
        .chain([
            Expectation {
                fault: "dual-covered".into(),
                series_class: "behavioral".into(),
                catherine_class: "finding".into(),
                detail: "a dual-covered row — `Method` row `shared` lies in the image of both \
                     `card_inj` and `transfer_inj`, falsifying the declared disjointness of \
                     an accepted coproduct; the pre-registered expectation is one \
                     violation/executed naming the row and both injections (today silently \
                     accepted, exit 0, findings [])"
                    .into(),
            },
            Expectation {
                fault: "clean-closed-table".into(),
                series_class: "behavioral".into(),
                catherine_class: "finding".into(),
                detail: "a clean closed table — `Method` = card | transfer | cash with every row \
                     covered exactly once; the pre-registered expectation is zero findings \
                     (the negative side of both defect arms: the check must not fire on a \
                     correct declaration)"
                    .into(),
            },
        ])
        .collect()
}
