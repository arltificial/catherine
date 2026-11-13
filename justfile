# Workspace CI gate (openspec change `add-model-document-kernel`, task 1).
ci:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    just check-wasm
    # Fail-closed complexity gate, full tree, no baseline (same command as
    # the lefthook pretender-strict hook and the CI workflow step).
    pretender check --mode gate

# wasm32 compile-only gate (openspec change `add-wasm-web-view`, task 4):
# the kernel crates stay wasm-optional — compilation is the gate, no
# browser harness is required to run it. The rustup line is idempotent:
# with rust-toolchain.toml pinning the toolchain, the target must be
# present for THAT toolchain, and CI's first gate runs before the
# workflow's dedicated target step.
check-wasm:
    rustup target add wasm32-unknown-unknown
    cargo check -p catherine-wasm --target wasm32-unknown-unknown

# Everything .github/workflows/ci.yml runs, for local pre-push parity. The
# bd-export freshness gate lives inside `cargo test --workspace` (the
# standard_conformance test) — CI never runs dolt.
ci-full:
    just ci
    openspec validate --changes --strict --no-interactive
    ah check

# Install the pinned ecosystem tools from versions.ddl.toml — the same
# script the workflow calls (Standardization §4; beads catherine-6p5).
install-tools:
    scripts/ci-install.sh

# Dependency advisory audit (RustSec). Used by the release workflow's
# security gate (beads catherine-z91); publication is blocked on any
# advisory.
advisory:
    cargo audit --version >/dev/null 2>&1 || cargo install --locked cargo-audit
    cargo audit

# Secret scan over tracked files. Used by the release workflow's security
# gate (beads catherine-z91); publication is blocked when a potential
# secret is found.
secret-scan:
    #!/usr/bin/env bash
    set -euo pipefail
    if git grep -nIE '(AKIA[0-9A-Z]{16}|ghp_[A-Za-z0-9]{36}|github_pat_[A-Za-z0-9_]{22,}|-----BEGIN [A-Z ]*PRIVATE KEY-----)' -- . 2>/dev/null; then
        echo "❌ potential secret detected in tracked files (see matches above)" >&2
        exit 1
    fi
    echo "✅ no potential secrets in tracked files"

# Build the mdBook book locally (requires mdbook, pinned to the same
# version as the Docs workflow). Runs the workflow's generation steps
# first so the local build matches what Pages ships (beads catherine-48f).
docs:
    #!/usr/bin/env bash
    set -euo pipefail
    if ! command -v mdbook >/dev/null 2>&1; then
        echo "mdbook not found — cargo install mdbook --version 0.5.4 --locked" >&2
        exit 127
    fi
    mkdir -p docs/generated/specs
    for dir in openspec/specs/*/; do
        cp "$dir/spec.md" "docs/generated/specs/$(basename "$dir").md"
    done
    VERSION="$(cargo metadata --no-deps --format-version 1 \
        | python3 -c 'import json,sys; print(next(p["version"] for p in json.load(sys.stdin)["packages"] if p["name"]=="catherine-cli"))')"
    TAG="$(git rev-parse --abbrev-ref HEAD)"
    cat > docs/generated/release.md <<EOF
    # Release Status

    > **Generated artifact** (just docs, same inputs as docs.yml).
    > Do not hand-edit — the repo source files remain authoritative.

    - **Version:** v${VERSION}
    - **Built from:** ${TAG} @ $(git rev-parse --short HEAD)
    EOF
    mdbook build

# Bundle the static site (web/): wasm-bindgen glue + esbuild bundle.
# Requires the wasm-bindgen CLI at the SAME version as the wasm-bindgen
# crate in Cargo.lock (the lockfile is the pin — a mismatched CLI breaks
# the generated glue) and `npm install` run in web/.
web-build:
    cargo build -p catherine-wasm --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/debug/catherine_wasm.wasm
    cd web && npx esbuild main.mjs --bundle --format=esm --outfile=app.js
