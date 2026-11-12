# catherine-analysis

The analysis layer (openspec change `add-bounded-analysis`, beads
`catherine-1ay`): the deterministic question-answerer agents ask
mid-conversation (DESIGN §4.2). Dependency direction **`analysis →
core`** — it consumes the kernel's public surfaces (`Schema`,
`Instance`, `TypedInstance`, `checks::check`) and nothing flows back.
Kernel-pure: genesis-free and serde-free (the CLI owns every envelope);
runtime dependency: `catherine-core` alone — bounded search is plain
hand-rolled enumeration over the finite row spaces, **no solver**.

## The decidability boundary — what the tool refuses to claim

Every analysis answer is either **a proof within stated bounds** or
**`UNKNOWN { scope }`**. This is the contract, not a disclaimer:

- **No unbounded claims.** Bounds are inputs. No code path, doc line, or
  help text implies exhaustion that did not happen. "No violation found"
  is never an output — the outcomes are exactly: a witness, a
  contradiction finding (only when exhaustion *completed* within
  bounds), or `UNKNOWN` with the exact scope explored (label `unknown`).
- **`UNKNOWN` is first-class.** It is never silently converted into
  clean, into a finding, or into an error. The scope names the explored
  bounds (per-object row counts + candidates actually enumerated).
- **Zero-size bounds → `UNKNOWN` with the empty scope, never a
  contradiction.** A bound that leaves no row space anywhere is a
  non-answer by rule (spec scenario), regardless of vacuous
  satisfiability.
- **No schema-equivalence claims, ever.** `distinguish` decides bounded
  behavioral equivalence over an explicit observation set — the set is
  part of the result and the verdict holds exactly over it.
- **Rewriting limits.** Redundancy is proven only by normal-form
  rewriting under the *declared accepted equations* (derivations rest on
  accepted declarations; proposed equations prove nothing). Rules are
  oriented lhs→rhs; an equation whose rhs contains its lhs is skipped
  (that orientation cannot terminate). The engine is deterministic but
  **not a confluence procedure** — no critical-pair completion — so a
  normal form may depend on rule order when rules overlap. The honest
  consequence: rewriting can *under*-report redundancy but never
  *over*-report it — every `redundant` finding carries a witnessed
  derivation. A derivation that hits the rewrite-step cap is not a proof
  and emits nothing.
- **Underconstraint relating is direct-only.** A declared equation
  relates a parallel pair iff one of its sides *is* each path, in
  either orientation; transitive chains are not followed. The accepted
  chain `p=q, q=r` still raises the `(p,r)` question even though the
  rewriter can derive `p=r` — the question-raiser prefers noise over
  silence (DESIGN §4.2), and congruence closure over the accepted set
  is a future capability, not v1. Pinned by
  `transitive_equation_chain_does_not_suppress_the_end_pair_question`.
- **Search witness contract.** A witness is a *fully resolved* instance:
  every arrow value resolves and every accepted equation holds on every
  applicable row (the satisfaction predicate is the kernel's own
  `check` — a candidate witnesses iff executing all checks on it emits
  nothing). Candidates that satisfy equations only by dangling
  short-circuits prove nothing, so they never witness. Consequence, and
  the honest limit of the equality-only kernel language (DESIGN §4.1
  keeps injectivity and disequality out of v1): pure path equalities are
  collapse-satisfiable (the one-row collapse satisfies every equation),
  so a proven contradiction arises exactly when the bounds make full
  resolution impossible. The engine is deterministic smallest-first
  (unresolved cells first, then row ids ascending), which folds the
  counterexample-minimality discipline into the enumeration — the
  returned witness is the first satisfying candidate in that order.
- **Schema diff (P10).** `schema_diff::diff` classifies a v1→v2 edit on
  the *enforced* equations — accepted `(lhs, rhs)` pairs; rationale,
  document order, and proposed equations are presentation and never
  classify an edit. `cosmetic` (presentation-only) and `accreted`
  (constraints only added) are structural and emit zero findings;
  `weakened` emits exactly one `violation`/`executed` finding with a
  witness instance v2 accepts and v1 rejects; `equivalent-in-scope` is
  reported only after the bounded enumeration of fully resolved
  instances exhausts with no separating instance, with the explored
  scope echoed; a budget hit is `unknown`, never equivalence; differing
  objects/arrows are `incomparable` (instance spaces across vocabularies
  need a declared schema map, not a guess). A mixed edit with no
  weakening witness but a witnessed strengthening is
  `strengthened-in-scope` — never rounded into `accreted`. The shared
  `search::enumerate` engine owns the candidate order and the budget
  rule for both `search` and the diff.
