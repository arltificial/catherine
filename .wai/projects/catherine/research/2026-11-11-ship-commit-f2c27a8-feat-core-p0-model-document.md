---
tags: [pipeline-run:epic-orchestrator-2026-11-11-catherine-p0-model-document-kernel-skeleton-catherine-bpz, pipeline-step:ship]
---

SHIP: commit f2c27a8 feat(core): P0 model document kernel (crates/, root Cargo.toml, Cargo.lock, justfile, tasks.md 17 ticks, wai trail); bd close catherine-bpz done; export refreshed + committed as follow-up (add -f needed: global gitignore covers .beads dir, jsonl tracked explicitly per precedent); beads db unstaged per Invariant 9; no push

Commands run: bd close catherine-bpz (exit 0); bd export > .beads/issues.jsonl + git add -f + commit 1f62581 (exit 0). Verified: `bd ready` now lists catherine-0gk (P1) as the only ready ticket; ship approved by user before commit; tests pass state carried from verify step (fresh cargo/just/openspec runs, same tree, no code changed after them).
