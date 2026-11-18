#!/usr/bin/env bash
# Purpose: the executable side of the `lean-kernel-model` espectacular
# contracts — one case per scenario of the capability spec, each asserting
# the observable the scenario names rather than re-running the whole
# `just lean` gate (`add-lean-kernel-model`).
#
# Responsibilities: dispatch on one case name (`twin-builds`, `parity`,
# `parity-divergence`, `soundness`, `equation-completeness`,
# `dangling-discipline`, `stop-gate`) and exit 0 only when that scenario's
# observable holds. The cases reuse the committed artifacts — the Lake
# project, the `lean-parity` oracle, `scripts/check-lean-axioms.sh`, the
# ledger decision, and the README pointer — so a contract failure names a
# real regression, not a harness accident.
#
# Rationale: the twin's scenarios are Lean-toolchain checks, not cargo
# tests, so the contracts bind through a `[[tests.shell]]` entry instead of
# the cargo runner. `parity-divergence` seeds the divergence through the
# documented `CATHERINE_BIN` seam (a wrapper that drops one finding from
# the envelope) rather than mutating the Lean tree: the scenario is that a
# mismatch is loud and attributed, and a seeded CLI-side mismatch proves
# the oracle still fails closed without a second multi-minute Lean build.
# Exit 1 is a failed scenario; exit 2 is a usage error.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

usage() {
	printf '%s\n' \
		"usage: scripts/check-lean-contract.sh <case>" \
		"" \
		"cases:" \
		"  twin-builds            the Lake project builds and the mirror round-trips" \
		"  parity                 the oracle agrees over the fixture corpus" \
		"  parity-divergence      a seeded mismatch exits 1 and names the fixture" \
		"  soundness              the soundness theorem is proved, not assumed" \
		"  equation-completeness  completeness and the label lemma are proved" \
		"  dangling-discipline    the dangling bundle is proved" \
		"  stop-gate              the hold decision is on the validated ledger" \
		"  -h, --help             show this message"
}

fail() {
	printf '❌ lean-contract: %s\n' "$*" >&2
	exit 1
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || fail "$1 not found on PATH"
}

axiom_line() {
	local declaration=$1
	(cd lean && lake env lean Catherine/Axioms.lean) |
		grep -F "'Catherine.Proofs.${declaration}'" || true
}

