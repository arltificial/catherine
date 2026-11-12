# ro5u review — catherine-ga7 (grounded-instance harness, bogart-side)

Lead-side review. Deliverables land in `at/bogart` (bogart commits
`bc51d1e`, `a8dafd8`, nothing pushed): `scripts/grounded_instances.py`
(spec-driven extraction: declarative `bogart-corpus-slice/1` spec →
read-only SQL → deterministic `.ct.json` + `<instance>.provenance.json`
sidecar; hard errors on ambiguous extraction), `scripts/grounded_seeded_defects.py`
(quantified gate: build slice → mutate deep copies → real binary with
`--json` → witnesses translated back to corpus row names → fail on any
miss/extra/wrong exit class), `test-resources/catherine-grounded/`
(spec + emitted instance + provenance), evidence docs, README extraction
contract. Catherine: zero kernel delta (adjudicated — `git -C ../catherine
status --short` empty; the emitted document is stock format 1.0).

Orchestrator verification (live, its own commands): parity check —
"instance block identical to test-resources/catherine-pilot/model1_fts_anchor.ct.json";
`catherine check` on the emitted instance → exit 0; seeded-defect gate →
"controls clean: 2/2 · gate: pass" (7/7 finding-bearing detections with
witnesses per the report, 1/1 structural rejection, 2/2 negative
controls); determinism claimed by repeated sha256 (`f69e1778…616c`).

Five passes:

1. Correctness: spec-driven extraction replaces per-model hand-coding;
   the emitted instance is BYTE-IDENTICAL to the hc6 pilot model
   (independent confirmation the generalization preserved the frozen
   shape); contradiction-with-data demonstrated on a real corpus row
   (`pg_70953`) with the witness rendered in corpus terms.
2. Contract drift: adapter lives in bogart (kernel-smallness invariant
   held — the specodelic cautionary note); provenance carried in a
   sidecar, so canonical hashing/parity untouched; stock format 1.0.
3. Tests: the quantified Must gate (hc6's precondition) is executable
   and green — 7/7 with witnesses, misses would fail loudly; multi-row
   injections demand exact multisets.
4. Negative matrix: structural rejections (1/1) and negative controls
   (2/2) are rows of the same gate. N/A beyond that — no CLI surface.
5. Regression risk: none in catherine (zero delta); bogart gains two
   scripts + data + docs.

Findings + dispositions:

- [NOTE] the contradiction class honesty: v1's `contradiction` finding
   is schema-level only (`catherine search`, exit 5) — the demonstration
   uses the violation class against real data and SAYS SO rather than
   mislabeling; recorded in the report.
- [DEFERRED, LOW] bogart pushes remain user-gated (both repos clean,
   nothing pushed).

max_critical = 0. Verdict: PASS.