#!/usr/bin/env python3
# Purpose: thin entry for the architecture policy gate (bead
#   catherine-kpn.9): flag parsing and the process exit only, dispatching
#   into scripts/architecture_policy_lib.py.
# Responsibilities:
# - own the CLI surface (--self-test, --bin, --root) and exit propagation;
#   every behavior — policy loading, extraction, model building, checking,
#   exit codes 0/3/4, failure-line formats, the self-test bundle — lives in
#   the library module
# Rationale: split to satisfy the fail-closed size gate (pretender
#   file_lines <= 300 per script, catherine-kpn.9). The pre-push hook and
#   `just check-architecture-policy` keep invoking this path unchanged, and
#   direct execution puts this directory on sys.path, so the plain import
#   resolves from any working directory.
import argparse
import sys
from pathlib import Path

from architecture_policy_lib import REPO, gate, self_test


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Gate the workspace dependency graph against "
                    "policy/architecture-policy.ct.json (bead catherine-kpn.9)")
    parser.add_argument("--self-test", action="store_true",
                        help="run the negative-control bundle and the "
                             "extractor test, then exit")
    parser.add_argument("--bin",
                        help="catherine CLI binary (default: $CATHERINE_BIN, "
                             "target/debug/catherine, target/release/catherine, $PATH)")
    parser.add_argument("--root", type=Path, default=REPO,
                        help="workspace root (default: this script's repo)")
    args = parser.parse_args()
    if args.self_test:
        return self_test(args.root)
    return gate(args.root, args.bin)


if __name__ == "__main__":
    sys.exit(main())
