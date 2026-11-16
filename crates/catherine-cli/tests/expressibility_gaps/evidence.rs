//! Purpose: git-backed verification of the `closed` rows' sealing evidence —
//! a fixing commit hash or an artifact path that is part of the change which
//! seals the row.
//! Responsibilities: resolve the repository root, accept a 7–40 hex token
//! only when `git cat-file -e <token>^{commit}` succeeds, accept a
//! repo-relative path only when `git ls-files --error-unmatch` reports it
//! tracked (index or HEAD), and require `gap` evidence to name the missing
//! capability.
//! Rationale: the pre-commit-correct notion of "committed evidence" is index
//! tracking (the artifact is staged in the same commit that flips the row),
//! and a shape-only check would accept `1a2b3c4` or an untracked file. Git
//! absence panics (fail closed), never a silent pass.

use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root: this binary lives at `crates/catherine-cli`.
pub(crate) fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root")
}

/// `true` when the evidence contains a commit that resolves in this
/// repository or a repo-relative path tracked by git (index or HEAD).
pub(crate) fn is_fixing_evidence(evidence: &str) -> bool {
    evidence
        .split(|c: char| c.is_whitespace() || c == '`')
        .any(|token| is_commit_hash(token) && commit_exists(token))
        || cites_artifact(evidence)
}

/// `true` when the evidence names a repo-relative path that git tracks (in
/// the index or in `HEAD`). An untracked file on disk is not evidence: the
/// path must be part of the change that seals the row.
fn cites_artifact(evidence: &str) -> bool {
    let candidate = evidence.trim().trim_matches('`');
    if !is_repo_relative(candidate) {
        return false;
    }
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["ls-files", "--error-unmatch", "--", candidate])
        .output()
        .unwrap_or_else(|error| {
            panic!(
                "git is required to verify fixing evidence `{candidate}` is tracked \
                 (index or HEAD): {error}"
            )
        });
    output.status.success()
}

fn is_repo_relative(candidate: &str) -> bool {
    if candidate.is_empty() {
        return false;
    }
    if candidate.chars().any(char::is_whitespace) {
        return false;
    }
    !Path::new(candidate).is_absolute()
}

/// `true` when `token` names a commit reachable in this repository. Git
/// absence is a panic (fail closed), never a silent pass.
fn commit_exists(token: &str) -> bool {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root())
        .args(["cat-file", "-e", &format!("{token}^{{commit}}")])
        .output()
        .unwrap_or_else(|error| {
            panic!("git is required to verify fixing commit `{token}`: {error}")
        });
    output.status.success()
}

fn is_commit_hash(token: &str) -> bool {
    (7..=40).contains(&token.len()) && token.bytes().all(|byte| byte.is_ascii_hexdigit())
}

pub(crate) fn names_missing_capability(evidence: &str) -> bool {
    evidence
        .to_ascii_lowercase()
        .strip_prefix("missing:")
        .is_some_and(|capability| !capability.trim().is_empty())
}
