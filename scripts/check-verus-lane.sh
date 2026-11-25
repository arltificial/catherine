#!/usr/bin/env bash
# Purpose: the Verus verification lane gate (openspec change
# `add-verus-verification-lane`) — the executable half of the
# `just verify-verus` recipe, mirroring scripts/check-kani-lane.sh.
#
# Responsibilities: (1) read `verus-version.toml` and refuse to run
# under any other installed release (the pin is the reproducibility
# contract); provisioning downloads the pinned zip, checks its sha256,
# installs the pinned rustup toolchain, and is idempotent;
# (2) verify every model proof unit in crates/catherine-verus/model/
# with the pinned release (--crate-type lib), parsing the
# `verification results::` line — a run with no such line is a unit
# error, never silently green; rlimit exhaustion is classified
# EXHAUSTED, not failed (three-valued outcomes: VERIFIED / ERROR /
# EXHAUSTED);
# (3) run the parity gate: the trusted driver
# (crates/catherine-verus/parity) and the kernel CLI
# (`catherine check --json`) each walk every corpus document in
# crates/catherine-verus/fixtures/parity-model/, and the normalized
# equation findings must match exactly; the corpus must emit at least
# one finding in total — the non-vacuity witness gate (two always-empty
# streams match trivially; the corpus must exercise real emissions);
# (4) run the guide tripwire corpus (crates/catherine-verus/fixtures/
# guide/): every vendored Verus-guide example verifies (or fails)
# exactly as `expectations.txt` records, measured against the same
# pinned release — an outcome drift is a toolchain or upstream regression;
# (5) cross-check every declared unit produced an outcome (no drops,
# both directions on the corpus and expectations) and record wall time
# per unit into target/verus-timings.txt;
# (6) exit nonzero on any model error, exhaustion, parity diff, empty
# corpus emission, expectation mismatch, or missing outcome.
#
# Rationale: deliberately NOT part of `just ci` (the kani-lane
# precedent: a pinned external toolchain must not gate every commit).
# No systemd memory scope, unlike the kani lane: verus runs here are
# sub-second concrete walks plus a bounded SMT phase under the release's
# own rlimits, and every invocation is wall-capped at 300 s.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

VERUS_TIMEOUT=300
PIN="verus-version.toml"
MODEL_DIR="crates/catherine-verus/model"
PARITY_DIR="crates/catherine-verus/parity"
PARITY_FIXTURES="crates/catherine-verus/fixtures/parity-model"
GUIDE_DIR="crates/catherine-verus/fixtures/guide"
TIMINGS="target/verus-timings.txt"

usage() {
	echo "usage: $0" >&2
	exit 2
}

fail() {
	echo "verus lane: $1" >&2
	exit 1
}

require_cmd() {
	command -v "$1" >/dev/null 2>&1 || fail "$1 not found on PATH"
}

pin_field() {
	awk -F'"' -v k="$1" '$0 ~ k" = " {print $2; exit}' "$PIN"
}

[ "${1:-}" = "--help" ] && usage
[ -f "$PIN" ] || fail "$PIN missing"
RELEASE="$(pin_field release)"
URL="$(pin_field url)"
SHA="$(pin_field sha256)"
TOOLCHAIN="$(pin_field toolchain)"
[ -n "$RELEASE" ] && [ -n "$URL" ] && [ -n "$SHA" ] && [ -n "$TOOLCHAIN" ] ||
	fail "$PIN is missing release/url/sha256/toolchain"

require_cmd jq
require_cmd cargo
require_cmd rustup
require_cmd sha256sum
require_cmd unzip
require_cmd timeout

# ---- provisioning -----------------------------------------------------
# The install layout mirrors the zip: <install>/ holds verus, z3, and
# the prebuilt vstd rlibs; the driver resolves them relative to itself.
VERUS_HOME="${VERUS_HOME:-$HOME/.local/share/verus}"
INSTALL="$VERUS_HOME/$RELEASE"
VERUS_BIN="$INSTALL/verus"

