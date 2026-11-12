//! Purpose: the kata-fixture harness shared with the P8/P9 validation
//! work (tasks.md task 10) — load a fixture, execute the kernel checks
//! exactly as the CLI drives them, and report the finding inventory a
//! ledger entry records.
//!
//! Responsibilities: [`run`] parses the fixture, builds the typed
//! instance, executes the kernel checks, and returns the finding list
//! plus the parse outcome. A fixture that does not parse is a recorded
//! outcome (the construction-time probe class), not an error thrown
//! away.
//!
//! Rationale: the harness consumes `catherine-core`'s public artifacts
//! only (parse, instance, checks, findings encoding) — the same
//! single execution path the CLI and the wasm boundary drive — so a
//! ledger entry's findings are the tool's findings verbatim.

use catherine_core::findings::Finding;
use catherine_core::instance::TypedInstance;
use std::path::Path;

/// One executed fixture run: the inventory the ledger entry records.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KataRun {
    /// The fixture path as given.
    pub fixture: String,
    /// `false` = the fixture did not parse (the construction-time probe
    /// class: a load that must be rejected, recorded as the outcome).
    pub parsed: bool,
    /// The kernel findings verbatim (empty when parsing failed).
    pub findings: Vec<Finding>,
    /// The parse failure, when [`KataRun::parsed`] is `false`.
    pub parse_error: Option<String>,
}

/// Execute the kernel checks over a kata fixture.
pub fn run(fixture: &Path) -> KataRun {
    let display = fixture.display().to_string();
    let bytes = match std::fs::read_to_string(fixture) {
        Ok(bytes) => bytes,
        Err(error) => {
            return KataRun {
                fixture: display,
                parsed: false,
                findings: Vec::new(),
                parse_error: Some(format!("{error}")),
            }
        }
    };
    let model = match catherine_core::parse::parse(&bytes) {
        Ok(model) => model,
        Err(error) => {
            return KataRun {
                fixture: display,
                parsed: false,
                findings: Vec::new(),
                parse_error: Some(format!("{error:?}")),
            }
        }
    };
    let typed = TypedInstance::build(&model.schema, &model.instance);
    KataRun {
        fixture: display,
        parsed: true,
        findings: catherine_core::checks::check(&model.schema, &typed),
        parse_error: None,
    }
}