- **Findings emit into the P2 taxonomy.** The analysis kinds with their
  pinned witness shapes and labels: `contradiction`/`redundant` carry
  `executed` (deterministic proofs the kernel computed);
  `underconstraint` carries `hypothesized` with the domain question
  "should these agree?". No new finding kinds; the P8 rule lens is the
  only producer of the reserved `UncoveredInputs` arm.
- **Rule lens (P8).** `rule_lens` analyses a rule presentation over a
  declared finite domain (`catherine-core::rule_lens`): gap →
  `underconstraint` (`hypothesized`, `UncoveredInputs` naming the class),
  overlap under `DeclaredDisjoint` → `violation` (`executed`), shadow and
  derivable → `redundant` (`executed`). A priority policy resolves
  intersections, so it reports no overlap finding; an absent policy
  leaves overlap a diagnostic mapped to no kind (Decision 1/3). The
  candidate enumeration is bounded by `max_candidates`; expiry is
  `UNKNOWN { scope }` with the declared column sizes, never a partial
  finding set.

## Scope caps (configured, not magic numbers)

| Cap | Default | Where |
|---|---|---|
| `max_candidates` | 256 | search's candidate budget (`--max-candidates`) |
| `max_path_length` | 3 | parallel-path enumeration (`--max-path-length`) |
| `max_rewrite_steps` | 64 | rewriting derivations per side |

All live in `bounds::ScopeCaps`; a cap hit is a non-answer, never a
quiet truncation dressed as success.

## The parity gate (query.rs port, task 9)

`src/query.rs` is **adapted, not authored**, from specodelic
(`charly/specodelic`, Apache-2.0, ships no NOTICE file)
`src/acset/query.rs` — attribution and adaptation notes are in its
header, mirroring the `instance.rs` precedent. The donor's contracts are
carried as tests under the donor's names in `tests/query_parity.rs`:

| Donor contract | Catherine test | What it pins |
|---|---|---|
| `seeds_exist` | `seeds_exist` | unknown seeds rejected (not ignored), all violations reported |
| `closure_terminates` | `closure_terminates` | one worklist + visited set; cyclic M terminates |
| `dangling_not_followed` | `dangling_not_followed` | a `None` morphism cell is no edge |
| `results_ordered` | `results_ordered` | `BTreeSet` results, sorted by construction |
| `fan_in_is_preimage_size` | `fan_in_is_preimage_size` | fan-in counts exactly the defined preimages |

Adapted, and why: specodelic's world is string-keyed (one global id
space per instance file; `BTreeSet<String>` seeds/results; morphism
vectors queried by name with no schema present). Catherine's kernel is
dense per-object `RowId` tables over a typed schema, so a node is
`(object, row)` (`query::NodeRef`) and the walk needs the schema's arrow
typing to interpret each morphism table — the query functions take
`(&Schema, &TypedInstance)`. `blast_radius` ports verbatim (the
parity-pinned mixed-closure fixpoint); `cyclic_nodes` ports as the
closure-based reading (a node is on a cycle through M iff some defined
value `s → t` has `s` reachable forward from `t`; trivial self-edges
included). Property tests (`proptest`) compare the ported closure
against an independent BFS reference on generated instances and re-pin
`fan_in` as a property.

## Layout

- `bounds.rs` — shared bound/scope types (`SearchBounds`, `SearchScope`),
  the engines' `ScopeCaps`, and the deduplicated witness builders (one
  construction site per P2 analysis finding shape).
- `search.rs` — bounded instance search (task 6) and the shared
  `enumerate` engine (found / exhausted-with-scope / unknown-with-scope)
  behind both search and schema diff.
- `rewrite.rs` — normal-form rewriting / redundancy (task 7).
- `parallel.rs` — parallel-path detection (task 8).
- `query.rs` — the ported traversals (task 9).
- `distinguish.rs` — bounded distinguishability over explicit
  observation sets (task 10).
- `schema_diff.rs` — v1→v2 edit classification (P10): cosmetic /
  accreted / weakened-with-witness / equivalent-in-scope.

Build: part of the workspace; `cargo test -p catherine-analysis` runs
the fixture suites (including `tests/schema_diff.rs`), the
parity/property gate, and the determinism assertions (repeated runs
serialize byte-identically; assertions are structural — no snapshots).
