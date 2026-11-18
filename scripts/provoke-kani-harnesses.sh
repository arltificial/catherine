#!/usr/bin/env bash
# Purpose: the phase-1 RED provocation evidence for the Kani lane
# (openspec change `add-kani-verification-lane`, tasks.md task 3) — EACH
# declared harness is seeded with a kernel defect in a SCRATCH copy and
# must produce a concrete Kani counterexample; a harness that cannot be
# provoked proves nothing and is rewritten, not waved through (design
# Decision 4). The unseeded side of the evidence is the lane itself:
# `scripts/check-kani-lane.sh` on unmutated kernel code reports proved.
#
# Responsibilities: for each case — copy `crates/catherine-core` to a
# gitignored scratch dir, apply ONE exact-string mutation (aborting
# loudly if the anchor is missing or not unique), run the corresponding
# harness under the pinned checker, and require the run to FAIL
# (`VERIFICATION:- FAILED`) with a counterexample. Seeds, one per
# harness:
#
# - build_panic_freedom ← typing-violation report arithmetic overflowed
#   (instance.rs): the build's violation branch must panic the build.
# - check_sortedness_panic_freedom ← `check`'s canonical sort replaced by
#   a reverse: the diff-stable sortedness invariant must fire.
# - check_equation_panic_freedom ← off-by-one iteration bound in the
#   equation walk: an out-of-bounds panic the harness must catch.
# - check_key_panic_freedom ← off-by-one table index in key_tuple: the
#   key walk must panic on the last row.
# - check_coproduct_panic_freedom ← absurd initial capacity in the
#   coproduct walk: allocation panic the harness must catch.
# - check_pullback_panic_freedom ← off-by-one table index in
#   leg_values: the pullback walk must panic on the last apex row.
# - soundness_equation ← the equation witness's lhs/rhs swapped at
#   construction: the independent re-evaluation must catch it.
# - soundness_key ← the key witness's rows un-ascending: the re-derived
#   ascending-pair claim must catch it.
# - soundness_coproduct ← the coproduct witness naming row+1: the
#   re-derived coverage claim must catch it on the disjointness input.
# - soundness_pullback ← the commutes witness's lhs/rhs swapped: the
#   re-derived disagreement must catch it.
# - soundness_dangling ← the dangling witness naming row+1: the
#   re-derived unresolved-cell claim must catch it.
#
# Rationale: provocation runs in scratch copies under target/ (never the
# working tree), so the seeds never touch committed code. Anchor strings
# are exact and counted — a refactor that moves them fails this script
# honestly instead of provoking nothing.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
	echo "usage: $0" >&2
	exit 2
}

fail() {
	echo "provocation: $1" >&2
	exit 1
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || fail "$1 not found on PATH"
}

pin_release() {
	awk -F'"' '/^release = /{print $2; exit}' kani-version.toml
}

scratch_root="$ROOT/target/kani-provocation"

# copy <case> — fresh scratch copy of the crate, detached from any
# parent workspace (the scratch lives under the repo's target/, and
# cargo's walk-up would otherwise refuse to run a non-member package).
copy() {
	local case="$1"
	rm -rf "$scratch_root/$case"
	mkdir -p "$scratch_root"
	cp -r "$ROOT/crates/catherine-core" "$scratch_root/$case"
	printf '\n[workspace]\n' >> "$scratch_root/$case/Cargo.toml"
}

# patch <case> <file> <expected-count> — read the exact anchor and
# replacement from stdin (two blocks separated by a line with only
# "----"), apply exactly once.
patch() {
	local case="$1" file="$2" expected="$3"
	python3 - "$scratch_root/$case/$file" "$expected" <<'PY'
import sys
path, expected = sys.argv[1], int(sys.argv[2])
blocks = sys.stdin.read().split("\n----\n")
if len(blocks) != 2:
    sys.exit("anchor stdin must be exactly two blocks")
anchor, replacement = blocks[0], blocks[1].rstrip("\n")
text = open(path).read()
count = text.count(anchor)
if count != expected:
    sys.exit(f"anchor found {count} times in {path}, expected {expected}")
open(path, "w").write(text.replace(anchor, replacement, 1))
PY
}

# run <case> <harness> — the seeded harness MUST fail with a
# counterexample; anything else is an unprovokable harness.
run() {
	local case="$1" harness="$2"
	local out
	out="$(cd "$scratch_root/$case" \
		&& TMPDIR="$scratch_root/$case/target" \
		cargo kani -p catherine-core --default-unwind 16 --harness "$harness" 2>&1)" || true
	if printf '%s\n' "$out" | grep -qi '^VERIFICATION:- FAILED'; then
		echo "provoked: $harness (seed: $case) — counterexample produced"
	else
		echo "$out" | tail -20
		fail "harness $harness could NOT be provoked by seed $case — rewrite the harness, do not wave it through"
	fi
}

