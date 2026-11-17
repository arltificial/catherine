#!/usr/bin/env bash
# check-lean-axioms.sh — the `sorry`-free, assumption-free audit for the Lean
# assurance twin (`add-lean-kernel-model`, task 11; beads `catherine-xvh`).
#
# Purpose: make the proof claims machine-checkable. `lean/Catherine/Axioms.lean`
# carries one `#print axioms` line per audited declaration; Lean's kernel
# tracks the axiom dependencies of every declaration and `#print axioms`
# reports them as one of exactly two shapes:
#
#   'Catherine.Proofs.soundness' depends on axioms: [propext, Classical.choice, Quot.sound]
#   'Catherine.Proofs.labelFor_agrees' does not depend on any axioms
#
# The audit asserts the axiom *set*, not a substring: every listed name must
# be in the allowlist {propext, Classical.choice, Quot.sound} (the classical
# axioms the twin's proofs legitimately use). An unfinished proof reports
# `sorryAx` in that set and a user-declared assumption reports its own bare
# name (`[cheat]`, with no `axiom` prefix), so both fail automatically and the
# offending name is printed. Any elaboration line that matches neither shape
# also fails — the audit never ignores unexpected output.
#
# Responsibilities: require the audit file (default
# `lean/Catherine/Axioms.lean`, overridable with `--file PATH` for self-tests);
# run `lake build` from `lean/` so the committed `lean-toolchain` pin is the
# one used, and fail on every `declaration uses ...sorry...` warning — that
# covers every declaration in every module, not only the audited ones; then
# elaborate the audit file with `lake env lean` and parse its output as above.
# Finally scan the source of every `lean/Catherine/**/*.lean` (plus the audit
# file itself when it lives outside that tree) for `axiom`/`private axiom`/
# `protected axiom` declarations and fail with file:line.
#
# Rationale: the parity oracle checks the twin's *semantics* against Rust; it
# says nothing about whether the theorems are proved or assumed. Lean does NOT
# echo source declaration lines in elaboration output — that is why a source
# scan, not an output scan, is what finds `axiom` declarations, and why the
# elaboration check must read the reported axiom list. Without this audit CI
# would accept a `sorry` or a hidden assumption silently and the proofs would
# be decoration. Exit 1 is the gate's red state; exit 2 is a usage error.

set -euo pipefail

ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
LEAN_DIR="$ROOT/lean"
AUDIT_FILE="$LEAN_DIR/Catherine/Axioms.lean"
AUDIT_FILE_EXPLICIT=0

usage() {
	printf '%s\n' \
		"usage: scripts/check-lean-axioms.sh [--file PATH]" \
		"" \
		"  --file PATH   elaborate PATH instead of lean/Catherine/Axioms.lean" \
		"  -h, --help    show this message"
}

