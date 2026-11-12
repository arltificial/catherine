//! Purpose: `ledger-check` — validate a ledger directory and report the
//! named rejections (or the derived project verdict shape) with exit
//! codes a gate can consume: 0 valid, 1 violations.
//!
//! Rationale: the validator is the RED gate of tasks.md task 1; the bin
//! is its operator surface, mirroring the library API one-to-one so the
//! orchestrator can re-run the discipline from the entries alone.

use std::path::PathBuf;

fn main() {
    let Some(dir) = std::env::args().nth(1) else {
        eprintln!("usage: ledger-check <ledger-dir>");
        std::process::exit(2);
    };
    match catherine_ledger::validate::validate_dir(&PathBuf::from(&dir)) {
        Ok(valid) => {
            let rollup = valid.rollup();
            let verdict = rollup
                .and_then(|entry| entry.content.rollup.as_ref())
                .map(|record| record.verdict.as_str())
                .unwrap_or("(no rollup)");
            println!(
                "ok: {} entries ({} kata), rollup verdict: {verdict}",
                valid.entries.len(),
                valid.kata_entries().len()
            );
        }
        Err(violations) => {
            for violation in &violations {
                eprintln!("violation: {violation:?}");
            }
            eprintln!("ledger refused: {} violations", violations.len());
            std::process::exit(1);
        }
    }
}
