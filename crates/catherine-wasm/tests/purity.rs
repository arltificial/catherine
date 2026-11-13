//! Purpose: kernel purity under the wasm build (openspec spec `wasm-ui`,
//! scenario "Kernel stays pure") — `catherine-core` and
//! `catherine-analysis` carry no wasm-bindgen and no genesis dependency,
//! so compiling them for `wasm32` links nothing browser-specific and
//! nothing from the sibling suite.
//!
//! Responsibilities: read the workspace dependency graph (`cargo
//! metadata --no-deps`) and assert both kernel packages are present and
//! free of those dependencies. The `wasm32` compile itself stays the
//! `just check-wasm` gate; this test pins the dependency set that gate
//! relies on, which compile success alone would not catch (a wasm-
//! compatible dependency could slip in).
//!
//! Rationale: the graph is the fact. Parsing manifests as text would
//! drift from resolution (features, target tables, renames); cargo
//! metadata reports what would actually link. `--offline` keeps the test
//! hermetic — no index access, and the workspace is already resolved by
//! the `cargo test` invocation running it.

use std::process::Command;

/// The packages that must stay wasm-agnostic: the kernel and its analysis
/// layer.
const KERNEL_CRATES: [&str; 2] = ["catherine-core", "catherine-analysis"];

/// The dependency prefixes the kernel may never link.
const FORBIDDEN_PREFIXES: [&str; 2] = ["wasm-bindgen", "genesis"];

#[test]
fn kernel_crates_link_no_wasm_bindgen_or_genesis() {
    let metadata = cargo_metadata();
    let packages = metadata["packages"].as_array().expect("packages array");
    for name in KERNEL_CRATES {
        let linked = first_forbidden_dependency(workspace_package(packages, name));
        assert!(
            linked.is_none(),
            "{name} must stay wasm-agnostic, but links {}",
            linked.unwrap_or("")
        );
    }
}

/// The workspace's resolved dependency graph (`cargo metadata --no-deps`),
/// queried from this crate's directory and `--offline`: no index access, and
/// the workspace is already resolved by the `cargo test` invocation.
fn cargo_metadata() -> serde_json::Value {
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("metadata is JSON")
}

/// The workspace package named `name`; its absence is a failure.
fn workspace_package<'a>(packages: &'a [serde_json::Value], name: &str) -> &'a serde_json::Value {
    packages
        .iter()
        .find(|package| package["name"] == name)
        .unwrap_or_else(|| panic!("{name} must be a workspace package"))
}

/// The first dependency of `package` matching a forbidden prefix, if any —
/// the one the kernel may never link.
fn first_forbidden_dependency(package: &serde_json::Value) -> Option<&str> {
    package["dependencies"]
        .as_array()
        .expect("dependencies array")
        .iter()
        .map(|dependency| dependency["name"].as_str().expect("dependency name"))
        .find(|dependency_name| forbidden_dependency(dependency_name))
}

/// Whether a dependency matches one of the forbidden prefixes.
fn forbidden_dependency(dependency_name: &str) -> bool {
    FORBIDDEN_PREFIXES
        .iter()
        .any(|prefix| dependency_name.starts_with(prefix))
}
