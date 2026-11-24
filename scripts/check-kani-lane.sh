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
# unwinding bound, ALWAYS inside a systemd user scope capped at
# MEMORY_BUDGET_MB — a CBMC blowup kills the harness run, never the
# machine (beads catherine-amh: an uncapped run OOM-killed the
# workstation at 15.4 GB on 2026-11-18); (3) parse the per-harness
# `VERIFICATION:-` line — a run with no such line is a harness error,
# never silently green; a CBMC timeout (emitted as `FAILED`) and a
# budget-kill (SIGKILL at the memory cap) are classified UNDETERMINED,
# not counterexamples; (4) cross-check every declared harness produced
# an outcome (no drops) and record peak RSS + wall per harness into
# target/kani-timings.txt; (5) exit nonzero on a counterexample, a
# harness error, or an undetermined outcome whose disposition is not
# recorded in docs/kani-lane.md (`<harness>: undetermined — ...`); exit
# 0 only when every declared harness is proved or
# undetermined-with-disposition.
#
# Rationale: deliberately NOT part of `just ci` (the Lean-gate
# precedent: a toolchain every checkout cannot assume must not gate
# every commit). Outcomes are echoed verbatim so the lane's aggregate
# names any harness that did not prove (spec: undetermined is loud,
# never green). Set KANI_HARNESS_FILTER=<regex> for a maintenance
# subset run — same enforcement and classification, only fewer
# harnesses; it never weakens the disposition rule.

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
# crates/catherine-kani/src/lib.rs (every #[kani::proof] fn).
DECLARED_HARNESSES=(
	build_panic_freedom
	build_panic_freedom_equation
	build_panic_freedom_key
	build_panic_freedom_coproduct
	build_panic_freedom_pullback
	check_sortedness_panic_freedom
	check_dangling_panic_freedom
	check_equation_panic_freedom
	check_key_panic_freedom
	check_coproduct_panic_freedom
	check_pullback_panic_freedom
	soundness_equation
	soundness_equation_rhs
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
# within it reports UNDETERMINED (cargo kani --harness-timeout, which
# requires -Z unstable-options) — loud, never silently green; each
# undetermined outcome needs a disposition in docs/kani-lane.md or the
# lane fails (spec requirement).
HARNESS_TIMEOUT=300

# The per-harness memory budget. cargo kani always runs inside a
# systemd user scope capped at MEMORY_BUDGET_MB with swap disabled, so
# the OOM-killer's target is the harness run, not the workstation.
# Overridable via KANI_MEMORY_BUDGET_MB; a machine without a working
# `systemd-run --user` cannot host a capped run, so the lane refuses
# rather than run unbounded (catherine-amh acceptance criterion 2).
MEMORY_BUDGET_MB="${KANI_MEMORY_BUDGET_MB:-8192}"
if ! systemd-run --user --scope -p MemoryMax=64M true >/dev/null 2>&1; then
	fail "no working systemd user scope — refusing to run cargo kani without a memory cap (catherine-amh: uncapped runs OOM-kill the machine)"
fi

# Per-harness resource record (peak RSS + wall) — catherine-amh
# deliverable 1; one line per harness per lane run.
TIMINGS="$ROOT/target/kani-timings.txt"
: > "$TIMINGS"

# Optional maintenance subset (same rules, fewer harnesses).
FILTER="${KANI_HARNESS_FILTER:-}"
run_harnesses=()
for harness in "${DECLARED_HARNESSES[@]}"; do
	if [ -z "$FILTER" ] || [[ "$harness" =~ $FILTER ]]; then
		run_harnesses+=("$harness")
	fi
done
[ "${#run_harnesses[@]}" -gt 0 ] || fail "KANI_HARNESS_FILTER=$FILTER matched none of the declared harnesses"

declare -a outcomes
lane_failed=0
for harness in "${run_harnesses[@]}"; do
	echo ":: kani harness $harness"
	timefile="$TMPDIR/$harness.time"
	logfile="$TMPDIR/$harness.log"
	set +e
		systemd-run --user --scope -p "MemoryMax=${MEMORY_BUDGET_MB}M" -p MemorySwapMax=0 \
		/usr/bin/time -v -o "$timefile" \
		cargo kani -Z unstable-options -p catherine-kani --default-unwind 16 \
		--harness-timeout "$HARNESS_TIMEOUT" --harness "$harness" > "$logfile" 2>&1
	rc=$?
	set -e
	out="$(cat "$logfile")"

	verdict_line="$(printf '%s\n' "$out" | grep -i '^VERIFICATION:-' | tail -1 || true)"
	peak_kb="$(grep -oE 'Maximum resident set size \(kbytes\): [0-9]+' "$timefile" 2>/dev/null | grep -oE '[0-9]+$' | tail -1 || true)"
	wall="$(grep -oE 'Elapsed \(wall clock\) time .*: .*' "$timefile" 2>/dev/null | sed 's/.*: //' | tail -1 || true)"

	# A CBMC timeout reaches the solver's deadline and prints FAILED,
	# indistinguishable from a counterexample by the verdict line
	# alone; the "CBMC timed out" banner is the discriminator. A
	# budget-kill surfaces as rc 137 (SIGKILL at MemoryMax). Both are
	# resource limits, not proof results: UNDETERMINED with a required
	# disposition — loud, never silently green, never "counterexample".
	# A MemoryMax breach TERM-inates the whole scope (rc 143) before the
	# kernel's cgroup OOM-kill lands the SIGKILL (rc 137) — both are the
	# budget arm (beads catherine-6g2: soundness_equation died at the
	# 2026-11-23 reduction wall as a scope TERM).
	if [ "$rc" -eq 137 ] || [ "$rc" -eq 143 ] \
		|| grep -q 'exited with non-zero status 137' "$timefile" 2>/dev/null \
		|| grep -q 'exited with non-zero status 143' "$timefile" 2>/dev/null; then
		kind="budget"
	elif [ -n "$verdict_line" ] && printf '%s' "$verdict_line" | grep -qi FAILED \
		&& printf '%s' "$out" | grep -qi 'CBMC timed out'; then
		kind="timeout"
	elif [ -z "$verdict_line" ]; then
		kind="error"
	elif printf '%s' "$verdict_line" | grep -qi SUCCESSFUL; then
		kind="proved"
	elif printf '%s' "$verdict_line" | grep -qi FAILED; then
		kind="counterexample"
	elif printf '%s' "$verdict_line" | grep -qi UNDETERMINED; then
		kind="undetermined"
	else
		kind="error"
	fi

	case "$kind" in
		proved)
			outcomes+=("$harness: proved")
			;;
		counterexample)
			outcomes+=("$harness: counterexample")
			echo "$out" | tail -30
			lane_failed=1
			;;
		timeout|undetermined|budget)
			case "$kind" in
				timeout)   reason="undetermined (CBMC timed out at ${HARNESS_TIMEOUT}s)" ;;
				budget)    reason="undetermined (memory budget ${MEMORY_BUDGET_MB}MB exceeded)" ;;
				undetermined) reason="undetermined" ;;
			esac
			if grep -qE "^(- )?${harness}: undetermined" docs/kani-lane.md; then
				outcomes+=("$harness: $reason (disposition recorded)")
			else
				outcomes+=("$harness: $reason (NO disposition — fails the lane)")
				echo "$out" | tail -20
				lane_failed=1
			fi
			;;
		error)
			outcomes+=("$harness: error")
			echo "$out" | tail -20
			lane_failed=1
			;;
	esac

	printf '%s exit=%s peak_KiB=%s wall=%s outcome=%s\n' \
		"$harness" "$rc" "${peak_kb:-unknown}" "${wall:-unknown}" \
		"${outcomes[${#outcomes[@]}-1]#"$harness": }" >> "$TIMINGS"
done

echo "kani lane outcomes:"
for outcome in "${outcomes[@]}"; do
	echo "  $outcome"
done

# Every declared harness must have produced an outcome — no silent drops.
if [ "${#outcomes[@]}" -ne "${#run_harnesses[@]}" ]; then
	fail "matched ${#run_harnesses[@]} harnesses but collected ${#outcomes[@]} outcomes"
fi

[ "$lane_failed" -eq 0 ] || fail "lane failed (see outcomes above)"
if [ -n "$FILTER" ]; then
	echo "kani lane: filtered run ($FILTER) — matched harnesses proved or undetermined-with-disposition"
else
	echo "kani lane: all declared harnesses proved or undetermined-with-disposition"
fi