while (($# > 0)); do
	case $1 in
	--file)
		if (($# < 2)); then
			printf '❌ check-lean-axioms: --file requires a path\n' >&2
			usage >&2
			exit 2
		fi
		AUDIT_FILE=$2
		AUDIT_FILE_EXPLICIT=1
		shift 2
		;;
	--file=*)
		AUDIT_FILE=${1#--file=}
		AUDIT_FILE_EXPLICIT=1
		shift
		;;
	-h | --help)
		usage
		exit 0
		;;
	*)
		printf '❌ check-lean-axioms: unknown argument %s\n' "$1" >&2
		usage >&2
		exit 2
		;;
	esac
done

# An explicit --file is a path the caller typed; resolve it against the
# invocation directory before the script cd's into lean/ for elaboration.
if ((AUDIT_FILE_EXPLICIT)) && [[ $AUDIT_FILE != /* ]]; then
	AUDIT_FILE="$PWD/$AUDIT_FILE"
fi

if [[ ! -f $AUDIT_FILE ]]; then
	printf '❌ check-lean-axioms: %s is absent — proofs are not yet stated.\n' "${AUDIT_FILE#"$ROOT"/}" >&2
	if ((AUDIT_FILE_EXPLICIT)); then
		printf '%s\n' "   (--file was given explicitly; check the path.)" >&2
		exit 2
	fi
	printf '%s\n' \
		"   The theorem statements land with add-lean-kernel-model task 7; until then this" \
		"   audit is red on purpose (the pre-proof state), not a tooling failure." >&2
	exit 1
fi

if ! command -v lake >/dev/null 2>&1; then
	printf '%s\n' \
		"❌ check-lean-axioms: lake not found on PATH — install elan (lean/lean-toolchain pins the version)." >&2
	exit 1
fi

trim() {
	local value=$1
	value="${value#"${value%%[![:space:]]*}"}"
	value="${value%"${value##*[![:space:]]}"}"
	printf '%s' "$value"
}

failed=0

# 1. Build the twin. A `sorry` is a build *warning*, so the build itself
# succeeds; `declaration uses 'sorry'` (backticked in Lean 4.36) is the
# compiler's report for every such declaration in every module, which is what
# closes the coverage gap of auditing only the listed `#print axioms` lines.
if ! build_output="$(cd -- "$LEAN_DIR" && lake build 2>&1)"; then
	printf '❌ check-lean-axioms: `lake build` failed\n' >&2
	printf '%s\n' "$build_output" >&2
	exit 1
fi

# Lean 4.36 quotes the token as `` `sorry` ``; earlier versions used `'sorry'`.
# Match either quote so a toolchain bump does not silently disable the gate.
sorry_re="declaration uses .?sorry.?"
while IFS= read -r line; do
	if [[ -n $line ]]; then
		printf '❌ check-lean-axioms: %s\n' "$line" >&2
		failed=1
	fi
done < <(printf '%s\n' "$build_output" | grep -E "$sorry_re" || true)

if ((failed != 0)); then
	printf '%s\n' "❌ check-lean-axioms: the twin contains a \`sorry\`; every declaration must be proved" >&2
	exit 1
fi

# 2. Elaborate the audit file. `lake env` supplies the package's LEAN_PATH, so
# a file outside the tree (the `--file` self-test path) still resolves imports.
if ! output="$(cd -- "$LEAN_DIR" && lake env lean "$AUDIT_FILE" 2>&1)"; then
	printf '❌ check-lean-axioms: `lake env lean %s` failed\n' "${AUDIT_FILE#"$ROOT"/}" >&2
	printf '%s\n' "$output" >&2
	exit 1
fi

# 3. Parse the reported axiom sets. Allowed: the classical axioms the proofs
# legitimately rest on. Everything else — `sorryAx`, a user-declared name — is
# a failure, and the audit reports the declaration and the axiom by name.
declare -A allowlist=([propext]=1 [Classical.choice]=1 [Quot.sound]=1)
audit_count=0
audit_re="^'([^']+)' (depends on axioms: \[(.*)\]|does not depend on any axioms)$"

while IFS= read -r line; do
	if [[ -z $line ]]; then
		continue
	fi
	if [[ ! $line =~ $audit_re ]]; then
		printf '❌ check-lean-axioms: unexpected elaboration output: %s\n' "$line" >&2
		failed=1
		continue
	fi
	name="${BASH_REMATCH[1]}"
	clause="${BASH_REMATCH[2]}"
	audit_count=$((audit_count + 1))
	if [[ $clause == 'does not depend on any axioms' ]]; then
		continue
	fi
	deps="${BASH_REMATCH[3]}"
	IFS=',' read -r -a reported <<<"$deps"
	for dep in "${reported[@]}"; do
		dep="$(trim "$dep")"
		if [[ -z $dep ]]; then
			continue
		fi
		if [[ -z ${allowlist[$dep]:-} ]]; then
			printf "❌ check-lean-axioms: '%s' depends on disallowed axiom '%s' (reported: [%s])\n" \
				"$name" "$dep" "$deps" >&2
			failed=1
		fi
	done
done < <(printf '%s\n' "$output")

if ((audit_count == 0)); then
	printf "❌ check-lean-axioms: no '#print axioms' output from %s — the audit asserts nothing\n" \
		"${AUDIT_FILE#"$ROOT"/}" >&2
	failed=1
fi

# 4. A declaration the audit never prints cannot be caught by step 3, so scan
# every module's source. Lean does not echo declarations, and an `axiom` can
# live in any proof module, not only in the audit file.
shopt -s globstar nullglob
scan_files=("$LEAN_DIR"/Catherine/**/*.lean)
shopt -u globstar nullglob

audit_real="$(realpath -- "$AUDIT_FILE")"
in_tree=0
for candidate in "${scan_files[@]}"; do
	if [[ $(realpath -- "$candidate") == "$audit_real" ]]; then
		in_tree=1
		break
	fi
done
if ((!in_tree)); then
	scan_files+=("$AUDIT_FILE")
fi

axiom_re='^[[:space:]]*((private|protected)[[:space:]]+)?axiom[[:space:]]+'
for source in "${scan_files[@]}"; do
	hits="$(grep -nE "$axiom_re" -- "$source" || true)"
	if [[ -z $hits ]]; then
		continue
	fi
	while IFS= read -r hit; do
		if [[ -n $hit ]]; then
			printf '❌ check-lean-axioms: axiom declaration at %s:%s\n' \
				"${source#"$ROOT"/}" "$hit" >&2
			failed=1
		fi
	done <<<"$hits"
done

if ((failed != 0)); then
	printf '%s\n' "❌ check-lean-axioms: proofs are not sorry-free / assumption-free" >&2
	exit 1
fi

printf '✅ check-lean-axioms: %d audited declaration(s) use only {propext, Classical.choice, Quot.sound}; no `sorry` in the build; no axiom declarations under lean/Catherine\n' \
	"$audit_count"