//! Purpose: `ledger-record` — seal one entry file into a ledger
//! directory under its computed content address, then re-validate the
//! whole directory. A record that makes the directory invalid is
//! removed again and the named rejections are printed: append-only with
//! the discipline enforced at write time, not after the fact.
//!
//! Responsibilities: read the candidate entry file, sealing it when it
//! declares no id (absent, empty, or the `auto` authoring placeholder),
//! and stage it under `<seq>-<id-prefix>.json`.
//!
//! Rationale: the record path enforces the negative-control-first rule
//! by construction (the validator runs over the staged ledger), so bad
//! science cannot quietly enter the ledger. Sealing is what makes the
//! content-addressed entry authorable by hand: the human writes the
//! content, the tool computes the address. A DECLARED id is never
//! rewritten — it takes the validator's own read path, so an address
//! that disagrees with the bytes stays a refusal.

use std::path::{Path, PathBuf};

use catherine_ledger::entry::{declared_id, seal_parsed, Entry};
use catherine_ledger::validate::{read_entry_file, validate_dir, Violation};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() != 3 {
        eprintln!("usage: ledger-record <ledger-dir> <entry.json>");
        std::process::exit(2);
    }
    let dir = PathBuf::from(&args[1]);
    let entry_file = Path::new(&args[2]);
    let entry = match candidate_entry(entry_file) {
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

    if let Err(violations) = validate_dir(&dir) {
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

/// Read the candidate entry, sealing it under its computed content
/// address when it declares no id. A declared id delegates to
/// [`read_entry_file`]: the integrity rule is the validator's, not a
/// second implementation here, so a wrong id is refused as `IdMismatch`
/// exactly as before.
fn candidate_entry(path: &Path) -> Result<Entry, Violation> {
    let bytes = std::fs::read_to_string(path)
        .map_err(|e| Violation::Unreadable(format!("{}: {e}", path.display())))?;
    let value: serde_json::Value = serde_json::from_str(&bytes)
        .map_err(|e| Violation::Unreadable(format!("{}: {e}", path.display())))?;
    if declared_id(&value).is_some() {
        return read_entry_file(path);
    }
    let file = path.display().to_string();
    seal_parsed(value).map_err(|e| Violation::Unreadable(format!("{file}: {e}")))
}
