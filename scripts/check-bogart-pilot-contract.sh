#!/usr/bin/env bash
# Purpose: the executable side of the `bogart-pilot` espectacular
# contracts — one case per scenario of the capability spec, each
# asserting the observable the scenario names rather than re-running
# bogart's CI (`add-bogart-pilot`, beads `catherine-hc6`).
#
# Responsibilities: dispatch on one case name (`drifted-anchor-is-caught`,
# `ci-fails-on-a-real-violation`, `noise-is-accounted`) and exit 0 only
# when that scenario's observable holds. The pilot's execution surface
# lives in the bogart repo (its CI runs the pinned catherine CLI over the
# committed models), so the cases exercise the committed artifacts this
# repo owns — the pilot models, the CI wiring, the report and evidence —
# plus, for the drifted-anchor scenario, the real RED fixture run
# against a locally built CLI at the sibling checkout. Exit 1 is a failed
# scenario; exit 2 is a usage error.
#
# Rationale: the sibling bogart checkout is part of the pilot's ground
# truth on this machine (the same layout the probe sources and corpus
# fixtures already reference); the shell cases run only under
# `ah check --run-tests` (the plain `ah check` gate validates contract
# existence, mirroring the Lean contracts' treatment).

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BOGART="${BOGART_REPO:-/var/home/sasha/para/areas/dev/gh/at/bogart}"

usage() {
	echo "usage: $0 <drifted-anchor-is-caught|ci-fails-on-a-real-violation|noise-is-accounted>" >&2
	exit 2
}

fail() {
	echo "bogart-pilot contract failed: $1" >&2
	exit 1
}

ensure_cli() {
	local bin="$ROOT/target/debug/catherine"
	[ -x "$bin" ] || cargo build -p catherine-cli >/dev/null 2>&1
	[ -x "$bin" ] || fail "catherine CLI not buildable"
	printf '%s' "$bin"
}

case_drifted_anchor_is_caught() {
	local bin models
	bin="$(ensure_cli)"
	models="$BOGART/test-resources/catherine-pilot"
	[ -f "$models/red_drifted_anchor.ct.json" ] \
		|| fail "bogart pilot red fixture missing at $models (sibling checkout required)"
	local code
	set +e
	"$bin" check "$models/red_drifted_anchor.ct.json" >/dev/null 2>&1
	code=$?
	set -e
	[ "$code" -eq 3 ] || fail "red fixture expected the violation exit code 3, got $code"
}

case_ci_fails_on_a_real_violation() {
	local ci="$BOGART/.github/workflows/ci.yml"
	[ -f "$ci" ] || fail "bogart ci.yml missing (sibling checkout required)"
	grep -q 'ref: 5b3f1aa' "$ci" || fail "ci.yml does not pin catherine 5b3f1aa"
	grep -q 'expected the violation exit code 3' "$ci" \
		|| fail "ci.yml lost the red-fixture exit-3 assertion"
	grep -q 'expected the lost-data exit code 8' "$ci" \
		|| fail "ci.yml lost the model4 exit-8 assertion"
	grep -q 'expected the underconstraint exit code 6' "$ci" \
		|| fail "ci.yml lost the model5 red exit-6 assertion"
}

case_noise_is_accounted() {
	local evidence="$BOGART/docs/evidence/catherine-pilot-2026-11-11.json"
	local report="$BOGART/docs/catherine-pilot-2026-11-11.md"
	[ -f "$evidence" ] || fail "pilot evidence json missing (sibling checkout required)"
	[ -f "$report" ] || fail "pilot report missing (sibling checkout required)"
	python3 - "$evidence" <<'PY'
import json, sys
d = json.load(open(sys.argv[1]))
triage = d.get("triage", {})
if triage.get("accounting") is None:
    sys.exit("triage accounting missing")
if triage.get("real", {}).get("count") is None or triage.get("noise", {}).get("count") is None:
    sys.exit("triage counts missing")
if d.get("closure_2026_11_18", {}).get("census_2026_11_18") is None:
    sys.exit("closure census missing")
if d["closure_2026_11_18"]["model4"].get("real_defects") != 0:
    sys.exit("model4 real_defects not recorded as 0")
if d["closure_2026_11_18"]["model5"]["red_fixture"].get("exit") != 6:
    sys.exit("model5 red exit not recorded as 6")
PY
	grep -q 'Noise (67 findings)' "$report" || fail "report lost the noise accounting section"
	grep -q '## Closure (2026-11-18)' "$report" || fail "report lost the closure section"
}

[[ $# -eq 1 ]] || usage

case $1 in
	drifted-anchor-is-caught) case_drifted_anchor_is_caught ;;
	ci-fails-on-a-real-violation) case_ci_fails_on_a_real_violation ;;
	noise-is-accounted) case_noise_is_accounted ;;
	*) usage ;;
esac