if [ ! -x "$VERUS_BIN" ]; then
	echo "verus lane: provisioning $RELEASE into $INSTALL"
	mkdir -p "$INSTALL" "$VERUS_HOME/tmp"
	zip="$VERUS_HOME/tmp/$RELEASE.zip"
	curl -fsSL -o "$zip" "$URL" || fail "download of $URL failed"
	echo "$SHA  $zip" | sha256sum -c - >/dev/null ||
		fail "pinned zip sha256 mismatch for $URL"
	unzip -o -q "$zip" -d "$VERUS_HOME/tmp/$RELEASE"
	# The zip unpacks one directory; move its payload into place.
	mv "$VERUS_HOME/tmp/$RELEASE"/*/* "$INSTALL"/ 2>/dev/null ||
		mv "$VERUS_HOME/tmp/$RELEASE"/* "$INSTALL"/
	rm -rf "$VERUS_HOME/tmp/$RELEASE" "$zip"
fi
"$VERUS_BIN" --version | grep -q "$RELEASE" ||
	fail "installed verus does not report the pinned release $RELEASE"

# The release pins its rustup toolchain by exact name; install it
# idempotently (minimal profile: the verifier drives rustc directly).
rustup toolchain list | grep -q "^$TOOLCHAIN" ||
	rustup toolchain install "$TOOLCHAIN" --profile minimal >/dev/null 2>&1 ||
	fail "toolchain $TOOLCHAIN install failed"

mkdir -p target
: > "$TIMINGS"

run_verus() { # <file> <outfile> [extra args...] — wall-capped; rc via $?
	local file="$1" out="$2"
	shift 2
	local t0 t1 rc
	t0=$(date +%s%N)
	timeout "$VERUS_TIMEOUT" "$VERUS_BIN" "$@" "$file" >"$out" 2>&1
	rc=$?
	t1=$(date +%s%N)
	printf '%s\t%dms\n' "$file" "$(( (t1 - t0) / 1000000 ))" >>"$TIMINGS"
	return $rc
}

# ---- 1. model proof units --------------------------------------------
echo "verus lane: model proof units ($MODEL_DIR)"
model_units=0
model_out=$(mktemp)
trap 'rm -f "$model_out"' EXIT
for unit in "$MODEL_DIR"/*.rs; do
	[ -e "$unit" ] || fail "no model proof units under $MODEL_DIR"
	rc=0
	run_verus "$unit" "$model_out" --crate-type lib || rc=$?
	out=$(cat "$model_out")
	if [ $rc -eq 124 ]; then
		status=EXHAUSTED
	else
		results=$(printf '%s\n' "$out" | grep -oE "verification results:: [0-9]+ verified, [0-9]+ errors" | tail -1 || true)
		if [ -z "$results" ]; then
			if printf '%s\n' "$out" | grep -qi "rlimit"; then
				status=EXHAUSTED
			else
				status=ERROR
			fi
		else
			errors=$(printf '%s' "$results" | sed -E 's/.*verified, ([0-9]+) errors/\1/')
			if [ "$errors" -eq 0 ]; then
				status=VERIFIED
			elif printf '%s\n' "$out" | grep -qi "rlimit"; then
				status=EXHAUSTED
			else
				status=ERROR
			fi
		fi
	fi
	case "$status" in
	VERIFIED) ;;
	*) fail "model unit $(basename "$unit"): $status
$(printf '%s\n' "$out" | tail -5)" ;;
	esac
	echo "  VERIFIED  $(basename "$unit")  ($results)"
	model_units=$((model_units + 1))
done
[ "$model_units" -gt 0 ] || fail "model produced no proof-unit outcomes"

# ---- 2. parity gate ---------------------------------------------------
echo "verus lane: parity gate ($PARITY_FIXTURES)"
( cd "$PARITY_DIR" && cargo build -q --offline 2>/dev/null || cargo build -q ) ||
	fail "parity driver build failed"
PARITY_BIN="$PARITY_DIR/target/debug/verus-parity"
[ -x "$PARITY_BIN" ] || fail "parity driver binary missing"
cargo build -q -p catherine-cli ||
	fail "kernel CLI build failed"
KERNEL_BIN="target/debug/catherine"

total_emissions=0
fixtures=0
for fixture in "$PARITY_FIXTURES"/*.ct.json; do
	[ -e "$fixture" ] || fail "no parity fixtures under $PARITY_FIXTURES"
	name=$(basename "$fixture")
	# The kernel's exit contract is nonzero when findings exist (2 + rank
	# class); the well-formedness signal is the envelope's `ok` field.
	envelope=$("$KERNEL_BIN" check --json "$fixture" 2>/dev/null) || true
	if ! printf '%s' "$envelope" | jq -e '.ok == true' >/dev/null 2>&1; then
		fail "kernel rejected corpus fixture $name: $envelope"
	fi
	kernel=$(printf '%s' "$envelope" |
		jq -r '.data.findings[]? | select(.kind=="violation" and .witness.equation != null) | "EQ \(.witness.equation) \(.witness.row) \(.witness.lhs) \(.witness.rhs) \(.label)"')
	model=$("$PARITY_BIN" "$fixture") ||
		fail "parity driver rejected corpus fixture $name (outside the verified fragment?)"
	if [ "$kernel" != "$model" ]; then
		fail "parity diff on $name
--- kernel
$kernel
--- model
$model"
	fi
	n=$(printf '%s' "$model" | grep -c . || true)
	total_emissions=$((total_emissions + n))
	fixtures=$((fixtures + 1))
	echo "  MATCH     $name  ($n equation findings)"
done
[ "$fixtures" -gt 0 ] || fail "parity corpus is empty"
[ "$total_emissions" -gt 0 ] ||
	fail "non-vacuity gate: corpus emitted 0 findings (a vacuous parity run proves nothing)"

# ---- 3. guide tripwire corpus ----------------------------------------
echo "verus lane: guide tripwire corpus ($GUIDE_DIR)"
EXPECTATIONS="$GUIDE_DIR/expectations.txt"
[ -f "$EXPECTATIONS" ] || fail "guide expectations file missing"
guide_units=0
guide_out=$(mktemp)
trap 'rm -f "$model_out" "$guide_out"' EXIT
while IFS=$'\t' read -r name expected; do
	[ -n "$name" ] || continue
	file="$GUIDE_DIR/$name"
	[ -f "$file" ] || fail "expectations name a missing guide file: $name"
	rc=0
	run_verus "$file" "$guide_out" </dev/null || rc=$?
	out=$(cat "$guide_out")
	if [ $rc -eq 124 ]; then
		actual="fail:timeout"
	elif [ $rc -eq 0 ]; then
		actual="success"
	else
		code=$( { printf '%s\n' "$out" | grep -oE "error\[E[0-9]+\]" | head -1 | sed -E 's/error\[(E[0-9]+)\]/\1/'; } || true)
		if [ -n "$code" ]; then
			actual="fail:$code"
		elif printf '%s\n' "$out" | grep -q "verification results:: .*[1-9][0-9]* errors"; then
			actual="fail:verification"
		else
			actual="fail:rc$rc"
		fi
	fi
	if [ "$actual" != "$expected" ]; then
		fail "guide outcome drift on $name: expected $expected, got $actual"
	fi
	guide_units=$((guide_units + 1))
done <"$EXPECTATIONS"
[ "$guide_units" -gt 0 ] || fail "guide expectations are empty"
# No silent drops the other direction: every vendored guide file must be
# named by the expectations file.
for file in "$GUIDE_DIR"/*.rs; do
	[ -e "$file" ] || continue
	cut -f1 "$EXPECTATIONS" | grep -qx "$(basename "$file")" ||
		fail "guide file $(basename "$file") has no expectation"
done

echo "verus lane: green — $model_units model units verified, parity over $fixtures fixtures ($total_emissions emissions), $guide_units guide files as expected"
exit 0
