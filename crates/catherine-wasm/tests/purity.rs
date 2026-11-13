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
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let output = Command::new(env!("CARGO"))
        .args([
            "metadata",
            "--no-deps",
            "--format-version",
            "1",
            "--offline",
        ])
        .current_dir(manifest_dir)
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("metadata is JSON");
    let packages = metadata["packages"].as_array().expect("packages array");

    for name in KERNEL_CRATES {
        let package = packages
            .iter()
            .find(|package| package["name"] == name)
            .unwrap_or_else(|| panic!("{name} must be a workspace package"));
        let dependencies = package["dependencies"]
            .as_array()
            .expect("dependencies array");
        for dependency in dependencies {
            let dependency_name = dependency["name"].as_str().expect("dependency name");
            for prefix in FORBIDDEN_PREFIXES {
                assert!(
                    !dependency_name.starts_with(prefix),
                    "{name} must stay wasm-agnostic, but links {dependency_name}"
                );
            }
        }
    }
}
