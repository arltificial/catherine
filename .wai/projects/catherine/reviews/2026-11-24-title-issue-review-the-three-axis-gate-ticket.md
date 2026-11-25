---
reviews: 2026-11-24-plan-landing-the-three-axis-gate-sequencing.md
verdict: needs-work
severity: {critical: 0, high: 3, medium: 12, low: 6}
---

#+TITLE: Issue review — the three-axis gate ticket set (catherine-kpn + 9 children)
#+DATE: 2026-11-24
#+OPTIONS: toc:2

System: Beads. Scope: the epic ~catherine-kpn~ and its nine children
(10 issues). Source: the wai design artifact "the three-axis gate" and plan
artifact "landing the three-axis gate" (2026-11-24).

Mode: tracer-bullet issue review (5 passes). Advisory — this review does not
edit tracker issues; every remediation below is given as an exact command.

* PASS 0: Pre-flight
Mechanical checks over the ticket set, then remediation and a re-run.

**[PRE-001] MEDIUM → FIXED** — all ten tickets lacked ~metadata.files~.
Remediation applied: ~bd update <id> --metadata '{"files":[...]}'.
Re-run: no ticket missing ~files~.

**[PRE-002] LOW → OPEN** — files claimed by three or more tickets:
- ~docs/ref/expressibility-dispositions.md~ → kpn.3, kpn.4, kpn.7, kpn.8
  (four tickets, three of which *flip different rows* of that table)
- ~README.md~ → kpn.1, kpn.2, kpn.3
- ~docs/PROPOSAL.md~ → kpn.1, kpn.2, kpn.3
Recommendation: serialize the registry writers (kpn.4 → kpn.7 → kpn.8) and the
doc trio (kpn.1 → kpn.2 → kpn.3), or add a co-modification note to each.

**[PRE-003] [PRE-004] → FIXED** — no ticket carried ~base_commit~. All ten
are now anchored at 3d379d7 (= HEAD).

**[PRE-005] MEDIUM → FIXED** — no estimates. Applied: 30/30/45/90/30/120/60/
90/120/120 minutes (epic 30).

**[PRE-006] → none** — no ticket exceeds the 120-minute scheduling midpoint;
the three largest are tagged ~complexity:l~.

**[PRE-007] → none in this set** — no ticket is claimed. (Repo-wide note:
~catherine-6g2~, the Kani lane ticket, holds an expired lease —
~bd reclaim~ is available if that session is dead.)

**Lane labels added** (repo convention is ~lane:afk~ / ~lane:design~ /
~lane:hitl~): afk → kpn.1, kpn.4, kpn.5, kpn.6, kpn.7; hitl → kpn.2, kpn.3,
kpn.8; design → kpn.9, epic.

* PASS 1: Completeness & Clarity

[CLRT-001] MEDIUM - catherine-kpn (epic)
Description: the epic's ~acceptance_criteria~ is empty; its done-when lives
only in the description prose.
Evidence: ~bd show catherine-kpn --json~ → ~acceptance_criteria: ""~.
Recommendation: ~bd update catherine-kpn --acceptance "Phase 0 corrections
landed; kpn.5-kpn.8 landed; the registry rows the landed capabilities close
are closed with sealing evidence and the rest stay gap and say why; catherine
check reports its execution scope; the architecture gate, if it lands, has its
policy, extractor test and negative-control bundle."~

