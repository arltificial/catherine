# Workspace CI gate (openspec change `add-model-document-kernel`, task 1).
ci:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
    just check-wasm

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

# Bundle the static site (web/): wasm-bindgen glue + esbuild bundle.
# Requires the wasm-bindgen CLI at the SAME version as the wasm-bindgen
# crate in Cargo.lock (the lockfile is the pin — a mismatched CLI breaks
# the generated glue) and `npm install` run in web/.
web-build:
    cargo build -p catherine-wasm --target wasm32-unknown-unknown
    wasm-bindgen --target web --out-dir web/pkg target/wasm32-unknown-unknown/debug/catherine_wasm.wasm
    cd web && npx esbuild main.mjs --bundle --format=esm --outfile=app.js
