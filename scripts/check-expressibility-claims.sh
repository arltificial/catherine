#!/usr/bin/env bash
# check-expressibility-claims.sh — the limitation-claim docs gate for the
# expressibility gap suite (`add-expressibility-gap-suite`, beads
# `catherine-j6g`).
#
# Purpose: keep every known expressibility limitation inside the registry at
# `docs/ref/expressibility-dispositions.md`. A limitation claim in the docs
# must name a disposition row id, so a stale or unregistered claim cannot live
# outside the executable table. The gate is deliberately textual (openspec
# design Decision 4): a fixed, case-insensitive phrase list, no NLP.
#
# Responsibilities: extract the row ids from the table's first column; scan
# README.md, CHANGELOG.md, `docs/**/*.md` and openspec close notes for claim
# lines; require each claim line to contain at least one row id; name the file
# and line and exit 1 otherwise.
#
# Close notes: this repository writes them as `## Close note` sections of
# `openspec/**/tasks.md` (and `design.md`), not as `close-note*.md` files. The
# section is extracted deterministically: it starts at a heading whose text is
# exactly `Close note` and ends at the next heading of the same or higher
# level (fewer or equal leading `#`); nested sub-sections stay inside it. Both
# shapes are scanned, `tasks.md`/`design.md` sections and literal
# `openspec/**/close-note*.md` files (kept as the fallback for that spelling).
#
# Out of scope, by design: `.beads/**`, `.whisper/**`, every line of
# `openspec/changes/**` outside a `## Close note` section, the two disposition
# tables themselves, and the regenerated `docs/generated/**` mirrors (they are
# rebuilt from the openspec sources, which are not docs claims).
#
# Rationale: the suite's value is completeness under drift; without the gate
# the table decays back to chat-only knowledge. Two fixed, case-insensitive
# phrase groups keep every `Claimed limitation` cell in the table rejected
# when fed as an unattributed claim (a non-tautological coverage test,
# `crates/catherine-cli/tests/expressibility_gate_rows.rs`):
#
#   Specific vocabulary (a claim on its own):
#     cannot be expressed | can't be expressed | cannot be stated |
#     cannot be declared | cannot be represented | nothing can express |
#     not expressible | no way to express | inexpressible |
#     silently ignored | is silently dropped | cannot be checked jointly
#
#   Generic phrases (a claim only in limitation context):
#     accepted and ignored | accepted without a finding | is not checked |
#     checks clean | is never evaluated | hand-picked |
#     expressible only with
#
# Context rule: a generic phrase fires only when the *same line* also carries
# a limitation cue — a tool subject (`catherine`, `the tool`, `the parser`,
# `the checker`, `the check`, `this tool`), a limitation subject (`nothing`,
# `no way`, `silently`), or a bounded modal/negation/limitation word (`not`,
# `never`, `no`, `none`, `without`, `cannot`, `can't`, `unable`, `absent`,
# `missing`, `unobserved`, `unchecked`, `unsupported`, `dangling`, `wrong`,
# `omitted`, `fails`, `only`, `requires`). "Bounded" means the cue must stand
# as a word: `no` does not fire on `north`, `not` does not fire on
# `annotation`. This keeps ordinary prose such as "the run checks clean on
# the fixture" or "a hand-picked observation" out of the gate while a real
# claim ("the instance checks clean with no warning") is still rejected.
# The synthetic path applies the same rule, so `CLAIM_GATE_SYNTHETIC` cannot
# launch a rejection from a line that is not a limitation claim.
# `CLAIM_GATE_SYNTHETIC=<line>` scans only that synthetic line — used by the
# test to prove the gate is not a no-op. `CLAIM_GATE_ROOT=<dir>` is a
# test-only override of the repository root so the close-note scanner can be
# pointed at a temporary tree; the plain invocation is unchanged.

set -euo pipefail

ROOT="${CLAIM_GATE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TABLE="$ROOT/docs/ref/expressibility-dispositions.md"

# Unconditional limitation vocabulary: a line carrying one of these is a
# limitation claim whatever the surrounding sentence.
SPECIFIC_PATTERN="cannot be expressed|can't be expressed|cannot be stated"
SPECIFIC_PATTERN+="|cannot be declared|cannot be represented|nothing can express"
SPECIFIC_PATTERN+="|not expressible|no way to express|inexpressible"
SPECIFIC_PATTERN+="|silently ignored|is silently dropped|cannot be checked jointly"

# Generic phrases are ordinary English on their own; they count only on a
# line that also carries a limitation cue (CONTEXT_PATTERN).
GENERIC_PATTERN="accepted and ignored|accepted without a finding|is not checked"
GENERIC_PATTERN+="|checks clean|is never evaluated|hand-picked|expressible only with"

# The limitation cue for generic phrases: a tool subject, a limitation
# subject, or a bounded modal/negation word. The boundary group keeps `no`
# from matching `north` and `not` from matching `annotation`.
CONTEXT_PATTERN="catherine|the tool|the parser|the checker|the check"
CONTEXT_PATTERN+="|this tool|nothing|no way|silently"
CONTEXT_PATTERN+="|(^|[^[:alnum:]])(not|never|no|none|without|cannot|can't|unable"
CONTEXT_PATTERN+="|absent|missing|unobserved|unchecked|unsupported|dangling"
CONTEXT_PATTERN+="|wrong|omitted|fails?|only|requires?)([^[:alnum:]]|$)"

if [[ ! -f "$TABLE" ]]; then
	echo "check-expressibility-claims: missing disposition table: $TABLE" >&2
	exit 1
fi