[CLRT-002] HIGH - catherine-kpn.2
Description: the ticket offers two mutually exclusive resolutions ("Either
the tool emits the advisory … or the four documentation locations stop
claiming it") with no decision recorded, so a fresh agent cannot start and the
done-criteria are ambiguous.
Evidence: acceptance criteria begin "Either … or …"; the design field says
"the choice is recorded in the change" — i.e. the ticket defers its own
decision to the implementer.
Recommendation: split — a decision ticket (~lane:design~, owner: the human)
plus one implementation ticket that names the chosen resolution; or record the
choice now: ~bd update catherine-kpn.2 --design "Decision: <a|b>. Rationale: …"~

[CLRT-003] HIGH - catherine-kpn.8
Description: same shape — "Option (a) is preferred; (b) is the acceptable
cheap form" — two different slices (reuse the classifier vs emit a hint),
different flip-protocol consequences, no decision.
Recommendation: split into kpn.8a (classifier reuse; flips both rows) and
kpn.8b (hint; leaves both rows gap), or record the choice.

[CLRT-004] MEDIUM - catherine-kpn.3
Description: "stand up one normative constraints surface" names no file; the
ticket's ~metadata.files~ lists five documents it would *edit* but not the new
artifact it would *create*.
Recommendation: name it (~docs/VALUE.md~ or a README section) and add it to
~metadata.files~.

[CLRT-005] MEDIUM - catherine-kpn.5
Description: "every answer" is unquantified — which commands? and the payload
field's name and shape are unspecified, so two implementers will produce
different payloads.
Recommendation: name the surface (all nine commands, or the four that carry
findings) and the field (~data.scope = {constraints_checked, rows_evaluated,
equations_over_zero_rows}~).

* PASS 2: Scope, Atomicity, Tracer-Bullet Shape

[SCOPE-001] MEDIUM - catherine-kpn.5
Description: horizontal, cross-cutting: six files across the CLI, the web
bundle and two openspec specs, at the 120-minute ceiling. It is not one
session's worth of one capability.
Recommendation: split at the seam — (a) kernel+CLI compute and render the
scope (crates/catherine-cli/src/render/envelope.rs and the scope computation;
gates: cargo test --workspace, ah check), (b) wasm/web types and the specs
(gates: just check-wasm, ah check --changes), wired
~bd dep add <b> --deps "blocked-by:<a>"~.

[SCOPE-002] MEDIUM - catherine-kpn.3
Description: three capabilities in one ticket (proposition rewrite, the
state-shape definition, a new constraints surface) across five files.
Recommendation: split into (a) the constraints surface with the axes and the
state-shape definition (the new file), (b) the proposition rewrite and the
repointing of PROPOSAL/DESIGN/USING.

[SCOPE-003] MEDIUM - catherine-kpn.8
Description: two candidate slices behind one ticket (see CLRT-003).

[SCOPE-004] LOW - catherine-kpn.1, kpn.2, kpn.3
Description: three tickets editing README.md and docs/PROPOSAL.md — not a
horizontal-decomposition problem (each is a distinct claim), but a
co-modification conflict (PRE-002).

* PASS 3: Dependencies and Ordering

[DEP-001] HIGH - the whole set
Description: no dependency edges exist between the nine children, even though
the plan sequences them and three of them flip *different rows of the same
registry file* (~docs/ref/expressibility-dispositions.md~) — a real
merge-conflict and flip-ordering hazard, and the flip protocol expects one
change per row.
Recommendation:
#+begin_example
bd dep add catherine-kpn.7 --deps "blocked-by:catherine-kpn.4"
bd dep add catherine-kpn.8 --deps "blocked-by:catherine-kpn.7"
bd dep add catherine-kpn.3 --deps "blocked-by:catherine-kpn.1"
#+end_example
(the last one serializes the README/PROPOSAL trio).

[DEP-002] MEDIUM - catherine-kpn.5
Description: its prerequisite — an openspec change for the payload shape — is
not tracked as an issue anywhere; the plan requires it (~ah check --changes
<name>~), so the ticket as written cannot start.
Recommendation: create the openspec change first, then link:
~bd create "openspec: execution scope in the payload" --type task --parent
catherine-kpn --labels lane:design,openspec~ and
~bd dep add catherine-kpn.5 --deps "blocked-by:<that id>"~.

[DEP-003] LOW - catherine-kpn.2 / catherine-kpn.5
Description: if the "implement the hint" resolution is chosen, both tickets
touch ~crates/catherine-cli/src/render/envelope.rs~ (the hints channel and the
scope field) — a co-modification note belongs in both bodies.

Positive: the HITL tickets (kpn.2, kpn.3, kpn.8) are isolated from the AFK
ones, so autonomous work continues around them — the property the lens asks
for.

* PASS 4: Plan, Story, and Spec Alignment

[ALIGN-001] MEDIUM - all children
Description: no ticket cites its plan section (the plan has numbered phases
0.1-0.4, 1.1-1.2, 2.1-2.2, 3.1); traceability exists only at the epic level
(its description names the wai artifacts).
Recommendation: append a ~Plan:~ line to each body, e.g. ~bd update
catherine-kpn.7 --append-notes "Plan: Phase 2.1 (landing-the-three-axis-gate,
2026-11-24)"~.

[ALIGN-002] LOW - the epic
Description: coverage is complete — the plan's nine items map one-to-one onto
the nine children (Phase 0 → kpn.1-4, Phase 1 → kpn.5-6, Phase 2 → kpn.7-8,
Phase 3 → kpn.9) and the flip protocol appears on exactly the two tickets
that can close registry rows — but the epic does not state that mapping, so a
reader cannot check coverage without opening the plan.

* PASS 5: Executability and Handoff

