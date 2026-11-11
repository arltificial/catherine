# Workspace CI gate (openspec change `add-model-document-kernel`, task 1).
ci:
    cargo fmt --all -- --check
    cargo clippy --workspace --all-targets -- -D warnings
    cargo test --workspace
