# Tasks: add-identity-arrows

One feature cycle (RED → GREEN) plus one refactor cycle. `KNOWN_MINOR`
bumps 6 → 7 at GREEN; documents without identity usage stay
hash-neutral (design Decision 4), so no sealed artifact moves. The
out-of-repo catherine-ugc re-model is a separate ticket and is **not**
covered here.

## RED

- [ ] 1. Confirm the pre-registered probe baseline: the gap-suite
      `identity-empty-path-equations` row (fixtures
      `identity_involution.ct.json`,
      `identity_memento_roundtrip.ct.json`) runs and observes today's
      exit-1 `equation path must be non-empty` rejection as
      `Limitation`; commit the vendored evidence baseline
      (`notes/identity-arrows-evidence/`: adjunction model with its
      four endomorphism stand-ins, the 1611-question underconstraint
      envelope, README numbers)
- [ ] 2. Kernel format tests (failing): an equation side may be `[]` at
      any known minor (fixtures' `1.3` included — no per-feature
      version floor, design Decision 5), both-sides-empty is rejected
      `IllTypedEquation` at the equation's node, the new `1.7` minor is
      accepted and `1.8` rejected at the `format_version` node,
      canonical bytes of an identity side are `[]`, and the
      hash-neutrality pins hold on the
      `absent_coproducts_are_hash_neutral` /
      `current_major_parses_a_1_5_document` patterns (absent identity
      usage → byte-identical canonical bytes and hash; pre-existing 1.0
      golden fixtures byte-identical)
- [ ] 3. Check/findings tests (failing): the flip of the gap-suite
      fixtures — one-sided-empty equations parse, execute on the
      domain derived from the non-empty side, violating rows emit one
      `violation`/`executed` with `witness.equation` and both composed
      row values (the identity side's value = the row's own id),
      satisfying rows emit nothing, findings JSON envelope shape
      unchanged; dangling-traversal rows stay
      not-evaluable-through-dangling, never identity
- [ ] 4. Engine tests (failing): `compile_equations` derives the domain
      from the first non-empty side (an empty lhs no longer compiles to
      the silently-skipped `domain: None`), and `engine_compose` over a
      zero-length path returns the start row — pinning the
      identity-neutral walk
- [ ] 5. Render tests (failing): `spelled()` in `diff.rs` renders a
      one-sided-empty equation as `[realize,abstract] = id_Spec` (the
      object from the counterpart side), `path_string` call sites and
      the `rules.rs` human line use the same spelling, and a bare empty
      path spells `id`

## GREEN

- [ ] 6. Parse: `check_path` accepts a one-sided empty path and rejects
      both-sides-empty (`IllTypedEquation`, equation node);
      `check_same_endpoints` restructured to the `Option<(src, tgt)>`
      rule (design Decision 3) so `path_endpoints` is never called on
      an empty path; unknown-arrow checks unchanged for non-empty
      sides
- [ ] 7. Compile/engine: domain derivation from the first non-empty
      side in `compile_equations`; the walk stays as-is (`engine_compose`
      is already identity-neutral) plus the Decision-4 test pin
- [ ] 8. Canonical: pin `[]` for the identity side (expected no change
      to `equation_value` — the pin test is the deliverable)
- [ ] 9. `KNOWN_MINOR` 6 → 7 in `crates/catherine-core/src/parse/schema.rs`
      and the version-gate hint strings (`errors.rs`, `cli.rs`,
      `negative_matrix.rs` expectations); author the `model-document`
      versioning MODIFIED delta at apply ("1.7 adds identity paths"),
      per the shared-requirement rule
- [ ] 10. Render: the single `id_<object>` spelling (design Decision 4)
      threaded through `diff.rs` `spelled()`, the violation lines, and
      the `rules.rs` human line
- [ ] 11. Flip the gap-suite identity fixtures' observation: the
      `identity-empty-path-equations` row reports `Cleared` with the
      executed-identity signature (violation findings carrying
      `witness.equation`), replacing the rejection signature

## REFACTOR

- [ ] 12. Single identity-semantics definition: the endpoint/domain
      derivation (Decision 3) lives once in `catherine-core` and
      compile, check, and render consume it — no second path-endpoint
      computation in the CLI renderers; gate caps respected without
      behavior change
- [ ] 13. Docs: CHANGELOG entry; a short note that identity is explicit
      (the empty path), never inferred, and declared id arrows are out
      of scope (proposal Scope boundary)

## Verification

- [ ] 14. `cargo test -p catherine-cli --test expressibility_gaps` —
      the identity rows pass as `Cleared`; `just ci`
- [ ] 15. `openspec validate --strict --all` green
- [ ] 16. Disposition row `identity-empty-path-equations` flipped
      closed with fixing evidence attached (the Cleared-run output over
      both fixtures)
- [ ] 17. Evidence re-model: `notes/identity-arrows-evidence/adjunction.ct.json`
      re-modeled without the `spec_round`/`impl_round`/`impl_t_round`/
      `test_round` endomorphisms, triangle laws stated as
      `[realize, abstract] = []`-style identity equations, reproducing
      the same 4 drift violations (drift case `mod_ddl_doctor`) under
      `catherine check`
- [ ] 18. Underconstraint meter: `catherine underconstraint` over the
      vendored corpus docs counts the `parallel_paths` findings —
      collapsed from 1611 to O(1) declared-law checks against the
      baseline envelope `notes/identity-arrows-evidence/underconstraint.json`;
      record the measured number next to the baseline