[EXEC-001] MEDIUM - catherine-kpn.7, kpn.5, kpn.6, kpn.8
Description: no ticket names its red gate as a runnable command. kpn.7 comes
closest (it names the fixture) but not the command that must go red.
Evidence: the gap-suite case exists at
~crates/catherine-cli/tests/expressibility_gaps/cases/migration.rs:85~
(~migrate_without_loss("nonfunctorial_schema_map.ct.json", …)~).
Recommendation: add to each body an explicit red line, e.g. for kpn.7:
"Red: ~cargo test -p catherine-cli --test expressibility_gaps~ fails — the
non-functorial map is accepted (exit 0, no finding). Green: the case asserts
the refusal (~partial_schema_map~, or a new labeled code named in the ticket)."

[EXEC-002] MEDIUM - catherine-kpn.5, kpn.7, kpn.8
Description: tickets that touch a public surface carry no anti-goals — the
reward-hacking surface is open. kpn.5 changes the JSON payload; kpn.7 changes
which documents are accepted; kpn.8 changes migrate's output.
Recommendation: add anti-goals, e.g. for kpn.5: "No new exit code; existing
payload fields unchanged; single-document runs keep today's bytes and exit
codes; the scope field is additive." For kpn.7: "No new finding kinds; a
well-typed map migrates byte-identically."

[EXEC-003] MEDIUM - catherine-kpn.1, kpn.4 (lane:afk without an objective gate)
Description: both are tagged ~lane:afk~ but the *claim* each fixes has no
machine check — a fresh agent can satisfy them by editing prose.
Recommendation: name the meter. kpn.1: "~! grep -rn 'smallest breaking
instance' README.md docs/PROPOSAL.md~ plus the grep of the replacement phrase
in both files". kpn.4: "~cargo test -p catherine-cli --test
expressibility_gaps~ stays green and the two rows' cells carry the stated
text" (grep-able). Otherwise tag both ~no-gate~ and keep them out of
autonomous scheduling.

[EXEC-004] MEDIUM - catherine-kpn.5, kpn.6, kpn.7, kpn.8
Description: no pass-to-pass regression suite named, so "nothing else broke"
is unfalsifiable.
Recommendation: name it — "~cargo test --workspace~ stays green (the contract
suite pins the parsed surface byte-identically)" plus the specific family for
each (e.g. kpn.6 → the search and diff CLI tests).

[EXEC-005] MEDIUM - catherine-kpn.9
Description: creates two new files (~scripts/check-architecture-policy.py~,
~policy/architecture-policy.ct.json~) with no file-header criterion, so every
implementing agent re-derives their intent; and it mixes a design act
(authoring the policy from the documents) with implementation.
Recommendation: split — (a) a design ticket (~lane:hitl~) that authors the
nine permitted edges with their source citations, (b) an AFK implementation
ticket. Add the file-header acceptance criterion: "both new files carry a
Purpose/Responsibilities/Rationale header".

[EXEC-006] LOW - catherine-kpn.2, kpn.3, kpn.8 (lane:hitl)
Description: the human-in-the-loop handoff point is not explicit — who
decides, against what, and what the implementer does afterwards.
Recommendation: each HITL body ends with "Handoff: <decision needed> — owner:
<who>; the implementing ticket starts only after the decision is recorded
here."

* Convergence checks
#+begin_example
After Pass 1: new CRITICAL 0 | new 5 | previous 1 | FP ~0% | ITERATE
After Pass 2: new CRITICAL 0 | new 4 | previous 6 | FP ~0% | ITERATE
After Pass 3: new CRITICAL 0 | new 3 | previous 10 | FP ~0% | ITERATE
After Pass 4: new CRITICAL 0 | new 2 | previous 13 | FP ~0% | ITERATE
After Pass 5: new CRITICAL 0 | new 6 | previous 15 | FP ~0% | ITERATE
#+end_example
The mechanical <10% criterion is not met, and the reason is structural: each
pass surfaces a *different class* (clarity, scope, dependencies, alignment,
executability) rather than new instances of one class. Every finding above is
an actionable edit with a command, not new scope — so the set converges on
remediation, not on more discovery.

* Verdict
NEEDS_UPDATES. Nothing is blocked (no CRITICAL), the set is well-founded and
traceable, and PASS 0 is remediated and green. What remains is a bounded list
of content edits: two tickets that defer their own decision (kpn.2, kpn.8),
one missing dependency chain for the registry flips, three tickets that need
their red gate, meters and anti-goals named, and two tickets that should split
(kpn.5, kpn.9).

* Issue quality assessment
- Clarity: Fair (two tickets are not startable without a decision)
- Scope: Fair (two cross-cutting tickets at the ceiling)
- Dependencies: Poor (no edges at all, despite shared files and a flip order)
- Completeness: Good (every plan item is covered; PASS 0 metadata is in place)