mapfile -t ROW_IDS < <(awk -F'|' '
	/^[[:space:]]*\|/ {
		id = $2
		gsub(/^[[:space:]]+|[[:space:]]+$/, "", id)
		if (id == "" || id == "Case id" || id ~ /^-+$/) next
		print id
	}' "$TABLE")

if [[ ${#ROW_IDS[@]} -eq 0 ]]; then
	echo "check-expressibility-claims: no row ids parsed from $TABLE" >&2
	exit 1
fi

# check_line FILE LINE TEXT — fail when a claim names no registered row id.
check_line() {
	local file="$1" line="$2" text="$3" id
	for id in "${ROW_IDS[@]}"; do
		if [[ $text == *"$id"* ]]; then
			return 0
		fi
	done
	printf '❌ %s:%s: limitation claim names no disposition row id: %s\n' \
		"$file" "$line" "$text" >&2
	return 1
}

# claim_hits FILE — print `LINE:TEXT` for every limitation claim in FILE.
# Specific vocabulary stands alone; a generic phrase must share its line with
# a limitation cue. The generic branch drops lines the specific branch already
# emitted, so each claim line is reported once.
claim_hits() {
	{
		grep -inE "$SPECIFIC_PATTERN" "$1" || true
		grep -inE "$CONTEXT_PATTERN" "$1" \
			| grep -iE "$GENERIC_PATTERN" \
			| grep -ivE "$SPECIFIC_PATTERN" || true
	}
}

# is_claim_line TEXT — succeed when TEXT is a limitation claim under the
# same specific/generic/context rule the file scan uses.
is_claim_line() {
	printf '%s\n' "$1" | grep -qiE "$SPECIFIC_PATTERN" && return 0
	printf '%s\n' "$1" | grep -qiE "$CONTEXT_PATTERN" \
		&& printf '%s\n' "$1" | grep -qiE "$GENERIC_PATTERN"
}

if [[ -n ${CLAIM_GATE_SYNTHETIC:-} ]]; then
	if ! is_claim_line "$CLAIM_GATE_SYNTHETIC"; then
		echo "✅ synthetic line is not a limitation claim"
		exit 0
	fi
	if check_line "<synthetic>" 1 "$CLAIM_GATE_SYNTHETIC"; then
		echo "✅ synthetic limitation claim names a disposition row id"
		exit 0
	fi
	exit 1
fi

# close_note_lines FILE — print `LINE:TEXT` for every line inside a
# `Close note` heading's section, up to the next heading at the same or a
# higher level (see the header rule). Deterministic awk range, no heuristics.
close_note_lines() {
	awk '
		/^#+[[:space:]]+Close note[[:space:]]*$/ {
			match($0, /^#+/)
			level = RLENGTH
			inside = 1
			next
		}
		inside {
			if ($0 ~ /^#+[[:space:]]/) {
				match($0, /^#+/)
				if (RLENGTH <= level) { inside = 0; next }
			}
			printf "%d:%s\n", NR, $0
		}
	' "$1"
}

plain_files=()
for name in README.md CHANGELOG.md; do
	if [[ -f $ROOT/$name ]]; then
		plain_files+=("$ROOT/$name")
	fi
done
if [[ ! -d $ROOT/docs ]]; then
	echo "check-expressibility-claims: missing docs directory: $ROOT/docs" >&2
	exit 1
fi
while IFS= read -r -d '' file; do
	case "$file" in
		*/docs/generated/* | */docs/ref/expressibility-dispositions.md | */docs/ref/pattern-kata-dispositions.md) ;;
		*) plain_files+=("$file") ;;
	esac
done < <(find "$ROOT/docs" -type f -name '*.md' -print0)

if [[ ! -d $ROOT/openspec ]]; then
	echo "check-expressibility-claims: missing openspec directory: $ROOT/openspec" >&2
	exit 1
fi
close_note_files=()
while IFS= read -r -d '' file; do
	close_note_files+=("$file")
done < <(find "$ROOT/openspec" -type f \
	\( -name 'tasks.md' -o -name 'design.md' -o -name 'close-note*.md' \) -print0)

for file in "${plain_files[@]}" "${close_note_files[@]}"; do
	if [[ ! -r $file ]]; then
		echo "check-expressibility-claims: unreadable scan target: $file" >&2
		exit 1
	fi
done

failed=0
claims=0
record_hits() {
	local file="$1" hits="$2" hit
	while IFS= read -r hit; do
		[[ -z $hit ]] && continue
		claims=$((claims + 1))
		check_line "${file#"$ROOT/"}" "${hit%%:*}" "${hit#*:}" || failed=1
	done <<< "$hits"
}

for file in "${plain_files[@]}"; do
	record_hits "$file" "$(claim_hits "$file")"
done

for file in "${close_note_files[@]}"; do
	case "$file" in
		*/close-note*.md) record_hits "$file" "$(claim_hits "$file")" ;;
		*)
			record_hits "$file" "$(
				{
					close_note_lines "$file" | grep -iE "$SPECIFIC_PATTERN" || true
					close_note_lines "$file" | grep -iE "$CONTEXT_PATTERN" \
						| grep -iE "$GENERIC_PATTERN" \
						| grep -ivE "$SPECIFIC_PATTERN" || true
				}
			)"
			;;
	esac
done

if [[ $failed -ne 0 ]]; then
	printf '%s\n' "❌ expressibility claim gate failed ($claims claim line(s) scanned)" >&2
	exit 1
fi

printf '✅ every limitation claim names a disposition row id (%s claim line(s) scanned)\n' "$claims"