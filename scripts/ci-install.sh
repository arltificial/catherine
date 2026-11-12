#!/usr/bin/env bash
# ci-install.sh — install the pinned ecosystem tools from versions.ddl.toml.
#
# Purpose: the single pinned-install recipe both CI
# (.github/workflows/ci.yml) and local pre-push setup run, so the gate
# environment is defined once. Versions are read from versions.ddl.toml at
# run time — the file is the only pin source (enforced by the
# standard_conformance test), and this script never installs `latest`.
#
# Responsibilities: parse the seven pins (wai, pretender, dont,
# dulce-de-leche, openspec, bd, ah); cargo-install the crates.io tools at
# their exact versions; npm-install @fission-ai/openspec pinned (user-owned
# prefix — no sudo); download bd from its pinned GitHub release
# (gastownhall/beads — no versioned homebrew formula), verifying the sha256
# against the release's checksums.txt; then verify each binary's --version
# contains its pin. Linux x86_64/arm64 for the bd asset; everything else is
# channel-agnostic.
#
# Rationale: cargo install skips tools already present at the requested
# version (local machines stay untouched when aligned), and the trailing
# --version sweep proves the pins landed rather than assuming it. bd is
# installed but never invoked by CI — the freshness gate is the
# standard_conformance test on .beads/issues.jsonl (CI never runs dolt).

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

pin() {
	local value
	value="$(sed -nE "s/^$1[[:space:]]*=[[:space:]]*\"([^\"]+)\".*/\\1/p" versions.ddl.toml)"
	if [[ -z "$value" ]]; then
		echo "ci-install: no pin for '$1' in versions.ddl.toml" >&2
		exit 1
	fi
	printf '%s' "$value"
}

WAI="$(pin wai)"
PRETENDER="$(pin pretender)"
DONT="$(pin dont)"
DDL="$(pin dulce-de-leche)" # crate dulce-de-leche, binary ddl
OPENSPEC="$(pin openspec)"
BD="$(pin bd)"
AH="$(pin ah)" # crate espectacular, binary ah

echo "ci-install: pins wai=$WAI pretender=$PRETENDER dont=$DONT dulce-de-leche=$DDL openspec=$OPENSPEC bd=$BD ah=$AH"

# crates.io tools, exact versions.
cargo install --version "$WAI" wai-cli
cargo install --version "$PRETENDER" pretender
cargo install --version "$DONT" dont-cli
cargo install --version "$DDL" dulce-de-leche
cargo install --version "$AH" espectacular

# openspec: npm @fission-ai/openspec, pinned, user-owned prefix (CI runners
# have no sudo-free global prefix we can assume).
NPM_PREFIX="$HOME/.npm-global"
mkdir -p "$NPM_PREFIX"
npm install --global --prefix "$NPM_PREFIX" "@fission-ai/openspec@$OPENSPEC"
export PATH="$NPM_PREFIX/bin:$PATH"
# Each Actions `run:` step is a fresh shell — the export above dies with
# this script, so hand the prefix to GITHUB_PATH for the later gate steps
# (unset locally; the export already covers direct local runs).
if [[ -n "${GITHUB_PATH:-}" ]]; then
	echo "$NPM_PREFIX/bin" >> "$GITHUB_PATH"
fi

# bd: pinned GitHub release download, checksum-verified.
case "$(uname -s)-$(uname -m)" in
	Linux-x86_64) BD_ASSET="beads_${BD}_linux_amd64.tar.gz" ;;
	Linux-aarch64) BD_ASSET="beads_${BD}_linux_arm64.tar.gz" ;;
	Darwin-arm64) BD_ASSET="beads_${BD}_darwin_arm64.tar.gz" ;;
	*)
		echo "ci-install: no pinned bd asset for $(uname -s)-$(uname -m)" >&2
		exit 1
		;;
esac
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
curl -sfL -o "$TMP/$BD_ASSET" \
	"https://github.com/gastownhall/beads/releases/download/v$BD/$BD_ASSET"
curl -sfL -o "$TMP/checksums.txt" \
	"https://github.com/gastownhall/beads/releases/download/v$BD/checksums.txt"
grep -F "$BD_ASSET" "$TMP/checksums.txt" > "$TMP/expected.sha256"
(cd "$TMP" && sha256sum -c expected.sha256)
tar -xzf "$TMP/$BD_ASSET" -C "$TMP" bd
install -m 0755 "$TMP/bd" "$HOME/.cargo/bin/bd"

# Prove the pins landed — every binary must report its pinned version.
check_version() {
	local binary="$1" want="$2"
	local got
	got="$("$binary" --version 2>&1)"
	if ! grep -qF "$want" <<<"$got"; then
		echo "ci-install: $binary --version ($got) does not contain pin $want" >&2
		exit 1
	fi
	echo "ci-install: $binary $(printf '%s' "$got" | head -1)"
}

check_version wai "$WAI"
check_version pretender "$PRETENDER"
check_version dont "$DONT"
check_version ddl "$DDL"
check_version openspec "$OPENSPEC"
check_version bd "$BD"
check_version ah "$AH"
echo "ci-install: all seven tools installed at their pinned versions"
