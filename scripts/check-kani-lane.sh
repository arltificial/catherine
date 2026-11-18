#!/usr/bin/env bash
# Purpose: the Kani verification lane gate (openspec change
# `add-kani-verification-lane`, beads `catherine-xdi`) — the executable
# half of the `just verify` recipe: provisioning check against the
# committed pin, one bounded-model-checking run per declared harness,
# the three-valued outcome (proved / counterexample / undetermined) per
# harness, and the disposition rule.
#
# Responsibilities: (1) read `kani-version.toml` and refuse to run under
# any other installed release (the pin is the reproducibility contract);
# (2) run `cargo kani` per declared harness with the repo's standard
# unwinding bound; (3) parse the per-harness `VERIFICATION:-` line — a
# run with no such line is a harness error, never silently green; (4)
# cross-check every declared harness produced an outcome (no drops);
# (5) exit nonzero on a counterexample, a harness error, or an
# undetermined outcome whose disposition is not recorded in
# docs/kani-lane.md (`<harness>: undetermined — ...`); exit 0 only when
# every declared harness is proved or undetermined-with-disposition.
#
# Rationale: deliberately NOT part of `just ci` (the Lean-gate
# precedent: a toolchain every checkout cannot assume must not gate
# every commit). Outcomes are echoed verbatim so the lane's aggregate
# names any harness that did not prove (spec: undetermined is loud,
# never green).

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
	echo "usage: $0" >&2
	exit 2
}

fail() {
	echo "kani lane: $1" >&2
	exit 1
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || fail "$1 not found on PATH"
}

pin_release() {
	local release
	release="$(awk -F'"' '/^release = /{print $2; exit}' kani-version.toml)"
	[ -n "$release" ] || fail "kani-version.toml has no release pin"
	printf '%s' "$release"
}

installed_release() {
	local version
	version="$(cargo kani --version 2>/dev/null | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
	[ -n "$version" ] || fail "cargo kani --version produced no version"
	printf '%s' "$version"
}

# The declared harness list — keep in sync with
# crates/catherine-core/src/kani_harnesses.rs (every #[kani::proof] fn).
DECLARED_HARNESSES=(
	build_panic_freedom
	check_sortedness_panic_freedom
	check_equation_panic_freedom
	check_key_panic_freedom
	check_coproduct_panic_freedom
	check_pullback_panic_freedom
	soundness_equation
	soundness_key
	soundness_coproduct
	soundness_pullback
	soundness_dangling
)

[[ $# -eq 0 ]] || usage
require_cmd cargo
require_cmd kani

PIN="$(pin_release)"
INSTALLED="$(installed_release)"
if [ "$PIN" != "$INSTALLED" ]; then
	fail "pin mismatch: kani-version.toml pins $PIN, installed release is $INSTALLED (provision: cargo install --locked kani-verifier@$PIN && cargo kani setup)"
fi
echo "kani lane: pin OK ($INSTALLED)"

TMPDIR="$ROOT/target/tmp"
export TMPDIR
mkdir -p "$TMPDIR"

# The per-harness wall budget. A harness that cannot produce its verdict
# within it reports UNDETERMINED (cargo kani --harness-timeout) — loud,
# never silently green; each undetermined outcome needs a disposition in
# docs/kani-lane.md or the lane fails (spec requirement).
HARNESS_TIMEOUT=300

declare -a outcomes
lane_failed=0
for harness in "${DECLARED_HARNESSES[@]}"; do
	echo ":: kani harness $harness"
	out="$(cargo kani -p catherine-core --default-unwind 16 --harness-timeout "$HARNESS_TIMEOUT" --harness "$harness" 2>&1)" || true
	verdict_line="$(printf '%s\n' "$out" | grep -i '^VERIFICATION:-' | tail -1 || true)"
	if [ -z "$verdict_line" ]; then
		outcomes+=("$harness: error")
		echo "$out" | tail -20
		lane_failed=1
		continue
	fi
	echo "$out" | grep -E '^Check [0-9]+:|^SUMMARY:|successfully verified harnesses' | tail -6 || true
	case "$verdict_line" in
		*SUCCESSFUL*)
			outcomes+=("$harness: proved")
			;;
		*FAILED*)
			outcomes+=("$harness: counterexample")
			echo "$out" | tail -30
			lane_failed=1
			;;
		*UNDETERMINED*)
			if grep -qE "^${harness}: undetermined" docs/kani-lane.md; then
				outcomes+=("$harness: undetermined (disposition recorded)")
			else
				outcomes+=("$harness: undetermined (NO disposition — fails the lane)")
				echo "$out" | tail -20
				lane_failed=1
			fi
			;;
		*)
			outcomes+=("$harness: error (unrecognized verdict line: $verdict_line)")
			echo "$out" | tail -20
			lane_failed=1
			;;
	esac
done

echo "kani lane outcomes:"
for outcome in "${outcomes[@]}"; do
	echo "  $outcome"
done

# Every declared harness must have produced an outcome — no silent drops.
if [ "${#outcomes[@]}" -ne "${#DECLARED_HARNESSES[@]}" ]; then
	fail "declared ${#DECLARED_HARNESSES[@]} harnesses but collected ${#outcomes[@]} outcomes"
fi

[ "$lane_failed" -eq 0 ] || fail "lane failed (see outcomes above)"
echo "kani lane: all declared harnesses proved or undetermined-with-disposition"