case_twin_builds() {
	( cd lean && lake build )
	local tmp
	tmp="$(mktemp -d)"
	trap 'rm -rf "$tmp"' RETURN
	# The mirror round-trips: the oracle projects every parity document to
	# the mirror JSON and decodes it again before the finding comparison.
	( cd lean && lake exe lean-parity --emit-mirror "$tmp/mirror" --corpus "$ROOT/fixtures/parity" >/dev/null )
	find "$tmp/mirror" -name '*.json' -print -quit | grep -q . ||
		fail "the mirror projection wrote no JSON"
	# The Cargo workspace gains no dependency on Lean.
	if grep -rInE '(^|[^a-zA-Z])lean([^a-zA-Z]|$)' Cargo.toml crates/*/Cargo.toml; then
		fail "a Cargo manifest references Lean"
	fi
	printf '✅ twin-builds: lake build green; mirror JSON emitted and decoded; no Cargo manifest references Lean\n'
}

case_parity() {
	local tmp
	tmp="$(mktemp -d)"
	trap 'rm -rf "$tmp"' RETURN
	require_cmd python3
	( cd lean && lake exe lean-parity --report "$tmp/report.json" >/dev/null )
	python3 - "$tmp/report.json" <<'PY'
import json
import sys

report = json.load(open(sys.argv[1]))
corpus = report["corpus"]
agreement = report["agreement"]
assert report["ok"] is True, "the report is not ok"
assert not report["divergences"], "divergences are non-empty"
assert not report["build_mismatches"], "build mismatches are non-empty"
assert corpus["rust_checkable"] > 0, "no document was compared against Rust"
assert corpus["total"] > corpus["rust_checkable"], "the corpus collapsed"
assert agreement["findings_bearing_agreeing"] > 0, "no findings-bearing document agreed"
PY
	printf '✅ parity: %s documents, %s Rust-compared, %s findings-bearing agreeing, 0 divergences\n' \
		"$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["corpus"]["total"])' "$tmp/report.json")" \
		"$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["corpus"]["rust_checkable"])' "$tmp/report.json")" \
		"$(python3 -c 'import json,sys;print(json.load(open(sys.argv[1]))["agreement"]["findings_bearing_agreeing"])' "$tmp/report.json")"
}

case_parity_divergence() {
	local tmp
	tmp="$(mktemp -d)"
	trap 'rm -rf "$tmp"' RETURN
	require_cmd python3
	local real="$ROOT/target/debug/catherine"
	if [[ -n ${CATHERINE_BIN:-} ]]; then
		real="$CATHERINE_BIN"
	fi
	[[ -x $real ]] || fail "catherine binary not found at $real (cargo build -p catherine-cli)"
	mkdir -p "$tmp/corpus/parity"
	cp -r fixtures/parity/dangling "$tmp/corpus/parity/dangling"

	cat >"$tmp/catherine-drop-finding" <<'WRAPPER'
#!/usr/bin/env bash
set -uo pipefail
out="$("$REAL_CATHERINE" "$@")"
code=$?
if [[ $code -ne 0 ]] && printf '%s' "$out" | grep -q '"findings"'; then
	out="$(printf '%s' "$out" | python3 -c 'import json, sys
document = json.load(sys.stdin)
findings = document["data"]["findings"]
if findings:
    findings.pop(0)
print(json.dumps(document))')"
fi
printf '%s' "$out"
exit "$code"
WRAPPER
	chmod +x "$tmp/catherine-drop-finding"

	local output code=0
	set +e
	output="$(
		cd lean && REAL_CATHERINE="$real" CATHERINE_BIN="$tmp/catherine-drop-finding" \
			lake exe lean-parity --corpus "$tmp/corpus" 2>&1
	)"
	code=$?
	set -e
	[[ $code -eq 1 ]] || fail "expected exit 1 from a seeded divergence, got $code"
	printf '%s' "$output" | grep -q 'parity/dangling' ||
		fail "the divergence did not name the fixture"
	printf '%s' "$output" | grep -q 'divergence' ||
		fail "the divergence was not reported"
	printf '✅ parity-divergence: a seeded finding mismatch exits 1, names parity/dangling, and stays loud\n'
}

case_soundness() {
	scripts/check-lean-axioms.sh >/dev/null
	local line
	line="$(axiom_line soundness)"
	[[ -n $line ]] || fail "the audit did not report Catherine.Proofs.soundness"
	[[ $line != *sorryAx* ]] || fail "soundness depends on sorryAx"
	[[ $line != *"'cheat'"* ]] || fail "soundness depends on a declared assumption"
	printf '✅ soundness: %s\n' "$line"
}

case_equation_completeness() {
	scripts/check-lean-axioms.sh >/dev/null
	local line
	line="$(axiom_line equation_completeness)"
	[[ -n $line ]] || fail "the audit did not report Catherine.Proofs.equation_completeness"
	[[ $line != *sorryAx* ]] || fail "equation_completeness depends on sorryAx"
	local label
	label="$(axiom_line labelFor_agrees)"
	[[ -n $label ]] || fail "the audit did not report Catherine.Proofs.labelFor_agrees"
	printf '✅ equation-completeness: %s / %s\n' "$line" "$label"
}

case_dangling_discipline() {
	scripts/check-lean-axioms.sh >/dev/null
	local line
	line="$(axiom_line dangling_discipline)"
	[[ -n $line ]] || fail "the audit did not report Catherine.Proofs.dangling_discipline"
	[[ $line != *sorryAx* ]] || fail "dangling_discipline depends on sorryAx"
	printf '✅ dangling-discipline: %s\n' "$line"
}

case_stop_gate() {
	require_cmd python3
	cargo run -q -p catherine-ledger --bin ledger-check -- ledger/lean
	local entry="ledger/lean/0001-cbef9219.json"
	[[ -f $entry ]] || fail "the gate decision entry is missing at $entry"
	python3 - "$entry" <<'PY'
import json
import sys

entry = json.load(open(sys.argv[1]))
assert entry["kind"] == "decision", "the entry is not a decision"
outcome = entry["decision"]["outcome"]
assert outcome in {"extend", "hold", "retire"}, f"unknown outcome {outcome!r}"
assert entry["decision"].get("scope"), "the decision states no scope"
PY
	grep -q 'ledger/lean/0001-cbef9219.json' lean/README.md ||
		fail "lean/README.md does not link the gate decision"
	printf '✅ stop-gate: ledger/lean validates and records a decision, linked from lean/README.md\n'
}

[[ $# -eq 1 ]] || {
	usage >&2
	exit 2
}

case $1 in
twin-builds) case_twin_builds ;;
parity) case_parity ;;
parity-divergence) case_parity_divergence ;;
soundness) case_soundness ;;
equation-completeness) case_equation_completeness ;;
dangling-discipline) case_dangling_discipline ;;
stop-gate) case_stop_gate ;;
-h | --help) usage ;;
*)
	printf '❌ lean-contract: unknown case `%s`\n' "$1" >&2
	usage >&2
	exit 2
	;;
esac