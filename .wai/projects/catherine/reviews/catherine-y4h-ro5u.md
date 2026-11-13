# ro5u review — catherine-y4h (seam agreement fixture + USING recipe)

Lead-side review. Deliverables: `crates/catherine-cli/tests/fixtures/seam_multi_path_agreement.ct.json`
(now part of the variant-coverage ratchet corpus, which enumerates that
directory), and the USING Recipe D "the seam agreement dbt cannot
express" with the captured executed run.

Five passes:

1. Correctness: the fixture's equation is typed end to end
   (`roll_frame ≡ roll_frame∘first_roll∘roll_frame`, Roll→Frame over the
   declared arrows). The executed run reports one `violation` with the
   witness `row 2, lhs 1, rhs 0` — the row's frame is `f1` while its
   frame's first roll belongs to `f0`. No `dangling` finding fires: every
   single-path reference resolves, which is exactly the contrast claimed
   (single-path existence passes; composed-path agreement fails).
2. Contract drift: fixture parses and every finding deserializes under
   the existing contract; the variant-coverage ratchet test passes with
   the fixture in its corpus (no new finding kind or CLI surface was
   invented for the seam).
3. Tests: the ratchet executes the fixture through the real binary; the
   USING run is its captured output. No new bespoke test asserts a copy
   of the engine's behavior.
4. Negative matrix: the companion claim is honest — parity on single-path
   existence, unique on composed-path agreement; the doc says catherine
   does not connect to a warehouse, so no dbt-replacement claim is made.
5. Regression risk: one data fixture + docs; no kernel or CLI code.

Findings + dispositions:

- [NOTE, ACCEPTED] The ticket said "in fixtures/"; the ratchet corpus
   lives at `crates/catherine-cli/tests/fixtures/`, and putting the
   document there is what makes the ratchet run it. A second copy under
   `fixtures/` would duplicate the corpus — rejected.
- [NOTE, RECORDED] "What a single-path FK check passes" is demonstrated
   structurally (no dangling findings on the same document), not against
   a live dbt run; the doc states the parity/unique split instead of
   claiming a measured dbt comparison.

max_critical = 0. Verdict: PASS.