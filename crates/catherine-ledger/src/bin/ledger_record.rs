//! Purpose: `ledger-record` — seal one entry file into a ledger
//! directory under its computed content address, then re-validate the
//! whole directory. A record that makes the directory invalid is
//! removed again and the named rejections are printed: append-only with
//! the discipline enforced at write time, not after the fact.
//!
//! Rationale: the record path enforces the negative-control-first rule
//! by construction (the validator runs over the staged ledger), so bad
//! science cannot quietly enter the ledger.

use std::path::{Path, PathBuf};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: ledger-record <ledger-dir> <entry.json>");
        std::process::exit(2);
    }
    let dir = PathBuf::from(&args[1]);
    let entry_file = Path::new(&args[2]);
    let entry = match catherine_ledger::validate::read_entry_file(entry_file) {
        Ok(entry) => entry,
        Err(violation) => {
            eprintln!("entry refused: {violation:?}");
            std::process::exit(1);
        }
    };
    let hex = entry.id.trim_start_matches("sha256:");
    let target = dir.join(format!("{:04}-{}.json", entry.content.seq, &hex[..8]));
    let staged = serde_json::to_string(&entry).expect("entry serializes");
    std::fs::write(&target, staged).expect("ledger file written");

    if let Err(violations) = catherine_ledger::validate::validate_dir(&dir) {
        for violation in &violations {
            eprintln!("violation: {violation:?}");
        }
        eprintln!(
            "record reverted: {} violations after appending {}",
            violations.len(),
            target.display()
        );
        std::fs::remove_file(&target).expect("staged record removed");
        std::process::exit(1);
    }
    println!("recorded {}", target.display());
}
