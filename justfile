# Workspace CI gate (openspec change `add-model-document-kernel`, task 1).
ci:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace

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
