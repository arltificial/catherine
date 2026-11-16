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
# the table decays back to chat-only knowledge. The phrase list is fixed and
# case-insensitive, chosen so every `Claimed limitation` cell in the table is
# itself rejected when fed as an unattributed claim (a non-tautological
# coverage test, `crates/catherine-cli/tests/expressibility_gate_rows.rs`):
#   cannot be expressed | can't be expressed | cannot be stated |
#   cannot be declared | cannot be represented | nothing can express |
#   not expressible | no way to express | inexpressible |
#   silently ignored | is silently dropped | accepted and ignored |
#   accepted without a finding | is not checked | checks clean |
#   is never evaluated | hand-picked | expressible only with |
#   cannot be checked jointly
# `CLAIM_GATE_SYNTHETIC=<line>` scans only that synthetic line — used by the
# test to prove the gate is not a no-op. `CLAIM_GATE_ROOT=<dir>` is a
# test-only override of the repository root so the close-note scanner can be
# pointed at a temporary tree; the plain invocation is unchanged.

set -euo pipefail

ROOT="${CLAIM_GATE_ROOT:-$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)}"
TABLE="$ROOT/docs/ref/expressibility-dispositions.md"

CLAIM_PATTERN="cannot be expressed|can't be expressed|cannot be stated|cannot be declared"
CLAIM_PATTERN+="|cannot be represented|nothing can express|not expressible"
CLAIM_PATTERN+="|no way to express|inexpressible|silently ignored"
CLAIM_PATTERN+="|is silently dropped|accepted and ignored|accepted without a finding"
CLAIM_PATTERN+="|is not checked|checks clean|is never evaluated|hand-picked"
CLAIM_PATTERN+="|expressible only with|cannot be checked jointly"

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

if [[ -n ${CLAIM_GATE_SYNTHETIC:-} ]]; then
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
	record_hits "$file" "$(grep -inE "$CLAIM_PATTERN" "$file" || true)"
done

for file in "${close_note_files[@]}"; do
	case "$file" in
		*/close-note*.md) record_hits "$file" "$(grep -inE "$CLAIM_PATTERN" "$file" || true)" ;;
		*) record_hits "$file" "$(close_note_lines "$file" | grep -iE "$CLAIM_PATTERN" || true)" ;;
	esac
done

if [[ $failed -ne 0 ]]; then
	printf '%s\n' "❌ expressibility claim gate failed ($claims claim line(s) scanned)" >&2
	exit 1
fi

printf '✅ every limitation claim names a disposition row id (%s claim line(s) scanned)\n' "$claims"