[[ $# -eq 0 ]] || usage
require_cmd cargo
require_cmd python3
INSTALLED="$(cargo kani --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
[ "$INSTALLED" = "$(pin_release)" ] || fail "pin mismatch: pinned $(pin_release), installed ${INSTALLED:-none}"

copy prov-build-overflow
patch prov-build-overflow src/instance.rs 1 <<'PY'
                        Some(value) => typing_violations.push(TypingViolation {
                            arrow: name.clone(),
                            src: RowId(idx),
                            value,
                        }),
----
                        Some(value) => typing_violations.push(TypingViolation {
                            arrow: name.clone(),
                            src: RowId(idx),
                            value: RowId(value.0 * usize::MAX),
                        }),
PY
run prov-build-overflow build_panic_freedom

copy prov-check-sort
patch prov-check-sort src/checks.rs 1 <<'PY'
    findings.sort();
    findings
----
    findings.reverse();
    findings
PY
run prov-check-sort check_sortedness_panic_freedom

copy prov-eq-off-by-one
patch prov-eq-off-by-one src/checks.rs 1 <<'PY'
        let Some(rows) = instance.rows().get(&arrow.src) else {
            continue;
        };
        for &row in rows {
----
        let Some(rows) = instance.rows().get(&arrow.src) else {
            continue;
        };
        for i in 0..=rows.len() {
            let row = rows[i];
PY
run prov-eq-off-by-one check_equation_panic_freedom

copy prov-key-off-by-one
patch prov-key-off-by-one src/checks.rs 1 <<'PY'
        tuple.push(table.get(row.0).copied().flatten()?);
----
        tuple.push(table[row.0 + 1].copied().flatten()?);
PY
run prov-key-off-by-one check_key_panic_freedom

copy prov-coproduct-capacity
patch prov-coproduct-capacity src/checks/coproducts.rs 1 <<'PY'
pub(super) fn coproduct_findings(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::new();
----
pub(super) fn coproduct_findings(schema: &Schema, instance: &TypedInstance) -> Vec<Finding> {
    let mut findings = Vec::with_capacity(usize::MAX);
PY
run prov-coproduct-capacity check_coproduct_panic_freedom

copy prov-pullback-off-by-one
patch prov-pullback-off-by-one src/checks/pullbacks.rs 1 <<'PY'
        let left = self.p1_table.get(row.0).copied().flatten()?;
----
        let left = self.p1_table[row.0 + 1].copied().flatten()?;
PY
run prov-pullback-off-by-one check_pullback_panic_freedom

copy prov-soundness-eq-swap
patch prov-soundness-eq-swap src/checks.rs 1 <<'PY'
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: index,
            row,
            lhs,
            rhs,
        })),
----
        kind: FindingKind::Violation(ViolationWitness::Equation(EquationWitness {
            equation: index,
            row,
            lhs: rhs,
            rhs: lhs,
        })),
PY
run prov-soundness-eq-swap soundness_equation

copy prov-soundness-key-swap
patch prov-soundness-key-swap src/checks.rs 1 <<'PY'
                    kind: FindingKind::Violation(ViolationWitness::Key(KeyWitness {
                        key: index,
                        rows: [lo, hi],
                    })),
----
                    kind: FindingKind::Violation(ViolationWitness::Key(KeyWitness {
                        key: index,
                        rows: [hi, lo],
                    })),
PY
run prov-soundness-key-swap soundness_key

copy prov-soundness-coproduct-row
patch prov-soundness-coproduct-row src/checks/coproducts.rs 1 <<'PY'
    let injections = covering_injections(images, row);
    (injections.len() != 1).then(|| witness_finding(label, index, row, injections))
----
    let injections = covering_injections(images, row);
    (injections.len() != 1).then(|| witness_finding(label, index, RowId(row.0 + 1), injections))
PY
run prov-soundness-coproduct-row soundness_coproduct

copy prov-soundness-pullback-swap
patch prov-soundness-pullback-swap src/checks/pullbacks.rs 1 <<'PY'
            PullbackWitness::Commutes {
                pullback: index,
                row,
                lhs,
                rhs,
            },
----
            PullbackWitness::Commutes {
                pullback: index,
                row,
                lhs: rhs,
                rhs: lhs,
            },
PY
run prov-soundness-pullback-swap soundness_pullback

copy prov-soundness-dangling-row
patch prov-soundness-dangling-row src/checks.rs 1 <<'PY'
                    kind: FindingKind::Dangling(DanglingWitness {
                        arrow: arrow.clone(),
                        src: RowId(idx),
                    }),
----
                    kind: FindingKind::Dangling(DanglingWitness {
                        arrow: arrow.clone(),
                        src: RowId(idx + 1),
                    }),
PY
run prov-soundness-dangling-row soundness_dangling

echo "provocation: all declared harnesses provoked (concrete counterexamples above)"
